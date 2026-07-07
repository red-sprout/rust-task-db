use crate::command::Command;
use crate::error::AppError;

pub fn parse_args(args: Vec<String>) -> Result<Command, AppError> {
    let mut iter = args.into_iter();
    let _program = iter.next();

    let Some(command) = iter.next() else {
        return Ok(Command::Help);
    };

    match command.as_str() {
        "add" => {
            let title = require_next(&mut iter, "Usage: rust-task add \"할 일\"")?;
            Ok(Command::Add { title })
        }
        "list" => Ok(Command::List),
        "done" => {
            let id = parse_id(require_next(&mut iter, "Usage: rust-task done 1")?)?;
            Ok(Command::Done { id })
        }
        "delete" => {
            let id = parse_id(require_next(&mut iter, "Usage: rust-task delete 1")?)?;
            Ok(Command::Delete { id })
        }
        "search" => {
            let keyword = require_next(&mut iter, "Usage: rust-task search rust")?;
            Ok(Command::Search { keyword })
        }
        "stats" => Ok(Command::Stats),
        "sql" => {
            let sql = require_next(&mut iter, "Usage: rust-task sql \"SELECT * FROM tasks\"")?;
            Ok(Command::Sql { sql })
        }
        "repl" => Ok(Command::Repl),
        "help" | "-h" | "--help" => Ok(Command::Help),
        other => Err(AppError::InvalidCommand(format!(
            "Unknown command: {other}"
        ))),
    }
}

fn require_next(
    iter: &mut impl Iterator<Item = String>,
    message: &str,
) -> Result<String, AppError> {
    iter.next()
        .ok_or_else(|| AppError::InvalidCommand(message.to_string()))
}

fn parse_id(value: String) -> Result<i64, AppError> {
    value
        .parse::<i64>()
        .map_err(|_| AppError::InvalidCommand(format!("id must be an integer: {value}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    #[test]
    fn parses_add_command() {
        let command = parse_args(args(&["rust-task", "add", "Rust"]));

        assert_eq!(
            command,
            Ok(Command::Add {
                title: "Rust".to_string()
            })
        );
    }

    #[test]
    fn parses_list_command() {
        let command = parse_args(args(&["rust-task", "list"]));

        assert_eq!(command, Ok(Command::List));
    }

    #[test]
    fn parses_done_command() {
        let command = parse_args(args(&["rust-task", "done", "1"]));

        assert_eq!(command, Ok(Command::Done { id: 1 }));
    }

    #[test]
    fn parses_delete_command() {
        let command = parse_args(args(&["rust-task", "delete", "1"]));

        assert_eq!(command, Ok(Command::Delete { id: 1 }));
    }

    #[test]
    fn parses_search_command() {
        let command = parse_args(args(&["rust-task", "search", "Rust"]));

        assert_eq!(
            command,
            Ok(Command::Search {
                keyword: "Rust".to_string()
            })
        );
    }

    #[test]
    fn parses_stats_command() {
        let command = parse_args(args(&["rust-task", "stats"]));

        assert_eq!(command, Ok(Command::Stats));
    }

    #[test]
    fn parses_sql_command() {
        let command = parse_args(args(&["rust-task", "sql", "SELECT * FROM tasks"]));

        assert_eq!(
            command,
            Ok(Command::Sql {
                sql: "SELECT * FROM tasks".to_string()
            })
        );
    }

    #[test]
    fn parses_repl_command() {
        let command = parse_args(args(&["rust-task", "repl"]));

        assert_eq!(command, Ok(Command::Repl));
    }

    #[test]
    fn parses_help_aliases() {
        assert_eq!(parse_args(args(&["rust-task", "help"])), Ok(Command::Help));
        assert_eq!(parse_args(args(&["rust-task", "-h"])), Ok(Command::Help));
        assert_eq!(
            parse_args(args(&["rust-task", "--help"])),
            Ok(Command::Help)
        );
    }

    #[test]
    fn no_command_returns_help() {
        let command = parse_args(args(&["rust-task"]));

        assert_eq!(command, Ok(Command::Help));
    }

    #[test]
    fn invalid_id_returns_error() {
        let command = parse_args(args(&["rust-task", "done", "abc"]));

        assert_eq!(
            command,
            Err(AppError::InvalidCommand(
                "id must be an integer: abc".to_string()
            ))
        );
    }

    #[test]
    fn missing_add_title_returns_error() {
        let command = parse_args(args(&["rust-task", "add"]));

        assert_eq!(
            command,
            Err(AppError::InvalidCommand(
                "Usage: rust-task add \"할 일\"".to_string()
            ))
        );
    }

    #[test]
    fn missing_done_id_returns_error() {
        let command = parse_args(args(&["rust-task", "done"]));

        assert_eq!(
            command,
            Err(AppError::InvalidCommand(
                "Usage: rust-task done 1".to_string()
            ))
        );
    }

    #[test]
    fn missing_search_keyword_returns_error() {
        let command = parse_args(args(&["rust-task", "search"]));

        assert_eq!(
            command,
            Err(AppError::InvalidCommand(
                "Usage: rust-task search rust".to_string()
            ))
        );
    }

    #[test]
    fn missing_sql_string_returns_error() {
        let command = parse_args(args(&["rust-task", "sql"]));

        assert_eq!(
            command,
            Err(AppError::InvalidCommand(
                "Usage: rust-task sql \"SELECT * FROM tasks\"".to_string()
            ))
        );
    }

    #[test]
    fn unknown_command_returns_error() {
        let command = parse_args(args(&["rust-task", "unknown"]));

        assert_eq!(
            command,
            Err(AppError::InvalidCommand(
                "Unknown command: unknown".to_string()
            ))
        );
    }
}
