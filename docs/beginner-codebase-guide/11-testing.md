# 테스트 코드 해설

## 테스트 구조

테스트는 `src/task.rs`, `src/cli.rs`, `src/error.rs`, `src/service.rs`, `src/repl.rs`, `src/repository/mod.rs`, `src/repository/gluesql_repository.rs`, `src/main.rs` 하단의 `#[cfg(test)] mod tests` 안에 있다. Step 12에서는 SledStorage 영속 저장 테스트가 추가되었다.

```rust
#[cfg(test)]
mod tests {
    use super::*;
}
```

`#[cfg(test)]`는 테스트를 실행할 때만 이 모듈을 컴파일하라는 뜻이다. 평소 `cargo run -- ...`으로 앱을 실행할 때는 이 테스트 모듈이 빠진다.

## 테스트 실행 명령어

```bash
cargo test
```

## 테스트 파일 목록

- `src/cli.rs`
- `src/task.rs`
- `src/error.rs`
- `src/service.rs`
- `src/repl.rs`
- `src/repository/mod.rs`
- `src/repository/gluesql_repository.rs`
- `src/main.rs`

## 테스트 종류

- CLI parser 단위 테스트
- AppError 단위 테스트
- service 단위 테스트
- JSON repository 단위 테스트
- GlueSQL repository 단위 테스트
- SQL 실행 모드 단위 테스트
- REPL 단위 테스트
- help 흐름 보조 테스트
- Task 생성과 TaskStats 계산 테스트
- SledStorage 영속 저장 테스트

CLI end-to-end 테스트는 코드에서 확인되지 않음.

## Step 12에서 달라진 점

Step 12는 새 CLI 명령을 추가하지 않고 저장소를 영속 저장소로 전환한다.

| 구분 | Step 11 | Step 12 |
| --- | --- | --- |
| 기본 CLI 저장소 | `MemoryStorage` | `SledStorage` |
| 테스트 개수 | 57개 | 58개 |
| 새 테스트 | 없음 | `persists_tasks_with_sled_storage` |
| 검증하는 것 | 기존 흐름 보강 | repository를 다시 열어도 Todo가 남는지 |

## Step 11에서 달라진 점

Step 11은 새 명령을 추가하는 단계가 아니다. 이미 있는 코드가 깨지지 않도록 테스트를 늘리는 단계다.

| 구분 | Step 10 | Step 11 |
| --- | --- | --- |
| 기능 | REPL 모드까지 구현 | 기능 변화 없음 |
| 테스트 개수 | 46개 | 57개 |
| 새로 보이는 테스트 문법 | `Cursor`, `Vec<u8>` | `matches!` |
| 주로 보강한 위치 | REPL 기본 흐름 | parser 실패, domain 생성, REPL 에러 지속, GlueSQL 에러 타입 |

## 대표 domain 테스트 해설

| 항목 | 내용 |
| --- | --- |
| 테스트 파일 | `src/task.rs` |
| 테스트 대상 | `Task::new`, `TaskStats::new` |
| 테스트가 검증하는 것 | 새 Todo의 기본 `done` 값과 통계 계산 |
| 실패하면 의심해야 할 코드 | `Task::new`, `TaskStats::new` |

핵심 코드:

```rust
#[test]
fn task_stats_new_calculates_todo_count() {
    let stats = TaskStats::new(5, 2);

    assert_eq!(stats.total, 5);
    assert_eq!(stats.done, 2);
    assert_eq!(stats.todo, 3);
}
```

## 대표 parser 테스트 해설

| 항목 | 내용 |
| --- | --- |
| 테스트 파일 | `src/cli.rs` |
| 테스트 대상 | `parse_args` |
| 테스트가 검증하는 것 | `add`, `search`, `stats`, `sql`, `repl` 같은 CLI 문자열이 `Command`로 바뀌는지 |
| 실패하면 의심해야 할 코드 | `parse_args`, `require_next`, `Command` |
| 초심자가 추가할 수 있는 테스트 | `help` alias 또는 인자 부족 테스트 |

given/when/then 구조:

```text
given: ["rust-task", "add", "Rust"]
when: parse_args(...)
then: Ok(Command::Add { title: "Rust" })
```

## 대표 service 테스트 해설

| 항목 | 내용 |
| --- | --- |
| 테스트 파일 | `src/service.rs` |
| 테스트 대상 | `TaskService::add`, `TaskService::list`, `TaskService::done`, `TaskService::delete`, `TaskService::search`, `TaskService::stats`, `TaskService::execute_sql` |
| 테스트가 검증하는 것 | service가 repository trait 메서드에 요청을 위임하는지 |
| 실패하면 의심해야 할 코드 | `TaskService`, `FakeTaskRepository`, `TaskRepository` |
| 초심자가 추가할 수 있는 테스트 | 없는 id가 service를 거쳐 `Err(AppError::NotFound)`로 올라오는지 |

given/when/then 구조:

```text
given: FakeTaskRepository를 넣은 TaskService
when: service.add("Rust")
then: Task가 추가되고 service.list()로 확인 가능
```

핵심 코드:

