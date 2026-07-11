use crate::error::AppError;
use crate::project::{Project, ProjectStats};
use crate::repository::{SqlResult, TaskManagementRepository, TaskRepository};
use crate::tag::Tag;
use crate::task::{Task, TaskDetail, TaskStats};
use futures::executor::block_on;
use gluesql::core::ast::Statement;
use gluesql::core::store::{GStore, GStoreMut, Planner};
use gluesql::prelude::{Glue, MemoryStorage, Payload, SledStorage, Value};
use std::path::Path;

pub struct GlueSqlTaskRepository<S = MemoryStorage>
where
    S: GStore + GStoreMut + Planner,
{
    glue: Glue<S>,
    transactional: bool,
    in_transaction: bool,
}

#[cfg(test)]
impl GlueSqlTaskRepository<MemoryStorage> {
    pub fn new() -> Result<Self, AppError> {
        let storage = MemoryStorage::default();
        let glue = Glue::new(storage);
        let mut repository = Self {
            glue,
            transactional: false,
            in_transaction: false,
        };

        repository.create_tables()?;

        Ok(repository)
    }
}

impl GlueSqlTaskRepository<SledStorage> {
    #[allow(dead_code)]
    pub fn persistent(path: impl AsRef<Path>) -> Result<Self, AppError> {
        let storage =
            SledStorage::new(path).map_err(|error| AppError::GlueSql(error.to_string()))?;
        let glue = Glue::new(storage);
        let mut repository = Self {
            glue,
            transactional: true,
            in_transaction: false,
        };

        repository.create_tables()?;

        Ok(repository)
    }
}

impl<S> GlueSqlTaskRepository<S>
where
    S: GStore + GStoreMut + Planner,
{
    pub(crate) fn from_storage(storage: S, transactional: bool) -> Result<Self, AppError> {
        let mut repository = Self {
            glue: Glue::new(storage),
            transactional,
            in_transaction: false,
        };
        repository.create_tables()?;
        Ok(repository)
    }

    pub(crate) fn storage(&self) -> &S {
        &self.glue.storage
    }

    pub(crate) fn storage_mut(&mut self) -> &mut S {
        &mut self.glue.storage
    }

    pub(crate) fn seed_lab_profile(&mut self, profile: &str) -> Result<(), AppError> {
        if profile == "small" {
            return self.seed();
        }
        let (projects, tasks, tags, skewed) = match profile {
            "medium" => (100usize, 100_000usize, 100usize, false),
            "large" => (250usize, 250_000usize, 200usize, false),
            "skewed" => (10usize, 10_000usize, 20usize, true),
            _ => {
                return Err(AppError::Domain(format!(
                    "Unknown lab seed profile: {profile}"
                )))
            }
        };
        let key = format!("query_lab_seed_{profile}");
        if metadata_value(self, &key)?.as_deref() == Some("1") {
            return Ok(());
        }
        self.transaction(|repository| {
            let project_start = reserve_ids(repository, "projects", projects)
                .map_err(|error| seed_profile_error(profile, "reserve project IDs", error))?;
            let tag_start = reserve_ids(repository, "tags", tags)
                .map_err(|error| seed_profile_error(profile, "reserve tag IDs", error))?;
            let task_start = reserve_ids(repository, "tasks", tasks)
                .map_err(|error| seed_profile_error(profile, "reserve task IDs", error))?;
            execute_batches(
                repository,
                (0..projects).map(|n| {
                    format!(
                        "INSERT INTO projects VALUES ({}, 'Lab {profile} Project {:04}');",
                        project_start + n as i64,
                        n + 1
                    )
                }),
            )
            .map_err(|error| seed_profile_error(profile, "insert projects", error))?;
            execute_batches(
                repository,
                (0..tags).map(|n| {
                    format!(
                        "INSERT INTO tags VALUES ({}, 'lab-{profile}-tag-{:04}');",
                        tag_start + n as i64,
                        n + 1
                    )
                }),
            )
            .map_err(|error| seed_profile_error(profile, "insert tags", error))?;
            execute_batches(
                repository,
                (0..tasks).map(|n| {
                    let project_offset = if skewed && n < tasks * 8 / 10 {
                        0
                    } else {
                        n % projects
                    };
                    let done = if skewed { n % 10 != 0 } else { n % 3 == 0 };
                    format!(
                        "INSERT INTO tasks (id, project_id, title, done, priority) VALUES ({}, {}, 'Lab {profile} Task {:07}', {}, {});",
                        task_start + n as i64,
                        project_start + project_offset as i64,
                        n + 1,
                        done,
                        (n % 5) + 1
                    )
                }),
            )
            .map_err(|error| seed_profile_error(profile, "insert tasks", error))?;
            execute_batches(
                repository,
                (0..tasks).map(|n| {
                    let tag_offset = if skewed && n < tasks * 8 / 10 {
                        0
                    } else {
                        n % tags
                    };
                    format!(
                        "INSERT INTO task_tags VALUES ({}, {});",
                        task_start + n as i64,
                        tag_start + tag_offset as i64
                    )
                }),
            )
            .map_err(|error| seed_profile_error(profile, "insert task-tag links", error))?;
            set_metadata(repository, &key, "1")
                .map_err(|error| seed_profile_error(profile, "write completion metadata", error))?;
            Ok(())
        })
    }
    fn create_tables(&mut self) -> Result<(), AppError> {
        self.execute(
            "CREATE TABLE IF NOT EXISTS projects (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL
             );
             CREATE TABLE IF NOT EXISTS tasks (
                id INTEGER PRIMARY KEY,
                project_id INTEGER,
                title TEXT NOT NULL,
                done BOOLEAN NOT NULL,
                priority INTEGER NOT NULL,
                FOREIGN KEY (project_id) REFERENCES projects(id)
             );
             CREATE TABLE IF NOT EXISTS tags (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL
             );
             CREATE TABLE IF NOT EXISTS task_tags (
                task_id INTEGER,
                tag_id INTEGER
             );
             CREATE TABLE IF NOT EXISTS id_sequences (
                entity TEXT PRIMARY KEY,
                next_id INTEGER NOT NULL
             );
             CREATE TABLE IF NOT EXISTS app_metadata (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
             );",
        )?;

        // Step 18까지의 3열 tasks 테이블을 그대로 가진 Sled DB도 연다.
        if self
            .execute("SELECT project_id, priority FROM tasks LIMIT 1;")
            .is_err()
        {
            self.execute(
                "ALTER TABLE tasks ADD COLUMN project_id INTEGER;
                 ALTER TABLE tasks ADD COLUMN priority INTEGER DEFAULT 3;",
            )?;
        }

        if self.transactional {
            for sql in [
                "CREATE INDEX idx_tasks_project_id ON tasks(project_id);",
                "CREATE INDEX idx_tasks_done ON tasks(done);",
                "CREATE INDEX idx_task_tags_tag_id ON task_tags(tag_id);",
            ] {
                if let Err(error) = self.execute(sql) {
                    if !error.to_string().contains("index name already exists") {
                        return Err(error);
                    }
                }
            }
        }

        Ok(())
    }

    fn execute(&mut self, sql: impl AsRef<str>) -> Result<Vec<Payload>, AppError> {
        block_on(self.glue.execute(sql)).map_err(|error| AppError::GlueSql(error.to_string()))
    }

    pub(crate) fn plan_sql(&mut self, sql: &str) -> Result<Vec<Statement>, AppError> {
        block_on(self.glue.plan(sql)).map_err(|error| AppError::GlueSql(error.to_string()))
    }

    pub(crate) fn execute_statement(&mut self, statement: &Statement) -> Result<Payload, AppError> {
        block_on(self.glue.execute_stmt(statement))
            .map_err(|error| AppError::GlueSql(error.to_string()))
    }

    fn transaction<T>(
        &mut self,
        operation: impl FnOnce(&mut Self) -> Result<T, AppError>,
    ) -> Result<T, AppError> {
        if !self.transactional || self.in_transaction {
            return operation(self);
        }
        self.execute("BEGIN;")?;
        self.in_transaction = true;
        let result = operation(self);
        self.in_transaction = false;
        match result {
            Ok(value) => {
                self.execute("COMMIT;")?;
                Ok(value)
            }
            Err(error) => {
                let _ = self.execute("ROLLBACK;");
                Err(error)
            }
        }
    }

    fn select_tasks(&mut self, sql: impl AsRef<str>) -> Result<Vec<Task>, AppError> {
        let payloads = self.execute(sql)?;
        let Some(Payload::Select { labels: _, rows }) = payloads.into_iter().last() else {
            return Err(AppError::GlueSql("expected SELECT result".to_string()));
        };

        rows.into_iter().map(row_to_task).collect()
    }
}

