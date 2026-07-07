# 초심자용 코드베이스 완전 해설서

## 이 문서 세트의 목적

이 문서 세트는 Rust를 처음 보는 사람이 현재 Step 13 코드와 문서만 보고도 `rust-task`의 구조, 실행 흐름, GlueSQL SledStorage 저장소, SQL 실행 모드, REPL 모드, search/stats, custom error, Service layer, Repository trait, 테스트, 수정 포인트를 이해하게 만드는 것이다.

현재 구현은 `Step 13. 최종 검증 및 문서 정합성 점검` 단계다. 기능 구현은 Step 12의 GlueSQL `SledStorage` 영속 저장 전환까지 완료되어 있다.

## 문서를 읽는 추천 순서

0. `README.md`
1. `16-run-guide.md`
2. `00-overview.md`
3. `01-project-map.md`
4. `02-reading-order.md`
5. `03-runtime-flow.md`
6. `04-feature-flows.md`
7. `05-file-by-file/00-index.md`
8. `06-language-from-code.md`
9. `08-data-model.md`
10. `10-error-handling.md`
11. `11-testing.md`
12. `12-practice-tasks.md`
13. `13-common-mistakes.md`
14. `14-glossary.md`
15. `15-beginner-faq.md`

## 각 문서의 역할

- `00-overview.md`: 현재 Step 13 프로젝트 큰 그림
- `01-project-map.md`: 실제 파일 지도
- `02-reading-order.md`: 초심자가 읽을 순서
- `03-runtime-flow.md`: `main()`부터 service, GlueSQL repository 호출까지 흐름
- `04-feature-flows.md`: add/list/done/delete 흐름
- `05-file-by-file/`: 파일별 상세 설명
- `06-language-from-code.md`: 현재 코드에서 배우는 Rust 문법
- `07-framework-and-libraries.md`: 언어 기능과 외부 crate 구분
- `08-data-model.md`: `Task`, `Command`, `TaskService`, `TaskRepository`, GlueSQL `tasks` table, REPL 데이터 흐름
- `09-configuration.md`: Cargo 설정과 dependency
- `10-error-handling.md`: `AppError` 기반 실패 처리
- `11-testing.md`: 테스트 코드 해설
- `12-practice-tasks.md`: 쉬운 실습
- `13-common-mistakes.md`: 초심자 실수
- `14-glossary.md`: 용어 사전
- `15-beginner-faq.md`: 지금까지 나온 질문과 답변 모음
- `16-run-guide.md`: 프로젝트 실행 방법, SQL/REPL 실행 방법, GlueSQL `SledStorage` 저장 위치
- `README.md`: GitHub 첫 화면용 요약, 실행 방법, 테스트 방법
- `docs/todo/step-13-progress.md`: 현재 Step 13 최종 검증 및 문서 정합성 점검 상태

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

1. `src/main.rs`
2. `src/service.rs`
3. `src/repository/mod.rs`
4. `src/task.rs`
5. `src/repository/gluesql_repository.rs`

## 다음 단계 안내

현재 단계는 Step 13이다. 새 기능은 추가하지 않고, Step 12까지 구현된 기능과 문서가 일치하는지 검증한다.
