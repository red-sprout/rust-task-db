#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Project {
    pub id: i64,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProjectStats {
    pub project: Project,
    pub total: usize,
    pub done: usize,
    pub todo: usize,
    pub completion_rate: f64,
}

impl ProjectStats {
    pub fn new(project: Project, total: usize, done: usize) -> Self {
        let completion_rate = if total == 0 {
            0.0
        } else {
            done as f64 * 100.0 / total as f64
        };
        Self {
            project,
            total,
            done,
            todo: total - done,
            completion_rate,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn calculates_rate() {
        let s = ProjectStats::new(
            Project {
                id: 1,
                name: "Lab".into(),
            },
            4,
            3,
        );
        assert_eq!(s.todo, 1);
        assert_eq!(s.completion_rate, 75.0);
    }
    #[test]
    fn empty_rate_is_zero() {
        assert_eq!(
            ProjectStats::new(
                Project {
                    id: 1,
                    name: "Lab".into()
                },
                0,
                0
            )
            .completion_rate,
            0.0
        );
    }
}
