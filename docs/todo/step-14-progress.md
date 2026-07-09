# Step 14 진행 상황: GlueSQL SledStorage 트랜잭션과 동시성 관찰

## 현재 상태

Step 14에서는 새 CLI 명령이나 새 외부 crate를 추가하지 않는다. 대신 현재 Todo table을 이용해 GlueSQL storage 구현체별 transaction/동시성 차이를 테스트와 문서로 관찰한다.

## Step 13에서 Step 14로 달라진 점

| 구분 | Step 13 | Step 14 |
| --- | --- | --- |
| 활성 저장소 | GlueSQL `SledStorage` | GlueSQL `SledStorage` |
| 새 CLI 명령 | 없음 | 없음 |
| 새 외부 crate | 없음 | 없음 |
| 핵심 작업 | 최종 검증과 문서 정합성 점검 | SledStorage transaction/snapshot/write lock 관찰 |
| 테스트 수 | 58개 | 62개 |

## 구현된 관찰 테스트

| 테스트 | 관찰하는 내용 |
| --- | --- |
| `memory_storage_rejects_explicit_transactions` | `MemoryStorage`는 명시적 `BEGIN`을 지원하지 않는다. |
| `sled_storage_rolls_back_uncommitted_insert` | `SledStorage`는 `ROLLBACK`으로 transaction 안의 insert를 취소한다. |
| `sled_storage_keeps_repeatable_read_snapshot_until_commit` | reader transaction은 commit 전 snapshot을 유지하고, commit 뒤 최신 값을 본다. |
| `sled_storage_reports_database_locked_for_competing_writes` | 열린 writer transaction이 있으면 다른 writer는 `database is locked` 에러를 받는다. |

## 초심자가 이해해야 할 핵심

GlueSQL은 SQL 실행 계층과 storage 계층을 분리한다. 그래서 transaction/동시성 특성도 `GlueSQL core` 하나로 설명하기보다 `MemoryStorage`, `SledStorage` 같은 storage 구현체별로 확인해야 한다.

현재 프로젝트는 `src/main.rs`의 CLI 실행 구조를 바꾸지 않는다. 동시성 관찰은 `src/repository/gluesql_repository.rs`의 테스트에서 수행한다.

```text
SledStorage::new(path)
-> SledStorage::clone()
-> GlueSqlTaskRepository { glue: Glue::new(cloned_storage) }
-> BEGIN / INSERT / COMMIT / ROLLBACK SQL 실행
-> snapshot, rollback, database is locked 결과 확인
```

## 주의할 점

같은 Sled 디렉터리를 동시에 관찰할 때 `SledStorage::new(path)`를 두 번 호출하면 OS 파일 락 때문에 실패할 수 있다. GlueSQL 예제와 현재 테스트는 먼저 만든 `SledStorage`를 `clone()`해서 여러 `Glue` 인스턴스에 넣는다.

## 완료 기준

- `cargo fmt --check` 통과
- `cargo check` 통과
- `cargo test` 통과
- README와 초심자 가이드가 Step 14의 테스트 목적을 설명

## 완료된 검증

| 명령 | 결과 |
| --- | --- |
| `cargo fmt --check` | 통과 |
| `cargo test` | 62개 테스트 통과 |

## 다음 단계

현재는 storage 동시성 관찰을 테스트 수준에서 다룬다. 사용자-facing CLI 명령, 웹 서버, async 앱 구조는 코드에서 확인되지 않으며 이후 요구가 있을 때 별도 단계로 검토한다.
