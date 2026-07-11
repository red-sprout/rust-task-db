use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum AppError {
    Io(std::io::Error),
    Json(serde_json::Error),
    GlueSql(String),
    NotFound(i64),
    InvalidCommand(String),
    Unsupported(String),
    Domain(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "I/O error: {error}"),
            Self::Json(error) => write!(formatter, "JSON error: {error}"),
            Self::GlueSql(message) => write!(formatter, "GlueSQL error: {message}"),
            Self::NotFound(id) => write!(formatter, "Task not found: {id}"),
            Self::InvalidCommand(message) => write!(formatter, "{message}"),
            Self::Unsupported(message) => write!(formatter, "{message}"),
            Self::Domain(message) => write!(formatter, "{message}"),
        }
    }
}

impl Error for AppError {}

impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

impl PartialEq for AppError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::NotFound(left), Self::NotFound(right)) => left == right,
            (Self::InvalidCommand(left), Self::InvalidCommand(right)) => left == right,
            (Self::GlueSql(left), Self::GlueSql(right)) => left == right,
            (Self::Unsupported(left), Self::Unsupported(right)) => left == right,
            (Self::Domain(left), Self::Domain(right)) => left == right,
            (Self::Io(left), Self::Io(right)) => left.to_string() == right.to_string(),
            (Self::Json(left), Self::Json(right)) => left.to_string() == right.to_string(),
            _ => false,
        }
    }
}

impl Eq for AppError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn displays_not_found_message() {
        let error = AppError::NotFound(404);

        assert_eq!(error.to_string(), "Task not found: 404");
    }

    #[test]
    fn displays_invalid_command_message_without_prefix() {
        let error = AppError::InvalidCommand("Unknown command: nope".to_string());

        assert_eq!(error.to_string(), "Unknown command: nope");
    }

    #[test]
    fn converts_io_error_into_app_error() {
        let error = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "blocked");
        let app_error = AppError::from(error);

        assert_eq!(app_error.to_string(), "I/O error: blocked");
    }

    #[test]
    fn displays_gluesql_error_message() {
        let error = AppError::GlueSql("table not found: tasks".to_string());

        assert_eq!(error.to_string(), "GlueSQL error: table not found: tasks");
    }

    #[test]
    fn displays_unsupported_message_without_prefix() {
        let error = AppError::Unsupported("SQL is not supported here".to_string());

        assert_eq!(error.to_string(), "SQL is not supported here");
    }

    #[test]
    fn displays_domain_message_without_prefix() {
        assert_eq!(
            AppError::Domain("project has tasks".into()).to_string(),
            "project has tasks"
        );
    }
}
