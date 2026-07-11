use crate::command::Command;
use crate::error::AppError;

pub fn parse_args(args: Vec<String>) -> Result<Command, AppError> {
    let mut values = args.into_iter().skip(1).collect::<Vec<_>>();
    if values.is_empty() {
        return Ok(Command::Help);
    }
    let command = values.remove(0);
    match command.as_str() {
        "add" => Ok(Command::Add {
            title: one(values, "Usage: rust-task add \"할 일\"")?,
        }),
        "list" => Ok(Command::List),
        "done" => Ok(Command::Done {
            id: id(one(values, "Usage: rust-task done 1")?)?,
        }),
        "delete" => Ok(Command::Delete {
            id: id(one(values, "Usage: rust-task delete 1")?)?,
        }),
        "search" => Ok(Command::Search {
            keyword: one(values, "Usage: rust-task search rust")?,
        }),
        "stats" => Ok(Command::Stats),
        "project" => parse_project(values),
        "task" => parse_task(values),
        "tag" => parse_tag(values),
        "seed" => Ok(Command::Seed),
        "sql" => Ok(Command::Sql {
            sql: one(values, "Usage: rust-task sql \"SELECT * FROM tasks\"")?,
        }),
        "repl" => Ok(Command::Repl),
        "help" | "-h" | "--help" => Ok(Command::Help),
        other => Err(invalid(format!("Unknown command: {other}"))),
    }
}

fn parse_project(mut v: Vec<String>) -> Result<Command, AppError> {
    let sub = take(
        &mut v,
        "Usage: rust-task project <add|list|show|delete|stats>",
    )?;
    match sub.as_str() {
        "add" => Ok(Command::ProjectAdd {
            name: one(v, "Usage: rust-task project add \"이름\"")?,
        }),
        "list" => Ok(Command::ProjectList),
        "show" => Ok(Command::ProjectShow {
            id: id(one(v, "Usage: rust-task project show 1")?)?,
        }),
        "delete" => Ok(Command::ProjectDelete {
            id: id(one(v, "Usage: rust-task project delete 1")?)?,
        }),
        "stats" => Ok(Command::ProjectStats {
            id: match v.len() {
                0 => None,
                1 => Some(id(v[0].clone())?),
                _ => return Err(invalid("Usage: rust-task project stats [id]")),
            },
        }),
        _ => Err(invalid(format!("Unknown project command: {sub}"))),
    }
}

fn parse_tag(mut v: Vec<String>) -> Result<Command, AppError> {
    let sub = take(&mut v, "Usage: rust-task tag <add|list|delete>")?;
    match sub.as_str() {
        "add" => Ok(Command::TagAdd {
            name: one(v, "Usage: rust-task tag add backend")?,
        }),
        "list" => Ok(Command::TagList),
        "delete" => Ok(Command::TagDelete {
            id: id(one(v, "Usage: rust-task tag delete 1")?)?,
        }),
        _ => Err(invalid(format!("Unknown tag command: {sub}"))),
    }
}

fn parse_task(mut v: Vec<String>) -> Result<Command, AppError> {
    let sub = take(
        &mut v,
        "Usage: rust-task task <add|list|show|done|delete|search|tag|untag|tags>",
    )?;
    match sub.as_str() {
        "add" => {
            let mut project_id = None;
            let mut priority = 3;
            let mut title = None;
            let mut tags = Vec::new();
            let mut i = 0;
            while i < v.len() {
                match v[i].as_str() {
                    "--project" => {
                        i += 1;
                        project_id = Some(id(v
                            .get(i)
                            .cloned()
                            .ok_or_else(|| invalid("--project requires id"))?)?);
                    }
                    "--priority" => {
                        i += 1;
                        priority = id(v
                            .get(i)
                            .cloned()
                            .ok_or_else(|| invalid("--priority requires 1..5"))?)?;
                    }
                    "--tag" => {
                        i += 1;
                        tags.push(
                            v.get(i)
                                .cloned()
                                .ok_or_else(|| invalid("--tag requires name"))?,
                        );
                    }
                    _ if title.is_none() => title = Some(v[i].clone()),
                    _ => return Err(invalid("task title must be one quoted argument")),
                }
                i += 1;
            }
            Ok(Command::TaskAdd {
                project_id,
                priority,
                title: title.ok_or_else(|| {
                    invalid("Usage: rust-task task add [--project 1] [--priority 3] [--tag name] \"할 일\"")
                })?,
                tags,
            })
        }
        "list" => {
            let mut project_id = None;
            let mut tag = None;
            let mut i = 0;
            while i < v.len() {
                match v[i].as_str() {
                    "--project" => {
                        i += 1;
                        project_id = Some(id(v
                            .get(i)
                            .cloned()
                            .ok_or_else(|| invalid("--project requires id"))?)?);
                    }
                    "--tag" => {
                        i += 1;
                        tag = Some(
                            v.get(i)
                                .cloned()
                                .ok_or_else(|| invalid("--tag requires name"))?,
                        );
                    }
                    x => return Err(invalid(format!("Unknown task list option: {x}"))),
                }
                i += 1;
            }
            Ok(Command::TaskList { project_id, tag })
        }
        "show" => Ok(Command::TaskShow {
            id: id(one(v, "Usage: rust-task task show 1")?)?,
        }),
        "done" => Ok(Command::TaskDone {
            id: id(one(v, "Usage: rust-task task done 1")?)?,
        }),
        "delete" => Ok(Command::TaskDelete {
            id: id(one(v, "Usage: rust-task task delete 1")?)?,
        }),
        "search" => Ok(Command::TaskSearch {
            keyword: one(v, "Usage: rust-task task search rust")?,
        }),
        "tag" => Ok(Command::TaskTag {
            id: id(take(&mut v, "Usage: rust-task task tag 1 backend")?)?,
            tag: one(v, "Usage: rust-task task tag 1 backend")?,
        }),
        "untag" => Ok(Command::TaskUntag {
            id: id(take(&mut v, "Usage: rust-task task untag 1 backend")?)?,
            tag: one(v, "Usage: rust-task task untag 1 backend")?,
        }),
        "tags" => Ok(Command::TaskTags {
            id: id(one(v, "Usage: rust-task task tags 1")?)?,
        }),
        _ => Err(invalid(format!("Unknown task command: {sub}"))),
    }
}

