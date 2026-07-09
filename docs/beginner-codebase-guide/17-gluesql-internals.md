# GlueSQL 내부 흐름과 Storage Adapter

## 이 문서의 목적

이 문서는 Notion의 GlueSQL 분석 리포트에서 다룬 Parser, Planner, Executor, Store, Storage Adapter 관점을 현재 `rust-task` 코드와 연결한다.

현재 프로젝트는 GlueSQL 내부 코드를 직접 수정하지 않는다. `src/repository/gluesql_repository.rs`에서 `Glue::execute`를 호출해 SQL 실행 흐름을 간접적으로 사용하고, 테스트로 storage별 차이를 관찰한다.

## 현재 코드에서 보이는 실행 흐름

파일 경로:

- `src/repository/gluesql_repository.rs`
- `src/repository/mod.rs`
- `src/main.rs`

현재 SQL 실행의 입구는 `GlueSqlTaskRepository::execute`다.

```rust
fn execute(&mut self, sql: impl AsRef<str>) -> Result<Vec<Payload>, AppError> {
    block_on(self.glue.execute(sql)).map_err(|error| AppError::GlueSql(error.to_string()))
}
```

코드 해석:

- `self.glue.execute(sql)`: GlueSQL public API에 SQL 문자열을 넘긴다.
- `block_on(...)`: GlueSQL API가 async이므로 동기 CLI 코드에서 끝까지 기다린다.
- `map_err(...)`: GlueSQL 내부 실패를 프로젝트 공통 에러인 `AppError::GlueSql`로 바꾼다.

프로젝트에서의 역할:

```text
사용자 SQL
-> GlueSqlTaskRepository::execute
-> Glue::execute
-> GlueSQL 내부 Parser / Planner / Executor
-> Storage trait 호출
-> MemoryStorage 또는 SledStorage
-> Payload
-> SqlResult
```

중요한 한계:

현재 프로젝트는 Parser, Planner, Executor 함수를 직접 호출하지 않는다. `Glue::execute` 내부에서 일어나는 일로 보고, 프로젝트 코드에서는 `Payload` 결과와 에러만 관찰한다.

## Parser, Planner, Executor를 어떻게 이해하면 되나

GlueSQL 내부 흐름은 개념적으로 아래와 같다.

```text
SQL String
-> Parser
-> AST
-> Planner
-> Plan
-> Executor
-> Store trait
-> Storage Backend
```

현재 프로젝트 코드와 연결하면 다음과 같다.

| GlueSQL 계층 | 역할 | 현재 프로젝트에서 보이는 지점 |
| --- | --- | --- |
| Parser | SQL 문자열을 구문 구조로 해석 | 직접 호출하지 않음. `Glue::execute` 내부 동작 |
| Planner | 실행 가능한 plan으로 변환 | 직접 호출하지 않음. `Glue::execute` 내부 동작 |
| Executor | plan을 실행하고 storage를 호출 | 직접 호출하지 않음. `Payload`로 결과 관찰 |
| Store trait | storage가 제공해야 하는 읽기/쓰기 인터페이스 | `GStore`, `GStoreMut`, `Planner` trait bound |
| Storage | 실제 데이터 저장 구현체 | `MemoryStorage`, `SledStorage` |

## `GlueSqlTaskRepository<S>`의 trait bound

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

- `S`: GlueSQL storage 타입 자리다.
- `MemoryStorage`: 테스트에서 사용하는 기본 generic 타입이다.
- `SledStorage`: CLI 실행에서 사용하는 영속 storage 타입이다.
- `GStore`: GlueSQL이 데이터를 읽기 위해 요구하는 기능 묶음이다.
- `GStoreMut`: GlueSQL이 데이터를 변경하기 위해 요구하는 기능 묶음이다.
- `Planner`: GlueSQL이 SQL 실행 계획을 준비할 때 storage와 연결되는 기능 묶음이다.

프로젝트에서의 역할:

`GlueSqlTaskRepository`는 storage가 무엇인지 몰라도 같은 Todo 메서드를 제공한다. 단, 그 storage가 GlueSQL 실행에 필요한 trait을 구현해야 한다.

```text
GlueSqlTaskRepository<MemoryStorage>
-> 테스트용 in-memory SQL table

GlueSqlTaskRepository<SledStorage>
-> CLI 기본 영속 SQL table
```

초심자가 수정할 수 있는 지점:

새 GlueSQL storage를 실제로 도입하려면 `Cargo.toml` feature/dependency, 생성자, 테스트, 문서를 함께 바꿔야 한다. Step 15에서는 새 storage를 도입하지 않는다.

## Storage별 기능 차이

GlueSQL은 SQL engine과 storage를 분리한다. 그래서 모든 storage가 같은 기능을 지원한다고 보면 안 된다.

