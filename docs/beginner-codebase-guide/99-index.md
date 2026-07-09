# 초심자용 코드베이스 완전 해설서

## 이 문서 세트의 목적

이 문서 세트는 Rust를 처음 보는 사람이 현재 Step 18 코드와 문서만 보고도 `rust-task`의 구조, 실행 흐름, GlueSQL SledStorage 저장소, SQL 실행 모드, REPL 모드, search/stats, custom error, Service layer, Repository trait, GlueSQL transaction 관찰 테스트, GlueSQL Engine/Storage Adapter 구조, Minimal Custom Storage 책임, Query Execution 변환 흐름, Storage별 기능 차이, 수정 포인트를 이해하게 만드는 것이다.

현재 구현은 `Step 18. Storage별 기능 비교표 고도화` 단계다. CLI 기능 구현은 Step 12의 GlueSQL `SledStorage` 영속 저장 전환까지 완료되어 있고, Step 18에서는 새 CLI 명령 없이 storage별 기능 차이를 문서로 분석한다.

## 문서를 읽는 추천 순서

0. [README.md](../../README.md)
1. [16-run-guide.md](16-run-guide.md)
2. [00-overview.md](00-overview.md)
3. [01-project-map.md](01-project-map.md)
4. [02-reading-order.md](02-reading-order.md)
5. [03-runtime-flow.md](03-runtime-flow.md)
6. [04-feature-flows.md](04-feature-flows.md)
7. [05-file-by-file/00-index.md](05-file-by-file/00-index.md)
8. [06-language-from-code.md](06-language-from-code.md)
9. [08-data-model.md](08-data-model.md)
10. [10-error-handling.md](10-error-handling.md)
11. [11-testing.md](11-testing.md)
12. [12-practice-tasks.md](12-practice-tasks.md)
13. [13-common-mistakes.md](13-common-mistakes.md)
14. [14-glossary.md](14-glossary.md)
15. [15-beginner-faq.md](15-beginner-faq.md)
16. [17-gluesql-internals.md](17-gluesql-internals.md)
17. [18-custom-storage.md](18-custom-storage.md)
18. [19-query-execution.md](19-query-execution.md)
19. [20-storage-comparison.md](20-storage-comparison.md)

## 각 문서의 역할

- [00-overview.md](00-overview.md): 현재 Step 18 프로젝트 큰 그림
- [01-project-map.md](01-project-map.md): 실제 파일 지도
- [02-reading-order.md](02-reading-order.md): 초심자가 읽을 순서
- [03-runtime-flow.md](03-runtime-flow.md): `main()`부터 service, GlueSQL repository 호출까지 흐름
- [04-feature-flows.md](04-feature-flows.md): add/list/done/delete 흐름
- [05-file-by-file/](05-file-by-file/00-index.md): 파일별 상세 설명
- [06-language-from-code.md](06-language-from-code.md): 현재 코드에서 배우는 Rust 문법
- [07-framework-and-libraries.md](07-framework-and-libraries.md): 언어 기능과 외부 crate 구분
- [08-data-model.md](08-data-model.md): `Task`, `Command`, `TaskService`, `TaskRepository`, GlueSQL `tasks` table, REPL 데이터 흐름
- [09-configuration.md](09-configuration.md): Cargo 설정과 dependency
- [10-error-handling.md](10-error-handling.md): `AppError` 기반 실패 처리
- [11-testing.md](11-testing.md): 테스트 코드 해설
- [12-practice-tasks.md](12-practice-tasks.md): 쉬운 실습
- [13-common-mistakes.md](13-common-mistakes.md): 초심자 실수
- [14-glossary.md](14-glossary.md): 용어 사전
- [15-beginner-faq.md](15-beginner-faq.md): 지금까지 나온 질문과 답변 모음
- [16-run-guide.md](16-run-guide.md): 프로젝트 실행 방법, SQL/REPL 실행 방법, GlueSQL `SledStorage` 저장 위치
- [17-gluesql-internals.md](17-gluesql-internals.md): GlueSQL Parser/Planner/Executor/Store 흐름과 Storage별 차이
- [18-custom-storage.md](18-custom-storage.md): Minimal Custom Storage를 만들 때 필요한 trait 책임과 구현 순서
- [19-query-execution.md](19-query-execution.md): Todo 명령별 SQL 생성과 `Payload` 변환 흐름
- [20-storage-comparison.md](20-storage-comparison.md): Storage별 기능 차이와 현재 코드 도입 여부 비교
- [README.md](../../README.md): GitHub 첫 화면용 요약, 실행 방법, 테스트 방법
- [docs/todo/step-13-progress.md](../todo/step-13-progress.md): Step 13 최종 검증 및 문서 정합성 점검 기록
- [docs/todo/step-14-progress.md](../todo/step-14-progress.md): Step 14 GlueSQL transaction/snapshot/write lock 관찰 기록
- [docs/todo/step-15-progress.md](../todo/step-15-progress.md): Step 15 GlueSQL Engine/Storage Adapter 분석 완료 상태
- [docs/todo/step-16-progress.md](../todo/step-16-progress.md): Step 16 Minimal Custom Storage 분석 완료 상태
- [docs/todo/step-17-progress.md](../todo/step-17-progress.md): Step 17 Query Execution 상세 분석 완료 상태
- [docs/todo/step-18-progress.md](../todo/step-18-progress.md): 현재 Step 18 Storage별 기능 비교표 고도화 상태

