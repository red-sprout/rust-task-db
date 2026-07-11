use crate::error::AppError;
use crate::project::{Project, ProjectStats};
use crate::repository::{SqlResult, TaskManagementRepository, TaskRepository};
use crate::tag::Tag;
use crate::task::{Task, TaskDetail, TaskStats};

pub struct TaskService<R> {
    repository: R,
}

impl<R> TaskService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

impl<R: TaskRepository> TaskService<R> {
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
    pub fn search(&mut self, k: &str) -> Result<Vec<Task>, AppError> {
        self.repository.search(k)
    }
    pub fn stats(&mut self) -> Result<TaskStats, AppError> {
        self.repository.stats()
    }
    pub fn execute_sql(&mut self, sql: String) -> Result<Vec<SqlResult>, AppError> {
        self.repository.execute_sql(sql)
    }
}

impl<R: TaskManagementRepository> TaskService<R> {
    pub fn add_project(&mut self, name: String) -> Result<Project, AppError> {
        self.repository.add_project(name)
    }
    pub fn list_projects(&mut self) -> Result<Vec<Project>, AppError> {
        self.repository.list_projects()
    }
    pub fn show_project(&mut self, id: i64) -> Result<Project, AppError> {
        self.repository.show_project(id)
    }
    pub fn delete_project(&mut self, id: i64) -> Result<Project, AppError> {
        self.repository.delete_project(id)
    }
    pub fn project_stats(&mut self, id: i64) -> Result<ProjectStats, AppError> {
        self.repository.project_stats(id)
    }
    pub fn all_project_stats(&mut self) -> Result<Vec<ProjectStats>, AppError> {
        self.repository.all_project_stats()
    }
    pub fn add_task_with_tags(
        &mut self,
        p: Option<i64>,
        priority: i64,
        title: String,
        tags: Vec<String>,
    ) -> Result<Task, AppError> {
        self.repository.add_task_with_tags(p, priority, title, tags)
    }
    pub fn list_tasks(&mut self, p: Option<i64>, tag: Option<&str>) -> Result<Vec<Task>, AppError> {
        self.repository.list_tasks(p, tag)
    }
    pub fn show_task(&mut self, id: i64) -> Result<TaskDetail, AppError> {
        self.repository.show_task(id)
    }
    pub fn add_tag(&mut self, name: String) -> Result<Tag, AppError> {
        self.repository.add_tag(name)
    }
    pub fn list_tags(&mut self) -> Result<Vec<Tag>, AppError> {
        self.repository.list_tags()
    }
    pub fn delete_tag(&mut self, id: i64) -> Result<Tag, AppError> {
        self.repository.delete_tag(id)
    }
    pub fn tag_task(&mut self, id: i64, tag: &str) -> Result<(), AppError> {
        self.repository.tag_task(id, tag)
    }
    pub fn untag_task(&mut self, id: i64, tag: &str) -> Result<(), AppError> {
        self.repository.untag_task(id, tag)
    }
    pub fn task_tags(&mut self, id: i64) -> Result<Vec<Tag>, AppError> {
        self.repository.task_tags(id)
    }
    pub fn seed(&mut self) -> Result<(), AppError> {
        self.repository.seed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Fake {
        tasks: Vec<Task>,
    }
    impl TaskRepository for Fake {
        fn add(&mut self, title: String) -> Result<Task, AppError> {
            let t = Task::new(self.tasks.len() as i64 + 1, title);
            self.tasks.push(t.clone());
            Ok(t)
        }
        fn find_all(&mut self) -> Result<Vec<Task>, AppError> {
            Ok(self.tasks.clone())
        }
        fn mark_done(&mut self, id: i64) -> Result<(), AppError> {
            self.tasks
                .iter_mut()
                .find(|t| t.id == id)
                .ok_or(AppError::NotFound(id))?
                .done = true;
            Ok(())
        }
        fn delete(&mut self, id: i64) -> Result<Task, AppError> {
            let i = self
                .tasks
                .iter()
                .position(|t| t.id == id)
                .ok_or(AppError::NotFound(id))?;
            Ok(self.tasks.remove(i))
        }
        fn search(&mut self, k: &str) -> Result<Vec<Task>, AppError> {
            Ok(self
                .tasks
                .iter()
                .filter(|t| t.title.to_lowercase().contains(&k.to_lowercase()))
                .cloned()
                .collect())
        }
        fn stats(&mut self) -> Result<TaskStats, AppError> {
            Ok(TaskStats::new(
                self.tasks.len(),
                self.tasks.iter().filter(|t| t.done).count(),
            ))
        }
        fn execute_sql(&mut self, _: String) -> Result<Vec<SqlResult>, AppError> {
            Err(AppError::Unsupported("fake SQL".into()))
        }
    }
    fn service() -> TaskService<Fake> {
        TaskService::new(Fake { tasks: vec![] })
    }
    #[test]
    fn add_delegates() {
        let mut s = service();
        assert_eq!(s.add("Rust".into()).unwrap(), Task::new(1, "Rust".into()));
    }
    #[test]
    fn list_delegates() {
        let mut s = service();
        s.add("Rust".into()).unwrap();
        assert_eq!(s.list().unwrap().len(), 1);
    }
    #[test]
    fn done_delegates() {
        let mut s = service();
        s.add("Rust".into()).unwrap();
        s.done(1).unwrap();
        assert!(s.list().unwrap()[0].done);
    }
    #[test]
    fn delete_delegates() {
        let mut s = service();
        s.add("Rust".into()).unwrap();
        assert_eq!(s.delete(1).unwrap().id, 1);
    }
    #[test]
    fn search_delegates() {
        let mut s = service();
        s.add("Rust".into()).unwrap();
        assert_eq!(s.search("rust").unwrap().len(), 1);
    }
    #[test]
    fn stats_delegates() {
        let mut s = service();
        s.add("Rust".into()).unwrap();
        assert_eq!(s.stats().unwrap(), TaskStats::new(1, 0));
    }
    #[test]
    fn sql_delegates() {
        let mut s = service();
        assert_eq!(
            s.execute_sql("SELECT 1".into()),
            Err(AppError::Unsupported("fake SQL".into()))
        );
    }
}