```rust
#[test]
fn add_delegates_to_repository() {
    let repository = FakeTaskRepository::new(Vec::new());
    let mut service = TaskService::new(repository);

    let task = service.add("Rust".to_string());

    assert_eq!(task, Ok(Task::new(1, "Rust".to_string())));
    assert_eq!(service.list(), Ok(vec![Task::new(1, "Rust".to_string())]));
}
```

## 대표 error 테스트 해설

| 항목 | 내용 |
| --- | --- |
| 테스트 파일 | `src/error.rs` |
| 테스트 대상 | `AppError` |
| 테스트가 검증하는 것 | `Display` 출력, `From<std::io::Error>` 변환, GlueSQL error 출력, Unsupported 출력 |
| 실패하면 의심해야 할 코드 | `impl fmt::Display for AppError`, `impl From<std::io::Error> for AppError` |

핵심 코드:

```rust
#[test]
fn displays_not_found_message() {
    let error = AppError::NotFound(404);

    assert_eq!(error.to_string(), "Task not found: 404");
}
```

## 대표 JSON repository 테스트 해설

| 항목 | 내용 |
| --- | --- |
| 테스트 파일 | `src/repository/mod.rs` |
| 테스트 대상 | `JsonTaskRepository::new`, `TaskRepository::add`, `TaskRepository::search`, `TaskRepository::stats` |
| 테스트가 검증하는 것 | 보존된 JSON 저장소의 Todo 저장/로드, 제목 검색, 통계 계산 |
| 실패하면 의심해야 할 코드 | `JsonTaskRepository::new`, `add`, `save_tasks`, `load_tasks`, `Task::new` |
| 초심자가 추가할 수 있는 테스트 | 없는 파일, 잘못된 JSON, 없는 id 처리 |

given/when/then 구조:

```text
given: 임시 JSON 파일 경로와 빈 JsonTaskRepository
when: repository.add("Rust")
then: 파일을 다시 읽은 repository에 Task가 들어 있음
```

핵심 코드:

```rust
#[test]
fn adds_task_and_saves_to_json_file() {
    let path = unique_test_path("add");
    let mut repository = JsonTaskRepository::new(&path).unwrap();

    let task = repository.add("Rust".to_string()).unwrap();
    let mut reloaded = JsonTaskRepository::new(&path).unwrap();
    let _ = fs::remove_file(&path);

    assert_eq!(task, Task::new(1, "Rust".to_string()));
    assert_eq!(reloaded.find_all(), Ok(vec![task]));
}
```

## 테스트 코드 읽는 법

```rust
let command = parse_args(args(&["rust-task", "done", "1"]));

assert_eq!(command, Ok(Command::Done { id: 1 }));
```

`assert_eq!`는 JUnit의 `assertEquals` 또는 AssertJ의 `assertThat(...).isEqualTo(...)`와 비슷한 테스트 assertion이다. 내부 비교에는 Rust의 `==`가 쓰이고, `Command`와 `Task`가 `==`로 비교되려면 `#[derive(PartialEq, Eq)]`가 필요하다.

| Rust | Java/JUnit | AssertJ |
| --- | --- | --- |
| `assert_eq!(a, b)` | `assertEquals(a, b)` | `assertThat(a).isEqualTo(b)` |
| `PartialEq` | `equals()` 구현 | `isEqualTo`가 의존하는 동등성 |

## `assert!(matches!(...))` 읽는 법

Step 11에서 GlueSQL 실패 타입을 확인할 때 아래 코드가 추가됐다.

```rust
assert!(matches!(result, Err(AppError::GlueSql(_))));
```

읽는 순서:

```text
result 값이
-> Err(...) 형태인지 보고
-> 그 안이 AppError::GlueSql(...)인지 확인한다.
```

`_`는 "안의 값은 지금 중요하지 않다"는 뜻이다. GlueSQL 내부 에러 메시지는 라이브러리 버전에 따라 달라질 수 있으므로, 이 테스트는 메시지 전체가 아니라 프로젝트가 올바른 에러 variant로 감쌌는지만 확인한다.

## 테스트 실패 시 확인할 것

- `Command`에 `PartialEq`가 있는가
- `Task`에 `PartialEq`가 있는가
- `Task`에 `Serialize`, `Deserialize`가 있는가
- `parse_args`가 `Ok(Command)`와 `Err(AppError)` 중 무엇을 반환하는가
- `TaskService`가 `TaskRepository` trait bound를 사용하는가
- `JsonTaskRepository::new`가 파일 없음과 JSON parsing 실패를 다르게 처리하는가
- `GlueSqlTaskRepository::new`가 테스트용 `MemoryStorage` table을 만들고, `GlueSqlTaskRepository::persistent`가 SledStorage table을 준비하는가
- GlueSQL `Payload::Select`의 row가 `Task`로 변환되는가
- GlueSQL `Payload`가 `SqlResult`로 변환되는가
- REPL 테스트가 `Cursor` 입력과 `Vec<u8>` 출력을 사용하는가
- `Task::new` 기본 `done` 값이 테스트 기대와 같은가
- `repository.add`에서 `task.clone()`을 제거하지 않았는가
- repository 테스트가 실제 `tasks.json`이 아니라 임시 파일 경로를 사용하는가

