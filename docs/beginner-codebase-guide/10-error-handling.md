# 에러 처리 흐름

## 에러 처리 큰 그림

현재 Step 11에도 custom error 타입 `AppError`가 있다. 실패 가능성은 `src/error.rs`의 `AppError` enum으로 모인다. REPL의 표준 입력/출력 실패도 `AppError::Io`로 표현한다.

```text
CLI parsing 실패
-> parse_args가 Err(AppError::InvalidCommand(...)) 반환
-> main에서 메시지와 help 출력
```

```text
GlueSQL 실행 실패
-> GlueSqlTaskRepository가 Err(AppError::GlueSql(...)) 반환
-> main에서 메시지 출력 후 종료
```

```text
id에 해당하는 Task 없음
-> repository가 Err(AppError::NotFound(id)) 반환
-> service를 거쳐 main에서 메시지 출력
```

```text
지원하지 않는 저장소 기능
-> JsonTaskRepository::execute_sql이 Err(AppError::Unsupported(...)) 반환
-> service를 거쳐 main에서 메시지 출력
```

## Step 5에서 Step 6으로 달라진 점

| 구분 | Step 5 | Step 6 |
| --- | --- | --- |
| 실패 표현 | `String` | `AppError` |
| CLI 입력 오류 | `Err("Unknown command...")` | `Err(AppError::InvalidCommand(...))` |
| 없는 Todo id | `Err("Task not found...")` | `Err(AppError::NotFound(id))` |
| 파일 오류 | 문자열 메시지 | `AppError::Io(error)` |
| JSON 오류 | 문자열 메시지 | `AppError::Json(error)` |
| 출력 방식 | 문자열 출력 | `Display` 구현을 통해 사람이 읽는 메시지 출력 |

핵심은 화면에 보이는 메시지가 크게 바뀐 것이 아니라, 코드 내부에서 실패 종류를 더 명확히 구분하게 됐다는 점이다.

## AppError 목록

| variant | 의미 | 현재 사용 위치 |
| --- | --- | --- |
| `AppError::Io(std::io::Error)` | 파일 읽기/쓰기 실패 | `src/repository/mod.rs` |
| `AppError::Json(serde_json::Error)` | JSON parsing 또는 JSON 변환 실패 | `src/repository/mod.rs` |
| `AppError::GlueSql(String)` | GlueSQL 실행 또는 row 변환 실패 | `src/repository/gluesql_repository.rs` |
| `AppError::NotFound(i64)` | id에 맞는 Todo 없음 | `src/repository/mod.rs` |
| `AppError::InvalidCommand(String)` | CLI 명령 또는 인자 오류 | `src/cli.rs` |
| `AppError::Unsupported(String)` | 현재 repository가 지원하지 않는 기능 | `src/repository/mod.rs` |

## AppError 코드

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

코드 해석:

- `Io`: Rust 표준 파일 I/O 에러를 담는다.
- `Json`: `serde_json` 에러를 담는다.
- `GlueSql`: GlueSQL 실행 실패 메시지를 담는다.
- `NotFound`: 찾지 못한 Todo id를 담는다.
- `InvalidCommand`: 사용자에게 보여줄 CLI 오류 메시지를 담는다.
- `Unsupported`: 현재 구현체가 지원하지 않는 기능 메시지를 담는다.

## Display 구현

```rust
impl fmt::Display for AppError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "I/O error: {error}"),
            Self::Json(error) => write!(formatter, "JSON error: {error}"),
            Self::GlueSql(message) => write!(formatter, "GlueSQL error: {message}"),
            Self::NotFound(id) => write!(formatter, "Task not found: {id}"),
            Self::InvalidCommand(message) => write!(formatter, "{message}"),
            Self::Unsupported(message) => write!(formatter, "{message}"),
        }
    }
}
```

`main.rs`는 여전히 아래처럼 출력한다.

```rust
Err(message) => eprintln!("{message}"),
```

여기서 `message` 변수의 실제 타입은 `AppError`다. `{message}`로 출력할 수 있는 이유는 `AppError`가 `Display`를 구현하기 때문이다.

## From 구현

```rust
impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}
```

```rust
impl From<serde_json::Error> for AppError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}
```

이 구현 덕분에 `?`가 `std::io::Error`나 `serde_json::Error`를 `AppError`로 바꿔서 반환할 수 있다.

## 실패가 발생하는 위치

| 위치 | 실패 상황 | 반환 |
| --- | --- | --- |
| `src/cli.rs` `require_next` | 필요한 인자가 없음 | `Err(AppError::InvalidCommand)` |
| `src/cli.rs` `parse_id` | id가 숫자가 아님 | `Err(AppError::InvalidCommand)` |
| `src/cli.rs` `parse_args` | 모르는 명령 | `Err(AppError::InvalidCommand)` |
| `src/repl.rs` `run_repl_with_io` | 입력 읽기 또는 출력 쓰기 실패 | `Err(AppError::Io)` |
| `src/repository/gluesql_repository.rs` `GlueSqlTaskRepository::new` | `tasks` table 생성 실패 | `Err(AppError::GlueSql)` |
| `src/repository/gluesql_repository.rs` `execute` | SQL 실행 실패 | `Err(AppError::GlueSql)` |
| `src/repository/gluesql_repository.rs` `execute_sql` | 사용자 SQL 실행 실패 | `Err(AppError::GlueSql)` |
| `src/repository/gluesql_repository.rs` `row_to_task` | SQL row를 `Task`로 바꾸지 못함 | `Err(AppError::GlueSql)` |
| `src/repository/gluesql_repository.rs` `payload_to_sql_result` | SQL 결과 변환 실패 | `Err(AppError::GlueSql)` |
| `src/repository/mod.rs` `JsonTaskRepository::execute_sql` | JSON 저장소에 SQL 직접 실행 요청 | `Err(AppError::Unsupported)` |
| `src/repository/mod.rs` `JsonTaskRepository::new` | 보존된 JSON 저장소의 파일 읽기 실패 또는 JSON parsing 실패 | `Err(AppError)` |
| `src/repository/mod.rs` `save_tasks` | 보존된 JSON 저장소의 JSON 변환 실패 또는 파일 쓰기 실패 | `Err(AppError)` |
| `src/repository/mod.rs` `mark_done` | id에 맞는 Task 없음 | `Err(AppError::NotFound)` |
| `src/repository/mod.rs` `delete` | id에 맞는 Task 없음 | `Err(AppError::NotFound)` |

## 없는 id 처리 예시

```rust
fn mark_done(&mut self, id: i64) -> Result<(), AppError> {
    let Some(task) = self.tasks.iter_mut().find(|task| task.id == id) else {
        return Err(AppError::NotFound(id));
    };

    task.done = true;
    self.save()
}
```

id에 맞는 Task를 못 찾으면 `AppError::NotFound(id)`를 반환한다.

## HTTP 상태 코드 또는 에러 코드

웹 서버가 아니므로 코드에서 확인되지 않음.

## 에러 처리 수정 실습

목표: 없는 id 메시지를 바꾼다.

수정 전:

```rust
Self::NotFound(id) => write!(formatter, "Task not found: {id}"),
```

수정 후:

```rust
Self::NotFound(id) => write!(formatter, "No task with id: {id}"),
```

## 초심자가 자주 하는 실수

- `parse::<i64>().unwrap()`을 사용해 잘못된 입력에서 panic을 내는 것
- `Result`와 `Option`을 같은 용도로 생각하는 것
- `?`가 동작하려면 `From` 변환이 필요할 수 있다는 점을 놓치는 것
- `AppError` variant를 추가하고 `Display`의 `match` 분기를 빼먹는 것
