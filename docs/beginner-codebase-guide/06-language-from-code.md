# 이 프로젝트 코드로 배우는 언어 문법

## 이 문서의 목적

현재 Step 18 코드에 실제 등장하는 Rust 문법과 외부 crate 사용만 설명한다. 기능 코드는 Step 12의 GlueSQL `SledStorage` 상태를 유지하고, Step 18에서는 Storage별 기능 차이를 문서로 분석한다. 새 Rust 문법이나 production code 변경은 추가하지 않았다.

## 이 프로젝트에서 자주 등장하는 문법 목록

`mod`, `use`, `derive`, `struct`, `enum`, `trait`, generic, trait bound, struct-like enum variant, `impl`, `Self`, `let`, `mut`, `Vec`, `String`, `Option`, `Result`, `?`, `match`, `loop`, custom error, `Display`, `From`, borrowing, mutable reference, slice, iterator, closure, `filter`, `count`, `collect`, `std::fs`, `Path`, `impl AsRef<Path>`, `BufRead`, `Write`, `Cursor`, serde derive, `block_on`, external crate API, `SledStorage`, `unreachable!`, test attribute, `matches!`

## 파일별로 등장하는 문법

- `src/command.rs`: `enum`, struct-like enum variant, `derive`, `Command::Sql`
- `src/error.rs`: custom error enum, `Display`, `Error`, `From`, manual `PartialEq`, `AppError::GlueSql`, `AppError::Unsupported`
- `src/cli.rs`: `Result<Command, AppError>`, `?`, iterator, `let Some(...) else`, `match`
- `src/service.rs`: generic struct, trait bound, `impl<R: TaskRepository>`, `&self`, `&mut self`
- `src/repository/mod.rs`: `trait`, `impl Trait for Type`, `&mut self`, `PathBuf`, `SqlResult`, 보존된 JSON 저장소
- `src/repository/gluesql_repository.rs`: external crate API, `block_on`, `Payload`, `Value`, `MemoryStorage`, `SledStorage`, generic storage 타입, SQL row 변환, `Payload -> SqlResult` 변환
- `src/repl.rs`: `loop`, `BufRead`, `Write`, `read_line`, `flush`, `Cursor`, mutable borrow
- `src/task.rs`: `struct`, `impl`, `Self`, `derive`, `Serialize`, `Deserialize`, `TaskStats`, domain tests
- `src/main.rs`: `mod`, `use`, `let mut`, `match`, enum pattern, `Result`, `&`, slice, `unreachable!`, tests

## 문법 이름

`impl AsRef<Path>`

### 한 줄 설명

`impl AsRef<Path>`는 `Path`, `PathBuf`, `&str`처럼 path로 볼 수 있는 값을 편하게 받기 위한 함수 인자 타입이다.

### 프로젝트 코드 예시

```rust
pub fn persistent(path: impl AsRef<Path>) -> Result<Self, AppError> {
    let storage =
        SledStorage::new(path).map_err(|error| AppError::GlueSql(error.to_string()))?;
    // ...
}
```

### 코드 해석

`persistent`는 `"data/rust-task-db"` 같은 문자열도 받을 수 있고, 테스트에서 만든 `PathBuf`도 받을 수 있다.

### 프로젝트에서의 역할

Step 12부터 CLI는 `"data/rust-task-db"`를 넘기고, 테스트는 임시 디렉터리 `PathBuf`를 넘긴다.

## 문법 이름

generic storage 타입

### 한 줄 설명

하나의 repository 코드가 여러 GlueSQL storage 타입을 받을 수 있게 만든다.

### 프로젝트 코드 예시

```rust
pub struct GlueSqlTaskRepository<S = MemoryStorage>
where
    S: GStore + GStoreMut + Planner,
{
    glue: Glue<S>,
}
```

### 코드 해석

`S`는 `MemoryStorage`나 `SledStorage`가 들어갈 자리다. `where` 뒤 조건은 GlueSQL이 SQL 실행을 위해 요구하는 storage 기능이다.

### 프로젝트에서의 역할

테스트는 `MemoryStorage`로 빠르게 돌리고, 실제 CLI는 `SledStorage`로 데이터를 디렉터리에 저장한다.

## 문법 이름

`matches!`

### 한 줄 설명

