use crate::error::AppError;
use crate::repository::{SqlResult, TaskRepository};
use crate::task::{Task, TaskStats};

pub struct TaskService<R: TaskRepository> {
    repository: R,
}

impl<R: TaskRepository> TaskService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub fn add(&mut self, title: String) -> Result<Task, AppError> {
        self.repository.add(title)
    }

    pub fn list(&mut self) -> Result<Vec<Task>, AppError> {
        self.repository.find_all()
    }

    pub fn done(&mut self, id: i64) -> Result<(), AppError> {
        self.repository.mark_done(id)
    }

    pub fn delete(&mut self, id: i64) -> Result<Task, AppError> {
        self.repository.delete(id)
    }

    pub fn search(&mut self, keyword: &str) -> Result<Vec<Task>, AppError> {
        self.repository.search(keyword)
    }

    pub fn stats(&mut self) -> Result<TaskStats, AppError> {
        self.repository.stats()
    }

    pub fn execute_sql(&mut self, sql: String) -> Result<Vec<SqlResult>, AppError> {
        self.repository.execute_sql(sql)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeTaskRepository {
        tasks: Vec<Task>,
    }

    impl FakeTaskRepository {
        fn new(tasks: Vec<Task>) -> Self {
            Self { tasks }
        }
    }

    impl TaskRepository for FakeTaskRepository {
        fn add(&mut self, title: String) -> Result<Task, AppError> {
            let id = self.tasks.iter().map(|task| task.id).max().unwrap_or(0) + 1;
            let task = Task::new(id, title);
            self.tasks.push(task.clone());
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
            Ok(())
        }

        fn delete(&mut self, id: i64) -> Result<Task, AppError> {
            let Some(index) = self.tasks.iter().position(|task| task.id == id) else {
                return Err(AppError::NotFound(id));
            };

            Ok(self.tasks.remove(index))
        }

        fn search(&mut self, keyword: &str) -> Result<Vec<Task>, AppError> {
            let keyword = keyword.to_lowercase();
            Ok(self
                .tasks
                .iter()
                .filter(|task| task.title.to_lowercase().contains(&keyword))
                .cloned()
                .collect())
        }

        fn stats(&mut self) -> Result<TaskStats, AppError> {
            let total = self.tasks.len();
            let done = self.tasks.iter().filter(|task| task.done).count();

            Ok(TaskStats::new(total, done))
        }

        fn execute_sql(&mut self, _sql: String) -> Result<Vec<SqlResult>, AppError> {
            Err(AppError::Unsupported(
                "FakeTaskRepository does not support SQL".to_string(),
            ))
        }
    }

    #[test]
    fn add_delegates_to_repository() {
        let repository = FakeTaskRepository::new(Vec::new());
        let mut service = TaskService::new(repository);

        let task = service.add("Rust".to_string());

        assert_eq!(task, Ok(Task::new(1, "Rust".to_string())));
        assert_eq!(service.list(), Ok(vec![Task::new(1, "Rust".to_string())]));
    }

    #[test]
    fn list_delegates_to_repository() {
        let repository = FakeTaskRepository::new(vec![Task::new(1, "Rust".to_string())]);
        let mut service = TaskService::new(repository);

        let tasks = service.list();

        assert_eq!(tasks, Ok(vec![Task::new(1, "Rust".to_string())]));
    }

    #[test]
    fn done_delegates_to_repository() {
        let repository = FakeTaskRepository::new(vec![Task::new(1, "Rust".to_string())]);
        let mut service = TaskService::new(repository);

        let result = service.done(1);
        let tasks = service.list().unwrap();

        assert_eq!(result, Ok(()));
        assert_eq!(tasks[0].done, true);
    }

    #[test]
    fn delete_delegates_to_repository() {
        let repository = FakeTaskRepository::new(vec![Task::new(1, "Rust".to_string())]);
        let mut service = TaskService::new(repository);

        let deleted = service.delete(1);

        assert_eq!(deleted, Ok(Task::new(1, "Rust".to_string())));
        assert_eq!(service.list(), Ok(Vec::new()));
    }

    #[test]
    fn search_delegates_to_repository() {
        let repository = FakeTaskRepository::new(vec![
            Task::new(1, "Rust".to_string()),
            Task::new(2, "GlueSQL".to_string()),
        ]);
        let mut service = TaskService::new(repository);

        let tasks = service.search("rust");

        assert_eq!(tasks, Ok(vec![Task::new(1, "Rust".to_string())]));
    }

    #[test]
    fn stats_delegates_to_repository() {
        let mut done = Task::new(1, "Rust".to_string());
        done.done = true;
        let repository = FakeTaskRepository::new(vec![done, Task::new(2, "GlueSQL".to_string())]);
        let mut service = TaskService::new(repository);

        let stats = service.stats();

        assert_eq!(stats, Ok(TaskStats::new(2, 1)));
    }

    #[test]
    fn execute_sql_delegates_to_repository() {
        let repository = FakeTaskRepository::new(Vec::new());
        let mut service = TaskService::new(repository);

        let result = service.execute_sql("SELECT * FROM tasks".to_string());

        assert_eq!(
            result,
            Err(AppError::Unsupported(
                "FakeTaskRepository does not support SQL".to_string()
            ))
        );
    }
}
