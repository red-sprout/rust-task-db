# 도메인 또는 기능 파일

## 포함된 파일 목록

- `src/repository/mod.rs`
- `src/repository/gluesql_repository.rs`
- `src/error.rs`
- `src/service.rs`
- `src/command.rs`
- `src/cli.rs`
- `src/repl.rs`
- `src/task.rs`
- `tasks.json`

## 이 파일 묶음의 역할

Custom error, Service layer, Repository trait, SQL 결과 타입, GlueSQL repository, CLI 명령의 타입, CLI parsing, REPL, Todo 데이터 모양, 보존된 JSON 저장 파일을 설명한다.

## 전체 연결 관계

```text
src/main.rs -> AppError
src/main.rs -> TaskService -> TaskRepository -> GlueSqlTaskRepository
GlueSqlTaskRepository -> GlueSQL SledStorage
GlueSqlTaskRepository -> SqlResult
JsonTaskRepository -> tasks.json 보존
src/cli.rs -> Command
src/main.rs -> cli::parse_args -> Command
src/main.rs -> repl::run_repl
src/main.rs -> Task::new -> src/task.rs
```

## 파일 경로

`src/error.rs`

### 이 파일의 역할

`AppError` enum과 `Display`, `Error`, `From` 구현을 정의한다.

### 핵심 코드 블록

```rust
pub enum AppError {
    Io(std::io::Error),
    Json(serde_json::Error),
    GlueSql(String),
    NotFound(i64),
    InvalidCommand(String),
    Unsupported(String),
}
```

### 코드 블록별 해설

- `Io`: 파일 읽기/쓰기 실패
- `Json`: JSON parsing 또는 변환 실패
- `GlueSql`: GlueSQL 실행 또는 row 변환 실패
- `NotFound`: id에 맞는 Todo가 없음
- `InvalidCommand`: CLI 명령 또는 인자 오류
- `Unsupported`: 현재 repository 구현체가 지원하지 않는 기능

## 파일 경로

`src/service.rs`

### 이 파일의 역할

`TaskService<R: TaskRepository>`를 정의한다.

### 핵심 코드 블록

```rust
pub struct TaskService<R: TaskRepository> {
    repository: R,
}
```

### 코드 블록별 해설

- `TaskService`: Todo 기능을 실행하는 service layer 타입이다.
- `<R: TaskRepository>`: `R`은 `TaskRepository` trait를 구현한 타입이어야 한다.
- `repository: R`: 실제 저장소 구현체를 service 안에 보관한다.

### 메서드 코드

```rust
impl<R: TaskRepository> TaskService<R> {
    pub fn add(&mut self, title: String) -> Result<Task, AppError> {
        self.repository.add(title)
    }

    pub fn list(&mut self) -> Result<Vec<Task>, AppError> {
        self.repository.find_all()
    }

    pub fn execute_sql(&mut self, sql: String) -> Result<Vec<SqlResult>, AppError> {
        self.repository.execute_sql(sql)
    }
}
```

Step 15 현재도 service가 add/list/done/delete/search/stats/sql/repl 요청을 repository에 위임한다. REPL도 내부적으로 `TaskService::execute_sql`을 사용한다. 실패 타입은 `AppError`다.

## 파일 경로

`src/repository/mod.rs`

### 이 파일의 역할

`TaskRepository` trait, `SqlResult`, 보존된 `JsonTaskRepository` 구현체를 정의하고, `GlueSqlTaskRepository`를 re-export한다.

### 핵심 코드 블록

```rust
pub enum SqlResult {
    Select {
        labels: Vec<String>,
        rows: Vec<Vec<String>>,
    },
    Affected {
        kind: String,
        count: usize,
    },
    Message(String),
}

pub trait TaskRepository {
    fn add(&mut self, title: String) -> Result<Task, AppError>;
    fn find_all(&mut self) -> Result<Vec<Task>, AppError>;
    fn mark_done(&mut self, id: i64) -> Result<(), AppError>;
    fn delete(&mut self, id: i64) -> Result<Task, AppError>;
    fn search(&mut self, keyword: &str) -> Result<Vec<Task>, AppError>;
    fn stats(&mut self) -> Result<TaskStats, AppError>;
    fn execute_sql(&mut self, sql: String) -> Result<Vec<SqlResult>, AppError>;
}
```

### 코드 블록별 해설

- `trait`: 저장소가 제공해야 하는 동작 약속이다.
- `SqlResult`: SQL 실행 결과를 출력하기 쉬운 프로젝트 타입으로 바꾼 enum이다.
- `&mut self`: Step 9의 GlueSQL 실행이 내부 storage를 mutable하게 빌리므로 조회 메서드에도 필요하다.
- `Result<..., AppError>`: 성공과 실패를 모두 표현한다.

