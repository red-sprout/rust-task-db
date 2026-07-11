use crate::error::AppError;

pub fn scenario_names() -> Vec<&'static str> {
    vec![
        "scan",
        "index",
        "join",
        "aggregate",
        "having",
        "subquery",
        "sort",
        "distinct",
        "mutation",
        "sargability",
        "selectivity",
        "all",
    ]
}

pub fn scenario_sql(name: &str) -> Result<Vec<&'static str>, AppError> {
    let queries = match name {
        "scan" => vec!["SELECT * FROM tasks;", "SELECT * FROM tasks WHERE id = 100;"],
        "index" => vec!["SELECT * FROM tasks WHERE project_id = 1;", "SELECT * FROM tasks WHERE done = TRUE;"],
        "join" => vec!["SELECT p.name, t.title FROM projects p JOIN tasks t ON t.project_id = p.id;", "SELECT t.title, tag.name FROM tasks t JOIN task_tags tt ON tt.task_id = t.id JOIN tags tag ON tag.id = tt.tag_id;", "SELECT t.id, t.title FROM tasks t JOIN projects p ON p.id = t.project_id WHERE p.id = 1 AND t.done = FALSE;"],
        "aggregate" => vec!["SELECT project_id, COUNT(*) FROM tasks GROUP BY project_id;", "SELECT MAX(priority), MIN(priority), COUNT(*) FROM tasks;", "SELECT p.id, p.name, COUNT(t.id) FROM projects p LEFT JOIN tasks t ON t.project_id = p.id GROUP BY p.id, p.name;"],
        "having" => vec!["SELECT project_id, COUNT(*) AS task_count FROM tasks GROUP BY project_id HAVING COUNT(*) >= 10;"],
        "subquery" => vec!["SELECT p.id, p.name FROM projects p WHERE (SELECT COUNT(*) FROM tasks t WHERE t.project_id = p.id) >= 10;"],
        "sort" => vec!["SELECT * FROM tasks WHERE project_id = 1 ORDER BY priority DESC, id ASC LIMIT 10;"],
        "distinct" => vec!["SELECT DISTINCT project_id FROM tasks;"],
        "mutation" => vec!["UPDATE tasks SET done = TRUE WHERE project_id = 1 AND priority >= 3;", "DELETE FROM task_tags WHERE tag_id = 1;"],
        "sargability" => vec!["SELECT * FROM tasks WHERE id = 10;", "SELECT * FROM tasks WHERE id + 1 = 11;", "SELECT * FROM tasks WHERE project_id = 1;", "SELECT * FROM tasks WHERE title = 'Rust';", "SELECT * FROM tasks WHERE LOWER(title) = 'rust';", "SELECT * FROM tasks WHERE title LIKE 'Rust%';", "SELECT * FROM tasks WHERE title LIKE '%Rust%';"],
        "selectivity" => vec!["SELECT * FROM tasks WHERE done = TRUE;", "SELECT * FROM tasks WHERE project_id = 1;", "SELECT t.* FROM tasks t JOIN task_tags tt ON tt.task_id = t.id WHERE tt.tag_id = 1;"],
        "all" => return scenario_names().into_iter().filter(|name| *name != "all").try_fold(Vec::new(), |mut all, name| { all.extend(scenario_sql(name)?); Ok(all) }),
        _ => return Err(AppError::Domain(format!("Unknown lab scenario: {name}"))),
    };
    Ok(queries)
}