impl<S> TaskRepository for GlueSqlTaskRepository<S>
where
    S: GStore + GStoreMut + Planner,
{
    fn add(&mut self, title: String) -> Result<Task, AppError> {
        self.add_task(None, 3, title)
    }

    fn find_all(&mut self) -> Result<Vec<Task>, AppError> {
        self.list_tasks(None, None)
    }

    fn mark_done(&mut self, id: i64) -> Result<(), AppError> {
        ensure_exists(self, id)?;

        self.execute(format!("UPDATE tasks SET done = TRUE WHERE id = {id};"))?;

        Ok(())
    }

    fn delete(&mut self, id: i64) -> Result<Task, AppError> {
        self.transaction(|repository| {
            let task = find_one(repository, id)?;
            repository.execute(format!(
                "DELETE FROM task_tags WHERE task_id = {id}; DELETE FROM tasks WHERE id = {id};"
            ))?;
            Ok(task)
        })
    }

    fn search(&mut self, keyword: &str) -> Result<Vec<Task>, AppError> {
        let pattern = sql_string(&format!("%{keyword}%"));

        self.select_tasks(format!(
            "SELECT id, project_id, title, done, priority FROM tasks WHERE title ILIKE {} ORDER BY id;",
            pattern
        ))
    }

    fn stats(&mut self) -> Result<TaskStats, AppError> {
        let total = select_count(self, "SELECT COUNT(*) FROM tasks;")?;
        let done = select_count(self, "SELECT COUNT(*) FROM tasks WHERE done = TRUE;")?;

        Ok(TaskStats::new(total, done))
    }

    fn execute_sql(&mut self, sql: String) -> Result<Vec<SqlResult>, AppError> {
        let payloads = self.execute(sql)?;

        payloads.into_iter().map(payload_to_sql_result).collect()
    }
}