`matches!`는 값이 특정 패턴과 맞는지 `true` 또는 `false`로 알려주는 macro다.

### 프로젝트 코드 예시

```rust
assert!(matches!(result, Err(AppError::GlueSql(_))));
```

### 코드 해석

`result`가 `Err(AppError::GlueSql(...))` 형태인지 확인한다. `_`는 괄호 안의 구체적인 값은 검사하지 않는다는 뜻이다.

### 프로젝트에서의 역할

Step 11의 `src/repository/gluesql_repository.rs` 테스트에서 잘못된 SQL이 `AppError::GlueSql`로 변환되는지 확인한다.

## 문법 이름

`trait`

### 한 줄 설명

trait는 어떤 타입이 제공해야 하는 동작의 약속이다.

### 프로젝트 코드 예시

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

### 코드 해석

`TaskRepository`를 구현하는 타입은 add/list/done/delete/search/stats/sql에 해당하는 저장소 동작을 제공해야 한다.

### 프로젝트에서의 역할

`TaskService`가 파일 저장 세부사항보다 repository 동작에 의존하게 만든다.

## 문법 이름

generic struct와 trait bound

### 한 줄 설명

generic은 타입을 나중에 정할 수 있게 하는 문법이고, trait bound는 그 타입이 어떤 trait를 구현해야 한다는 조건이다.

### 프로젝트 코드 예시

```rust
pub struct TaskService<R: TaskRepository> {
    repository: R,
}
```

```rust
impl<R: TaskRepository> TaskService<R> {
    pub fn add(&mut self, title: String) -> Result<Task, AppError> {
        self.repository.add(title)
    }
}
```

### 코드 해석

`R`은 repository 타입을 담는 자리다. 단, 아무 타입이나 올 수 있는 것이 아니라 `TaskRepository` trait를 구현한 타입만 올 수 있다.

### 프로젝트에서의 역할

현재 실행에서는 `R`이 `GlueSqlTaskRepository`가 된다. 테스트에서는 `FakeTaskRepository`를 넣어 service만 따로 검증한다.

## 문법 이름

`impl Trait for Type`

### 한 줄 설명

특정 타입이 trait의 약속을 실제로 구현한다는 뜻이다.

### 프로젝트 코드 예시

```rust
impl TaskRepository for JsonTaskRepository {
    fn find_all(&mut self) -> Result<Vec<Task>, AppError> {
        Ok(self.tasks.clone())
    }
}
```

### 코드 해석

`JsonTaskRepository`가 `TaskRepository` trait의 메서드를 실제 코드로 구현한다.

Step 9의 활성 구현체는 `GlueSqlTaskRepository`다.

```rust
impl TaskRepository for GlueSqlTaskRepository {
    fn find_all(&mut self) -> Result<Vec<Task>, AppError> {
        self.select_tasks("SELECT id, title, done FROM tasks ORDER BY id;")
    }
}
```

`find_all`이 데이터를 바꾸지 않는 조회처럼 보여도 `&mut self`를 쓰는 이유는 GlueSQL의 `execute`가 내부 storage를 mutable하게 빌리기 때문이다.

## 문법 이름

enum으로 결과 형태 나누기

### 한 줄 설명

enum은 값이 여러 형태 중 하나일 수 있음을 표현한다.

