# 테스트 파일

Step 40 현재 테스트는 100개다. 관계형 repository 테스트는 Project CRUD/통계, priority, Tag 연결/중복/해제/filter, 상세 JOIN, 삭제 정리, Seed idempotency, Sled secondary-index Planner 위임과 Step 18 schema migration 후 column-safe INSERT를 검증한다.

## 포함된 파일 목록

- `src/cli.rs`의 `#[cfg(test)] mod tests`
- `src/task.rs`의 `#[cfg(test)] mod tests`
- `src/error.rs`의 `#[cfg(test)] mod tests`
- `src/service/mod.rs`의 `#[cfg(test)] mod tests`
- `src/repl.rs`의 `#[cfg(test)] mod tests`
- `src/repository/mod.rs`의 `#[cfg(test)] mod tests`
- `src/repository/gluesql_repository.rs`의 `#[cfg(test)] mod tests`
- `src/main.rs`의 `#[cfg(test)] mod tests`

## 이 파일 묶음의 역할

Step 2의 CLI parser, Step 3에서 시작한 JSON 저장/로드, Step 4의 repository 동작, Step 5의 service 위임 흐름, Step 6의 custom error, Step 8의 GlueSQL repository, Step 9의 SQL 실행 모드, Step 10의 REPL 모드가 기대대로 동작하는지 검증한다. Step 11에서는 이 흐름을 더 촘촘한 테스트로 보강했고, Step 12에서는 SledStorage 영속 저장 테스트를 추가했다.

## 전체 연결 관계

```text
cargo test
-> src/cli.rs tests -> parse_args -> Command
-> src/task.rs tests -> Task::new / TaskStats::new
-> src/error.rs tests -> AppError -> Display / From
-> src/service/mod.rs tests -> TaskService -> FakeTaskRepository
-> src/repl.rs tests -> run_repl_with_io -> TaskService -> GlueSqlTaskRepository
-> src/repository/mod.rs tests -> JsonTaskRepository -> tasks.json 형식의 임시 JSON file
-> src/repository/gluesql_repository.rs tests -> GlueSqlTaskRepository -> GlueSQL MemoryStorage
-> src/main.rs tests -> help 흐름 보조 확인
```

## 파일 경로

`src/task.rs`

### 이 파일의 역할

`Task::new`가 기본 Todo 값을 올바르게 만들고, `TaskStats::new`가 전체/완료/미완료 개수를 계산하는지 테스트한다.

### 핵심 코드 블록

```rust
#[test]
fn task_new_sets_id_title_and_default_done() {
    let task = Task::new(1, "Rust".to_string());

    assert_eq!(task.id, 1);
    assert_eq!(task.title, "Rust");
    assert!(!task.done);
}
```

### 코드 블록별 해설

- `Task::new(...)`: Todo 하나를 만드는 생성 함수다.
- `task.id`, `task.title`: 생성자가 받은 값을 그대로 넣었는지 확인한다.
- `assert!(!task.done)`: 새 Todo는 아직 완료되지 않았는지 확인한다.

## 파일 경로

`src/error.rs`

### 이 파일의 역할

`AppError`의 출력 메시지와 에러 변환을 테스트한다.

### 핵심 코드 블록

```rust
#[test]
fn displays_invalid_command_message_without_prefix() {
    let error = AppError::InvalidCommand("Unknown command: nope".to_string());

    assert_eq!(error.to_string(), "Unknown command: nope");
}
```

## 파일 경로

`src/service/mod.rs`

### 이 파일의 역할

`TaskService`가 `TaskRepository` trait에 의존해 add/list/done/delete/search/stats/sql 요청을 위임하는지 테스트한다.

### 핵심 코드 블록

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

### 코드 블록별 해설

- `FakeTaskRepository`: 테스트용 repository 구현체
- `TaskService::new(repository)`: service에 fake repository를 넣는다.
- `service.add(...)`: service 메서드를 호출한다.
- `service.list()`: add 결과가 repository에 반영됐는지 확인한다.

## 파일 경로

`src/cli.rs`

### 이 파일의 역할

`parse_args`가 CLI 문자열을 올바른 `Command`로 바꾸는지 테스트한다.

### 핵심 코드 블록

```rust
#[test]
fn parses_add_command() {
    let command = parse_args(args(&["rust-task", "add", "Rust"]));

    assert_eq!(
        command,
        Ok(Command::Add {
            title: "Rust".to_string()
        })
    );
}
```

### 코드 블록별 해설

- `args(&["rust-task", "add", "Rust"])`: 테스트용 CLI 인자 Vec를 만든다.
- `parse_args(...)`: 실제 parser 호출
- `Ok(Command::Add { ... })`: 기대하는 성공 결과
- `assert_eq!`: 실제 결과와 기대 결과 비교

## 파일 경로

`src/repl.rs`

### 이 파일의 역할

REPL이 `.schema`, `.exit`, `.quit`, 빈 줄, SQL line, SQL 실패 후 계속 입력받는 흐름을 처리하는지 테스트한다.