### 구현체 코드

```rust
impl TaskRepository for JsonTaskRepository {
    fn add(&mut self, title: String) -> Result<Task, AppError> {
        let id = next_id(&self.tasks);
        let task = Task::new(id, title);

        self.tasks.push(task.clone());
        self.save()?;

        Ok(task)
    }
}
```

`JsonTaskRepository`는 `tasks.json`을 사용하는 저장소 구현체다. Step 9에서는 삭제하지 않고 보존한다. 다만 SQL 직접 실행은 GlueSQL 저장소에서만 지원하므로 `JsonTaskRepository::execute_sql`은 `AppError::Unsupported`를 반환한다.

## 파일 경로

`src/repository/gluesql_repository.rs`

### 이 파일의 역할

`GlueSqlTaskRepository<S>`를 정의한다. Step 15 현재 `main.rs`가 사용하는 활성 저장소는 `GlueSqlTaskRepository<SledStorage>`이며, `sql` 명령과 `repl` 안의 SQL도 여기서 실행한다. 테스트에서는 `GlueSqlTaskRepository<MemoryStorage>`도 사용하고, `SledStorage::clone()`으로 두 repository가 같은 DB를 관찰하는 transaction 테스트도 둔다. `GStore`, `GStoreMut`, `Planner` trait bound 해설은 [17-gluesql-internals.md](../17-gluesql-internals.md)에서 다룬다.

핵심 코드:

```rust
pub struct GlueSqlTaskRepository<S = MemoryStorage>
where
    S: GStore + GStoreMut + Planner,
{
    glue: Glue<S>,
}
```

코드 해석:

- `S`: GlueSQL storage 타입이 들어가는 자리다.
- `MemoryStorage`: 테스트에서 빠르게 쓰는 메모리 저장소다.
- `SledStorage`: Step 12 CLI 기본 실행에서 쓰는 파일 기반 저장소다.
- `GStore + GStoreMut + Planner`: GlueSQL의 `Glue<S>`가 SQL 실행을 위해 요구하는 storage 조건이다.

영속 저장소 생성 코드:

```rust
pub fn persistent(path: impl AsRef<Path>) -> Result<Self, AppError> {
    let storage =
        SledStorage::new(path).map_err(|error| AppError::GlueSql(error.to_string()))?;
    let glue = Glue::new(storage);
    let mut repository = Self { glue };

    repository.create_tasks_table()?;

    Ok(repository)
}
```

읽는 법:

```text
path로 SledStorage를 연다.
-> Glue::new(storage)로 SQL 실행 엔진을 만든다.
-> tasks table이 없으면 만든다.
-> repository를 반환한다.
```

### 핵심 코드 블록

```rust
pub struct GlueSqlTaskRepository<S = MemoryStorage>
where
    S: GStore + GStoreMut + Planner,
{
    glue: Glue<S>,
}
```

```rust
fn execute(&mut self, sql: impl AsRef<str>) -> Result<Vec<Payload>, AppError> {
    block_on(self.glue.execute(sql)).map_err(|error| AppError::GlueSql(error.to_string()))
}
```

```rust
fn execute_sql(&mut self, sql: String) -> Result<Vec<SqlResult>, AppError> {
    self.execute(sql)?
        .into_iter()
        .map(payload_to_sql_result)
        .collect()
}
```

### 코드 블록별 해설

- `Glue<S>`: GlueSQL 엔진과 storage 구현체를 함께 들고 있는 값이다. 현재 CLI의 `S`는 `SledStorage`다.
- `block_on`: GlueSQL의 async 실행을 동기 CLI 흐름 안에서 기다린다.
- `Payload`: GlueSQL 실행 결과다. `SELECT` 결과는 row 목록으로 들어온다.
- `Value`: SQL row 안의 개별 값이다. `Value::I64`, `Value::Str`, `Value::Bool`을 `Task` 필드로 바꾼다.
- `payload_to_sql_result`: GlueSQL 결과를 `main.rs`가 출력하기 쉬운 `SqlResult`로 바꾼다.
- `AppError::GlueSql`: GlueSQL 실행 또는 변환 실패를 앱 에러로 표현한다.

## 파일 경로

`src/command.rs`

### 이 파일의 역할

`Command` enum을 정의한다.

### 핵심 코드 블록

```rust
#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Add { title: String },
    List,
    Done { id: i64 },
    Delete { id: i64 },
    Search { keyword: String },
    Stats,
    Sql { sql: String },
    Repl,
    Help,
}
```

### 코드 블록별 해설