### 프로젝트 코드 예시

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
```

### 코드 해석

`SELECT` 결과는 column 이름과 row 목록이 필요하다. `INSERT`, `UPDATE`, `DELETE` 결과는 몇 건이 바뀌었는지가 중요하다. 그래서 Step 9는 SQL 실행 결과를 하나의 struct로 억지로 맞추지 않고 `SqlResult` enum으로 나눈다.

### 프로젝트에서의 역할

`src/repository/gluesql_repository.rs`가 GlueSQL `Payload`를 `SqlResult`로 바꾸고, `src/main.rs`의 `print_sql_results`가 variant별로 다르게 출력한다.

## 문법 이름

iterator, `iter`, `iter_mut`, `into_iter`

### 한 줄 설명

iterator는 여러 값을 한 번에 통째로 다루지 않고, 앞에서부터 하나씩 꺼내 처리할 수 있게 해주는 흐름이다.

### 프로젝트 코드 예시

```rust
fn next_id(tasks: &[Task]) -> i64 {
    tasks.iter().map(|task| task.id).max().unwrap_or(0) + 1
}
```

```rust
fn mark_done(&mut self, id: i64) -> Result<(), AppError> {
    let Some(task) = self.tasks.iter_mut().find(|task| task.id == id) else {
        return Err(AppError::NotFound(id));
    };

    task.done = true;
    self.save()
}
```

```rust
pub fn parse_args(args: Vec<String>) -> Result<Command, AppError> {
    let mut iter = args.into_iter();
    let _program = iter.next();
    // ...
}
```

### 코드 해석

`tasks.iter()`:

```text
tasks 안의 Task를 하나씩 읽기 전용으로 본다.
각 item의 타입은 &Task에 가깝다.
Task 자체를 수정하지 않는다.
```

`tasks.iter_mut()`:

```text
tasks 안의 Task를 하나씩 수정 가능한 상태로 본다.
각 item의 타입은 &mut Task에 가깝다.
그래서 task.done = true 같은 수정이 가능하다.
```

`args.into_iter()`:

```text
args Vec<String> 안의 String을 하나씩 소유권째 꺼낸다.
꺼낸 String은 Command::Add { title } 같은 곳으로 이동할 수 있다.
```

### `for`와의 차이

`for`는 반복을 실행하는 문법이고, `iter()` / `iter_mut()` / `into_iter()`는 반복할 흐름을 만드는 방법이다.

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
tasks를 하나씩 꺼내서
각 task마다 print_task(task)를 실행한다.
```

이 `for`는 내부적으로 반복 가능한 흐름을 사용한다. 직접 `tasks.iter()`라고 쓰지 않아도 Rust가 `tasks`를 반복 가능한 대상으로 다룬다.

### `iter`, `iter_mut`, `into_iter` 비교

| 표현 | 꺼내는 방식 | 꺼낸 값으로 할 수 있는 일 | 현재 코드 위치 |
| --- | --- | --- | --- |
| `iter()` | 읽기용으로 빌림 | 값 읽기 | `next_id` |
| `iter_mut()` | 수정 가능하게 빌림 | 값 수정 | `mark_done` |
| `into_iter()` | 소유권째 꺼냄 | 값 이동 | `parse_args` |

### 프로젝트에서의 역할

- `next_id`: 모든 Task의 id를 읽어서 가장 큰 id를 찾는다.
- `mark_done`: id가 같은 Task를 찾아 `done` 값을 바꾼다.
- `delete`: id가 같은 Task의 위치를 찾아 삭제한다.
- `search`: 제목에 keyword가 포함된 Task만 남긴다.
- `stats`: 완료된 Task 개수를 센다.
- `parse_args`: CLI 인자를 앞에서부터 하나씩 소비한다.

### Step 7의 검색 코드

```rust
self.tasks
    .iter()
    .filter(|task| task.title.to_lowercase().contains(&keyword))
    .cloned()
    .collect()
```

읽는 법:

```text
tasks를 하나씩 본다.
-> title에 keyword가 들어간 task만 남긴다.
-> 참조가 아니라 Task 값으로 복제한다.
-> Vec<Task>로 모은다.
```

### Step 7의 통계 코드

```rust
let done = self.tasks.iter().filter(|task| task.done).count();
```

읽는 법:

```text
tasks를 하나씩 본다.
-> done이 true인 task만 남긴다.
-> 남은 개수를 센다.
```

### 초심자가 자주 하는 오해

`iter()`는 Vec를 복사하는 것이 아니다. Vec 안의 값을 하나씩 빌려 보는 흐름을 만든다.

`iter_mut()`는 Vec 자체를 새로 만드는 것이 아니다. Vec 안의 값을 수정 가능한 참조로 하나씩 빌려준다.

`into_iter()`는 값을 소유권째 꺼내므로, 보통 원래 Vec를 그 뒤에 다시 쓰기 어렵다.

### 직접 수정해볼 수 있는 예시

`mark_done`에서 `iter_mut()`를 `iter()`로 바꿔보면 `task.done = true`에서 컴파일 오류가 난다. 읽기 전용 참조로는 Task를 수정할 수 없기 때문이다.

## 문법 이름

`std::fs`

### 한 줄 설명

파일을 읽고 쓰는 Rust 표준 라이브러리 모듈이다.

### 프로젝트 코드 예시

```rust
fs::read_to_string(path)
fs::write(path, contents)
```

