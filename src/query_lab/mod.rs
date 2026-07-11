mod plan;
mod report;
pub(crate) mod runtime;
mod scenarios;

use crate::command::AnalyzeFormat;
use crate::error::AppError;
use crate::repository::GlueSqlTaskRepository;
use gluesql::core::store::{GStore, GStoreMut, Planner};
use gluesql::prelude::{Payload, SledStorage};
use runtime::{MetricSource, TracingStorage};
use std::{path::Path, time::Instant};

pub use report::AnalysisReport;
pub use scenarios::{scenario_names, scenario_sql};

pub fn analyze<S>(
    repository: &mut GlueSqlTaskRepository<S>,
    sql: &str,
    execute_runtime: bool,
) -> Result<AnalysisReport, AppError>
where
    S: GStore + GStoreMut + Planner + MetricSource,
{
    let statements = repository.plan_sql(sql)?;
    let raw_plan = format!("{statements:#?}");
    let plans: Vec<plan::PlanNode> = statements.iter().map(plan::statement_to_node).collect();
    let access_paths = plan::access_paths(&plans);
    repository.storage().reset_metrics();
    let started = Instant::now();
    let mut result_rows = 0;
    let mut affected_rows = 0;
    if execute_runtime {
        for statement in &statements {
            match repository.execute_statement(statement)? {
                Payload::Select { rows, .. } => result_rows += rows.len(),
                Payload::Insert(count) | Payload::Update(count) | Payload::Delete(count) => {
                    affected_rows += count
                }
                _ => {}
            }
        }
    }
    let metrics = repository.storage().snapshot_metrics();
    Ok(AnalysisReport {
        sql: sql.to_string(),
        plans,
        access_paths,
        raw_plan,
        result_rows,
        affected_rows,
        elapsed_micros: if execute_runtime {
            started.elapsed().as_micros()
        } else {
            0
        },
        metrics,
        limitations: vec![
            "Public GlueSQL APIs do not expose per-operator input/output row counts.".into(),
        ],
    })
}

pub fn persistent_traced(
    path: impl AsRef<Path>,
) -> Result<GlueSqlTaskRepository<TracingStorage<SledStorage>>, AppError> {
    let mut storage =
        SledStorage::new(path).map_err(|error| AppError::GlueSql(error.to_string()))?;
    // GlueSQL SledStorage의 기본 stale-lock timeout은 1시간이다. CLI가 transaction
    // 도중 비정상 종료됐다면 다음 실행에서 오래 기다리지 않고 복구하도록 시작 시에만
    // 60초 timeout을 사용하고, 초기화 뒤 기본 1시간 정책으로 되돌린다.
    storage.set_transaction_timeout(Some(60_000));
    let mut repository = GlueSqlTaskRepository::from_storage(TracingStorage::new(storage), true)?;
    repository
        .storage_mut()
        .inner_mut()
        .set_transaction_timeout(Some(3_600_000));
    Ok(repository)
}