impl<S> TaskManagementRepository for GlueSqlTaskRepository<S>
where
    S: GStore + GStoreMut + Planner,
{
    fn add_project(&mut self, name: String) -> Result<Project, AppError> {
        let name = required_name(name, "project")?;
        self.transaction(|repository| {
            let id = allocate_id(repository, "projects")?;
            repository.execute(format!(
                "INSERT INTO projects VALUES ({id}, {});",
                sql_string(&name)
            ))?;
            Ok(Project { id, name })
        })
    }

    fn list_projects(&mut self) -> Result<Vec<Project>, AppError> {
        let rows = select_rows(self, "SELECT id, name FROM projects ORDER BY id;")?;
        rows.into_iter().map(row_to_project).collect()
    }

    fn show_project(&mut self, id: i64) -> Result<Project, AppError> {
        select_rows(
            self,
            &format!("SELECT id, name FROM projects WHERE id = {id};"),
        )?
        .into_iter()
        .next()
        .map(row_to_project)
        .transpose()?
        .ok_or_else(|| AppError::Domain(format!("Project not found: {id}")))
    }

    fn delete_project(&mut self, id: i64) -> Result<Project, AppError> {
        self.transaction(|repository| {
            let project = repository.show_project(id)?;
            if select_count(
                repository,
                &format!("SELECT COUNT(*) FROM tasks WHERE project_id = {id};"),
            )? > 0
            {
                return Err(AppError::Domain("project has tasks".to_string()));
            }
            repository.execute(format!("DELETE FROM projects WHERE id = {id};"))?;
            Ok(project)
        })
    }

    fn project_stats(&mut self, id: i64) -> Result<ProjectStats, AppError> {
        let project = self.show_project(id)?;
        let total = select_count(
            self,
            &format!("SELECT COUNT(*) FROM tasks WHERE project_id = {id};"),
        )?;
        let done = select_count(
            self,
            &format!("SELECT COUNT(*) FROM tasks WHERE project_id = {id} AND done = TRUE;"),
        )?;
        Ok(ProjectStats::new(project, total, done))
    }

    fn all_project_stats(&mut self) -> Result<Vec<ProjectStats>, AppError> {
        let rows = select_rows(self, "SELECT p.id, p.name, COUNT(t.id), COUNT(CASE WHEN t.done = TRUE THEN 1 END) FROM projects p LEFT JOIN tasks t ON t.project_id = p.id GROUP BY p.id, p.name ORDER BY p.id;")?;
        rows.into_iter().map(row_to_project_stats).collect()
    }

    fn add_task(
        &mut self,
        project_id: Option<i64>,
        priority: i64,
        title: String,
    ) -> Result<Task, AppError> {
        self.add_task_with_tags(project_id, priority, title, Vec::new())
    }

    fn add_task_with_tags(
        &mut self,
        project_id: Option<i64>,
        priority: i64,
        title: String,
        tags: Vec<String>,
    ) -> Result<Task, AppError> {
        let title = required_name(title, "task title")?;
        if !(1..=5).contains(&priority) {
            return Err(AppError::Domain(
                "priority must be between 1 and 5".to_string(),
            ));
        }
        let unique = tags
            .iter()
            .map(|tag| tag.trim().to_lowercase())
            .collect::<std::collections::BTreeSet<_>>();
        if unique.len() != tags.len() {
            return Err(AppError::Domain("duplicate tag option".to_string()));
        }
        self.transaction(|repository| {
            if let Some(id) = project_id {
                repository.show_project(id)?;
            }
            let resolved = tags
                .iter()
                .map(|name| {
                    find_tag(repository, name)?
                        .ok_or_else(|| AppError::Domain(format!("Tag not found: {name}")))
                })
                .collect::<Result<Vec<_>, _>>()?;
            let id = allocate_id(repository, "tasks")?;
            let task = Task::with_project(id, project_id, title, priority);
            let project = project_id.map_or_else(|| "NULL".to_string(), |id| id.to_string());
            repository.execute(format!(
                "INSERT INTO tasks (id, project_id, title, done, priority) VALUES ({id}, {project}, {}, FALSE, {priority});",
                sql_string(&task.title)
            ))?;
            for tag in resolved {
                repository.execute(format!("INSERT INTO task_tags VALUES ({id}, {});", tag.id))?;
            }
            Ok(task)
        })
    }

    fn list_tasks(
        &mut self,
        project_id: Option<i64>,
        tag: Option<&str>,
    ) -> Result<Vec<Task>, AppError> {
        let sql = if let Some(tag) = tag {
            format!("SELECT t.id, t.project_id, t.title, t.done, t.priority FROM tasks t JOIN task_tags tt ON tt.task_id = t.id JOIN tags tag ON tag.id = tt.tag_id WHERE tag.name ILIKE {} ORDER BY t.priority DESC, t.id ASC;", sql_string(tag))
        } else if let Some(id) = project_id {
            format!("SELECT id, project_id, title, done, priority FROM tasks WHERE project_id = {id} ORDER BY priority DESC, id ASC;")
        } else {
            "SELECT id, project_id, title, done, priority FROM tasks ORDER BY id ASC;".to_string()
        };
        self.select_tasks(sql)
    }

    fn show_task(&mut self, id: i64) -> Result<TaskDetail, AppError> {
        let task = find_one(self, id)?;
        let project_name = match task.project_id {
            Some(project_id) => Some(self.show_project(project_id)?.name),
            None => None,
        };
        let tags = self.task_tags(id)?;
        Ok(TaskDetail {
            task,
            project_name,
            tags,
        })
    }

    fn add_tag(&mut self, name: String) -> Result<Tag, AppError> {
        let name = required_name(name, "tag")?;
        self.transaction(|repository| {
            if find_tag(repository, &name)?.is_some() {
                return Err(AppError::Domain("tag name already exists".to_string()));
            }
            let id = allocate_id(repository, "tags")?;
            repository.execute(format!(
                "INSERT INTO tags VALUES ({id}, {});",
                sql_string(&name)
            ))?;
            Ok(Tag { id, name })
        })
    }

    fn list_tags(&mut self) -> Result<Vec<Tag>, AppError> {
        select_rows(self, "SELECT id, name FROM tags ORDER BY id;")?
            .into_iter()
            .map(row_to_tag)
            .collect()
    }

    fn delete_tag(&mut self, id: i64) -> Result<Tag, AppError> {
        self.transaction(|repository| {
            let tag = select_rows(
                repository,
                &format!("SELECT id, name FROM tags WHERE id = {id};"),
            )?
            .into_iter()
            .next()
            .map(row_to_tag)
            .transpose()?
            .ok_or_else(|| AppError::Domain(format!("Tag not found: {id}")))?;
            repository.execute(format!(
                "DELETE FROM task_tags WHERE tag_id = {id}; DELETE FROM tags WHERE id = {id};"
            ))?;
            Ok(tag)
        })
    }

    fn tag_task(&mut self, task_id: i64, tag_name: &str) -> Result<(), AppError> {
        self.transaction(|repository| {
            ensure_exists(repository, task_id)?;
            let tag = find_tag(repository, tag_name)?
                .ok_or_else(|| AppError::Domain(format!("Tag not found: {tag_name}")))?;
            let count = select_count(
                repository,
                &format!(
                    "SELECT COUNT(*) FROM task_tags WHERE task_id = {task_id} AND tag_id = {};",
                    tag.id
                ),
            )?;
            if count > 0 {
                return Err(AppError::Domain("task already has tag".to_string()));
            }
            repository.execute(format!(
                "INSERT INTO task_tags VALUES ({task_id}, {});",
                tag.id
            ))?;
            Ok(())
        })
    }

    fn untag_task(&mut self, task_id: i64, tag_name: &str) -> Result<(), AppError> {
        ensure_exists(self, task_id)?;
        let tag = find_tag(self, tag_name)?
            .ok_or_else(|| AppError::Domain(format!("Tag not found: {tag_name}")))?;
        self.execute(format!(
            "DELETE FROM task_tags WHERE task_id = {task_id} AND tag_id = {};",
            tag.id
        ))?;
        Ok(())
    }

    fn task_tags(&mut self, task_id: i64) -> Result<Vec<Tag>, AppError> {
        ensure_exists(self, task_id)?;
        let sql = format!("SELECT tag.id, tag.name FROM tags tag JOIN task_tags tt ON tt.tag_id = tag.id WHERE tt.task_id = {task_id} ORDER BY tag.id;");
        select_rows(self, &sql)?
            .into_iter()
            .map(row_to_tag)
            .collect()
    }

    fn seed(&mut self) -> Result<(), AppError> {
        if metadata_value(self, "seed_version")?.as_deref() == Some("1") {
            return Ok(());
        }
        self.transaction(|repository| {
            cleanup_partial_seed(repository)?;
            let mut projects = Vec::new();
            for number in 1..=10 {
                projects.push(repository.add_project(format!("Query Lab {number:02}"))?);
            }
            for number in 1..=20 {
                repository.add_tag(format!("tag-{number:02}"))?;
            }
            for number in 1..=1000 {
                let project_id = projects[(number - 1) % projects.len()].id;
                let task = repository.add_task(
                    Some(project_id),
                    (number as i64 % 5) + 1,
                    format!("Seed task {number:04}"),
                )?;
                if number % 3 == 0 {
                    repository.mark_done(task.id)?;
                }
                repository.tag_task(task.id, &format!("tag-{:02}", (number % 20) + 1))?;
                if number % 4 == 0 {
                    repository.tag_task(task.id, &format!("tag-{:02}", ((number + 7) % 20) + 1))?;
                }
            }
            set_metadata(repository, "seed_version", "1")?;
            Ok(())
        })
    }
}

