mod cli;
mod command;
mod error;
mod project;
mod query_lab;
mod repl;
mod repository;
mod service;
mod tag;
mod task;

use command::Command;
use repository::SqlResult;
use service::TaskService;
use task::{Task, TaskStats};

fn main() {
    let command = match cli::parse_args(std::env::args().collect()) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{e}");
            print_help();
            return;
        }
    };
    if command == Command::Help {
        print_help();
        return;
    }
    let repository = match query_lab::persistent_traced("data/rust-task-db") {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{e}");
            return;
        }
    };
    let mut service = TaskService::new(repository);
    if let Err(error) = run(command, &mut service) {
        eprintln!("{error}");
    }
}

fn run<S>(
    command: Command,
    service: &mut TaskService<repository::GlueSqlTaskRepository<S>>,
) -> Result<(), error::AppError>
where
    S: gluesql::core::store::GStore
        + gluesql::core::store::GStoreMut
        + gluesql::core::store::Planner
        + query_lab::runtime::MetricSource,
{
    match command {
        Command::Analyze {
            sql,
            plan,
            runtime,
            raw_plan,
            format,
        } => {
            let execute_runtime = runtime || (!plan && !raw_plan);
            let report = query_lab::analyze(service.repository_mut(), &sql, execute_runtime)?;
            println!(
                "{}",
                query_lab::render_report(&report, format, plan, runtime, raw_plan)?
            );
        }
        Command::LabList => println!("{}", query_lab::scenario_names().join("\n")),
        Command::LabRun { scenario } => {
            for sql in query_lab::scenario_sql(&scenario)? {
                let report = query_lab::analyze(service.repository_mut(), sql, true)?;
                println!(
                    "{}",
                    query_lab::render_report(
                        &report,
                        command::AnalyzeFormat::Tree,
                        true,
                        true,
                        false
                    )?
                );
            }
        }
        Command::LabSeed { profile } => {
            service.repository_mut().seed_lab_profile(&profile)?;
            println!("Query Lab seed ready: {profile}");
        }
        Command::Add { title } => print_task(&service.add(title)?),
        Command::List => print_tasks(&service.list()?),
        Command::Done { id } | Command::TaskDone { id } => {
            service.done(id)?;
            println!("Done: {id}");
        }
        Command::Delete { id } | Command::TaskDelete { id } => print_task(&service.delete(id)?),
        Command::Search { keyword } | Command::TaskSearch { keyword } => {
            print_tasks(&service.search(&keyword)?)
        }
        Command::Stats => print_stats(&service.stats()?),
        Command::ProjectAdd { name } => {
            let p = service.add_project(name)?;
            println!("{} | {}", p.id, p.name);
        }
        Command::ProjectList => {
            for p in service.list_projects()? {
                println!("{} | {}", p.id, p.name);
            }
        }
        Command::ProjectShow { id } => {
            let p = service.show_project(id)?;
            println!("{} | {}", p.id, p.name);
        }
        Command::ProjectDelete { id } => {
            let p = service.delete_project(id)?;
            println!("Deleted: {} | {}", p.id, p.name);
        }
        Command::ProjectStats { id } => {
            let stats = match id {
                Some(id) => vec![service.project_stats(id)?],
                None => service.all_project_stats()?,
            };
            for s in stats {
                println!(
                    "{} | {} | total={} | done={} | todo={} | completion={:.1}%",
                    s.project.id, s.project.name, s.total, s.done, s.todo, s.completion_rate
                );
            }
        }
        Command::TaskAdd {
            project_id,
            priority,
            title,
            tags,
        } => print_task(&service.add_task_with_tags(project_id, priority, title, tags)?),
        Command::TaskList { project_id, tag } => {
            print_tasks(&service.list_tasks(project_id, tag.as_deref())?)
        }
        Command::TaskShow { id } => {
            let d = service.show_task(id)?;
            print_task(&d.task);
            println!("project: {}", d.project_name.as_deref().unwrap_or("(none)"));
            println!(
                "tags: {}",
                d.tags
                    .iter()
                    .map(|t| t.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
        Command::TagAdd { name } => {
            let t = service.add_tag(name)?;
            println!("{} | {}", t.id, t.name);
        }
        Command::TagList => {
            for t in service.list_tags()? {
                println!("{} | {}", t.id, t.name);
            }
        }
        Command::TagDelete { id } => {
            let t = service.delete_tag(id)?;
            println!("Deleted: {} | {}", t.id, t.name);
        }
        Command::TaskTag { id, tag } => {
            service.tag_task(id, &tag)?;
            println!("Tagged: {id} | {tag}");
        }
        Command::TaskUntag { id, tag } => {
            service.untag_task(id, &tag)?;
            println!("Untagged: {id} | {tag}");
        }
        Command::TaskTags { id } => {
            for t in service.task_tags(id)? {
                println!("{} | {}", t.id, t.name);
            }
        }
        Command::Seed => {
            service.seed()?;
            println!("Seed ready: 10 projects, 1000 tasks, 20 tags");
        }
        Command::Sql { sql } => print_sql_results(&service.execute_sql(sql)?),
        Command::Repl => repl::run_repl(service)?,
        Command::Help => unreachable!(),
    }
    Ok(())
}

fn print_task(t: &Task) {
    println!(
        "{} | project={} | priority={} | {} | {}",
        t.id,
        t.project_id
            .map_or_else(|| "NULL".into(), |id| id.to_string()),
        t.priority,
        t.title,
        t.done
    );
}
fn print_tasks(v: &[Task]) {
    for t in v {
        print_task(t)
    }
}
fn print_stats(s: &TaskStats) {
    println!("total: {}\ndone: {}\ntodo: {}", s.total, s.done, s.todo)
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
            SqlResult::Affected { kind, count } => println!("{kind}: {count}"),
            SqlResult::Message(m) => println!("{m}"),
        }
    }
}
fn print_help() {
    println!("rust-task: relational task management CLI\n\nQuery Lab: analyze [options] \"SQL\", lab list, lab run <scenario>\nProject: project add|list|show|delete|stats\nTask: task add|list|show|done|delete|search|tag|untag|tags\nTag: tag add|list|delete\nOther: seed, sql, repl\nLegacy: add, list, done, delete, search, stats");
}