### 코드 해석

- `read_to_string`: 파일 전체를 문자열로 읽는다.
- `write`: 문자열을 파일에 쓴다.

### 프로젝트에서의 역할

`tasks.json`을 읽고 저장하기 위해 사용한다.

## 문법 이름

serde derive

### 한 줄 설명

`Task`를 JSON으로 저장하거나 JSON에서 읽을 수 있게 만드는 외부 crate 기능이다.

### 프로젝트 코드 예시

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct Task {
    pub id: i64,
    pub title: String,
    pub done: bool,
}
```

### 코드 해석

- `Serialize`: `Task`를 JSON 문자열로 바꿀 수 있게 한다.
- `Deserialize`: JSON 문자열을 `Task`로 바꿀 수 있게 한다.

### 프로젝트에서의 역할

`serde_json::to_string_pretty(tasks)`와 `serde_json::from_str(&contents)`가 동작하려면 필요하다.

## 문법 이름

`impl AsRef<Path>`

### 한 줄 설명

파일 경로처럼 다룰 수 있는 값을 인자로 받겠다는 뜻이다.

### 프로젝트 코드 예시

```rust
fn load_tasks(path: impl AsRef<Path>) -> Result<Vec<Task>, AppError>
```

### 코드 해석

`"tasks.json"` 같은 문자열도 받을 수 있고, 테스트에서 만든 `PathBuf`도 받을 수 있다.

## 문법 이름

`mod`와 `use`

### 한 줄 설명

`mod`는 다른 Rust 파일을 모듈로 등록하고, `use`는 그 안의 이름을 현재 파일에서 편하게 쓰게 가져온다.

### 프로젝트 코드 예시

```rust
mod cli;
mod command;
mod repository;
mod service;
mod task;

use command::Command;
use repository::JsonTaskRepository;
use service::TaskService;
use task::Task;
```

### 코드 해석

- `mod cli;`: `src/cli.rs` 파일을 `cli` 모듈로 포함한다.
- `mod command;`: `src/command.rs` 파일을 `command` 모듈로 포함한다.
- `mod repository;`: `src/repository/mod.rs` 파일을 `repository` 모듈로 포함한다.
- `mod service;`: `src/service.rs` 파일을 `service` 모듈로 포함한다.
- `mod task;`: `src/task.rs` 파일을 `task` 모듈로 포함한다.
- `use command::Command;`: `Command` 타입을 현재 파일에서 바로 쓰게 한다.
- `use repository::JsonTaskRepository;`: JSON repository 타입을 현재 파일에서 바로 쓰게 한다.
- `use service::TaskService;`: service 타입을 현재 파일에서 바로 쓰게 한다.

### 이 프로젝트에서 쓰인 이유

명령 모델, CLI parser, service, repository, Todo 모델을 파일로 나누고 `main.rs`에서 연결하기 위해서다.

## 문법 이름

`enum`

### 한 줄 설명

여러 가능한 형태 중 정확히 하나를 표현하는 타입이다.

### 프로젝트 코드 예시

```rust
pub enum Command {
    Add { title: String },
    List,
    Done { id: i64 },
    Delete { id: i64 },
    Help,
}
```

### 코드 해석

- `Command::Add { title }`: 제목을 가진 add 명령
- `Command::List`: 추가 값이 없는 list 명령
- `Command::Done { id }`: id를 가진 done 명령
- `Command::Delete { id }`: id를 가진 delete 명령
- `Command::Help`: help 출력 명령

### 프로젝트에서의 역할

CLI 문자열을 바로 실행하지 않고, 먼저 의미 있는 타입으로 바꾼다. 그래서 `main.rs`는 문자열 위치나 id parsing을 몰라도 된다.

### 수정 포인트

새 CLI 명령을 만들 때는 `Command`에 variant를 먼저 추가한다.

## 문법 이름

struct-like enum variant

### 한 줄 설명

enum의 한 종류가 이름 붙은 필드를 가지는 형태다.

### 프로젝트 코드 예시

```rust
Add { title: String }
Done { id: i64 }
```

### 코드 해석

`Add`는 제목이 필요하고, `Done`은 id가 필요하다. 필드 이름이 있으므로 `main.rs`에서 아래처럼 꺼낼 수 있다.

```rust
Command::Add { title } => {
    let task = add_task(&mut tasks, title);
}
```

### 프로젝트에서의 역할

명령마다 필요한 값이 다르다는 사실을 타입에 담는다.

## 문법 이름

`Result`

### 한 줄 설명

성공 또는 실패를 타입으로 표현한다.

### 프로젝트 코드 예시

```rust
pub fn parse_args(args: Vec<String>) -> Result<Command, AppError>
```

### 코드 해석

- `Ok(Command)`: CLI parsing 성공
- `Err(AppError)`: CLI parsing 실패와 메시지

### 프로젝트에서의 역할

잘못된 명령, 빠진 제목, 숫자가 아닌 id를 panic 없이 처리한다.

### 초심자가 자주 하는 오해

`Result`는 예외가 아니다. 함수의 반환값이다. 그래서 호출한 쪽에서 `match`로 처리한다.

## 문법 이름

`?`

### 한 줄 설명

`Result`가 `Err`이면 현재 함수에서 바로 반환하고, `Ok`이면 안의 값을 꺼낸다.

### 프로젝트 코드 예시

```rust
let id = parse_id(require_next(&mut iter, "Usage: rust-task done 1")?)?;
```

### 코드 해석

1. `require_next(...)`가 `Err(AppError)`이면 `parse_args`가 바로 그 에러를 반환한다.
2. 성공하면 `String` id 값을 꺼낸다.
3. `parse_id(...)`가 `Err(AppError)`이면 다시 바로 반환한다.
4. 성공하면 `i64` id 값을 꺼낸다.

### 프로젝트에서의 역할

CLI parsing 실패 처리를 짧게 쓴다.

## 문법 이름

`derive`

### 한 줄 설명

자주 쓰는 trait 구현을 컴파일러가 자동으로 만들어주는 attribute다.

### 프로젝트 코드 예시

```rust
#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Add { title: String },
    List,
}
```

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Task {
    pub id: i64,
    pub title: String,
    pub done: bool,
}
```

