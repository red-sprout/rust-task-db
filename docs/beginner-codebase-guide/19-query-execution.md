# Query Execution 상세 분석

## 이 문서의 목적

이 문서는 현재 Todo 명령이 `src/repository/gluesql_repository.rs` 안에서 어떤 SQL로 바뀌고, GlueSQL `Payload`가 어떤 프로젝트 타입으로 변환되는지 설명한다.

중요한 범위 제한:

- Step 17에서는 새 CLI 명령을 추가하지 않는다.
- 새 외부 crate를 추가하지 않는다.
- GlueSQL Parser, Planner, Executor를 직접 호출하지 않는다.
- 현재 코드는 `Glue::execute` public API를 통해 query execution 결과를 관찰한다.

## 전체 Query Execution 흐름

```text
터미널 입력
-> src/cli.rs parse_args
-> src/command.rs Command
-> src/main.rs match command
-> src/service/mod.rs TaskService
-> src/repository/mod.rs TaskRepository
-> src/repository/gluesql_repository.rs GlueSqlTaskRepository
-> execute(sql)
-> Glue::execute(sql)
-> Payload
-> Task / TaskStats / SqlResult
-> src/main.rs 출력 함수
```

현재 프로젝트에서 GlueSQL 호출이 실제로 모이는 함수는 `execute`다.

```rust
fn execute(&mut self, sql: impl AsRef<str>) -> Result<Vec<Payload>, AppError> {
    block_on(self.glue.execute(sql)).map_err(|error| AppError::GlueSql(error.to_string()))
}
```

코드 해석:

- `self.glue.execute(sql)`: GlueSQL에 SQL 문자열을 넘긴다.
- `block_on`: async API를 동기 CLI 안에서 기다린다.
- 성공하면 `Vec<Payload>`를 받는다.
- 실패하면 GlueSQL 에러 문자열을 `AppError::GlueSql`로 감싼다.

## Todo 명령별 SQL

| Todo 명령 | service 메서드 | repository 메서드 | 실제 SQL |
| --- | --- | --- | --- |
| `add` | `TaskService::add` | `GlueSqlTaskRepository::add` | `SELECT id, title, done FROM tasks ORDER BY id;` 후 `INSERT INTO tasks VALUES (...)` |
| `list` | `TaskService::list` | `find_all` | `SELECT id, title, done FROM tasks ORDER BY id;` |
| `done` | `TaskService::done` | `mark_done` | `SELECT id, title, done FROM tasks WHERE id = ...;` 후 `UPDATE tasks SET done = TRUE WHERE id = ...;` |
| `delete` | `TaskService::delete` | `delete` | `SELECT id, title, done FROM tasks WHERE id = ...;` 후 `DELETE FROM tasks WHERE id = ...;` |
| `search` | `TaskService::search` | `search` | `SELECT id, title, done FROM tasks WHERE title ILIKE ... ORDER BY id;` |
| `stats` | `TaskService::stats` | `stats` | `SELECT COUNT(*) FROM tasks;`와 `SELECT COUNT(*) FROM tasks WHERE done = TRUE;` |
| `sql` | `TaskService::execute_sql` | `execute_sql` | 사용자가 입력한 SQL |
| `repl` | `run_repl` | `execute_sql` | REPL 한 줄에 입력한 SQL |

## add 명령 실행

`add`는 바로 `INSERT`만 하지 않는다. 먼저 기존 Todo 목록을 읽어 다음 id를 계산한다.

```rust
fn add(&mut self, title: String) -> Result<Task, AppError> {
    let id = next_id(&self.find_all()?);
    let task = Task::new(id, title);
    let title = sql_string(&task.title);

    self.execute(format!(
        "INSERT INTO tasks VALUES ({}, {}, {});",
        task.id, title, task.done
    ))?;

    Ok(task)
}
```

실행 순서:

