# 초심자 질문 모음

## `mod task;`는 무엇인가?

`src/main.rs`의 코드:

```rust
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
```

의미:

```text
mod cli;
-> src/cli.rs 파일을 cli 모듈로 포함한다.

mod command;
-> src/command.rs 파일을 command 모듈로 포함한다.

mod error;
-> src/error.rs 파일을 error 모듈로 포함한다.

mod repl;
-> src/repl.rs 파일을 repl 모듈로 포함한다.

mod repository;
-> src/repository/mod.rs 파일을 repository 모듈로 포함한다.

mod service;
-> src/service.rs 파일을 service 모듈로 포함한다.

mod task;
-> src/task.rs 파일을 task 모듈로 포함한다.

use command::Command;
-> command 모듈 안의 Command 타입을 main.rs에서 Command라고 바로 쓰게 가져온다.

use repository::{GlueSqlTaskRepository, SqlResult};
-> repository 모듈 안의 GlueSQL 저장소 타입과 SQL 출력 결과 타입을 main.rs에서 쓰게 가져온다.

use service::TaskService;
-> service 모듈 안의 TaskService 타입을 main.rs에서 쓰게 가져온다.

use task::{Task, TaskStats};
-> task 모듈 안의 Task 타입과 통계 타입을 main.rs에서 쓰게 가져온다.
```

`mod task;`가 없으면 Rust는 `src/task.rs` 파일을 현재 crate의 모듈로 알지 못한다.

## `#[derive(Clone, Debug, PartialEq, Eq)]`는 무엇인가?

컴파일러에게 `Task` 타입의 기본 기능을 자동 구현해달라는 attribute다.

| 항목 | 의미 | 프로젝트에서 쓰이는 이유 |
| --- | --- | --- |
| `Clone` | 값 복제 가능 | `task.clone()` |
| `Debug` | 디버그 출력 가능 | 테스트 실패 시 값 표시 |
| `PartialEq` | `==`, `assert_eq!` 비교 가능 | 테스트에서 `Task` 비교 |
| `Eq` | 완전한 동등성 표시 | 동등성 trait 조합 |

## `assert_eq!`는 equals인가, assertThat인가?

`assert_eq!` 자체는 테스트 assertion이다. JUnit의 `assertEquals`, AssertJ의 `assertThat(...).isEqualTo(...)`에 가깝다.

실제 비교 가능성은 `PartialEq`가 담당한다. Java로 치면 `equals()` 구현에 해당하는 부분이다. 현재는 `Task`와 `Command` 모두 테스트 비교를 위해 `PartialEq`, `Eq`를 derive한다.

```rust
assert_eq!(task, Task::new(1, "Rust".to_string()));
```

## `impl Task`는 무엇인가?

`Task` 타입에 함수나 메서드를 붙이는 블록이다.

```rust
impl Task {
    pub fn new(id: i64, title: String) -> Self {
        Self {
            id,
            title,
            done: false,
        }
    }
}
```

`Task::new(...)`로 호출할 수 있다. 여기서 `Self`는 `Task`를 뜻한다.

## `let`과 `mut`는 무엇인가?

| 코드 | 의미 |
| --- | --- |
| `let name = value;` | 바꿀 수 없는 변수 선언 |
| `let mut name = value;` | 바꿀 수 있는 변수 선언 |

현재 코드:

```rust
let mut tasks = Vec::new();
let command = match cli::parse_args(std::env::args().collect()) {
    Ok(command) => command,
    Err(message) => {
        eprintln!("{message}");
        print_help();
        return;
    }
};
```

`tasks`는 Todo를 추가/삭제해야 해서 `mut`가 필요하다. `command`는 한 번 정해진 뒤 바꾸지 않으므로 `mut`가 없다.

## `match command`는 switch 문인가?

지금 사용 방식은 Java의 `switch`와 비슷하게 이해해도 된다.

```rust
match command {
    Command::Add { title } => { /* add */ }
    Command::List => { /* list */ }
    Command::Help => print_help(),
}
```

다만 Rust의 `match`는 Java switch보다 강하다. `Command::Add { title }`처럼 enum 안의 값도 동시에 꺼낼 수 있다.

`src/cli.rs` 안에서는 아직 문자열 명령을 비교하기 위해 아래처럼 `as_str()`를 사용한다.

```rust
match command.as_str() {
    "add" => { /* Command::Add 생성 */ }
    "list" => Ok(Command::List),
    other => Err(format!("Unknown command: {other}")),
}
```

`as_str()`는 `String`을 `&str`로 빌려 `"add"` 같은 문자열 리터럴과 비교하기 위해 사용한다.

## `add` 분기는 어떻게 읽는가?

