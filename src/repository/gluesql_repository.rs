use crate::error::AppError;
use crate::repository::{SqlResult, TaskRepository};
use crate::task::{Task, TaskStats};
use futures::executor::block_on;
use gluesql::prelude::{Glue, MemoryStorage, Payload, Value};

pub struct GlueSqlTaskRepository {
    glue: Glue<MemoryStorage>,
}

impl GlueSqlTaskRepository {
    pub fn new() -> Result<Self, AppError> {
        let storage = MemoryStorage::default();
        let glue = Glue::new(storage);
        let mut repository = Self { glue };

        repository.execute(
            "CREATE TABLE tasks (
                id INTEGER,
                title TEXT,
                done BOOLEAN
            );",
        )?;

        Ok(repository)
    }

    fn execute(&mut self, sql: impl AsRef<str>) -> Result<Vec<Payload>, AppError> {
        block_on(self.glue.execute(sql)).map_err(|error| AppError::GlueSql(error.to_string()))
    }

    fn select_tasks(&mut self, sql: impl AsRef<str>) -> Result<Vec<Task>, AppError> {
        let payloads = self.execute(sql)?;
        let Some(Payload::Select { labels: _, rows }) = payloads.into_iter().last() else {
            return Err(AppError::GlueSql("expected SELECT result".to_string()));
        };

        rows.into_iter().map(row_to_task).collect()
    }
}

impl TaskRepository for GlueSqlTaskRepository {
    fn add(&mut self, title: String) -> Result<Task, AppError> {
        let id = next_id(&self.find_all()?);
        let task = Task::new(id, title);
        let title = sql_string(&task.title);

        self.execute(format!(
            "INSERT INTO tasks VALUES ({}, {}, {});",
            task.id, title, task.done
        ))?;

        Ok(task)
    }

    fn find_all(&mut self) -> Result<Vec<Task>, AppError> {
        self.select_tasks("SELECT id, title, done FROM tasks ORDER BY id;")
    }

    fn mark_done(&mut self, id: i64) -> Result<(), AppError> {
        ensure_exists(self, id)?;

        self.execute(format!("UPDATE tasks SET done = TRUE WHERE id = {id};"))?;

        Ok(())
    }

    fn delete(&mut self, id: i64) -> Result<Task, AppError> {
        let task = find_one(self, id)?;

        self.execute(format!("DELETE FROM tasks WHERE id = {id};"))?;

        Ok(task)
    }

    fn search(&mut self, keyword: &str) -> Result<Vec<Task>, AppError> {
        let pattern = sql_string(&format!("%{keyword}%"));

        self.select_tasks(format!(
            "SELECT id, title, done FROM tasks WHERE title ILIKE {} ORDER BY id;",
            pattern
        ))
    }

    fn stats(&mut self) -> Result<TaskStats, AppError> {
        let total = select_count(self, "SELECT COUNT(*) FROM tasks;")?;
        let done = select_count(self, "SELECT COUNT(*) FROM tasks WHERE done = TRUE;")?;

        Ok(TaskStats::new(total, done))
    }

    fn execute_sql(&mut self, sql: String) -> Result<Vec<SqlResult>, AppError> {
        let payloads = self.execute(sql)?;

        payloads.into_iter().map(payload_to_sql_result).collect()
    }
}

fn find_one(repository: &mut GlueSqlTaskRepository, id: i64) -> Result<Task, AppError> {
    repository
        .select_tasks(format!(
            "SELECT id, title, done FROM tasks WHERE id = {id};"
        ))?
        .into_iter()
        .next()
        .ok_or(AppError::NotFound(id))
}

fn ensure_exists(repository: &mut GlueSqlTaskRepository, id: i64) -> Result<(), AppError> {
    find_one(repository, id).map(|_| ())
}