| Storage | 현재 프로젝트에서의 상태 | 관찰한 기능 | 주의할 점 |
| --- | --- | --- | --- |
| `MemoryStorage` | 테스트용으로 직접 사용 | CRUD, search, stats, SQL 실행 | 명시적 `BEGIN`은 지원하지 않는다. |
| `SledStorage` | CLI 기본 저장소 | 영속 저장, `BEGIN`, `COMMIT`, `ROLLBACK`, snapshot, write lock | 같은 path를 동시에 두 번 열지 말고 `clone()`을 사용한다. |
| `SharedMemoryStorage` | 코드에서 확인되지 않음 | Notion 분석 기준 참고 대상 | `Arc<RwLock<MemoryStorage>>` 기반 동시 접근 패턴을 공부할 때 본다. |
| `JsonStorage` | 코드에서 확인되지 않음 | Notion 분석 기준 참고 대상 | 현재 프로젝트의 `JsonTaskRepository`와 다르다. |
| `MongoStorage` | 코드에서 확인되지 않음 | Notion 분석 기준 참고 대상 | Document DB 위에 SQL layer를 얹는 구조 분석 후보이다. |
| `CompositeStorage` | 코드에서 확인되지 않음 | Notion 분석 기준 참고 대상 | 여러 storage를 묶는 구조 분석 후보이다. |

## 현재 테스트가 보여주는 storage 경계

파일 경로:

- `src/repository/mod.rs`
- `src/repository/gluesql_repository.rs`

### SQL 미지원 repository

`JsonTaskRepository`는 이 프로젝트가 직접 만든 JSON 저장소다. GlueSQL storage가 아니므로 SQL 실행을 지원하지 않는다.

```rust
fn execute_sql(&mut self, _sql: String) -> Result<Vec<SqlResult>, AppError> {
    Err(AppError::Unsupported(
        "SQL command is only supported by GlueSqlTaskRepository".to_string(),
    ))
}
```

테스트는 이 경계를 고정한다.

```rust
#[test]
fn json_repository_reports_sql_as_unsupported() {
    let path = unique_test_path("unsupported-sql");
    let mut repository = JsonTaskRepository::new(&path).unwrap();

    let result = repository.execute_sql("SELECT * FROM tasks;".to_string());
    let _ = fs::remove_file(&path);

    assert_eq!(
        result,
        Err(AppError::Unsupported(
            "SQL command is only supported by GlueSqlTaskRepository".to_string()
        ))
    );
}
```

### MemoryStorage transaction 미지원

`MemoryStorage`는 Todo CRUD 테스트에는 충분하지만 명시적 transaction은 지원하지 않는다.

```rust
#[test]
fn memory_storage_rejects_explicit_transactions() {
    let mut repository = GlueSqlTaskRepository::new().unwrap();

    let result = repository.execute_sql("BEGIN;".to_string());

    assert!(
        matches!(result, Err(AppError::GlueSql(message)) if message.contains("transaction is not supported"))
    );
}
```

### SledStorage transaction 관찰

`SledStorage`는 명시적 `BEGIN`, `COMMIT`, `ROLLBACK`을 관찰할 수 있다.

```rust
#[test]
fn sled_storage_commits_explicit_transaction() {
    let path = unique_sled_path("commit");
    let _ = fs::remove_dir_all(&path);
    let mut repository = GlueSqlTaskRepository::persistent(&path).unwrap();

    repository
        .execute_sql(
            "
            BEGIN;
            INSERT INTO tasks VALUES (1, 'committed', FALSE);
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
```

Nested transaction은 실패로 올라온다.

```rust
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
```

## `Payload`와 `SqlResult`

GlueSQL의 실행 결과는 `Payload`로 돌아온다. 프로젝트는 이를 CLI 출력에 맞는 `SqlResult`로 바꾼다.

```rust
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
        other => Ok(SqlResult::Message(format!("{other:?}"))),
    }
}
```

프로젝트에서의 역할:

`src/main.rs`는 GlueSQL `Payload`를 직접 알지 않는다. `print_sql_results`는 `SqlResult`만 보고 출력한다.

```text
GlueSQL Payload
-> payload_to_sql_result
-> SqlResult
-> print_sql_results
-> terminal
```

## 기여 전략과 현재 프로젝트의 연결

Notion 분석 리포트의 기여 전략은 "큰 기능 구현보다 테스트와 문서부터 시작"이다. 현재 프로젝트도 같은 흐름을 따른다.

| 기여 전략 | 현재 프로젝트에서의 대응 |
| --- | --- |
| Test Suite 먼저 읽기 | `src/repository/gluesql_repository.rs` 테스트로 storage 차이 관찰 |
| Storage trait 이해 | `GStore + GStoreMut + Planner` trait bound 문서화 |
| 작은 문서 PR | README, 초심자 가이드, 단계 문서 정합성 유지 |
| 작은 기능 테스트 | `BEGIN`, `COMMIT`, nested transaction 실패 테스트 추가 |

## 이후 단계 예정

현재 코드에서 확인되지 않음:

- 직접 custom GlueSQL storage 구현
- GlueSQL upstream PR 작성
- `SharedMemoryStorage` dependency 추가
- `JsonStorage`, `MongoStorage`, `CompositeStorage` 실제 연결
- Parser, Planner, Executor 내부 함수 직접 호출

이 항목들은 이후 단계 예정으로만 둔다.