### 핵심 코드 블록

```rust
let input = Cursor::new(
    "INSERT INTO tasks VALUES (1, 'Rust', FALSE);\nSELECT id, title, done FROM tasks;\n.quit\n",
);
let mut output = Vec::new();

run_repl_with_io(&mut service, input, &mut output).unwrap();
```

### 코드 블록별 해설

- `Cursor::new(...)`: 테스트용 가짜 입력을 만든다.
- `Vec::new()`: 테스트용 가짜 출력 버퍼를 만든다.
- `run_repl_with_io`: 실제 stdin/stdout 대신 테스트 입력/출력을 넣어 REPL을 검증한다.

### Step 11에서 추가된 REPL 실패 흐름 테스트

```rust
#[test]
fn continues_after_sql_error() {
    let mut service = TaskService::new(GlueSqlTaskRepository::new().unwrap());
    let input = Cursor::new("SELECT * FROM missing_table;\n.schema\n.exit\n");
    let mut output = Vec::new();

    run_repl_with_io(&mut service, input, &mut output).unwrap();

    let output = String::from_utf8(output).unwrap();
    assert!(output.contains("Error:"));
    assert!(output.contains("CREATE TABLE tasks"));
}
```

읽는 법:

```text
잘못된 SQL을 먼저 입력한다.
-> REPL은 에러를 출력한다.
-> 프로그램을 종료하지 않고 다음 `.schema` 명령을 처리한다.
```

## 파일 경로

`src/repository/mod.rs`

### 이 파일의 역할

보존된 `JsonTaskRepository`가 Todo를 추가, 조회, 완료, 삭제하고 JSON 파일에 저장하는지 테스트한다.

### 핵심 코드 블록

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

### 코드 블록별 해설

- `unique_test_path("add")`: 실제 `tasks.json`을 건드리지 않기 위한 임시 파일 경로를 만든다.
- `JsonTaskRepository::new(&path)`: 임시 파일을 사용하는 repository를 만든다.
- `repository.add(...)`: Todo를 추가하고 저장한다.
- `JsonTaskRepository::new(&path)`: 같은 파일을 다시 읽어 저장 결과를 확인한다.
- `fs::remove_file(&path)`: 테스트가 만든 임시 파일을 지운다.

## 파일 경로

`src/repository/gluesql_repository.rs`

### 이 파일의 역할

`GlueSqlTaskRepository`가 GlueSQL `MemoryStorage`에서 add/list/done/delete/search/stats/sql을 처리하는지 테스트하고, `SledStorage`에서 데이터를 다시 열어도 유지되는지 테스트한다. Step 14에서는 `MemoryStorage`와 `SledStorage`의 transaction 차이, rollback, snapshot, write lock 충돌도 같은 파일에서 관찰한다.

### 핵심 코드 블록

```rust
#[test]
fn adds_and_lists_tasks_with_gluesql() {
    let mut repository = GlueSqlTaskRepository::new().unwrap();

    let task = repository.add("Rust".to_string()).unwrap();

    assert_eq!(task, Task::new(1, "Rust".to_string()));
    assert_eq!(repository.find_all(), Ok(vec![task]));
}
```

### 코드 블록별 해설

- `GlueSqlTaskRepository::new()`: 테스트마다 새 in-memory DB와 `tasks` table을 만든다.
- `repository.add(...)`: SQL `INSERT` 흐름을 실행한다.
- `repository.find_all()`: 같은 repository 인스턴스 안에서 SQL `SELECT`로 조회한다.
- `MemoryStorage`: 테스트가 끝나면 데이터가 사라진다.

### Step 12에서 추가된 SledStorage 영속 저장 테스트

```rust
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
```

읽는 법:

```text
임시 SledStorage 경로를 만든다.
-> 첫 번째 repository에서 Todo를 추가한다.
-> repository를 스코프 밖으로 보내 닫는다.
-> 같은 경로로 두 번째 repository를 만든다.
-> 이전 Todo가 다시 조회되는지 확인한다.
```

### Step 14에서 추가된 transaction과 동시성 관찰 테스트

```rust
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
            INSERT INTO tasks VALUES (2, 'after', FALSE);
            COMMIT;
            "
            .to_string(),
        )
        .unwrap();

    assert_eq!(
        reader.find_all(),
        Ok(vec![Task::new(1, "before".to_string())])
    );
}
```

읽는 법:

```text
writer와 reader가 같은 SledStorage를 clone해서 나눠 가진다.
-> reader가 BEGIN으로 읽기 시점을 잡는다.
-> writer가 새 Todo를 INSERT하고 COMMIT한다.
-> reader는 아직 자기 transaction 안이라 이전 snapshot만 본다.
-> reader가 COMMIT한 뒤에는 최신 Todo를 볼 수 있다.
```