```text
add("Rust 공부")
-> find_all()
-> SELECT id, title, done FROM tasks ORDER BY id;
-> next_id(&tasks)
-> Task::new(id, title)
-> sql_string(&task.title)
-> INSERT INTO tasks VALUES (...)
-> Ok(task)
```

초심자가 볼 포인트:

- id 생성은 database auto increment가 아니라 `next_id` 함수가 담당한다.
- 제목은 `sql_string`을 거쳐 작은따옴표를 escape한다.
- `INSERT`의 GlueSQL 결과는 버리고, 앱이 만든 `Task`를 반환한다.

## SELECT 결과가 Task로 바뀌는 흐름

Todo 목록을 반환하는 query는 `select_tasks`를 지난다.

```rust
fn select_tasks(&mut self, sql: impl AsRef<str>) -> Result<Vec<Task>, AppError> {
    let payloads = self.execute(sql)?;
    let Some(Payload::Select { labels: _, rows }) = payloads.into_iter().last() else {
        return Err(AppError::GlueSql("expected SELECT result".to_string()));
    };

    rows.into_iter().map(row_to_task).collect()
}
```

`row_to_task`는 GlueSQL row를 Todo domain model로 바꾼다.

```rust
fn row_to_task(row: Vec<Value>) -> Result<Task, AppError> {
    match row.as_slice() {
        [Value::I64(id), Value::Str(title), Value::Bool(done)] => Ok(Task {
            id: *id,
            title: title.clone(),
            done: *done,
        }),
        values => Err(AppError::GlueSql(format!(
            "expected task row [I64, Str, Bool], got {values:?}"
        ))),
    }
}
```

코드 해석:

- `Payload::Select`가 아니면 `expected SELECT result` 에러가 된다.
- row 모양은 `[I64, Str, Bool]`이어야 한다.
- SQL column 순서가 `id, title, done`이 아니면 변환 실패 가능성이 있다.
- 그래서 `find_all`, `find_one`, `search`는 모두 `SELECT id, title, done ...` 순서를 유지한다.

## done과 delete의 존재 확인 흐름

`done`과 `delete`는 없는 id를 조용히 무시하지 않는다. 먼저 `find_one`을 실행한다.

```rust
fn find_one<S>(repository: &mut GlueSqlTaskRepository<S>, id: i64) -> Result<Task, AppError>
where
    S: GStore + GStoreMut + Planner,
{
    repository
        .select_tasks(format!(
            "SELECT id, title, done FROM tasks WHERE id = {id};"
        ))?
        .into_iter()
        .next()
        .ok_or(AppError::NotFound(id))
}
```

`mark_done`은 `ensure_exists`로 존재를 확인한 뒤 update한다.

```rust
fn mark_done(&mut self, id: i64) -> Result<(), AppError> {
    ensure_exists(self, id)?;

    self.execute(format!("UPDATE tasks SET done = TRUE WHERE id = {id};"))?;

    Ok(())
}
```

`delete`는 삭제 전에 찾은 `Task`를 반환값으로 보관한다.

```rust
fn delete(&mut self, id: i64) -> Result<Task, AppError> {
    let task = find_one(self, id)?;

    self.execute(format!("DELETE FROM tasks WHERE id = {id};"))?;

    Ok(task)
}
```

## stats와 COUNT 결과 변환

`stats`는 두 번의 `COUNT` query를 실행한다.

```rust
fn stats(&mut self) -> Result<TaskStats, AppError> {
    let total = select_count(self, "SELECT COUNT(*) FROM tasks;")?;
    let done = select_count(self, "SELECT COUNT(*) FROM tasks WHERE done = TRUE;")?;

    Ok(TaskStats::new(total, done))
}
```

`select_count`는 첫 번째 row의 첫 번째 값이 `Value::I64`인지 확인한다.

```rust
match row.first() {
    Some(Value::I64(value)) => Ok(*value as usize),
    Some(value) => Err(AppError::GlueSql(format!(
        "expected COUNT to return I64, got {value:?}"
    ))),
    None => Err(AppError::GlueSql("COUNT row was empty".to_string())),
}
```

