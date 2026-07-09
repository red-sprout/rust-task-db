# Storage별 기능 비교표

## 이 문서의 목적

이 문서는 현재 프로젝트에서 직접 사용하는 저장소와 GlueSQL 문서/분석에서 비교 대상으로 보는 storage를 한 표에 정리한다.

중요한 범위 제한:

- Step 18에서는 새 CLI 명령을 추가하지 않는다.
- 새 외부 crate를 추가하지 않는다.
- 기본 실행 저장소는 GlueSQL `SledStorage` 그대로다.
- `SharedMemoryStorage`, `JsonStorage`, `MongoStorage`, `CompositeStorage`는 현재 코드에 도입하지 않는다.
- 아래 표에서 "코드에서 확인되지 않음"은 현재 repository production/test code에 직접 연결하지 않았다는 뜻이다.

## 현재 프로젝트의 저장소 구분

현재 프로젝트에는 이름이 비슷한 저장소가 여럿 등장한다.

| 이름 | 종류 | 현재 사용 여부 | 헷갈리기 쉬운 점 |
| --- | --- | --- | --- |
| `JsonTaskRepository` | 프로젝트 자체 repository 구현체 | 보존된 코드와 테스트에서 사용 | GlueSQL `JsonStorage`가 아니다. |
| `MemoryStorage` | GlueSQL storage | 단위 테스트에서 사용 | 빠르지만 명시적 transaction은 지원하지 않는다. |
| `SledStorage` | GlueSQL storage | CLI 기본 실행 저장소 | `data/rust-task-db`에 데이터가 유지된다. |
| `SharedMemoryStorage` | GlueSQL storage | 코드에서 확인되지 않음 | `Arc<RwLock<MemoryStorage>>` 패턴 분석 대상이다. |
| `JsonStorage` | GlueSQL storage | 코드에서 확인되지 않음 | `tasks.json` 기반 `JsonTaskRepository`와 다르다. |
| `MongoStorage` | GlueSQL storage | 코드에서 확인되지 않음 | 외부 MongoDB 연결이 필요할 수 있어 현재 단계에 넣지 않는다. |
| `CompositeStorage` | GlueSQL storage | 코드에서 확인되지 않음 | 여러 storage를 조합하는 확장 주제다. |

## 기능 비교 매트릭스

| 저장소 | 현재 프로젝트 상태 | 영속성 | 동시 접근 관점 | transaction 관점 | SQL 실행 관점 | 학습 포인트 |
| --- | --- | --- | --- | --- | --- | --- |
| `JsonTaskRepository` | 보존된 구현체와 테스트 | `tasks.json` 파일 | 별도 동시성 제어 코드 없음 | 지원하지 않음 | `execute_sql`은 `AppError::Unsupported` | 앱 계층 `TaskRepository` 구현 예시 |
| `MemoryStorage` | GlueSQL repository 테스트 | 프로세스 메모리 | 테스트 한 repository 중심 | 명시적 `BEGIN` 실패 관찰 | Todo CRUD, search, stats, SQL 실행 테스트 | 빠른 단위 테스트와 GlueSQL 기본 흐름 |
| `SledStorage` | CLI 기본 저장소와 transaction 테스트 | `data/rust-task-db` 또는 테스트 path | 같은 path를 두 번 열지 않고 `clone()` 사용 | `BEGIN`, `COMMIT`, `ROLLBACK`, snapshot, write lock 관찰 | 현재 Todo 기능 전체 실행 | 영속 저장과 storage별 transaction 차이 |
| `SharedMemoryStorage` | 코드에서 확인되지 않음 | 메모리 | `Arc<RwLock<MemoryStorage>>` 기반 참고 대상 | 현재 코드에서 확인되지 않음 | GlueSQL storage로 분석 가능 | coarse-grained read/write lock 패턴 |
| `JsonStorage` | 코드에서 확인되지 않음 | JSON storage 계열 | 현재 코드에서 확인되지 않음 | 현재 코드에서 확인되지 않음 | GlueSQL storage로 분석 가능 | 파일 기반 SQL storage와 앱 JSON repository 차이 |
| `MongoStorage` | 코드에서 확인되지 않음 | 외부 MongoDB | 외부 DB 설정에 따름 | 현재 코드에서 확인되지 않음 | GlueSQL storage로 분석 가능 | document DB 위에 SQL layer를 얹는 구조 |
| `CompositeStorage` | 코드에서 확인되지 않음 | 조합한 storage에 따름 | 조합한 storage에 따름 | 조합한 storage에 따름 | GlueSQL storage로 분석 가능 | storage를 역할별로 나누는 adapter 구조 |