fn take(v: &mut Vec<String>, message: &str) -> Result<String, AppError> {
    if v.is_empty() {
        Err(invalid(message))
    } else {
        Ok(v.remove(0))
    }
}
fn one(v: Vec<String>, message: &str) -> Result<String, AppError> {
    if v.len() == 1 {
        Ok(v[0].clone())
    } else {
        Err(invalid(message))
    }
}
fn id(v: String) -> Result<i64, AppError> {
    v.parse()
        .map_err(|_| invalid(format!("id must be an integer: {v}")))
}
fn invalid(message: impl Into<String>) -> AppError {
    AppError::InvalidCommand(message.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn p(v: &[&str]) -> Result<Command, AppError> {
        parse_args(v.iter().map(|s| s.to_string()).collect())
    }
    #[test]
    fn legacy_commands_stay_supported() {
        assert_eq!(
            p(&["x", "add", "Rust"]),
            Ok(Command::Add {
                title: "Rust".into()
            })
        );
        assert_eq!(p(&["x", "list"]), Ok(Command::List));
        assert_eq!(p(&["x", "done", "1"]), Ok(Command::Done { id: 1 }));
    }
    #[test]
    fn parses_project_commands() {
        assert_eq!(
            p(&["x", "project", "add", "Lab"]),
            Ok(Command::ProjectAdd { name: "Lab".into() })
        );
        assert_eq!(
            p(&["x", "project", "stats", "2"]),
            Ok(Command::ProjectStats { id: Some(2) })
        );
    }
    #[test]
    fn parses_all_project_stats() {
        assert_eq!(
            p(&["x", "project", "stats"]),
            Ok(Command::ProjectStats { id: None })
        );
    }
    #[test]
    fn parses_task_add_tags() {
        let command = p(&[
            "x", "task", "add", "--tag", "backend", "--tag", "sql", "Plan",
        ]);
        assert_eq!(
            command,
            Ok(Command::TaskAdd {
                project_id: None,
                priority: 3,
                title: "Plan".into(),
                tags: vec!["backend".into(), "sql".into()]
            })
        );
    }
    #[test]
    fn parses_task_options() {
        assert_eq!(
            p(&[
                "x",
                "task",
                "add",
                "--project",
                "1",
                "--priority",
                "5",
                "Plan"
            ]),
            Ok(Command::TaskAdd {
                project_id: Some(1),
                priority: 5,
                title: "Plan".into(),
                tags: Vec::new()
            })
        );
        assert_eq!(
            p(&["x", "task", "list", "--tag", "backend"]),
            Ok(Command::TaskList {
                project_id: None,
                tag: Some("backend".into())
            })
        );
    }
    #[test]
    fn parses_tag_and_seed() {
        assert_eq!(
            p(&["x", "tag", "add", "backend"]),
            Ok(Command::TagAdd {
                name: "backend".into()
            })
        );
        assert_eq!(p(&["x", "seed"]), Ok(Command::Seed));
    }
    #[test]
    fn rejects_bad_id() {
        assert_eq!(
            p(&["x", "done", "no"]),
            Err(AppError::InvalidCommand("id must be an integer: no".into()))
        );
    }
    #[test]
    fn help_aliases() {
        assert_eq!(p(&["x"]), Ok(Command::Help));
        assert_eq!(p(&["x", "--help"]), Ok(Command::Help));
    }
    #[test]
    fn parses_task_relationship_commands() {
        assert_eq!(
            p(&["x", "task", "tag", "1", "backend"]),
            Ok(Command::TaskTag {
                id: 1,
                tag: "backend".into()
            })
        );
        assert_eq!(
            p(&["x", "task", "untag", "1", "backend"]),
            Ok(Command::TaskUntag {
                id: 1,
                tag: "backend".into()
            })
        );
        assert_eq!(
            p(&["x", "task", "tags", "1"]),
            Ok(Command::TaskTags { id: 1 })
        );
    }
    #[test]
    fn parses_project_delete() {
        assert_eq!(
            p(&["x", "project", "delete", "3"]),
            Ok(Command::ProjectDelete { id: 3 })
        );
    }
    #[test]
    fn rejects_unknown_subcommand() {
        assert!(p(&["x", "tag", "wat"]).is_err());
    }
}
