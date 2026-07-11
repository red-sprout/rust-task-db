use super::plan::PlanNode;
use super::runtime::MetricsSnapshot;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct AnalysisReport {
    pub sql: String,
    pub plans: Vec<PlanNode>,
    pub access_paths: Vec<String>,
    pub raw_plan: String,
    pub result_rows: usize,
    pub affected_rows: usize,
    pub elapsed_micros: u128,
    pub metrics: MetricsSnapshot,
    pub limitations: Vec<String>,
}

pub fn render_tree(
    report: &AnalysisReport,
    plan_only: bool,
    runtime_only: bool,
    raw: bool,
) -> String {
    let mut output = format!(
        "SQL\n────────────────────────────────────────\n{}\n",
        report.sql
    );
    if !runtime_only || plan_only {
        output.push_str("\nQUERY PLAN\n────────────────────────────────────────\n");
        for node in &report.plans {
            render_node(node, "", true, &mut output);
        }
        output.push_str("\nACCESS PATH\n────────────────────────────────────────\n");
        if report.access_paths.is_empty() {
            output.push_str("No table access path is present in this Statement.\n");
        }
        for path in &report.access_paths {
            output.push_str(&format!("- {path}\n"));
        }
    }
    if !plan_only || runtime_only {
        let metrics = &report.metrics;
        output.push_str(&format!("\nRUNTIME STATISTICS\n────────────────────────────────────────\nfetch_data calls        : {}\nscan_data calls         : {}\nscan_indexed_data calls : {}\nrows consumed           : {}\nrows returned           : {}\nrows affected           : {}\nappend/insert/delete    : {}/{}/{}\nelapsed                 : {} µs\n", metrics.fetch_data_calls, metrics.scan_data_calls, metrics.scan_indexed_data_calls, metrics.rows_consumed, report.result_rows, report.affected_rows, metrics.append_data_calls, metrics.insert_data_calls, metrics.delete_data_calls, report.elapsed_micros));
    }
    if raw {
        output.push_str(&format!(
            "\nRAW PLAN\n────────────────────────────────────────\n{}\n",
            report.raw_plan
        ));
    }
    output.push_str("\nLIMITATIONS\n────────────────────────────────────────\n");
    for limitation in &report.limitations {
        output.push_str(&format!("- {limitation}\n"));
    }
    output
}

fn render_node(node: &PlanNode, prefix: &str, last: bool, output: &mut String) {
    output.push_str(prefix);
    output.push_str(if last { "└─ " } else { "├─ " });
    output.push_str(&node.label);
    output.push('\n');
    let next = format!("{prefix}{}", if last { "   " } else { "│  " });
    for (index, child) in node.children.iter().enumerate() {
        render_node(child, &next, index + 1 == node.children.len(), output);
    }
}