## 현재 코드 증거

### JsonTaskRepository

파일 경로: `src/repository/mod.rs`

```rust
fn execute_sql(&mut self, _sql: String) -> Result<Vec<SqlResult>, AppError> {
    Err(AppError::Unsupported(
        "SQL command is only supported by GlueSqlTaskRepository".to_string(),
    ))
}
```

코드 해석:

- `JsonTaskRepository`는 Todo CRUD/search/stats는 구현한다.
- SQL 직접 실행은 지원하지 않는다.
- 그래서 Step 15 테스트는 `AppError::Unsupported`를 확인한다.

### MemoryStorage

파일 경로: `src/repository/gluesql_repository.rs`

```rust
#[cfg(test)]
impl GlueSqlTaskRepository<MemoryStorage> {
    pub fn new() -> Result<Self, AppError> {
        let storage = MemoryStorage::default();
        let glue = Glue::new(storage);
        let mut repository = Self { glue };

        repository.create_tasks_table()?;

        Ok(repository)
    }
}
```

코드 해석:

- `MemoryStorage`는 테스트 helper에서 사용한다.
- 빠르게 `tasks` table을 만들고 Todo 기능을 검증한다.
- 명시적 transaction은 `memory_storage_rejects_explicit_transactions` 테스트에서 실패를 관찰한다.

### SledStorage

파일 경로: `src/repository/gluesql_repository.rs`

```rust
impl GlueSqlTaskRepository<SledStorage> {
    pub fn persistent(path: impl AsRef<Path>) -> Result<Self, AppError> {
        let storage =
            SledStorage::new(path).map_err(|error| AppError::GlueSql(error.to_string()))?;
        let glue = Glue::new(storage);
        let mut repository = Self { glue };

        repository.create_tasks_table()?;

        Ok(repository)
    }
}
```

코드 해석:

- `SledStorage`는 CLI 기본 실행 저장소다.
- `src/main.rs`는 `GlueSqlTaskRepository::persistent("data/rust-task-db")`를 호출한다.
- 테스트에서는 rollback, commit, nested transaction, repeatable read snapshot, write lock을 관찰한다.

## Storage 선택 기준

현재 프로젝트 기준으로 저장소를 고르는 질문은 아래 순서가 좋다.

| 질문 | 답이 yes이면 |
| --- | --- |
| 빠른 단위 테스트가 필요한가? | `MemoryStorage` |
| CLI 실행 간 데이터가 남아야 하는가? | `SledStorage` |
| 이전 JSON 학습 단계를 보존해야 하는가? | `JsonTaskRepository` |
| 여러 thread가 같은 메모리 DB를 봐야 하는가? | `SharedMemoryStorage` 분석 후보 |
| 파일 기반 GlueSQL storage 자체를 분석하고 싶은가? | `JsonStorage` 분석 후보 |
| 외부 document DB와 SQL layer 연결을 보고 싶은가? | `MongoStorage` 분석 후보 |
| storage 역할을 나눠 조합하는 구조를 보고 싶은가? | `CompositeStorage` 분석 후보 |

## 현재 단계에서 도입하지 않는 이유

`SharedMemoryStorage`, `JsonStorage`, `MongoStorage`, `CompositeStorage`를 지금 코드에 추가하지 않는 이유:

- 새 외부 설정이나 dependency가 필요할 수 있다.
- CLI Todo 학습보다 storage 실험이 중심이 될 수 있다.
- 현재 기본 저장소인 `SledStorage`와 역할이 겹칠 수 있다.
- Step 18의 목표는 기능 추가가 아니라 비교 기준을 명확히 하는 것이다.

## 초심자가 수정할 수 있는 지점

문서 실습으로는 아래를 추천한다.

1. `src/repository/gluesql_repository.rs`에서 `MemoryStorage`와 `SledStorage` 생성 코드를 비교한다.
2. `src/repository/mod.rs`에서 `JsonTaskRepository::execute_sql`이 왜 `Unsupported`인지 확인한다.
3. `docs/beginner-codebase-guide/17-gluesql-internals.md`의 간단 storage 표와 이 문서의 상세 비교표를 함께 읽는다.
4. 어떤 storage가 현재 코드에 실제로 들어와 있고, 어떤 storage가 분석 후보인지 표시해본다.

코드에서 확인되지 않음:

- `SharedMemoryStorage` dependency 활성화
- GlueSQL `JsonStorage` 연결
- GlueSQL `MongoStorage` 연결
- GlueSQL `CompositeStorage` 연결
- CLI에서 storage를 선택하는 옵션
