use crate::error::AppError;
use crate::repository::{SqlResult, TaskRepository};
use crate::service::TaskService;
use std::io::{self, BufRead, Write};

const PROMPT: &str = "rust-task> ";
const SCHEMA: &str = "CREATE TABLE tasks (\n  id INTEGER,\n  title TEXT,\n  done BOOLEAN\n);";

pub fn run_repl<R: TaskRepository>(service: &mut TaskService<R>) -> Result<(), AppError> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    run_repl_with_io(service, stdin.lock(), &mut stdout)
}

fn run_repl_with_io<R, Input, Output>(
    service: &mut TaskService<R>,
    mut input: Input,
    output: &mut Output,
) -> Result<(), AppError>
where
    R: TaskRepository,
    Input: BufRead,
    Output: Write,
{
    writeln!(output, "rust-task SQL REPL")?;
    writeln!(output, "Type .schema, .exit, or .quit")?;

    loop {
        write!(output, "{PROMPT}")?;
        output.flush()?;

        let mut line = String::new();
        let bytes_read = input.read_line(&mut line)?;
        if bytes_read == 0 {
            break;
        }

        let command = line.trim();
        if command.is_empty() {
            continue;
        }

        match command {
            ".exit" | ".quit" => break,
            ".schema" => writeln!(output, "{SCHEMA}")?,
            sql => match service.execute_sql(sql.to_string()) {
                Ok(results) => write_sql_results(output, &results)?,
                Err(message) => writeln!(output, "{message}")?,
            },
        }
    }

    Ok(())
}

fn write_sql_results(output: &mut impl Write, results: &[SqlResult]) -> Result<(), AppError> {
    for result in results {
        match result {
            SqlResult::Select { labels, rows } => {
                writeln!(output, "{}", labels.join(" | "))?;
                for row in rows {
                    writeln!(output, "{}", row.join(" | "))?;
                }
            }
            SqlResult::Affected { kind, count } => {
                writeln!(output, "{kind}: {count}")?;
            }
            SqlResult::Message(message) => writeln!(output, "{message}")?,
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::GlueSqlTaskRepository;
    use std::io::Cursor;

    #[test]
    fn exits_repl_with_exit_command() {
        let repository = GlueSqlTaskRepository::new().unwrap();
        let mut service = TaskService::new(repository);
        let input = Cursor::new(".exit\n");
        let mut output = Vec::new();

        run_repl_with_io(&mut service, input, &mut output).unwrap();

        let output = String::from_utf8(output).unwrap();
        assert!(output.contains("rust-task SQL REPL"));
        assert!(output.contains("rust-task> "));
    }

    #[test]
    fn prints_schema_command() {
        let repository = GlueSqlTaskRepository::new().unwrap();
        let mut service = TaskService::new(repository);
        let input = Cursor::new(".schema\n.quit\n");
        let mut output = Vec::new();

        run_repl_with_io(&mut service, input, &mut output).unwrap();

        let output = String::from_utf8(output).unwrap();
        assert!(output.contains("CREATE TABLE tasks"));
        assert!(output.contains("id INTEGER"));
        assert!(output.contains("done BOOLEAN"));
    }

    #[test]
    fn executes_sql_in_one_repl_session() {
        let repository = GlueSqlTaskRepository::new().unwrap();
        let mut service = TaskService::new(repository);
        let input = Cursor::new(
            "INSERT INTO tasks VALUES (1, 'Rust', FALSE);\nSELECT id, title, done FROM tasks;\n.quit\n",
        );
        let mut output = Vec::new();

        run_repl_with_io(&mut service, input, &mut output).unwrap();

        let output = String::from_utf8(output).unwrap();
        assert!(output.contains("insert: 1"));
        assert!(output.contains("id | title | done"));
        assert!(output.contains("1 | Rust | false"));
    }

    #[test]
    fn skips_empty_lines() {
        let repository = GlueSqlTaskRepository::new().unwrap();
        let mut service = TaskService::new(repository);
        let input = Cursor::new("\n   \n.quit\n");
        let mut output = Vec::new();

        run_repl_with_io(&mut service, input, &mut output).unwrap();

        let output = String::from_utf8(output).unwrap();
        assert!(!output.contains("GlueSQL error"));
    }

    #[test]
    fn continues_after_sql_error() {
        let repository = GlueSqlTaskRepository::new().unwrap();
        let mut service = TaskService::new(repository);
        let input = Cursor::new(
            "SELECT * FROM missing_table;\nINSERT INTO tasks VALUES (1, 'Rust', FALSE);\nSELECT id, title, done FROM tasks;\n.quit\n",
        );
        let mut output = Vec::new();

        run_repl_with_io(&mut service, input, &mut output).unwrap();

        let output = String::from_utf8(output).unwrap();
        assert!(output.contains("GlueSQL error"));
        assert!(output.contains("insert: 1"));
        assert!(output.contains("1 | Rust | false"));
    }
}
