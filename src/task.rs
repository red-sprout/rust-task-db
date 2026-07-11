use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct Task {
    pub id: i64,
    #[serde(default)]
    pub project_id: Option<i64>,
    pub title: String,
    pub done: bool,
    #[serde(default = "default_priority")]
    pub priority: i64,
}

impl Task {
    pub fn new(id: i64, title: String) -> Self {
        Self {
            id,
            project_id: None,
            title,
            done: false,
            priority: default_priority(),
        }
    }

    pub fn with_project(id: i64, project_id: Option<i64>, title: String, priority: i64) -> Self {
        Self {
            id,
            project_id,
            title,
            done: false,
            priority,
        }
    }
}

fn default_priority() -> i64 {
    3
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskDetail {
    pub task: Task,
    pub project_name: Option<String>,
    pub tags: Vec<crate::tag::Tag>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskStats {
    pub total: usize,
    pub done: usize,
    pub todo: usize,
}

impl TaskStats {
    pub fn new(total: usize, done: usize) -> Self {
        Self {
            total,
            done,
            todo: total - done,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_new_sets_id_title_and_default_done() {
        let task = Task::new(7, "Rust 공부".to_string());

        assert_eq!(task.id, 7);
        assert_eq!(task.title, "Rust 공부");
        assert_eq!(task.done, false);
    }

    #[test]
    fn task_stats_new_calculates_todo_count() {
        let stats = TaskStats::new(5, 2);

        assert_eq!(stats.total, 5);
        assert_eq!(stats.done, 2);
        assert_eq!(stats.todo, 3);
    }

    #[test]
    fn task_new_uses_unassigned_project_and_default_priority() {
        let task = Task::new(1, "Rust".into());
        assert_eq!(task.project_id, None);
        assert_eq!(task.priority, 3);
    }

    #[test]
    fn task_with_project_keeps_relationship_and_priority() {
        let task = Task::with_project(1, Some(7), "Planner".into(), 5);
        assert_eq!((task.project_id, task.priority), (Some(7), 5));
    }
}