fn seed_profile_error(profile: &str, stage: &str, error: AppError) -> AppError {
    AppError::GlueSql(format!("lab seed {profile} failed during {stage}: {error}"))
}

fn metadata_value<S>(
    repository: &mut GlueSqlTaskRepository<S>,
    key: &str,
) -> Result<Option<String>, AppError>
where
    S: GStore + GStoreMut + Planner,
{
    let rows = select_rows(
        repository,
        &format!(
            "SELECT value FROM app_metadata WHERE key = {};",
            sql_string(key)
        ),
    )?;
    match rows.first().and_then(|row| row.first()) {
        Some(Value::Str(value)) => Ok(Some(value.clone())),
        None => Ok(None),
        value => Err(AppError::GlueSql(format!(
            "invalid metadata value: {value:?}"
        ))),
    }
}

fn set_metadata<S>(
    repository: &mut GlueSqlTaskRepository<S>,
    key: &str,
    value: &str,
) -> Result<(), AppError>
where
    S: GStore + GStoreMut + Planner,
{
    repository.execute(format!(
        "DELETE FROM app_metadata WHERE key = {}; INSERT INTO app_metadata VALUES ({}, {});",
        sql_string(key),
        sql_string(key),
        sql_string(value)
    ))?;
    Ok(())
}

fn cleanup_partial_seed<S>(repository: &mut GlueSqlTaskRepository<S>) -> Result<(), AppError>
where
    S: GStore + GStoreMut + Planner,
{
    let task_rows = select_rows(
        repository,
        "SELECT id FROM tasks WHERE title ILIKE 'Seed task %';",
    )?;
    for row in task_rows {
        if let [Value::I64(id)] = row.as_slice() {
            repository.execute(format!(
                "DELETE FROM task_tags WHERE task_id = {id}; DELETE FROM tasks WHERE id = {id};"
            ))?;
        }
    }
    let tag_rows = select_rows(repository, "SELECT id FROM tags WHERE name ILIKE 'tag-%';")?;
    for row in tag_rows {
        if let [Value::I64(id)] = row.as_slice() {
            repository.execute(format!(
                "DELETE FROM task_tags WHERE tag_id = {id}; DELETE FROM tags WHERE id = {id};"
            ))?;
        }
    }
    let project_rows = select_rows(
        repository,
        "SELECT id FROM projects WHERE name ILIKE 'Query Lab %';",
    )?;
    for row in project_rows {
        if let [Value::I64(id)] = row.as_slice() {
            if select_count(
                repository,
                &format!("SELECT COUNT(*) FROM tasks WHERE project_id = {id};"),
            )? > 0
            {
                return Err(AppError::Domain(
                    "seed project contains non-seed tasks".to_string(),
                ));
            }
            repository.execute(format!("DELETE FROM projects WHERE id = {id};"))?;
        }
    }
    Ok(())
}

fn find_one<S>(repository: &mut GlueSqlTaskRepository<S>, id: i64) -> Result<Task, AppError>
where
    S: GStore + GStoreMut + Planner,
{
    repository
        .select_tasks(format!(
            "SELECT id, project_id, title, done, priority FROM tasks WHERE id = {id};"
        ))?
        .into_iter()
        .next()
        .ok_or(AppError::NotFound(id))
}

fn ensure_exists<S>(repository: &mut GlueSqlTaskRepository<S>, id: i64) -> Result<(), AppError>
where
    S: GStore + GStoreMut + Planner,
{
    find_one(repository, id).map(|_| ())
}

fn select_count<S>(repository: &mut GlueSqlTaskRepository<S>, sql: &str) -> Result<usize, AppError>
where
    S: GStore + GStoreMut + Planner,
{
    let payloads = repository.execute(sql)?;
    let Some(Payload::Select { labels: _, rows }) = payloads.into_iter().last() else {
        return Err(AppError::GlueSql("expected COUNT result".to_string()));
    };

    let Some(row) = rows.first() else {
        return Ok(0);
    };

    match row.first() {
        Some(Value::I64(value)) => Ok(*value as usize),
        Some(value) => Err(AppError::GlueSql(format!(
            "expected COUNT to return I64, got {value:?}"
        ))),
        None => Err(AppError::GlueSql("COUNT row was empty".to_string())),
    }
}

fn select_rows<S>(
    repository: &mut GlueSqlTaskRepository<S>,
    sql: &str,
) -> Result<Vec<Vec<Value>>, AppError>
where
    S: GStore + GStoreMut + Planner,
{
    let payloads = repository.execute(sql)?;
    match payloads.into_iter().last() {
        Some(Payload::Select { rows, .. }) => Ok(rows),
        _ => Err(AppError::GlueSql("expected SELECT result".to_string())),
    }
}

fn allocate_id<S>(repository: &mut GlueSqlTaskRepository<S>, table: &str) -> Result<i64, AppError>
where
    S: GStore + GStoreMut + Planner,
{
    let max_rows = select_rows(repository, &format!("SELECT MAX(id) FROM {table};"))?;
    let max_next = match max_rows.first().and_then(|row| row.first()) {
        Some(Value::I64(id)) => Ok(*id),
        Some(Value::Null) | None => Ok(0),
        value => Err(AppError::GlueSql(format!(
            "expected MAX(id), got {value:?}"
        ))),
    }? + 1;
    let rows = select_rows(
        repository,
        &format!(
            "SELECT next_id FROM id_sequences WHERE entity = {};",
            sql_string(table)
        ),
    )?;
    let stored = match rows.first().and_then(|row| row.first()) {
        Some(Value::I64(id)) => Some(*id),
        None => None,
        value => {
            return Err(AppError::GlueSql(format!(
                "invalid sequence row: {value:?}"
            )))
        }
    };
    let id = stored.map_or(max_next, |value| value.max(max_next));
    if stored.is_some() {
        repository.execute(format!(
            "UPDATE id_sequences SET next_id = {} WHERE entity = {};",
            id + 1,
            sql_string(table)
        ))?;
    } else {
        repository.execute(format!(
            "INSERT INTO id_sequences VALUES ({}, {});",
            sql_string(table),
            id + 1
        ))?;
    }
    Ok(id)
}