초심자가 볼 포인트:

- `TaskStats` 계산은 Rust iterator가 아니라 GlueSQL `COUNT` 결과를 사용한다.
- COUNT 결과가 비어 있으면 `0`으로 처리하는 경로가 있다.
- COUNT 값의 타입이 다르면 `AppError::GlueSql`이 된다.

## SQL 명령과 REPL의 Payload 변환

사용자 SQL은 Todo domain model로 바꾸지 않는다. 대신 CLI 출력용 `SqlResult`로 바꾼다.

```rust
fn execute_sql(&mut self, sql: String) -> Result<Vec<SqlResult>, AppError> {
    let payloads = self.execute(sql)?;

    payloads.into_iter().map(payload_to_sql_result).collect()
}
```

`payload_to_sql_result`의 핵심 매핑:

| GlueSQL `Payload` | 프로젝트 `SqlResult` | 출력 의미 |
| --- | --- | --- |
| `Payload::Select { labels, rows }` | `SqlResult::Select { labels, rows }` | table 형태 출력 |
| `Payload::Insert(count)` | `SqlResult::Affected { kind: "insert", count }` | 변경 건수 출력 |
| `Payload::Update(count)` | `SqlResult::Affected { kind: "update", count }` | 변경 건수 출력 |
| `Payload::Delete(count)` | `SqlResult::Affected { kind: "delete", count }` | 변경 건수 출력 |
| `Payload::Create` | `SqlResult::Message("create ok")` | 성공 메시지 출력 |
| 기타 payload | `SqlResult::Message(format!("{other:?}"))` | debug 형태 메시지 |

`Value`는 `value_to_string`으로 문자열이 된다.

```rust
fn value_to_string(value: Value) -> String {
    match value {
        Value::Bool(value) => value.to_string(),
        Value::I64(value) => value.to_string(),
        Value::Str(value) => value,
        Value::Null => "NULL".to_string(),
        other => format!("{other:?}"),
    }
}
```

실제 코드는 `I8`, `I16`, `I32`, `I128`, unsigned integer, float도 문자열로 바꾼다.

## 실패가 올라오는 지점

| 실패 지점 | 실제 에러 |
| --- | --- |
| GlueSQL SQL 실행 실패 | `AppError::GlueSql(error.to_string())` |
| SELECT를 기대했는데 다른 payload가 온 경우 | `AppError::GlueSql("expected SELECT result")` |
| COUNT를 기대했는데 다른 payload가 온 경우 | `AppError::GlueSql("expected COUNT result")` |
| Task row 타입이 `[I64, Str, Bool]`이 아닌 경우 | `AppError::GlueSql("expected task row ...")` |
| COUNT 값이 `I64`가 아닌 경우 | `AppError::GlueSql("expected COUNT to return I64 ...")` |
| id에 맞는 row가 없는 경우 | `AppError::NotFound(id)` |
| `JsonTaskRepository`에 SQL 실행을 요청한 경우 | `AppError::Unsupported(...)` |

## 수정할 때 주의할 점

초심자가 `tasks` table column을 바꾸려면 다음 위치를 함께 봐야 한다.

| 바꾸는 것 | 같이 확인할 파일/함수 |
| --- | --- |
| column 추가 | `create_tasks_table`, `row_to_task`, `find_all`, `find_one`, `search`, 출력 함수 |
| column 순서 변경 | `row_to_task`의 `[Value::I64, Value::Str, Value::Bool]` 패턴 |
| id 생성 방식 변경 | `add`, `find_all`, `next_id` |
| SQL 출력 방식 변경 | `SqlResult`, `payload_to_sql_result`, `value_to_string`, `print_sql_results` |
| `stats` 계산 변경 | `stats`, `select_count`, `TaskStats::new` |

현재 코드에서 확인되지 않음:

- prepared statement
- query optimizer 직접 제어
- app code에서 Parser/Planner/Executor 직접 호출
- custom row mapper trait
- database auto increment
