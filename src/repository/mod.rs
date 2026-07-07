mod gluesql_repository;

use crate::error::AppError;
use crate::task::{Task, TaskStats};
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

pub use gluesql_repository::GlueSqlTaskRepository;

pub trait TaskRepository {
    fn add(&mut self, title: String) -> Result<Task, AppError>;
    fn find_all(&mut self) -> Result<Vec<Task>, AppError>;
    fn mark_done(&mut self, id: i64) -> Result<(), AppError>;
    fn delete(&mut self, id: i64) -> Result<Task, AppError>;
    fn search(&mut self, keyword: &str) -> Result<Vec<Task>, AppError>;
    fn stats(&mut self) -> Result<TaskStats, AppError>;
    fn execute_sql(&mut self, sql: String) -> Result<Vec<SqlResult>, AppError>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SqlResult {
    Select {
        labels: Vec<String>,
        rows: Vec<Vec<String>>,
    },
    Affected {
        kind: String,
        count: usize,
    },
    Message(String),
}

#[allow(dead_code)]
pub struct JsonTaskRepository {
    path: PathBuf,
    tasks: Vec<Task>,
}

#[allow(dead_code)]
impl JsonTaskRepository {
    pub fn new(path: impl AsRef<Path>) -> Result<Self, AppError> {
        let path = path.as_ref().to_path_buf();
        let tasks = load_tasks(&path)?;

        Ok(Self { path, tasks })
    }

    fn save(&self) -> Result<(), AppError> {
        save_tasks(&self.path, &self.tasks)
    }
}

impl TaskRepository for JsonTaskRepository {
    fn add(&mut self, title: String) -> Result<Task, AppError> {
        let id = next_id(&self.tasks);
        let task = Task::new(id, title);

        self.tasks.push(task.clone());
        self.save()?;

        Ok(task)
    }

    fn find_all(&mut self) -> Result<Vec<Task>, AppError> {
        Ok(self.tasks.clone())
    }

    fn mark_done(&mut self, id: i64) -> Result<(), AppError> {
        let Some(task) = self.tasks.iter_mut().find(|task| task.id == id) else {
            return Err(AppError::NotFound(id));
        };

        task.done = true;
        self.save()
    }

    fn delete(&mut self, id: i64) -> Result<Task, AppError> {
        let Some(index) = self.tasks.iter().position(|task| task.id == id) else {
            return Err(AppError::NotFound(id));
        };

        let task = self.tasks.remove(index);
        self.save()?;

        Ok(task)
    }

    fn search(&mut self, keyword: &str) -> Result<Vec<Task>, AppError> {
        let keyword = keyword.to_lowercase();
        let tasks = self
            .tasks
            .iter()
            .filter(|task| task.title.to_lowercase().contains(&keyword))
            .cloned()
            .collect();

        Ok(tasks)
    }

    fn stats(&mut self) -> Result<TaskStats, AppError> {
        let total = self.tasks.len();
        let done = self.tasks.iter().filter(|task| task.done).count();

        Ok(TaskStats::new(total, done))
    }