## 이 문서만 보고 할 수 있어야 하는 것

- `src/cli.rs`의 `parse_args`가 CLI 문자열을 `Command`로 바꾸는 흐름 설명
- `search`, `stats`가 CLI에서 service와 repository를 거쳐 실행되는 방식 설명
- `src/service.rs`의 `TaskService<R: TaskRepository>`가 repository에 의존하는 방식 설명
- `src/repository/mod.rs`의 `TaskRepository`와 보존된 `JsonTaskRepository` 설명
- `src/repository/gluesql_repository.rs`의 `GlueSqlTaskRepository` 설명
- `src/repository/mod.rs`의 `SqlResult` 설명
- `rust-task sql "..."` 실행 흐름 설명
- `rust-task repl` 실행 흐름 설명
- `src/task.rs`의 serde derive 설명
- Step 11에서 보강된 테스트가 무엇을 지키는지 설명
- Step 12에서 `SledStorage`로 데이터가 유지되는 이유 설명
- Step 12에서 `GlueSqlTaskRepository`가 GlueSQL `SledStorage`에 SQL을 직접 실행하는 방식 설명
- Step 14에서 `MemoryStorage`와 `SledStorage`의 transaction 차이를 테스트로 설명
- Step 14에서 `SledStorage::clone()`으로 같은 Sled DB를 여러 `Glue` 인스턴스에 연결하는 이유 설명
- Step 15에서 `Glue::execute`가 Parser/Planner/Executor/Store 흐름을 감싼다는 점 설명
- Step 15에서 `GStore`, `GStoreMut`, `Planner` trait bound가 필요한 이유 설명
- Step 16에서 custom storage를 만들 때 읽기/쓰기/transaction/index 책임이 어떻게 나뉘는지 설명
- Step 16에서 `TaskRepository`와 GlueSQL `Store` 계층이 서로 다른 추상화임을 설명
- Step 17에서 Todo 명령이 어떤 SQL을 만들고 `Payload`가 어떤 프로젝트 타입으로 바뀌는지 설명
- Step 17에서 `row_to_task`, `select_count`, `payload_to_sql_result`의 역할 설명
- Step 18에서 현재 코드에 실제 도입된 storage와 문서 비교 대상 storage를 구분
- Step 18에서 `JsonTaskRepository`, `MemoryStorage`, `SledStorage`, `SharedMemoryStorage`, `JsonStorage`, `MongoStorage`, `CompositeStorage` 차이를 설명
- Step 10에서 REPL 안에서는 같은 저장소 인스턴스가 유지되는 이유 설명
- `mod`, `derive`, `impl`, `match`, `Result`, `?`, `std::fs`, `Serialize`, `Deserialize`, `block_on`이 무엇인지 설명
- `cargo run`, `cargo test`, `cargo check`로 현재 프로젝트 실행/검증

## 프로젝트를 이해하는 핵심 흐름

```text
터미널 입력
-> src/cli.rs parse_args()
-> src/command.rs Command
-> src/main.rs
-> src/service.rs TaskService
-> TaskRepository trait
-> GlueSqlTaskRepository
-> GlueSQL SledStorage
-> SQL 직접 실행, REPL SQL 실행, 또는 Todo 명령 실행
-> src/main.rs match command
-> 터미널 출력
```

## 처음 읽는 사람이 가장 먼저 봐야 할 5개 파일

1. [src/main.rs](../../src/main.rs)
2. [src/service.rs](../../src/service.rs)
3. [src/repository/mod.rs](../../src/repository/mod.rs)
4. [src/task.rs](../../src/task.rs)
5. [src/repository/gluesql_repository.rs](../../src/repository/gluesql_repository.rs)

## 다음 단계 안내

현재 단계는 Step 18이다. 새 CLI 명령은 추가하지 않고, Step 12까지 구현된 기능 위에서 Storage별 기능 비교표를 문서로 고도화한다.
