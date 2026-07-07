mod cli;
mod command;
mod error;
mod repl;
mod repository;
mod service;
mod task;

use command::Command;
use repository::{GlueSqlTaskRepository, SqlResult};
use service::TaskService;
use task::{Task, TaskStats};

fn main() {
    let command = match cli::parse_args(std::env::args().collect()) {
        Ok(command) => command,
        Err(message) => {
            eprintln!("{message}");
            print_help();
            return;
        }
    };

    if command == Command::Help {
        print_help();
        return;
    }

    let repository = match GlueSqlTaskRepository::persistent("data/rust-task-db") {
        Ok(repository) => repository,
        Err(message) => {
            eprintln!("{message}");
            return;
        }
    };
    let mut service = TaskService::new(repository);

    match command {
        Command::Add { title } => match service.add(title) {
            Ok(task) => {
                println!("Added:");
                print_task(&task);
            }
            Err(message) => eprintln!("{message}"),
        },
        Command::List => match service.list() {
            Ok(tasks) => {
                println!("List:");
                print_tasks(&tasks);
            }
            Err(message) => eprintln!("{message}"),
        },
        Command::Done { id } => match service.done(id) {
            Ok(()) => {
                println!("Done:\n{id}");
            }
            Err(message) => eprintln!("{message}"),
        },
        Command::Delete { id } => match service.delete(id) {
            Ok(task) => {
                println!("Deleted:");
                print_task(&task);
            }
            Err(message) => eprintln!("{message}"),
        },
        Command::Search { keyword } => match service.search(&keyword) {
            Ok(tasks) => {
                println!("Search:");
                print_tasks(&tasks);
            }
            Err(message) => eprintln!("{message}"),
        },
        Command::Stats => match service.stats() {
            Ok(stats) => {
                println!("Stats:");
                print_stats(&stats);
            }
            Err(message) => eprintln!("{message}"),
        },
        Command::Sql { sql } => match service.execute_sql(sql) {
            Ok(results) => {
                println!("SQL:");
                print_sql_results(&results);
            }
            Err(message) => eprintln!("{message}"),
        },
        Command::Repl => {
            if let Err(message) = repl::run_repl(&mut service) {
                eprintln!("{message}");
            }
        }
        Command::Help => unreachable!("help command returns before loading tasks"),
    }
}

fn print_task(task: &Task) {
    println!("{} | {} | {}", task.id, task.title, task.done);
}

fn print_tasks(tasks: &[Task]) {
    for task in tasks {
        print_task(task);
    }
}

fn print_stats(stats: &TaskStats) {
    println!("total: {}", stats.total);
    println!("done: {}", stats.done);
    println!("todo: {}", stats.todo);
}

fn print_sql_results(results: &[SqlResult]) {
    for result in results {
        match result {
            SqlResult::Select { labels, rows } => {
                println!("{}", labels.join(" | "));
                for row in rows {
                    println!("{}", row.join(" | "));
                }
            }
            SqlResult::Affected { kind, count } => {
                println!("{kind}: {count}");
            }
            SqlResult::Message(message) => println!("{message}"),
        }
    }
}

fn print_help() {
    println!("rust-task Step 12: Persistent GlueSQL Todo CLI");
    println!();
    println!("Usage:");
    println!("  rust-task add \"Rust 공부\"");
    println!("  rust-task list");
    println!("  rust-task done 1");
    println!("  rust-task delete 1");
    println!("  rust-task search rust");
    println!("  rust-task stats");
    println!("  rust-task sql \"SELECT * FROM tasks\"");
    println!("  rust-task repl");
    println!();
    println!("Note: Step 12 stores GlueSQL data under data/rust-task-db.");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prints_help_before_repository_is_loaded() {
        let command = Command::Help;

        assert_eq!(command, Command::Help);
    }
}