fn reserve_ids<S>(
    repository: &mut GlueSqlTaskRepository<S>,
    table: &str,
    count: usize,
) -> Result<i64, AppError>
where
    S: GStore + GStoreMut + Planner,
{
    let start = allocate_id(repository, table)?;
    repository.execute(format!(
        "UPDATE id_sequences SET next_id = {} WHERE entity = {};",
        start + count as i64,
        sql_string(table)
    ))?;
    Ok(start)
}

fn execute_batches<S>(
    repository: &mut GlueSqlTaskRepository<S>,
    statements: impl IntoIterator<Item = String>,
) -> Result<(), AppError>
where
    S: GStore + GStoreMut + Planner,
{
    let mut batch = String::new();
    let mut count = 0;
    for statement in statements {
        batch.push_str(&statement);
        count += 1;
        if count == 500 {
            repository.execute(&batch)?;
            batch.clear();
            count = 0;
        }
    }
    if !batch.is_empty() {
        repository.execute(batch)?;
    }
    Ok(())
}

fn required_name(value: String, field: &str) -> Result<String, AppError> {
    let value = value.trim().to_string();
    if value.is_empty() {
        Err(AppError::Domain(format!("{field} must not be empty")))
    } else {
        Ok(value)
    }
}

fn row_to_project(row: Vec<Value>) -> Result<Project, AppError> {
    match row.as_slice() {
        [Value::I64(id), Value::Str(name)] => Ok(Project {
            id: *id,
            name: name.clone(),
        }),
        values => Err(AppError::GlueSql(format!(
            "invalid project row: {values:?}"
        ))),
    }
}

fn row_to_project_stats(row: Vec<Value>) -> Result<ProjectStats, AppError> {
    match row.as_slice() {
        [Value::I64(id), Value::Str(name), Value::I64(total), Value::I64(done)] => {
            Ok(ProjectStats::new(
                Project {
                    id: *id,
                    name: name.clone(),
                },
                *total as usize,
                *done as usize,
            ))
        }
        values => Err(AppError::GlueSql(format!(
            "invalid project stats row: {values:?}"
        ))),
    }
}

fn row_to_tag(row: Vec<Value>) -> Result<Tag, AppError> {
    match row.as_slice() {
        [Value::I64(id), Value::Str(name)] => Ok(Tag {
            id: *id,
            name: name.clone(),
        }),
        values => Err(AppError::GlueSql(format!("invalid tag row: {values:?}"))),
    }
}

fn find_tag<S>(
    repository: &mut GlueSqlTaskRepository<S>,
    name: &str,
) -> Result<Option<Tag>, AppError>
where
    S: GStore + GStoreMut + Planner,
{
    select_rows(
        repository,
        &format!(
            "SELECT id, name FROM tags WHERE name ILIKE {};",
            sql_string(name)
        ),
    )?
    .into_iter()
    .next()
    .map(row_to_tag)
    .transpose()
}

fn row_to_task(row: Vec<Value>) -> Result<Task, AppError> {
    match row.as_slice() {
        [Value::I64(id), project_id, Value::Str(title), Value::Bool(done), Value::I64(priority)] => {
            Ok(Task {
                id: *id,
                project_id: match project_id {
                    Value::I64(id) => Some(*id),
                    Value::Null => None,
                    value => {
                        return Err(AppError::GlueSql(format!(
                            "expected project id, got {value:?}"
                        )))
                    }
                },
                title: title.clone(),
                done: *done,
                priority: *priority,
            })
        }
        values => Err(AppError::GlueSql(format!(
            "expected task row [I64, I64/Null, Str, Bool, I64], got {values:?}"
        ))),
    }
}

