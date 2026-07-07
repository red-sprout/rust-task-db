use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct Task {
    pub id: i64,
    pub title: String,
    pub done: bool,
}

impl Task {
    pub fn new(id: i64, title: String) -> Self {
        Self {
            id,
            title,
            done: false,
        }
    }
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
}