- `enum`: 여러 명령 형태 중 하나를 표현하는 타입이다.
- `Add { title: String }`: 제목이 필요한 add 명령이다.
- `Done { id: i64 }`: id가 필요한 done 명령이다.
- `Search { keyword: String }`: 검색어가 필요한 search 명령이다.
- `Stats`: 추가 값이 필요 없는 통계 명령이다.
- `Sql { sql: String }`: 사용자가 직접 입력한 SQL 문자열을 담는다.
- `Repl`: SQL REPL 실행 요청을 표현한다.
- `List`: 추가 값이 필요 없는 명령이다.
- `Debug`, `PartialEq`, `Eq`: parser 테스트에서 비교하고 실패 값을 보기 위해 붙였다.

### 초심자가 수정할 수 있는 부분

새 명령을 추가하려면 먼저 여기에 variant를 추가한다. Step 15 현재는 새 명령을 추가하지 않고 `Repl`까지 유지한다.

## 파일 경로

`src/cli.rs`

### 이 파일의 역할

CLI 인자 `Vec<String>`을 `Result<Command, AppError>`로 변환한다.

### 핵심 코드 블록

```rust
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
        "sql" => {
            let sql = require_next(&mut iter, "Usage: rust-task sql \"SELECT * FROM tasks\"")?;
            Ok(Command::Sql { sql })
        }
        "repl" => Ok(Command::Repl),
        other => Err(format!("Unknown command: {other}")),
    }
}
```

위 코드는 핵심만 줄인 예시다. 실제 파일에는 `done`, `delete`, `search`, `stats`, `sql`, `repl`, `help`, id parsing도 포함되어 있다.

### 코드 블록별 해설

- `args.into_iter()`: `Vec<String>`의 값을 하나씩 소유권째 꺼낸다.
- `_program`: 첫 번째 인자는 실행 파일 이름이라 현재는 쓰지 않는다.
- `let Some(command) = ... else`: 명령이 없으면 help를 반환한다.
- `command.as_str()`: `String`을 `&str`로 빌려 문자열 literal과 비교한다.
- `Result<Command, AppError>`: 성공하면 `Command`, 실패하면 앱 에러다.
- `?`: `Err(AppError)`가 나오면 즉시 반환한다.

### 초심자가 수정할 수 있는 부분

명령 사용법 메시지는 `require_next`를 호출하는 문자열을 바꾸면 된다.

### 수정 전 코드

```rust
let title = require_next(&mut iter, "Usage: rust-task add \"할 일\"")?;
```

### 수정 후 코드

```rust
let title = require_next(&mut iter, "Please write a task title.")?;
```

## 파일 경로

`src/task.rs`

### 이 파일의 역할

`Task` struct, `Task::new`, JSON 변환 derive를 정의한다.

### 핵심 코드 블록

```rust
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct Task {
    pub id: i64,
    pub title: String,
    pub done: bool,
}
```

### 코드 블록별 해설

- `Clone`: `add_task`에서 `task.clone()`을 가능하게 한다.
- `Debug`: 테스트 실패 시 값을 보기 좋게 한다.
- `Deserialize`: JSON 문자열을 `Task`로 읽을 수 있게 한다.
- `PartialEq`, `Eq`: `assert_eq!` 비교에 필요하다.
- `Serialize`: `Task`를 JSON 문자열로 저장할 수 있게 한다.
- `pub`: `main.rs`에서 필드에 접근할 수 있게 한다.

### 이 파일에서 사용된 언어 문법

`struct`, `derive`, `impl`, associated function, `Self`, serde derive

### 초심자가 수정할 수 있는 부분

`Task::new`의 기본 완료 상태를 바꿀 수 있다.

### 수정 전 코드

```rust
done: false,
```

### 수정 후 코드

```rust
done: true,
```

### 수정 시 영향받는 파일

`src/main.rs`의 `creates_task`, `adds_task_to_memory_vec`, `deletes_task` 테스트 기대값이 바뀐다.

## 파일 경로

`tasks.json`

### 이 파일의 역할

Todo 목록을 JSON 배열로 저장한다.

### 핵심 코드 블록

```json
[]
```

Todo가 있으면 아래처럼 저장된다.

```json
[
  {
    "id": 1,
    "title": "Rust 공부",
    "done": false
  }
]
```

### 코드 블록별 해설

- `[]`: Todo가 없는 상태
- `id`: `Task.id`
- `title`: `Task.title`
- `done`: `Task.done`

### 초심자가 수정할 수 있는 부분

학습 목적으로 `tasks.json`을 직접 비우고 싶다면 `[]`로 바꿀 수 있다. JSON 문법이 깨지면 `load_tasks`가 `Failed to parse tasks.json` 에러를 낸다.