fn select_count(repository: &mut GlueSqlTaskRepository, sql: &str) -> Result<usize, AppError> {
    let payloads = repository.execute(sql)?;
    let Some(Payload::Select { labels: _, rows }) = payloads.into_iter().last() else {
        return Err(AppError::GlueSql("expected COUNT result".to_string()));
    };

    let Some(row) = rows.first() else {
        return Ok(0);
    };

    match row.first() {
        Some(Value::I64(value)) => Ok(*value as usize),
        Some(value) => Err(AppError::GlueSql(format!(
            "expected COUNT to return I64, got {value:?}"
        ))),
        None => Err(AppError::GlueSql("COUNT row was empty".to_string())),
    }
}

fn row_to_task(row: Vec<Value>) -> Result<Task, AppError> {
    match row.as_slice() {
        [Value::I64(id), Value::Str(title), Value::Bool(done)] => Ok(Task {
            id: *id,
            title: title.clone(),
            done: *done,
        }),
        values => Err(AppError::GlueSql(format!(
            "expected task row [I64, Str, Bool], got {values:?}"
        ))),
    }
}

fn next_id(tasks: &[Task]) -> i64 {
    tasks.iter().map(|task| task.id).max().unwrap_or(0) + 1
}

fn sql_string(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn payload_to_sql_result(payload: Payload) -> Result<SqlResult, AppError> {
    match payload {
        Payload::Select { labels, rows } => Ok(SqlResult::Select {
            labels,
            rows: rows
                .into_iter()
                .map(|row| row.into_iter().map(value_to_string).collect())
                .collect(),
        }),
        Payload::Insert(count) => Ok(SqlResult::Affected {
            kind: "insert".to_string(),
            count,
        }),
        Payload::Update(count) => Ok(SqlResult::Affected {
            kind: "update".to_string(),
            count,
        }),
        Payload::Delete(count) => Ok(SqlResult::Affected {
            kind: "delete".to_string(),
            count,
        }),
        Payload::Create => Ok(SqlResult::Message("create ok".to_string())),
        Payload::DropTable(count) => Ok(SqlResult::Affected {
            kind: "drop table".to_string(),
            count,
        }),
        other => Ok(SqlResult::Message(format!("{other:?}"))),
    }
}

fn value_to_string(value: Value) -> String {
    match value {
        Value::Bool(value) => value.to_string(),
        Value::I8(value) => value.to_string(),
        Value::I16(value) => value.to_string(),
        Value::I32(value) => value.to_string(),
        Value::I64(value) => value.to_string(),
        Value::I128(value) => value.to_string(),
        Value::U8(value) => value.to_string(),
        Value::U16(value) => value.to_string(),
        Value::U32(value) => value.to_string(),
        Value::U64(value) => value.to_string(),
        Value::U128(value) => value.to_string(),
        Value::F32(value) => value.to_string(),
        Value::F64(value) => value.to_string(),
        Value::Str(value) => value,
        Value::Null => "NULL".to_string(),
        other => format!("{other:?}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds_and_lists_tasks_with_gluesql() {
        let mut repository = GlueSqlTaskRepository::new().unwrap();

        let task = repository.add("Rust".to_string()).unwrap();

        assert_eq!(task, Task::new(1, "Rust".to_string()));
        assert_eq!(repository.find_all(), Ok(vec![task]));
    }

    #[test]
    fn marks_task_done_with_gluesql() {
        let mut repository = GlueSqlTaskRepository::new().unwrap();
        repository.add("Rust".to_string()).unwrap();

        repository.mark_done(1).unwrap();
        let tasks = repository.find_all().unwrap();

        assert_eq!(tasks[0].done, true);
    }

    #[test]
    fn deletes_task_with_gluesql() {
        let mut repository = GlueSqlTaskRepository::new().unwrap();
        repository.add("Rust".to_string()).unwrap();

        let deleted = repository.delete(1).unwrap();

        assert_eq!(deleted, Task::new(1, "Rust".to_string()));
        assert_eq!(repository.find_all(), Ok(Vec::new()));
    }

    #[test]
    fn searches_tasks_case_insensitively_with_gluesql() {
        let mut repository = GlueSqlTaskRepository::new().unwrap();
        let rust = repository.add("Rust 공부".to_string()).unwrap();
        repository.add("GlueSQL 붙이기".to_string()).unwrap();

        let result = repository.search("rust");

        assert_eq!(result, Ok(vec![rust]));
    }

    #[test]
    fn calculates_stats_with_gluesql_count() {
        let mut repository = GlueSqlTaskRepository::new().unwrap();
        repository.add("Rust 공부".to_string()).unwrap();
        repository.add("GlueSQL 붙이기".to_string()).unwrap();
        repository.mark_done(1).unwrap();

        assert_eq!(repository.stats(), Ok(TaskStats::new(2, 1)));
    }

    #[test]
    fn calculates_empty_stats_with_gluesql_count() {
        let mut repository = GlueSqlTaskRepository::new().unwrap();

        assert_eq!(repository.stats(), Ok(TaskStats::new(0, 0)));
    }

    #[test]
    fn returns_not_found_when_gluesql_task_is_missing() {
        let mut repository = GlueSqlTaskRepository::new().unwrap();

        assert_eq!(repository.delete(404), Err(AppError::NotFound(404)));
        assert_eq!(repository.mark_done(404), Err(AppError::NotFound(404)));
    }

    #[test]
    fn escapes_single_quote_in_title() {
        let mut repository = GlueSqlTaskRepository::new().unwrap();
        let task = repository.add("Rust's ownership".to_string()).unwrap();

        assert_eq!(repository.find_all(), Ok(vec![task]));
    }

    #[test]
    fn executes_select_sql_with_gluesql() {
        let mut repository = GlueSqlTaskRepository::new().unwrap();
        repository.add("Rust".to_string()).unwrap();

        let result = repository
            .execute_sql("SELECT id, title, done FROM tasks;".to_string())
            .unwrap();

        assert_eq!(
            result,
            vec![SqlResult::Select {
                labels: vec!["id".to_string(), "title".to_string(), "done".to_string()],
                rows: vec![vec![
                    "1".to_string(),
                    "Rust".to_string(),
                    "false".to_string()
                ]]
            }]
        );
    }

    #[test]
    fn executes_insert_update_delete_sql_with_gluesql() {
        let mut repository = GlueSqlTaskRepository::new().unwrap();

        let insert = repository
            .execute_sql("INSERT INTO tasks VALUES (1, 'Rust', FALSE);".to_string())
            .unwrap();
        let update = repository
            .execute_sql("UPDATE tasks SET done = TRUE WHERE id = 1;".to_string())
            .unwrap();
        let delete = repository
            .execute_sql("DELETE FROM tasks WHERE id = 1;".to_string())
            .unwrap();

        assert_eq!(
            insert,
            vec![SqlResult::Affected {
                kind: "insert".to_string(),
                count: 1
            }]
        );
        assert_eq!(
            update,
            vec![SqlResult::Affected {
                kind: "update".to_string(),
                count: 1
            }]
        );
        assert_eq!(
            delete,
            vec![SqlResult::Affected {
                kind: "delete".to_string(),
                count: 1
            }]
        );
    }

    #[test]
    fn add_uses_next_id_after_delete() {
        let mut repository = GlueSqlTaskRepository::new().unwrap();
        repository.add("first".to_string()).unwrap();
        repository.add("second".to_string()).unwrap();
        repository.delete(2).unwrap();

        let task = repository.add("third".to_string()).unwrap();

        assert_eq!(task, Task::new(2, "third".to_string()));
    }

    #[test]
    fn invalid_sql_returns_gluesql_error() {
        let mut repository = GlueSqlTaskRepository::new().unwrap();

        let result = repository.execute_sql("SELECT * FROM missing_table;".to_string());

        assert!(matches!(result, Err(AppError::GlueSql(_))));
    }
}