`sled_repository_pair` helper는 같은 path를 `SledStorage::new(path)`로 두 번 열지 않는다. Sled는 같은 DB 디렉터리에 OS 파일 락을 잡기 때문에 동시에 두 번 열면 실패할 수 있다. 그래서 먼저 만든 `SledStorage`를 `clone()`해서 두 `GlueSqlTaskRepository<TracingStorage<SledStorage>>`에 넣는다.

```rust
fn sled_repository_pair(
    path: impl AsRef<std::path::Path>,
) -> (
    GlueSqlTaskRepository<TracingStorage<SledStorage>>,
    GlueSqlTaskRepository<TracingStorage<SledStorage>>,
) {
    let storage = SledStorage::new(path).unwrap();
    let first_storage = storage.clone();
    let second_storage = storage;
    let mut first = GlueSqlTaskRepository {
        glue: Glue::new(first_storage),
    };
    let mut second = GlueSqlTaskRepository {
        glue: Glue::new(second_storage),
    };

    first.create_tasks_table().unwrap();
    second.create_tasks_table().unwrap();

    (first, second)
}
```

Step 14의 나머지 테스트:

- `memory_storage_rejects_explicit_transactions`: `MemoryStorage`는 명시적 `BEGIN`을 지원하지 않음을 확인한다.
- `sled_storage_rolls_back_uncommitted_insert`: `ROLLBACK` 후 transaction 안의 insert가 사라지는지 확인한다.
- `sled_storage_reports_database_locked_for_competing_writes`: writer transaction이 열려 있으면 다른 writer가 `database is locked`를 받는지 확인한다.

### Step 11에서 추가된 GlueSQL 실패 타입 테스트

```rust
#[test]
fn invalid_sql_returns_gluesql_error() {
    let mut repository = GlueSqlTaskRepository::new().unwrap();

    let result = repository.execute_sql("SELECT * FROM missing_table".to_string());

    assert!(matches!(result, Err(AppError::GlueSql(_))));
}
```

코드 해석:

- `execute_sql(...)`: 잘못된 SQL을 GlueSQL에 보낸다.
- `matches!(...)`: 결과가 특정 패턴과 맞는지 확인한다.
- `Err(AppError::GlueSql(_))`: 실패가 `AppError::GlueSql` variant로 감싸져 올라오는지 확인한다.
- `_`: GlueSQL 내부 에러 메시지는 테스트에서 구체적으로 고정하지 않겠다는 뜻이다.

### SQL 실행 테스트

```rust
#[test]
fn executes_select_sql_with_gluesql() {
    let mut repository = GlueSqlTaskRepository::new().unwrap();
    repository.add("Rust".to_string()).unwrap();

    let results = repository
        .execute_sql("SELECT id, title, done FROM tasks".to_string())
        .unwrap();

    assert_eq!(
        results,
        vec![SqlResult::Select {
            labels: vec!["id".to_string(), "title".to_string(), "done".to_string()],
            rows: vec![vec!["1".to_string(), "Rust".to_string(), "false".to_string()]],
        }]
    );
}
```

코드 해석:

- `execute_sql(...)`: 사용자가 입력한 SQL 문자열을 GlueSQL에 전달한다.
- `SqlResult::Select`: SELECT 결과의 column 이름과 row 값을 문자열로 들고 있다.
- 이 테스트는 Step 9의 핵심인 `Payload -> SqlResult` 변환을 확인한다.

## 파일 경로

`src/main.rs`

### 이 파일의 역할

help 명령이 repository를 만들기 전에 처리되는 흐름을 보조로 확인한다.

### 핵심 코드 블록

```rust
#[test]
fn prints_help_before_repository_is_loaded() {
    let command = Command::Help;

    assert_eq!(command, Command::Help);
}
```

## 이 파일에서 사용된 언어 문법

attribute, macro, module, `use super::*`, `Result` 비교, enum 비교, trait method 호출, `matches!`

## 이 파일에서 사용된 프레임워크/라이브러리 기능

Rust 내장 test harness

## 초심자가 수정할 수 있는 부분

새 명령을 추가하면 `src/cli.rs`에 parser 테스트를 먼저 추가할 수 있다. 저장소 동작이 바뀌면 `src/repository/mod.rs`에 repository 테스트를 추가한다.

## 수정 전 코드

```rust
assert_eq!(command, Ok(Command::List));
```

## 수정 후 코드

```rust
assert_eq!(command, Ok(Command::Help));
```

## 수정 시 영향받는 파일

테스트 기대값만 바꾸면 production code에는 영향이 없다. 새 명령을 추가하면 `src/command.rs`, `src/cli.rs`, `src/main.rs`, 필요하면 `src/repository/mod.rs`를 함께 수정해야 한다.

## 이 파일을 이해한 뒤 알아야 하는 것

Step 18 당시에는 총 65개 테스트였다. Step 40 현재는 기존 동작과 관계형 기능, Seed idempotency, Sled Planner 위임, migration column-order 회귀 검증을 합쳐 총 100개 테스트가 있다. 정확한 목록은 `cargo test -- --list`로 확인한다.