### 코드 해석

| 항목 | 의미 | 프로젝트에서 쓰이는 이유 |
| --- | --- | --- |
| `Clone` | 값을 명시적으로 복제 가능 | `Task`의 `task.clone()` |
| `Debug` | 디버그 출력 가능 | 테스트 실패 시 값 표시 |
| `PartialEq` | `==`, `assert_eq!` 비교 가능 | 테스트에서 `Task`, `Command` 비교 |
| `Eq` | 완전한 동등성 표시 | 동등성 타입임을 표시 |

## 문법 이름

`struct`와 `impl`

### 한 줄 설명

`struct`는 데이터 모양을 만들고, `impl`은 그 타입에 함수를 붙인다.

### 프로젝트 코드 예시

```rust
pub struct Task {
    pub id: i64,
    pub title: String,
    pub done: bool,
}

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

### 코드 해석

- `Task`: Todo 한 건의 타입
- `Task::new(...)`: 새 Task를 만드는 associated function
- `Self`: 이 `impl Task` 안에서는 `Task`를 뜻한다.

## 문법 이름

`let`과 `mut`

### 한 줄 설명

`let`은 변수를 만들고, `mut`는 그 변수를 나중에 바꿀 수 있게 한다.

### 프로젝트 코드 예시

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

### 코드 해석

- `let mut tasks`: Todo를 추가/삭제해야 하므로 수정 가능한 변수다.
- `let command`: 한 번 정해진 뒤 바꾸지 않으므로 `mut`가 없다.

## 문법 이름

`Option`

### 한 줄 설명

값이 있을 수도 없을 수도 있음을 타입으로 표현한다.

### 프로젝트 코드 예시

```rust
fn delete_task(tasks: &mut Vec<Task>, id: i64) -> Option<Task>
```

### 코드 해석

삭제할 Task가 있으면 `Some(Task)`, 없으면 `None`을 반환한다.

### 프로젝트에서의 역할

없는 id를 panic 없이 표현하기 위해서다.

## 문법 이름

`match`

### 한 줄 설명

값의 모양이나 내용에 따라 실행할 코드를 고르는 분기 문법이다.

### 프로젝트 코드 예시

```rust
match command {
    Command::Add { title } => { /* add 처리 */ }
    Command::List => { /* list 처리 */ }
    Command::Help => print_help(),
}
```

### 코드 해석

Java의 `switch`처럼 분기하지만, 문자열뿐 아니라 enum 내부 값도 꺼낼 수 있다.

### 프로젝트에서의 역할

`Command` 종류에 따라 실행할 기능을 고른다.

## 문법 이름

borrowing과 mutable reference

### 한 줄 설명

값을 소유하지 않고 빌려 쓰며, `&mut`는 수정 가능한 빌림이다.

### 프로젝트 코드 예시

```rust
fn add_task(tasks: &mut Vec<Task>, title: String) -> Task
fn print_task(task: &Task)
fn print_tasks(tasks: &[Task])
```

### 코드 해석

- `&mut Vec<Task>`: 목록을 수정한다.
- `&Task`: Task를 읽기만 한다.
- `&[Task]`: Task 여러 개를 slice로 읽기만 한다.

## 초심자가 자주 헷갈리는 문법

- `String` vs `&str`
- `Command::Add { title }`처럼 enum 안의 값 꺼내기
- `Result`와 `Option`의 차이
- `?`가 아무 곳에서나 되는 것이 아니라 반환 타입이 맞아야 한다는 점
- `iter()` vs `iter_mut()` vs `into_iter()`
- `Some(())`의 `()` 의미
- 마지막 줄에 세미콜론이 없으면 반환값이 된다는 점
- `#[cfg(test)]`와 `#[test]`의 차이
- `assert_eq!`는 assertion이고, 실제 비교 가능성은 `PartialEq`가 만든다는 점