    fn execute_sql(&mut self, _sql: String) -> Result<Vec<SqlResult>, AppError> {
        Err(AppError::Unsupported(
            "SQL command is only supported by GlueSqlTaskRepository".to_string(),
        ))
    }
}

#[allow(dead_code)]
fn load_tasks(path: impl AsRef<Path>) -> Result<Vec<Task>, AppError> {
    let path = path.as_ref();

    match fs::read_to_string(path) {
        Ok(contents) => serde_json::from_str(&contents).map_err(AppError::from),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(Vec::new()),
        Err(error) => Err(AppError::from(error)),
    }
}

#[allow(dead_code)]
fn save_tasks(path: impl AsRef<Path>, tasks: &[Task]) -> Result<(), AppError> {
    let path = path.as_ref();
    let contents = serde_json::to_string_pretty(tasks)?;

    fs::write(path, contents)?;

    Ok(())
}

#[allow(dead_code)]
fn next_id(tasks: &[Task]) -> i64 {
    tasks.iter().map(|task| task.id).max().unwrap_or(0) + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_tasks_file_loads_empty_vec() {
        let path = unique_test_path("missing");

        let mut repository = JsonTaskRepository::new(&path).unwrap();

        assert_eq!(repository.find_all(), Ok(Vec::new()));
    }

    #[test]
    fn adds_task_and_saves_to_json_file() {
        let path = unique_test_path("add");
        let mut repository = JsonTaskRepository::new(&path).unwrap();

        let task = repository.add("Rust".to_string()).unwrap();
        let mut reloaded = JsonTaskRepository::new(&path).unwrap();
        let _ = fs::remove_file(&path);

        assert_eq!(task, Task::new(1, "Rust".to_string()));
        assert_eq!(reloaded.find_all(), Ok(vec![task]));
    }

    #[test]
    fn marks_task_done_and_saves_to_json_file() {
        let path = unique_test_path("done");
        let mut repository = JsonTaskRepository::new(&path).unwrap();
        repository.add("Rust".to_string()).unwrap();

        repository.mark_done(1).unwrap();
        let mut reloaded = JsonTaskRepository::new(&path).unwrap();
        let _ = fs::remove_file(&path);

        assert_eq!(reloaded.find_all().unwrap()[0].done, true);
    }

    #[test]
    fn deletes_task_and_saves_to_json_file() {
        let path = unique_test_path("delete");
        let mut repository = JsonTaskRepository::new(&path).unwrap();
        repository.add("Rust".to_string()).unwrap();

        let deleted = repository.delete(1).unwrap();
        let mut reloaded = JsonTaskRepository::new(&path).unwrap();
        let _ = fs::remove_file(&path);

        assert_eq!(deleted, Task::new(1, "Rust".to_string()));
        assert_eq!(reloaded.find_all(), Ok(Vec::new()));
    }

    #[test]
    fn missing_done_returns_error() {
        let path = unique_test_path("missing-done");
        let mut repository = JsonTaskRepository::new(&path).unwrap();

        let result = repository.mark_done(404);

        assert_eq!(result, Err(AppError::NotFound(404)));
    }

    #[test]
    fn missing_delete_returns_error() {
        let path = unique_test_path("missing-delete");
        let mut repository = JsonTaskRepository::new(&path).unwrap();

        let result = repository.delete(404);

        assert_eq!(result, Err(AppError::NotFound(404)));
    }

    #[test]
    fn invalid_json_returns_error() {
        let path = unique_test_path("invalid-json");

        fs::write(&path, "{ invalid json").unwrap();
        let result = JsonTaskRepository::new(&path);
        let _ = fs::remove_file(&path);

        assert!(result.is_err());
    }

    #[test]
    fn searches_tasks_by_title_case_insensitive() {
        let path = unique_test_path("search");
        let mut repository = JsonTaskRepository::new(&path).unwrap();
        let rust = repository.add("Rust 공부".to_string()).unwrap();
        repository.add("GlueSQL 붙이기".to_string()).unwrap();

        let result = repository.search("rust");
        let _ = fs::remove_file(&path);

        assert_eq!(result, Ok(vec![rust]));
    }

    #[test]
    fn calculates_task_stats() {
        let path = unique_test_path("stats");
        let mut repository = JsonTaskRepository::new(&path).unwrap();
        repository.add("Rust 공부".to_string()).unwrap();
        repository.add("GlueSQL 붙이기".to_string()).unwrap();
        repository.mark_done(1).unwrap();

        let result = repository.stats();
        let _ = fs::remove_file(&path);

        assert_eq!(result, Ok(TaskStats::new(2, 1)));
    }

    fn unique_test_path(name: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        path.push(format!("rust-task-{name}-{nanos}.json"));
        path
    }
}