## 초심자가 추가할 수 있는 테스트

repository 없는 id 테스트 예시:

```rust
#[test]
fn missing_done_returns_error() {
    let path = unique_test_path("missing-done");
    let mut repository = JsonTaskRepository::new(&path).unwrap();

    let result = repository.mark_done(404);

    assert_eq!(result, Err(AppError::NotFound(404)));
}
```

parser 인자 부족 테스트 예시:

```rust
#[test]
fn missing_add_title_returns_error() {
    let command = parse_args(args(&["rust-task", "add"]));

    assert_eq!(
        command,
        Err(AppError::InvalidCommand(
            "Usage: rust-task add \"할 일\"".to_string()
        ))
    );
}
```

## 현재 테스트 개수

현재 Step 13에서도 기능 테스트 수는 총 58개다. Step 13은 새 기능 테스트를 추가하기보다 기존 테스트가 현재 코드와 문서 설명을 뒷받침하는지 확인한다.

- `src/task.rs`: domain 테스트 2개
- `src/cli.rs`: CLI parser 테스트 16개
- `src/error.rs`: error 테스트 5개
- `src/service.rs`: service 테스트 7개
- `src/repl.rs`: REPL 테스트 5개
- `src/repository/mod.rs`: repository 테스트 9개
- `src/repository/gluesql_repository.rs`: GlueSQL repository 테스트 13개
- `src/main.rs`: help 흐름 보조 테스트 1개

## 테스트 추가 실습

`cargo run -- help`, `cargo run -- -h`, `cargo run -- --help`에 해당하는 parser 입력이 모두 `Ok(Command::Help)`인지 확인하는 테스트를 추가한다.
## 대표 GlueSQL repository 테스트 해설

| 항목 | 내용 |
| --- | --- |
| 테스트 파일 | `src/repository/gluesql_repository.rs` |
| 테스트 대상 | `GlueSqlTaskRepository::new`, `GlueSqlTaskRepository::persistent`, `add`, `find_all`, `mark_done`, `delete`, `search`, `stats`, `execute_sql` |
| 테스트가 검증하는 것 | GlueSQL `MemoryStorage`에서 SQL Todo 기능이 동작하고, `SledStorage`에서 데이터가 유지되는지 |
| 실패하면 의심해야 할 코드 | `execute`, `execute_sql`, `payload_to_sql_result`, `select_tasks`, `row_to_task`, `select_count`, SQL 문자열 |
| 초심자가 추가할 수 있는 테스트 | 제목에 작은따옴표가 들어간 경우, 없는 id 처리 |

given/when/then 구조:

```text
given: 새 GlueSqlTaskRepository
when: repository.add("Rust") 후 repository.find_all()
then: 같은 repository 인스턴스 안에서 Task가 조회됨
```

핵심 코드:

```rust
#[test]
fn adds_and_lists_tasks_with_gluesql() {
    let mut repository = GlueSqlTaskRepository::new().unwrap();

    let task = repository.add("Rust".to_string()).unwrap();

    assert_eq!(task, Task::new(1, "Rust".to_string()));
    assert_eq!(repository.find_all(), Ok(vec![task]));
}
```

## 대표 SQL 실행 테스트 해설

| 항목 | 내용 |
| --- | --- |
| 테스트 파일 | `src/repository/gluesql_repository.rs` |
| 테스트 대상 | `GlueSqlTaskRepository::execute_sql` |
| 테스트가 검증하는 것 | 사용자가 입력한 SELECT/INSERT/UPDATE/DELETE SQL이 `SqlResult`로 변환되는지 |
| 실패하면 의심해야 할 코드 | `execute_sql`, `payload_to_sql_result`, `value_to_string`, `print_sql_results` |

핵심 코드:

```rust
let results = repository
    .execute_sql("SELECT id, title, done FROM tasks".to_string())
    .unwrap();
```

읽는 법:

```text
SQL 문자열을 repository에 전달한다.
-> GlueSQL이 Payload를 반환한다.
-> payload_to_sql_result가 SqlResult::Select로 바꾼다.
-> 테스트가 labels와 rows를 비교한다.
```

## 대표 REPL 테스트 해설

| 항목 | 내용 |
| --- | --- |
| 테스트 파일 | `src/repl.rs` |
| 테스트 대상 | `run_repl_with_io` |
| 테스트가 검증하는 것 | `.schema`, `.exit`, `.quit`, REPL 안 SQL 실행 |
| 실패하면 의심해야 할 코드 | `run_repl_with_io`, `write_sql_results`, `Command::Repl` |

핵심 코드:

```rust
let input = Cursor::new(
    "INSERT INTO tasks VALUES (1, 'Rust', FALSE);\nSELECT id, title, done FROM tasks;\n.quit\n",
);
let mut output = Vec::new();
```

읽는 법:

```text
Cursor로 사용자가 칠 입력을 미리 만든다.
-> Vec<u8>에 출력이 쌓이게 한다.
-> run_repl_with_io를 실행한다.
-> output 문자열에 insert/select 결과가 있는지 확인한다.
```
