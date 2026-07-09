# Step 15 진행 상황: GlueSQL Engine/Storage Adapter 분석 보강

## 현재 상태

Step 15에서는 새 CLI 명령이나 새 외부 crate를 추가하지 않는다. Notion의 GlueSQL 분석 리포트 기준으로 SQL 실행 흐름과 Storage Adapter 구조를 현재 Todo 프로젝트 코드와 연결해 문서화하고, storage 지원 경계 테스트를 보강한다.

## Step 14에서 Step 15로 달라진 점

| 구분 | Step 14 | Step 15 |
| --- | --- | --- |
| 활성 저장소 | GlueSQL `SledStorage` | GlueSQL `SledStorage` |
| 새 CLI 명령 | 없음 | 없음 |
| 새 외부 crate | 없음 | 없음 |
| 핵심 작업 | SledStorage transaction/snapshot/write lock 관찰 | GlueSQL engine 흐름과 Storage Adapter 구조 분석 |
| 테스트 수 | 62개 | 65개 |

## 구현된 보강 테스트

| 테스트 | 관찰하는 내용 |
| --- | --- |
| `json_repository_reports_sql_as_unsupported` | SQL 미지원 repository는 `AppError::Unsupported`로 앱 경계에 실패를 올린다. |
| `sled_storage_commits_explicit_transaction` | `SledStorage`는 명시적 `BEGIN`/`COMMIT`으로 insert를 확정한다. |
| `sled_storage_rejects_nested_transaction` | `SledStorage`는 nested transaction을 `AppError::GlueSql` 경계로 보고한다. |

## 초심자가 이해해야 할 핵심

현재 프로젝트는 GlueSQL 내부 Parser, Planner, Executor를 직접 구현하거나 직접 호출하지 않는다. 대신 `GlueSqlTaskRepository::execute`가 `Glue::execute`를 호출하고, GlueSQL이 내부에서 SQL 문자열을 실행 결과 `Payload`로 바꾼다.

```text
사용자 SQL
-> GlueSqlTaskRepository::execute
-> Glue::execute
-> Parser / Planner / Executor
-> Store trait 계층
-> MemoryStorage 또는 SledStorage
-> Payload
-> SqlResult
```

`GlueSqlTaskRepository<S>`의 `S`는 아무 타입이나 들어갈 수 없다. 현재 코드의 `where S: GStore + GStoreMut + Planner` 조건은 GlueSQL이 SQL 실행에 필요한 storage 기능을 요구한다는 뜻이다.

## Storage별 위치

| Storage | 현재 프로젝트 상태 |
| --- | --- |
| `MemoryStorage` | 테스트용. 빠르고 명시 transaction은 미지원이다. |
| `SledStorage` | 기본 CLI 저장소. 영속 저장, rollback, snapshot, write lock, commit, nested transaction 실패를 관찰한다. |
| `SharedMemoryStorage` | `Arc<RwLock<MemoryStorage>>` 기반 참고 대상. 현재 dependency에는 직접 추가하지 않는다. |
| `JsonStorage` | GlueSQL의 별도 storage 참고 대상. 현재 프로젝트의 `JsonTaskRepository`와는 다른 구현이며 코드에 미도입이다. |
| `MongoStorage` | Document DB 위 SQL Layer 분석 후보. 현재 코드에 미도입이다. |
| `CompositeStorage` | 서로 다른 storage 조합 분석 후보. 현재 코드에 미도입이다. |

## 완료 기준

- `cargo fmt --check` 통과
- `cargo check` 통과
- `cargo test` 통과
- README, AGENTS, roadmap, 초심자 가이드가 Step 15와 65개 테스트를 일관되게 설명

## 완료된 검증

| 명령 | 결과 |
| --- | --- |
| `cargo fmt --check` | 통과 |
| `cargo test` | 65개 테스트 통과 |

## 다음 단계

다음 확장 후보는 실제 GlueSQL upstream 기여 실습, Store trait 최소 구현 예제, 또는 MongoStorage/CompositeStorage 분석이다. 현재 코드에는 아직 도입하지 않았으며, 이후 단계 예정이다.