Step 16 현재도 `add` 처리는 CLI parsing, GlueSQL SledStorage repository 생성, service 생성, Todo 추가, SQL 실행으로 나뉜다. 실패가 생기면 `AppError`로 표현된다.

`src/cli.rs`:

```rust
"add" => {
    let title = require_next(&mut iter, "Usage: rust-task add \"할 일\"")?;
    Ok(Command::Add { title })
}
```

`src/main.rs`:

```rust
Command::Add { title } => match service.add(title) {
    Ok(task) => {
        println!("Added:");
        print_task(&task);
    }
    Err(message) => eprintln!("{message}"),
},
```

읽는 순서:

```text
parse_args가 명령어 "add"를 확인한다.
-> require_next로 제목을 꺼낸다.
-> Command::Add { title }을 만든다.
-> main이 Command::Add 분기를 실행한다.
-> GlueSqlTaskRepository::persistent로 GlueSQL SledStorage와 tasks table을 준비한다.
-> TaskService::new로 service를 만든다.
-> service.add로 Todo 추가 요청을 보낸다.
-> service가 repository.add에 위임한다.
-> repository 내부에서 INSERT SQL을 실행한다.
```

`require_next`는 값이 없을 때 panic하지 않고 `Err(AppError::InvalidCommand(...))`를 반환한다.

## `tasks.json`은 왜 생겼나?

Step 1과 Step 2에서는 Todo가 메모리에만 있었다. 프로그램이 끝나면 데이터도 사라졌다.

Step 3부터 Step 7까지는 `tasks.json`에 저장했다. Step 16 현재는 `main.rs`가 `TaskService`를 호출하고, 실제 저장 책임은 `GlueSqlTaskRepository<SledStorage>`가 맡는다. 실패는 `AppError`로 표현한다.

```text
cargo run -- add "Rust 공부"
-> GlueSQL SledStorage에 INSERT

cargo run -- list
-> data/rust-task-db에 저장된 값을 다시 읽음
```

그래서 Step 12에서는 별도 `cargo run` 명령끼리 데이터가 이어진다. 저장 위치는 `data/rust-task-db`다.

## `load_tasks`는 전체적으로 무슨 일을 하나?

`load_tasks`는 `tasks.json` 파일을 읽어서 Rust의 `Vec<Task>`로 바꾸는 함수다. Step 16 현재는 기본 실행 경로가 아니라, 보존된 `JsonTaskRepository` 안에 있다.

현재 코드:

```rust
fn load_tasks(path: impl AsRef<Path>) -> Result<Vec<Task>, AppError> {
    let path = path.as_ref();

    match fs::read_to_string(path) {
        Ok(contents) => serde_json::from_str(&contents).map_err(AppError::from),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(Vec::new()),
        Err(error) => Err(AppError::from(error)),
    }
}
```

큰 흐름:

```text
tasks.json 경로를 받는다.
-> 파일을 문자열로 읽는다.
-> 읽은 JSON 문자열을 Vec<Task>로 바꾼다.
-> 파일이 없으면 빈 Vec를 준다.
-> 읽기 실패나 JSON 오류는 AppError로 돌려준다.
```

## `load_tasks`의 반환 타입은 왜 `Result<Vec<Task>, AppError>`인가?

파일 읽기는 실패할 수 있다. JSON parsing도 실패할 수 있다. 그래서 성공과 실패를 모두 표현해야 한다.

```rust
Result<Vec<Task>, AppError>
```

뜻:

```text
Ok(Vec<Task>)
-> Todo 목록 읽기 성공

Err(AppError)
-> 파일 읽기 또는 JSON parsing 실패
```

## `load_tasks`에서 파일이 없으면 왜 에러가 아닌가?

처음 실행하면 `tasks.json`이 아직 없을 수 있다. 이 상황은 프로그램 입장에서는 자연스러운 시작 상태다.

그래서 아래처럼 처리한다.

```rust
Err(error) if error.kind() == ErrorKind::NotFound => Ok(Vec::new()),
```

뜻:

```text
파일이 없음
-> 아직 Todo가 없다는 뜻으로 본다.
-> 빈 Vec<Task>로 시작한다.
```

## `JsonTaskRepository`는 무엇인가?

`JsonTaskRepository`는 `tasks.json`을 사용하는 Todo 저장소다.

현재 코드:

```rust
pub struct JsonTaskRepository {
    path: PathBuf,
    tasks: Vec<Task>,
}
```

의미:

```text
path
-> 어떤 JSON 파일을 사용할지 기억한다.

tasks
-> 파일에서 읽어온 Todo 목록을 메모리에 들고 있다.
```

`main.rs`는 `load_tasks`와 `save_tasks`를 직접 부르지 않는다. 대신 아래처럼 service 메서드를 호출한다.