fn sql_string(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn payload_to_sql_result(payload: Payload) -> Result<SqlResult, AppError> {
    match payload {
        Payload::Select { labels, rows } => Ok(SqlResult::Select {
            labels,
            rows: rows
                .into_iter()
                .map(|row| row.into_iter().map(value_to_string).collect())
                .collect(),
        }),
        Payload::Insert(count) => Ok(SqlResult::Affected {
            kind: "insert".to_string(),
            count,
        }),
        Payload::Update(count) => Ok(SqlResult::Affected {
            kind: "update".to_string(),
            count,
        }),
        Payload::Delete(count) => Ok(SqlResult::Affected {
            kind: "delete".to_string(),
            count,
        }),
        Payload::Create => Ok(SqlResult::Message("create ok".to_string())),
        Payload::DropTable(count) => Ok(SqlResult::Affected {
            kind: "drop table".to_string(),
            count,
        }),
        other => Ok(SqlResult::Message(format!("{other:?}"))),
    }
}

fn value_to_string(value: Value) -> String {
    match value {
        Value::Bool(value) => value.to_string(),
        Value::I8(value) => value.to_string(),
        Value::I16(value) => value.to_string(),
        Value::I32(value) => value.to_string(),
        Value::I64(value) => value.to_string(),
        Value::I128(value) => value.to_string(),
        Value::U8(value) => value.to_string(),
        Value::U16(value) => value.to_string(),
        Value::U32(value) => value.to_string(),
        Value::U64(value) => value.to_string(),
        Value::U128(value) => value.to_string(),
        Value::F32(value) => value.to_string(),
        Value::F64(value) => value.to_string(),
        Value::Str(value) => value,
        Value::Null => "NULL".to_string(),
        other => format!("{other:?}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn adds_and_lists_tasks_with_gluesql() {
        let mut repository = GlueSqlTaskRepository::new().unwrap();

        let task = repository.add("Rust".to_string()).unwrap();

        assert_eq!(task, Task::new(1, "Rust".to_string()));
        assert_eq!(repository.find_all(), Ok(vec![task]));
    }

    #[test]
    fn inserts_task_after_migrating_step_18_column_order() {
        let mut legacy = Glue::new(MemoryStorage::default());
        block_on(legacy.execute(
            "CREATE TABLE tasks (
                id INTEGER PRIMARY KEY,
                title TEXT NOT NULL,
                done BOOLEAN NOT NULL
            );
            ALTER TABLE tasks ADD COLUMN project_id INTEGER;
            ALTER TABLE tasks ADD COLUMN priority INTEGER DEFAULT 3;
            CREATE TABLE projects (id INTEGER PRIMARY KEY, name TEXT NOT NULL);
            CREATE TABLE tags (id INTEGER PRIMARY KEY, name TEXT NOT NULL);
            CREATE TABLE task_tags (task_id INTEGER, tag_id INTEGER);
            CREATE TABLE id_sequences (entity TEXT PRIMARY KEY, next_id INTEGER NOT NULL);
            CREATE TABLE app_metadata (key TEXT PRIMARY KEY, value TEXT NOT NULL);",
        ))
        .unwrap();
        let mut repository = GlueSqlTaskRepository {
            glue: legacy,
            transactional: false,
            in_transaction: false,
        };
        let project = repository.add_project("Migrated".into()).unwrap();
        let task = repository
            .add_task(Some(project.id), 4, "column-safe insert".into())
            .unwrap();

        assert_eq!(task.project_id, Some(project.id));
        assert_eq!(task.title, "column-safe insert");
        assert_eq!(task.priority, 4);
    }

    #[test]
    fn marks_task_done_with_gluesql() {
        let mut repository = GlueSqlTaskRepository::new().unwrap();
        repository.add("Rust".to_string()).unwrap();

        repository.mark_done(1).unwrap();
        let tasks = repository.find_all().unwrap();

        assert_eq!(tasks[0].done, true);
    }

    #[test]
    fn deletes_task_with_gluesql() {
        let mut repository = GlueSqlTaskRepository::new().unwrap();
        repository.add("Rust".to_string()).unwrap();

        let deleted = repository.delete(1).unwrap();

        assert_eq!(deleted, Task::new(1, "Rust".to_string()));
        assert_eq!(repository.find_all(), Ok(Vec::new()));
    }

    #[test]
    fn searches_tasks_case_insensitively_with_gluesql() {
        let mut repository = GlueSqlTaskRepository::new().unwrap();
        let rust = repository.add("Rust 공부".to_string()).unwrap();
        repository.add("GlueSQL 붙이기".to_string()).unwrap();

        let result = repository.search("rust");

        assert_eq!(result, Ok(vec![rust]));
    }

    #[test]
    fn calculates_stats_with_gluesql_count() {
        let mut repository = GlueSqlTaskRepository::new().unwrap();
        repository.add("Rust 공부".to_string()).unwrap();
        repository.add("GlueSQL 붙이기".to_string()).unwrap();
        repository.mark_done(1).unwrap();

        assert_eq!(repository.stats(), Ok(TaskStats::new(2, 1)));
    }

    #[test]
    fn calculates_empty_stats_with_gluesql_count() {
        let mut repository = GlueSqlTaskRepository::new().unwrap();

        assert_eq!(repository.stats(), Ok(TaskStats::new(0, 0)));
    }

    #[test]
    fn manages_projects_and_rejects_delete_when_tasks_exist() {
        let mut repository = GlueSqlTaskRepository::new().unwrap();
        let project = repository.add_project("GlueSQL 분석".into()).unwrap();
        assert_eq!(repository.show_project(project.id), Ok(project.clone()));
        repository
            .add_task(Some(project.id), 3, "Planner".into())
            .unwrap();
        assert_eq!(
            repository.delete_project(project.id),
            Err(AppError::Domain("project has tasks".into()))
        );
    }

    #[test]
    fn validates_project_and_task_fields() {
        let mut repository = GlueSqlTaskRepository::new().unwrap();
        assert!(repository.add_project("   ".into()).is_err());
        assert!(repository.add_task(None, 0, "bad".into()).is_err());
        assert!(repository.add_task(Some(404), 3, "bad".into()).is_err());
    }

    #[test]
    fn lists_project_tasks_by_priority_then_id_and_calculates_stats() {
        let mut repository = GlueSqlTaskRepository::new().unwrap();
        let project = repository.add_project("Lab".into()).unwrap();
        repository
            .add_task(Some(project.id), 1, "low".into())
            .unwrap();
        let high = repository
            .add_task(Some(project.id), 5, "high".into())
            .unwrap();
        repository.mark_done(high.id).unwrap();
        let tasks = repository.list_tasks(Some(project.id), None).unwrap();
        assert_eq!(
            tasks.iter().map(|t| t.priority).collect::<Vec<_>>(),
            vec![5, 1]
        );
        assert_eq!(
            repository.project_stats(project.id),
            Ok(ProjectStats::new(project, 2, 1))
        );
    }

    #[test]
    fn aggregates_all_projects_with_left_join() {
        let mut repository = GlueSqlTaskRepository::new().unwrap();
        let empty = repository.add_project("empty".into()).unwrap();
        let active = repository.add_project("active".into()).unwrap();
        let task = repository
            .add_task(Some(active.id), 3, "done".into())
            .unwrap();
        repository.mark_done(task.id).unwrap();
        let stats = repository.all_project_stats().unwrap();
        assert_eq!(
            stats,
            vec![
                ProjectStats::new(empty, 0, 0),
                ProjectStats::new(active, 1, 1)
            ]
        );
    }

    #[test]
    fn creates_task_and_tags_together() {
        let mut repository = GlueSqlTaskRepository::new().unwrap();
        repository.add_tag("backend".into()).unwrap();
        repository.add_tag("sql".into()).unwrap();
        let task = repository
            .add_task_with_tags(
                None,
                4,
                "Planner".into(),
                vec!["backend".into(), "sql".into()],
            )
            .unwrap();
        assert_eq!(repository.task_tags(task.id).unwrap().len(), 2);
    }

    #[test]
    fn prevents_case_insensitive_duplicate_tags() {
        let mut repository = GlueSqlTaskRepository::new().unwrap();
        repository.add_tag("Backend".into()).unwrap();
        assert_eq!(
            repository.add_tag("backend".into()),
            Err(AppError::Domain("tag name already exists".into()))
        );
    }

    #[test]
    fn tags_untags_and_lists_tasks_by_tag() {
        let mut repository = GlueSqlTaskRepository::new().unwrap();
        let task = repository.add("API".into()).unwrap();
        repository.add_tag("backend".into()).unwrap();
        repository.tag_task(task.id, "BACKEND").unwrap();
        assert_eq!(
            repository.tag_task(task.id, "backend"),
            Err(AppError::Domain("task already has tag".into()))
        );
        assert_eq!(
            repository.list_tasks(None, Some("backend")).unwrap(),
            vec![task.clone()]
        );
        assert_eq!(repository.show_task(task.id).unwrap().tags.len(), 1);
        repository.untag_task(task.id, "backend").unwrap();
        assert!(repository.task_tags(task.id).unwrap().is_empty());
    }

    #[test]
    fn deleting_task_or_tag_cleans_join_rows() {
        let mut repository = GlueSqlTaskRepository::new().unwrap();
        let first = repository.add("one".into()).unwrap();
        let second = repository.add("two".into()).unwrap();
        let tag = repository.add_tag("backend".into()).unwrap();
        repository.tag_task(first.id, "backend").unwrap();
        repository.tag_task(second.id, "backend").unwrap();
        repository.delete(first.id).unwrap();
        assert_eq!(
            select_count(
                &mut repository,
                "SELECT COUNT(*) FROM task_tags WHERE task_id = 1;"
            )
            .unwrap(),
            0
        );
        repository.delete_tag(tag.id).unwrap();
        assert_eq!(
            select_count(&mut repository, "SELECT COUNT(*) FROM task_tags;").unwrap(),
            0
        );
    }

    #[test]
    fn deletes_empty_project() {
        let mut repository = GlueSqlTaskRepository::new().unwrap();
        let project = repository.add_project("empty".into()).unwrap();
        assert_eq!(repository.delete_project(project.id), Ok(project));
        assert!(repository.list_projects().unwrap().is_empty());
    }

    #[test]
    fn task_detail_includes_project_and_tags() {
        let mut repository = GlueSqlTaskRepository::new().unwrap();
        let project = repository.add_project("Lab".into()).unwrap();
        let task = repository
            .add_task(Some(project.id), 4, "JOIN".into())
            .unwrap();
        repository.add_tag("sql".into()).unwrap();
        repository.tag_task(task.id, "sql").unwrap();
        let detail = repository.show_task(task.id).unwrap();
        assert_eq!(detail.project_name.as_deref(), Some("Lab"));
        assert_eq!(detail.tags[0].name, "sql");
    }

    #[test]
    fn untag_requires_existing_task_and_tag() {
        let mut repository = GlueSqlTaskRepository::new().unwrap();
        assert_eq!(
            repository.untag_task(404, "missing"),
            Err(AppError::NotFound(404))
        );
    }

    #[test]
    fn tag_names_are_trimmed() {
        let mut repository = GlueSqlTaskRepository::new().unwrap();
        assert_eq!(
            repository.add_tag(" backend ".into()).unwrap().name,
            "backend"
        );
    }

    #[test]
    fn seed_creates_expected_counts_and_is_idempotent() {
        let mut repository = GlueSqlTaskRepository::new().unwrap();
        repository.seed().unwrap();
        repository.seed().unwrap();
        assert_eq!(repository.list_projects().unwrap().len(), 10);
        assert_eq!(repository.find_all().unwrap().len(), 1000);
        assert_eq!(repository.list_tags().unwrap().len(), 20);
        assert!(select_count(&mut repository, "SELECT COUNT(*) FROM task_tags;").unwrap() > 1000);
        assert_eq!(
            metadata_value(&mut repository, "seed_version")
                .unwrap()
                .as_deref(),
            Some("1")
        );
    }

    #[test]
    fn seed_recovers_reserved_partial_data_before_rebuilding() {
        let mut repository = GlueSqlTaskRepository::new().unwrap();
        let project = repository.add_project("Query Lab 01".into()).unwrap();
        repository.add_tag("tag-01".into()).unwrap();
        repository
            .add_task(Some(project.id), 1, "Seed task 0001".into())
            .unwrap();
        repository.seed().unwrap();
        assert_eq!(repository.list_projects().unwrap().len(), 10);
        assert_eq!(repository.find_all().unwrap().len(), 1000);
        assert_eq!(repository.list_tags().unwrap().len(), 20);
    }

    #[test]
    fn task_add_rejects_duplicate_tag_options() {
        let mut repository = GlueSqlTaskRepository::new().unwrap();
        repository.add_tag("backend".into()).unwrap();
        let result = repository.add_task_with_tags(
            None,
            3,
            "task".into(),
            vec!["backend".into(), "BACKEND".into()],
        );
        assert_eq!(result, Err(AppError::Domain("duplicate tag option".into())));
        assert!(repository.find_all().unwrap().is_empty());
    }

    #[test]
    fn returns_not_found_when_gluesql_task_is_missing() {
        let mut repository = GlueSqlTaskRepository::new().unwrap();

        assert_eq!(repository.delete(404), Err(AppError::NotFound(404)));
        assert_eq!(repository.mark_done(404), Err(AppError::NotFound(404)));
    }

    #[test]
    fn escapes_single_quote_in_title() {
        let mut repository = GlueSqlTaskRepository::new().unwrap();
        let task = repository.add("Rust's ownership".to_string()).unwrap();

        assert_eq!(repository.find_all(), Ok(vec![task]));
    }

    #[test]
    fn executes_select_sql_with_gluesql() {
        let mut repository = GlueSqlTaskRepository::new().unwrap();
        repository.add("Rust".to_string()).unwrap();

        let result = repository
            .execute_sql("SELECT id, title, done FROM tasks;".to_string())
            .unwrap();

        assert_eq!(
            result,
            vec![SqlResult::Select {
                labels: vec!["id".to_string(), "title".to_string(), "done".to_string()],
                rows: vec![vec![
                    "1".to_string(),
                    "Rust".to_string(),
                    "false".to_string()
                ]]
            }]
        );
    }

    #[test]
    fn executes_insert_update_delete_sql_with_gluesql() {
        let mut repository = GlueSqlTaskRepository::new().unwrap();

        let insert = repository
            .execute_sql("INSERT INTO tasks VALUES (1, NULL, 'Rust', FALSE, 3);".to_string())
            .unwrap();
        let update = repository
            .execute_sql("UPDATE tasks SET done = TRUE WHERE id = 1;".to_string())
            .unwrap();
        let delete = repository
            .execute_sql("DELETE FROM tasks WHERE id = 1;".to_string())
            .unwrap();

        assert_eq!(
            insert,
            vec![SqlResult::Affected {
                kind: "insert".to_string(),
                count: 1
            }]
        );
        assert_eq!(
            update,
            vec![SqlResult::Affected {
                kind: "update".to_string(),
                count: 1
            }]
        );
        assert_eq!(
            delete,
            vec![SqlResult::Affected {
                kind: "delete".to_string(),
                count: 1
            }]
        );
    }

    #[test]
    fn sequence_does_not_reuse_id_after_delete() {
        let mut repository = GlueSqlTaskRepository::new().unwrap();
        repository.add("first".to_string()).unwrap();
        repository.add("second".to_string()).unwrap();
        repository.delete(2).unwrap();

        let task = repository.add("third".to_string()).unwrap();

        assert_eq!(task, Task::new(3, "third".to_string()));
    }

    #[test]
    fn invalid_sql_returns_gluesql_error() {
        let mut repository = GlueSqlTaskRepository::new().unwrap();

        let result = repository.execute_sql("SELECT * FROM missing_table;".to_string());

        assert!(matches!(result, Err(AppError::GlueSql(_))));
    }

    #[test]
    fn persists_tasks_with_sled_storage() {
        let path = unique_sled_path("persist");
        let _ = fs::remove_dir_all(&path);

        {
            let mut repository = GlueSqlTaskRepository::persistent(&path).unwrap();
            repository.add("Rust".to_string()).unwrap();
        }

        {
            let mut repository = GlueSqlTaskRepository::persistent(&path).unwrap();

            assert_eq!(
                repository.find_all(),
                Ok(vec![Task::new(1, "Rust".to_string())])
            );
        }

        let _ = fs::remove_dir_all(&path);
    }

    #[test]
    fn memory_storage_rejects_explicit_transactions() {
        let mut repository = GlueSqlTaskRepository::new().unwrap();

        let result = repository.execute_sql("BEGIN;".to_string());

        assert!(
            matches!(result, Err(AppError::GlueSql(message)) if message.contains("transaction is not supported"))
        );
    }

    #[test]
    fn sled_storage_rolls_back_uncommitted_insert() {
        let path = unique_sled_path("rollback");
        let _ = fs::remove_dir_all(&path);
        let mut repository = GlueSqlTaskRepository::persistent(&path).unwrap();

        repository
            .execute_sql(
                "
                BEGIN;
                INSERT INTO tasks VALUES (1, NULL, 'temporary', FALSE, 3);
                ROLLBACK;
                "
                .to_string(),
            )
            .unwrap();

        assert_eq!(repository.find_all(), Ok(Vec::new()));

        let _ = fs::remove_dir_all(&path);
    }

    #[test]
    fn sled_transaction_rolls_back_sequence_allocation() {
        let path = unique_sled_path("sequence-rollback");
        let _ = fs::remove_dir_all(&path);
        let mut repository = GlueSqlTaskRepository::persistent(&path).unwrap();
        let result: Result<(), AppError> = repository.transaction(|repository| {
            assert_eq!(allocate_id(repository, "tasks")?, 1);
            Err(AppError::Domain("force rollback".into()))
        });
        assert_eq!(result, Err(AppError::Domain("force rollback".into())));
        assert_eq!(repository.add("after rollback".into()).unwrap().id, 1);
        let _ = fs::remove_dir_all(&path);
    }

    #[test]
    fn sled_storage_commits_explicit_transaction() {
        let path = unique_sled_path("commit");
        let _ = fs::remove_dir_all(&path);
        let mut repository = GlueSqlTaskRepository::persistent(&path).unwrap();

        repository
            .execute_sql(
                "
                BEGIN;
                INSERT INTO tasks VALUES (1, NULL, 'committed', FALSE, 3);
                COMMIT;
                "
                .to_string(),
            )
            .unwrap();

        assert_eq!(
            repository.find_all(),
            Ok(vec![Task::new(1, "committed".to_string())])
        );

        let _ = fs::remove_dir_all(&path);
    }

    #[test]
    fn sled_storage_rejects_nested_transaction() {
        let path = unique_sled_path("nested-transaction");
        let _ = fs::remove_dir_all(&path);
        let mut repository = GlueSqlTaskRepository::persistent(&path).unwrap();

        repository.execute_sql("BEGIN;".to_string()).unwrap();
        let result = repository.execute_sql("BEGIN;".to_string());

        assert!(
            matches!(result, Err(AppError::GlueSql(message)) if message.contains("nested transaction is not supported"))
        );

        repository.execute_sql("ROLLBACK;".to_string()).unwrap();
        let _ = fs::remove_dir_all(&path);
    }

    #[test]
    fn sled_storage_keeps_repeatable_read_snapshot_until_commit() {
        let path = unique_sled_path("snapshot");
        let _ = fs::remove_dir_all(&path);
        let (mut writer, mut reader) = sled_repository_pair(&path);

        writer.add("before".to_string()).unwrap();

        reader.execute_sql("BEGIN;".to_string()).unwrap();
        writer
            .execute_sql(
                "
                BEGIN;
                INSERT INTO tasks VALUES (2, NULL, 'after', FALSE, 3);
                COMMIT;
                "
                .to_string(),
            )
            .unwrap();

        assert_eq!(
            reader.find_all(),
            Ok(vec![Task::new(1, "before".to_string())])
        );

        reader.execute_sql("COMMIT;".to_string()).unwrap();

        assert_eq!(
            reader.find_all(),
            Ok(vec![
                Task::new(1, "before".to_string()),
                Task::new(2, "after".to_string())
            ])
        );

        let _ = fs::remove_dir_all(&path);
    }

    #[test]
    fn sled_storage_reports_database_locked_for_competing_writes() {
        let path = unique_sled_path("write-lock");
        let _ = fs::remove_dir_all(&path);
        let (mut first, mut second) = sled_repository_pair(&path);

        first
            .execute_sql(
                "
                BEGIN;
                INSERT INTO tasks VALUES (1, NULL, 'first writer', FALSE, 3);
                "
                .to_string(),
            )
            .unwrap();

        let result = second.execute_sql(
            "INSERT INTO tasks VALUES (2, NULL, 'second writer', FALSE, 3);".to_string(),
        );

        assert!(
            matches!(result, Err(AppError::GlueSql(message)) if message.contains("database is locked"))
        );

        first.execute_sql("ROLLBACK;".to_string()).unwrap();
        let _ = fs::remove_dir_all(&path);
    }

    fn sled_repository_pair(
        path: impl AsRef<std::path::Path>,
    ) -> (
        GlueSqlTaskRepository<SledStorage>,
        GlueSqlTaskRepository<SledStorage>,
    ) {
        let storage = SledStorage::new(path).unwrap();
        let first_storage = storage.clone();
        let second_storage = storage;
        let mut first = GlueSqlTaskRepository {
            glue: Glue::new(first_storage),
            transactional: true,
            in_transaction: false,
        };
        let mut second = GlueSqlTaskRepository {
            glue: Glue::new(second_storage),
            transactional: true,
            in_transaction: false,
        };

        first.create_tables().unwrap();
        second.create_tables().unwrap();

        (first, second)
    }

    fn unique_sled_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("rust-task-db-{name}-{}", std::process::id()))
    }
}
