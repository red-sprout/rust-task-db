use gluesql::core::ast::{
    IndexItem, JoinExecutor, JoinOperator, Query, SetExpr, Statement, TableFactor, ToSql,
};
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PlanNode {
    pub label: String,
    pub evidence: Evidence,
    pub children: Vec<PlanNode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub enum Evidence {
    Direct,
    SafeInterpretation,
}

pub fn statement_to_node(statement: &Statement) -> PlanNode {
    match statement {
        Statement::Query(query) => query_node(query),
        Statement::Update {
            table_name,
            selection,
            ..
        } => PlanNode::leaf(format!("Update [{table_name}] filter={}", expr(selection))),
        Statement::Delete {
            table_name,
            selection,
        } => PlanNode::leaf(format!("Delete [{table_name}] filter={}", expr(selection))),
        Statement::Insert {
            table_name, source, ..
        } => PlanNode::branch(format!("Insert [{table_name}]"), vec![query_node(source)]),
        other => PlanNode::leaf(format!("Statement [{}]", variant(other))),
    }
}

fn query_node(query: &Query) -> PlanNode {
    let SetExpr::Select(select) = &query.body else {
        return PlanNode::leaf("Values".into());
    };
    let mut node = relation_node(&select.from.relation);
    for join in &select.from.joins {
        let kind = match &join.join_operator {
            JoinOperator::Inner(_) => "INNER",
            JoinOperator::LeftOuter(_) => "LEFT",
        };
        let executor = match &join.join_executor {
            JoinExecutor::NestedLoop => "NestedLoop".into(),
            JoinExecutor::Hash {
                key_expr,
                value_expr,
                ..
            } => format!(
                "Hash key={} value={}",
                key_expr.to_sql(),
                value_expr.to_sql()
            ),
        };
        node = PlanNode::branch(
            format!("Join [{kind}, {executor}]"),
            vec![node, relation_node(&join.relation)],
        );
    }
    if let Some(selection) = &select.selection {
        node = PlanNode::branch(format!("Filter [{}]", selection.to_sql()), vec![node]);
    }
    if !select.group_by.is_empty() || select.having.is_some() {
        let groups = select
            .group_by
            .iter()
            .map(ToSql::to_sql)
            .collect::<Vec<_>>()
            .join(", ");
        node = PlanNode::branch(
            format!(
                "Aggregate [group_by={groups}, having={}]",
                expr(&select.having)
            ),
            vec![node],
        );
    }
    if select.distinct {
        node = PlanNode::branch("Distinct".into(), vec![node]);
    }
    let labels = select
        .projection
        .iter()
        .map(|item| format!("{item:?}"))
        .collect::<Vec<_>>()
        .join(", ");
    node = PlanNode::branch(format!("Projection [{labels}]"), vec![node]);
    if !query.order_by.is_empty() {
        node = PlanNode::branch(
            format!(
                "Sort [{}]",
                query
                    .order_by
                    .iter()
                    .map(ToSql::to_sql)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            vec![node],
        );
    }
    if let Some(limit) = &query.limit {
        node = PlanNode::branch(format!("Limit [{}]", limit.to_sql()), vec![node]);
    }
    node
}

fn relation_node(factor: &TableFactor) -> PlanNode {
    match factor {
        TableFactor::Table { name, index, .. } => match index {
            Some(IndexItem::PrimaryKey(key)) => {
                PlanNode::leaf(format!("Primary Key Lookup [{name}, key={}]", key.to_sql()))
            }
            Some(IndexItem::NonClustered {
                name: index,
                asc,
                cmp_expr,
            }) => PlanNode::leaf(format!(
                "Index Scan [{name}, index={index}, direction={asc:?}, condition={cmp_expr:?}]"
            )),
            None => PlanNode::leaf(format!("Table Scan [{name}]")),
        },
        TableFactor::Derived { subquery, .. } => {
            PlanNode::branch("Derived Subquery".into(), vec![query_node(subquery)])
        }
        other => PlanNode::leaf(format!("Table Factor [{other:?}]")),
    }
}

fn expr(value: &Option<gluesql::core::ast::Expr>) -> String {
    value
        .as_ref()
        .map(ToSql::to_sql)
        .unwrap_or_else(|| "none".into())
}
fn variant(statement: &Statement) -> &'static str {
    match statement {
        Statement::CreateTable { .. } => "CREATE TABLE",
        Statement::CreateIndex { .. } => "CREATE INDEX",
        Statement::DropTable { .. } => "DROP TABLE",
        Statement::StartTransaction => "BEGIN",
        Statement::Commit => "COMMIT",
        Statement::Rollback => "ROLLBACK",
        _ => "OTHER",
    }
}

impl PlanNode {
    fn leaf(label: String) -> Self {
        Self {
            label,
            evidence: Evidence::Direct,
            children: vec![],
        }
    }
    fn branch(label: String, children: Vec<Self>) -> Self {
        Self {
            label,
            evidence: Evidence::SafeInterpretation,
            children,
        }
    }
}

pub fn access_paths(nodes: &[PlanNode]) -> Vec<String> {
    fn collect(node: &PlanNode, paths: &mut Vec<String>) {
        if node.label.contains("Table Scan")
            || node.label.contains("Index Scan")
            || node.label.contains("Primary Key Lookup")
        {
            paths.push(node.label.clone());
        }
        for child in &node.children {
            collect(child, paths);
        }
    }
    let mut paths = Vec::new();
    for node in nodes {
        collect(node, &mut paths);
    }
    paths
}

#[cfg(test)]
mod tests {
    use super::*;
    use gluesql::core::ast::{Expr, Literal};
    #[test]
    fn tree_node_serializes_to_json() {
        let node = PlanNode::leaf("Table Scan [tasks]".into());
        assert!(serde_json::to_string(&node).unwrap().contains("Table Scan"));
    }
    #[test]
    fn renders_primary_key_access_path() {
        let node = relation_node(&TableFactor::Table {
            name: "tasks".into(),
            alias: None,
            index: Some(IndexItem::PrimaryKey(Expr::Literal(Literal::Number(
                10.into(),
            )))),
        });
        assert!(node.label.contains("Primary Key Lookup"));
    }
    #[test]
    fn renders_non_clustered_index_access_path() {
        let node = relation_node(&TableFactor::Table {
            name: "tasks".into(),
            alias: None,
            index: Some(IndexItem::NonClustered {
                name: "idx_tasks_project_id".into(),
                asc: Some(true),
                cmp_expr: None,
            }),
        });
        assert!(node.label.contains("Index Scan"));
        assert!(node.label.contains("idx_tasks_project_id"));
    }
    #[test]
    fn renders_full_scan_access_path() {
        let node = relation_node(&TableFactor::Table {
            name: "tasks".into(),
            alias: None,
            index: None,
        });
        assert_eq!(node.label, "Table Scan [tasks]");
    }
}