```rust
let repository = GlueSqlTaskRepository::persistent("data/rust-task-db")?;
let mut service = TaskService::new(repository);
service.add(title)?;
```

현재 `main.rs`에서는 `?` 대신 `match`로 에러를 출력한다.

## `TaskService<R: TaskRepository>`는 무엇인가?

`TaskService`는 `main.rs`와 repository 사이에 있는 service layer다.

```rust
pub struct TaskService<R: TaskRepository> {
    repository: R,
}
```

읽는 법:

```text
TaskService는 repository를 안에 가진다.
R은 TaskRepository를 구현한 타입이어야 한다.
현재 실행에서는 R이 GlueSqlTaskRepository<SledStorage>다.
```

현재는 service가 추가 검증을 하지 않고 repository에 위임한다.

```rust
pub fn add(&mut self, title: String) -> Result<Task, AppError> {
    self.repository.add(title)
}
```

## `TaskRepository` trait는 무엇인가?

`TaskRepository`는 저장소가 제공해야 하는 동작의 약속이다.

```rust
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

읽는 법:

```text
이 trait를 구현하는 저장소는 add/list/done/delete/search/stats/sql 동작을 제공해야 한다.
현재 구현체는 JsonTaskRepository와 GlueSqlTaskRepository 두 개다.
현재 main.rs가 사용하는 활성 구현체는 GlueSqlTaskRepository다.
```

## `serde_json::from_str`는 `load_tasks`에서 무슨 역할인가?

`fs::read_to_string`은 파일을 그냥 문자열로만 읽는다.

예:

```json
[
  {
    "id": 1,
    "title": "Rust 공부",
    "done": false
  }
]
```

이 상태는 아직 Rust의 `Vec<Task>`가 아니다. 그냥 JSON 모양의 문자열이다.

```rust
serde_json::from_str(&contents)
```

이 코드가 JSON 문자열을 Rust 값으로 바꾼다.

```text
JSON 문자열
-> Vec<Task>
```

## `map_err`는 `load_tasks`에서 왜 쓰나?

`serde_json::from_str`가 실패하면 serde_json 전용 에러 타입이 나온다. 그런데 `load_tasks`는 실패 타입을 `String`으로 정했다.

그래서 `map_err`로 에러를 문자열 메시지로 바꾼다.

```rust
.map_err(|error| format!("Failed to parse {}: {error}", path.display()))
```

읽는 법:

```text
JSON parsing 에러가 나면
-> 사람이 읽을 수 있는 String 메시지로 바꾼다.
```

## `Result<Command, AppError>`는 무엇인가?

`src/cli.rs`의 `parse_args` 반환 타입이다.

```rust
pub fn parse_args(args: Vec<String>) -> Result<Command, AppError>
```

의미:

```text
Ok(Command)
-> CLI parsing 성공

Err(AppError)
-> CLI parsing 실패. 현재는 주로 AppError::InvalidCommand
```

## `AppError`는 무엇인가?

`AppError`는 이 앱에서 발생할 수 있는 실패를 한 타입으로 모은 enum이다.

```rust
pub enum AppError {
    Io(std::io::Error),
    Json(serde_json::Error),
    NotFound(i64),
    InvalidCommand(String),
}
```

Step 5에서는 실패가 대부분 `String`이었다. Step 6에서는 실패 종류를 `AppError` variant로 나눈다.

```text
잘못된 명령 -> AppError::InvalidCommand
없는 id -> AppError::NotFound
파일 실패 -> AppError::Io
JSON 실패 -> AppError::Json
```

## `?`는 무엇인가?

`Result`가 실패하면 현재 함수에서 바로 반환하고, 성공하면 안의 값을 꺼낸다.

```rust
let title = require_next(&mut iter, "Usage: rust-task add \"할 일\"")?;
```

`require_next`가 `Err(...)`이면 `parse_args`도 바로 `Err(...)`를 반환한다.

## `#[cfg(test)]`는 무엇인가?

조건부 컴파일 attribute다.

```rust
#[cfg(test)]
mod tests {
    // ...
}
```

의미:

```text
cargo test일 때만 tests 모듈을 컴파일한다.
cargo run일 때는 tests 모듈을 포함하지 않는다.
```

`#[test]`는 개별 함수를 테스트 함수로 표시한다.

## 반복자가 무엇인가?

반복자는 `Vec` 같은 여러 값을 앞에서부터 하나씩 꺼내 처리할 수 있게 해주는 흐름이다.

예를 들어 `Vec<Task>`가 아래처럼 있다고 생각한다.

```text
Task 1
Task 2
Task 3
```

`tasks.iter()`는 이 값을 한 번에 통째로 주는 것이 아니라 아래처럼 하나씩 보게 해준다.

