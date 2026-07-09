# 초심자 실습

## 실습 이름: 응답 메시지 변경

## 난이도

쉬움

## 목표

`Added:`를 `Task added:`로 바꾼다.

## 배우는 개념

`println!`

## 수정할 파일

`src/main.rs`

## 수정 전 코드

```rust
println!("Added:");
```

## 수정 후 코드

```rust
println!("Task added:");
```

## 왜 이렇게 수정하는가

출력 메시지만 바꾸는 변경이라 Step 6의 service/repository/error 구조를 건드리지 않는다.

## 동작 확인 방법

```bash
cargo run -- add "Rust 공부"
```

## 실패할 경우 확인할 것

문자열 따옴표와 세미콜론을 확인한다.

## 관련 문서

[04-feature-flows.md](04-feature-flows.md)

## 실습 이름: parser 테스트 추가

## 난이도

중간

## 목표

`add` 명령에 제목이 없을 때 `Err(AppError::InvalidCommand)`가 나오는지 테스트한다.

## 배우는 개념

`Result`, `assert_eq!`, `Command`

## 수정할 파일

`src/cli.rs`

## 수정 전 코드

코드에서 확인되지 않음.

## 수정 후 코드

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

## 왜 이렇게 수정하는가

Step 2에서 들어온 CLI parser는 Step 11에서도 그대로 중요하다. 이 테스트는 CLI 문자열을 `Result<Command, AppError>`로 바꾸는 흐름을 확인한다.

## 동작 확인 방법

```bash
cargo test missing_add_title_returns_error
```

## 실패할 경우 확인할 것

테스트가 `src/cli.rs`의 `mod tests` 안에 있는지 확인한다.

## 관련 문서

[11-testing.md](11-testing.md)

## 실습 이름: 빈 목록 메시지 추가

## 난이도

중간

## 목표

`list` 결과가 비어 있을 때 `No tasks`를 출력한다.

## 배우는 개념

slice, `is_empty`, `if`

## 수정할 파일

`src/main.rs`

## 수정 전 코드

```rust
fn print_tasks(tasks: &[Task]) {
    for task in tasks {
        print_task(task);
    }
}
```

## 수정 후 코드

```rust
fn print_tasks(tasks: &[Task]) {
    if tasks.is_empty() {
        println!("No tasks");
    } else {
        for task in tasks {
            print_task(task);
        }
    }
}
```

## 왜 이렇게 수정하는가

현재 `list`는 빈 Vec면 `List:`만 출력한다. 빈 상태를 명확히 보여주려면 출력 함수를 바꾸면 된다.

## 동작 확인 방법

```bash
cargo run -- list
```

## 실습 이름: SledStorage rollback 테스트 추가

## 난이도

중간

## 목표

`src/repository/gluesql_repository.rs`에 transaction 안에서 `INSERT`한 뒤 `ROLLBACK`하면 Todo가 남지 않는 테스트를 직접 읽고 변형한다.

## 배우는 개념

`BEGIN`, `ROLLBACK`, `Result`, `assert_eq!`, GlueSQL storage별 transaction 차이

## 수정할 파일

`src/repository/gluesql_repository.rs`

## 수정 전 코드

```rust
repository
    .execute_sql(
        "
        BEGIN;
        INSERT INTO tasks VALUES (1, 'temporary', FALSE);
        ROLLBACK;
        "
        .to_string(),
    )
    .unwrap();
```

## 수정 후 코드

```rust
repository
    .execute_sql(
        "
        BEGIN;
        INSERT INTO tasks VALUES (1, 'temporary', FALSE);
        COMMIT;
        "
        .to_string(),
    )
    .unwrap();
```

## 왜 이렇게 수정하는가

`ROLLBACK`은 transaction 안의 변경을 취소하고, `COMMIT`은 변경을 확정한다. 같은 테스트 구조에서 마지막 SQL만 바꿔도 결과가 달라진다.

## 동작 확인 방법

```bash
cargo test sled_storage_rolls_back_uncommitted_insert
```

## 주의

`MemoryStorage`는 명시적 transaction을 지원하지 않는다. 이 실습은 `GlueSqlTaskRepository::persistent`로 만든 `SledStorage` repository에서 확인한다.

## 실습 이름: Storage 비교표에서 코드 증거 찾기

## 난이도

쉬움

## 목표

[20-storage-comparison.md](20-storage-comparison.md)의 표를 보고 현재 코드에 실제로 등장하는 storage와 문서 비교 대상 storage를 구분한다.

## 배우는 개념

GlueSQL storage, repository 구현체, 코드에서 확인되지 않음

## 확인할 파일

- `src/repository/mod.rs`
- `src/repository/gluesql_repository.rs`
- `docs/beginner-codebase-guide/20-storage-comparison.md`

## 확인할 코드

```rust
pub struct JsonTaskRepository {
    path: PathBuf,
    tasks: Vec<Task>,
}
```

```rust
let storage = MemoryStorage::default();
```

```rust
SledStorage::new(path)
```

## 왜 이렇게 확인하는가

Step 18에서는 새 storage를 추가하지 않는다. 대신 `JsonTaskRepository`, `MemoryStorage`, `SledStorage`는 현재 코드에서 확인하고, `SharedMemoryStorage`, `JsonStorage`, `MongoStorage`, `CompositeStorage`는 현재 코드에 없는 비교 대상으로 구분한다.

## 동작 확인 방법

```bash
rg -n "JsonTaskRepository|MemoryStorage|SledStorage|SharedMemoryStorage|JsonStorage|MongoStorage|CompositeStorage" src docs/beginner-codebase-guide/20-storage-comparison.md
```

## 실패할 경우 확인할 것

`SharedMemoryStorage`, `JsonStorage`, `MongoStorage`, `CompositeStorage`는 현재 `src/` 코드에 직접 등장하지 않아야 한다.
