use crate::error::AppError;
use crate::repository::{SqlResult, TaskRepository};
use crate::task::{Task, TaskStats};
use futures::executor::block_on;
use gluesql::core::store::{GStore, GStoreMut, Planner};
use gluesql::prelude::{Glue, MemoryStorage, Payload, SledStorage, Value};
use std::path::Path;

pub struct GlueSqlTaskRepository<S = MemoryStorage>
where
    S: GStore + GStoreMut + Planner,
{
    glue: Glue<S>,
}

#[cfg(test)]
impl GlueSqlTaskRepository<MemoryStorage> {
    pub fn new() -> Result<Self, AppError> {
        let storage = MemoryStorage::default();
        let glue = Glue::new(storage);
        let mut repository = Self { glue };

        repository.create_tasks_table()?;

        Ok(repository)
    }
}

impl GlueSqlTaskRepository<SledStorage> {
    pub fn persistent(path: impl AsRef<Path>) -> Result<Self, AppError> {
        let storage =
            SledStorage::new(path).map_err(|error| AppError::GlueSql(error.to_string()))?;
        let glue = Glue::new(storage);
        let mut repository = Self { glue };

        repository.create_tasks_table()?;

        Ok(repository)
    }
}

impl<S> GlueSqlTaskRepository<S>
where
    S: GStore + GStoreMut + Planner,
{
    fn create_tasks_table(&mut self) -> Result<(), AppError> {
        self.execute(
            "CREATE TABLE IF NOT EXISTS tasks (
                id INTEGER,
                title TEXT,
                done BOOLEAN
            );",
        )?;

        Ok(())
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

impl<S> TaskRepository for GlueSqlTaskRepository<S>
where
    S: GStore + GStoreMut + Planner,
{
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

fn find_one<S>(repository: &mut GlueSqlTaskRepository<S>, id: i64) -> Result<Task, AppError>
where
    S: GStore + GStoreMut + Planner,
{
    repository
        .select_tasks(format!(
            "SELECT id, title, done FROM tasks WHERE id = {id};"
        ))?
        .into_iter()
        .next()
        .ok_or(AppError::NotFound(id))
}

fn ensure_exists<S>(repository: &mut GlueSqlTaskRepository<S>, id: i64) -> Result<(), AppError>
where
    S: GStore + GStoreMut + Planner,
{
    find_one(repository, id).map(|_| ())
}

fn select_count<S>(repository: &mut GlueSqlTaskRepository<S>, sql: &str) -> Result<usize, AppError>
where
    S: GStore + GStoreMut + Planner,
{
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
    use std::fs;
    use std::path::PathBuf;

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

    #[test]
    fn persists_tasks_with_sled_storage() {
        let path = unique_sled_path("persist");
        let _ = fs::remove_dir_all(&path);

        {
            let mut repository = GlueSqlTaskRepository::persistent(&path).unwrap();
            repository.add("Rust".to_string()).unwrap();
        }

        {
            let mut repository = GlueSqlTaskRepository::persistent(&path).unwrap();

            assert_eq!(
                repository.find_all(),
                Ok(vec![Task::new(1, "Rust".to_string())])
            );
        }

        let _ = fs::remove_dir_all(&path);
    }

    #[test]
    fn memory_storage_rejects_explicit_transactions() {
        let mut repository = GlueSqlTaskRepository::new().unwrap();

        let result = repository.execute_sql("BEGIN;".to_string());

        assert!(
            matches!(result, Err(AppError::GlueSql(message)) if message.contains("transaction is not supported"))
        );
    }

    #[test]
    fn sled_storage_rolls_back_uncommitted_insert() {
        let path = unique_sled_path("rollback");
        let _ = fs::remove_dir_all(&path);
        let mut repository = GlueSqlTaskRepository::persistent(&path).unwrap();

        repository
            .execute_sql(
                "
                BEGIN;
                INSERT INTO tasks VALUES (1, 'temporary', FALSE);
                ROLLBACK;
                "
                .to_string(),
            )
            .unwrap();

        assert_eq!(repository.find_all(), Ok(Vec::new()));

        let _ = fs::remove_dir_all(&path);
    }

    #[test]
    fn sled_storage_commits_explicit_transaction() {
        let path = unique_sled_path("commit");
        let _ = fs::remove_dir_all(&path);
        let mut repository = GlueSqlTaskRepository::persistent(&path).unwrap();

        repository
            .execute_sql(
                "
                BEGIN;
                INSERT INTO tasks VALUES (1, 'committed', FALSE);
                COMMIT;
                "
                .to_string(),
            )
            .unwrap();

        assert_eq!(
            repository.find_all(),
            Ok(vec![Task::new(1, "committed".to_string())])
        );

        let _ = fs::remove_dir_all(&path);
    }

    #[test]
    fn sled_storage_rejects_nested_transaction() {
        let path = unique_sled_path("nested-transaction");
        let _ = fs::remove_dir_all(&path);
        let mut repository = GlueSqlTaskRepository::persistent(&path).unwrap();

        repository.execute_sql("BEGIN;".to_string()).unwrap();
        let result = repository.execute_sql("BEGIN;".to_string());

        assert!(
            matches!(result, Err(AppError::GlueSql(message)) if message.contains("nested transaction is not supported"))
        );

        repository.execute_sql("ROLLBACK;".to_string()).unwrap();
        let _ = fs::remove_dir_all(&path);
    }

    #[test]
    fn sled_storage_keeps_repeatable_read_snapshot_until_commit() {
        let path = unique_sled_path("snapshot");
        let _ = fs::remove_dir_all(&path);
        let (mut writer, mut reader) = sled_repository_pair(&path);

        writer.add("before".to_string()).unwrap();

        reader.execute_sql("BEGIN;".to_string()).unwrap();
        writer
            .execute_sql(
                "
                BEGIN;
                INSERT INTO tasks VALUES (2, 'after', FALSE);
                COMMIT;
                "
                .to_string(),
            )
            .unwrap();

        assert_eq!(
            reader.find_all(),
            Ok(vec![Task::new(1, "before".to_string())])
        );

        reader.execute_sql("COMMIT;".to_string()).unwrap();

        assert_eq!(
            reader.find_all(),
            Ok(vec![
                Task::new(1, "before".to_string()),
                Task::new(2, "after".to_string())
            ])
        );

        let _ = fs::remove_dir_all(&path);
    }

    #[test]
    fn sled_storage_reports_database_locked_for_competing_writes() {
        let path = unique_sled_path("write-lock");
        let _ = fs::remove_dir_all(&path);
        let (mut first, mut second) = sled_repository_pair(&path);

        first
            .execute_sql(
                "
                BEGIN;
                INSERT INTO tasks VALUES (1, 'first writer', FALSE);
                "
                .to_string(),
            )
            .unwrap();

        let result =
            second.execute_sql("INSERT INTO tasks VALUES (2, 'second writer', FALSE);".to_string());

        assert!(
            matches!(result, Err(AppError::GlueSql(message)) if message.contains("database is locked"))
        );

        first.execute_sql("ROLLBACK;".to_string()).unwrap();
        let _ = fs::remove_dir_all(&path);
    }

    fn sled_repository_pair(
        path: impl AsRef<std::path::Path>,
    ) -> (
        GlueSqlTaskRepository<SledStorage>,
        GlueSqlTaskRepository<SledStorage>,
    ) {
        let storage = SledStorage::new(path).unwrap();
        let first_storage = storage.clone();
        let second_storage = storage;
        let mut first = GlueSqlTaskRepository {
            glue: Glue::new(first_storage),
        };
        let mut second = GlueSqlTaskRepository {
            glue: Glue::new(second_storage),
        };

        first.create_tasks_table().unwrap();
        second.create_tasks_table().unwrap();

        (first, second)
    }

    fn unique_sled_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("rust-task-db-{name}-{}", std::process::id()))
    }
}