pub fn render_report(
    report: &AnalysisReport,
    format: AnalyzeFormat,
    plan_only: bool,
    runtime_only: bool,
    raw: bool,
) -> Result<String, AppError> {
    match format {
        AnalyzeFormat::Json => serde_json::to_string_pretty(report).map_err(AppError::from),
        AnalyzeFormat::Tree => Ok(report::render_tree(report, plan_only, runtime_only, raw)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::TaskRepository;
    use gluesql::prelude::MemoryStorage;

    fn repository() -> GlueSqlTaskRepository<TracingStorage<MemoryStorage>> {
        GlueSqlTaskRepository::from_storage(TracingStorage::new(MemoryStorage::default()), false)
            .unwrap()
    }

    #[test]
    fn runtime_counts_scan_and_iterator_rows() {
        let mut repository = repository();
        repository.add("one".into()).unwrap();
        repository.add("two".into()).unwrap();
        let report = analyze(&mut repository, "SELECT * FROM tasks;", true).unwrap();
        assert_eq!(report.metrics.scan_data_calls, 1);
        assert_eq!(report.metrics.rows_consumed, 2);
        assert_eq!(report.result_rows, 2);
    }

    #[test]
    fn primary_key_plan_uses_fetch_data() {
        let mut repository = repository();
        repository.add("one".into()).unwrap();
        let report = analyze(&mut repository, "SELECT * FROM tasks WHERE id = 1;", true).unwrap();
        assert!(
            report.plans[0].children.iter().any(contains_primary_key)
                || contains_primary_key(&report.plans[0])
        );
        assert_eq!(report.metrics.fetch_data_calls, 1);
        assert_eq!(report.metrics.rows_consumed, 1);
    }

    #[test]
    fn tracing_storage_delegates_sled_secondary_index_planning() {
        let path = std::env::temp_dir().join(format!(
            "rust-task-query-lab-index-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let storage = SledStorage::new(path).unwrap();
        let mut repository =
            GlueSqlTaskRepository::from_storage(TracingStorage::new(storage), true).unwrap();

        let report = analyze(
            &mut repository,
            "SELECT * FROM tasks WHERE project_id = 1;",
            true,
        )
        .unwrap();

        assert!(contains_label(&report.plans[0], "Index Scan [tasks"));
        assert_eq!(report.metrics.scan_indexed_data_calls, 1);
        assert_eq!(report.metrics.scan_data_calls, 0);
    }

    #[test]
    fn metrics_reset_for_each_query() {
        let mut repository = repository();
        repository.add("one".into()).unwrap();
        let _ = analyze(&mut repository, "SELECT * FROM tasks;", true).unwrap();
        let second = analyze(&mut repository, "SELECT * FROM tasks;", true).unwrap();
        assert_eq!(second.metrics.scan_data_calls, 1);
        assert_eq!(second.metrics.rows_consumed, 1);
    }

    #[test]
    fn renders_join_plan_from_planned_statement() {
        let mut repository = repository();
        let report = analyze(
            &mut repository,
            "SELECT p.name, t.title FROM projects p JOIN tasks t ON t.project_id = p.id;",
            true,
        )
        .unwrap();
        assert!(contains_label(&report.plans[0], "Join [INNER"));
    }

    #[test]
    fn renders_aggregate_sort_and_limit() {
        let mut repository = repository();
        let report = analyze(&mut repository, "SELECT project_id, COUNT(*) FROM tasks GROUP BY project_id ORDER BY project_id LIMIT 10;", true).unwrap();
        for label in ["Aggregate", "Sort", "Limit"] {
            assert!(contains_label(&report.plans[0], label), "missing {label}");
        }
    }

    #[test]
    fn json_report_contains_plan_and_metrics() {
        let mut repository = repository();
        let report = analyze(&mut repository, "SELECT * FROM tasks;", true).unwrap();
        let json = render_report(&report, AnalyzeFormat::Json, false, false, false).unwrap();
        assert!(json.contains("scan_data_calls"));
        assert!(json.contains("Table Scan"));
    }

    #[test]
    fn scenario_list_and_scan_queries_are_available() {
        assert!(scenario_names().contains(&"join"));
        assert_eq!(scenario_sql("scan").unwrap().len(), 2);
        assert!(scenario_sql("missing").is_err());
    }

    #[test]
    fn plan_only_does_not_execute_mutation() {
        let mut repository = repository();
        let report = analyze(
            &mut repository,
            "INSERT INTO tasks VALUES (1, NULL, 'planned', FALSE, 3);",
            false,
        )
        .unwrap();
        assert_eq!(report.affected_rows, 0);
        assert!(repository.find_all().unwrap().is_empty());
    }

    #[test]
    fn tree_report_contains_required_sections() {
        let mut repository = repository();
        let report = analyze(&mut repository, "SELECT * FROM tasks;", true).unwrap();
        let output = render_report(&report, AnalyzeFormat::Tree, false, false, false).unwrap();
        for section in [
            "SQL",
            "QUERY PLAN",
            "ACCESS PATH",
            "RUNTIME STATISTICS",
            "LIMITATIONS",
        ] {
            assert!(output.contains(section));
        }
    }

    #[test]
    fn scan_scenario_executes_all_queries() {
        let mut repository = repository();
        for sql in scenario_sql("scan").unwrap() {
            analyze(&mut repository, sql, true).unwrap();
        }
    }

    #[test]
    fn mutation_reports_affected_rows_and_storage_writes() {
        let mut repository = repository();
        repository.add("one".into()).unwrap();
        let report = analyze(
            &mut repository,
            "UPDATE tasks SET done = TRUE WHERE id = 1;",
            true,
        )
        .unwrap();
        assert_eq!(report.affected_rows, 1);
        assert!(report.metrics.insert_data_calls + report.metrics.append_data_calls > 0);
    }

    fn contains_primary_key(node: &plan::PlanNode) -> bool {
        node.label.contains("Primary Key Lookup") || node.children.iter().any(contains_primary_key)
    }
    fn contains_label(node: &plan::PlanNode, label: &str) -> bool {
        node.label.contains(label)
            || node
                .children
                .iter()
                .any(|child| contains_label(child, label))
    }
}