## 다른 언어 사용자 관점의 비교

- `struct`: Java/Kotlin의 data class에 가까움
- `enum`: Java enum보다 강하고, 각 variant가 데이터를 가질 수 있음
- `Option`: Kotlin nullable과 비슷하지만 null이 없음
- `Result`: 성공/실패를 반환값으로 표현
- `match`: switch보다 강한 패턴 매칭
- `Vec`: Java `ArrayList`와 비슷한 growable list

## 문법 이름

`BufRead`와 `Write`

### 한 줄 설명

`BufRead`는 줄 단위로 입력을 읽기 위한 trait이고, `Write`는 출력 대상에 문자열을 쓰기 위한 trait이다.

### 프로젝트 코드 예시

```rust
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
    // ...
}
```

### 코드 해석

`run_repl_with_io`는 실제 터미널 입력만 받는 함수가 아니다. `BufRead`를 구현한 값이면 입력으로 받을 수 있고, `Write`를 구현한 값이면 출력으로 받을 수 있다.

### 프로젝트에서의 역할

실제 실행에서는 stdin/stdout을 쓰고, 테스트에서는 `Cursor`와 `Vec<u8>`을 넣어 REPL을 검증한다.

### `Cursor`는 왜 쓰는가?

`Cursor`는 문자열이나 byte 배열을 파일/터미널 입력처럼 읽게 해주는 테스트용 도구로 볼 수 있다.

```rust
let input = Cursor::new(".schema\n.quit\n");
let mut output = Vec::new();

run_repl_with_io(&mut service, input, &mut output).unwrap();
```

읽는 법:

```text
".schema"를 입력한 것처럼 만든다.
-> ".quit"을 입력한 것처럼 만든다.
-> output Vec에 REPL 출력이 쌓인다.
```

이 구조 덕분에 사람이 직접 키보드로 입력하지 않아도 REPL을 테스트할 수 있다.

## 문법 이름

`loop`와 `break`

### 한 줄 설명

`loop`는 계속 반복하고, `break`는 반복을 끝낸다.

### 프로젝트 코드 예시

```rust
loop {
    write!(output, "{PROMPT}")?;
    let mut line = String::new();
    let bytes_read = input.read_line(&mut line)?;

    if bytes_read == 0 {
        break;
    }
}
```

### 코드 해석

REPL은 사용자가 종료 명령을 입력하거나 입력이 끝날 때까지 계속 줄을 읽어야 한다. 그래서 `loop`를 사용한다.

### `continue`는 무엇인가?

REPL 코드에는 빈 줄을 무시하기 위해 `continue`가 나온다.

```rust
if command.is_empty() {
    continue;
}
```

읽는 법:

```text
입력이 빈 문자열이면
아래 SQL 실행 분기로 내려가지 않고
다시 loop 처음으로 돌아간다.
```