```text
Task 1을 본다
-> Task 2를 본다
-> Task 3을 본다
-> 끝
```

현재 코드:

```rust
tasks.iter().map(|task| task.id).max()
```

읽는 순서:

```text
tasks를 하나씩 읽는다.
-> 각 task에서 id만 꺼낸다.
-> 가장 큰 id를 찾는다.
```

## `iter()`와 `iter_mut()`는 무엇이 다른가?

`iter()`는 읽기용이고, `iter_mut()`는 수정용이다.

| 코드 | 의미 | 수정 가능 여부 | 현재 코드 |
| --- | --- | --- | --- |
| `tasks.iter()` | Task를 하나씩 읽는다. | 수정 불가 | `next_id` |
| `tasks.iter_mut()` | Task를 하나씩 수정 가능하게 본다. | 수정 가능 | `mark_done` |

그래서 `mark_done`은 `iter_mut()`를 쓴다.

```rust
let Some(task) = tasks.iter_mut().find(|task| task.id == id) else {
    return None;
};

task.done = true;
```

여기서 `task.done = true`로 값을 바꿔야 하므로 읽기용 `iter()`로는 부족하다.

## `for`와 `iter()`는 뭐가 다른가?

`for`는 반복을 실행하는 문법이고, `iter()`는 반복할 흐름을 만드는 메서드다.

현재 코드:

```rust
fn print_tasks(tasks: &[Task]) {
    for task in tasks {
        print_task(task);
    }
}
```

읽는 법:

```text
tasks 안의 Task를 하나씩 꺼낸다.
각 Task마다 print_task를 실행한다.
```

한 줄로 정리하면:

```text
iter()
-> 하나씩 볼 수 있는 흐름 만들기

for
-> 그 흐름을 실제로 돌면서 코드 실행하기
```

## `into_iter()`는 무엇인가?

`into_iter()`는 값을 빌려 보는 것이 아니라 소유권째 하나씩 꺼낸다.

현재 코드:

```rust
let mut iter = args.into_iter();
let _program = iter.next();
```

`args`는 `Vec<String>`이다. `into_iter()`를 쓰면 안의 `String`들이 하나씩 이동한다.

이 프로젝트에서는 CLI 인자를 앞에서부터 소비하기 위해 쓴다.

```text
첫 번째: 실행 파일 이름
두 번째: 명령어 add/list/done/delete
세 번째: title 또는 id
```

## REPL 모드는 코드에서 어떻게 구현되는가?

REPL은 `src/repl.rs`에 있다. 핵심 구조는 아래와 같다.

```text
run_repl
-> stdin/stdout 준비
-> run_repl_with_io
-> loop
-> read_line
-> .schema/.exit/.quit/SQL 분기
-> SQL이면 service.execute_sql
-> 결과 출력
-> 다시 loop
```

`src/main.rs`에서는 `Command::Repl`일 때 REPL을 실행한다.

```rust
Command::Repl => {
    if let Err(message) = repl::run_repl(&mut service) {
        eprintln!("{message}");
    }
}
```

`src/repl.rs`에서는 한 줄씩 입력을 읽는다.

```rust
let mut line = String::new();
let bytes_read = input.read_line(&mut line)?;
```

읽은 줄은 `trim()`으로 공백과 엔터를 제거한다.

```rust
let command = line.trim();
```

그 다음 입력값을 분기한다.

```rust
match command {
    ".exit" | ".quit" => break,
    ".schema" => writeln!(output, "{SCHEMA}")?,
    sql => match service.execute_sql(sql.to_string()) {
        Ok(results) => write_sql_results(output, &results)?,
        Err(message) => writeln!(output, "{message}")?,
    },
}
```

한 줄로 정리하면:

```text
REPL은 loop 안에서 한 줄을 읽고, 특수 명령이면 직접 처리하고, 나머지는 SQL로 실행하는 구조다.
```

## REPL 안에서 INSERT 후 SELECT가 이어지는 이유는?

`cargo run -- repl`은 repository와 service를 한 번 만든 뒤, 같은 service를 REPL loop 안에서 계속 사용한다.

```text
GlueSqlTaskRepository::persistent("data/rust-task-db")
-> TaskService::new(repository)
-> repl::run_repl(&mut service)
-> service.execute_sql 반복 호출
```

그래서 REPL 안에서는 같은 GlueSQL `SledStorage` repository 인스턴스가 유지된다.

반대로 아래처럼 `cargo run`을 두 번 나눠 실행하면 프로세스가 다르므로 데이터가 이어지지 않을 수 있다.

```bash
cargo run -- sql "INSERT INTO tasks VALUES (1, 'Rust', FALSE);"
cargo run -- sql "SELECT * FROM tasks;"
```
