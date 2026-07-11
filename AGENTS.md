# Codex Agent Guide for rust-task

이 저장소는 Rust 문법을 단계별로 학습하기 위한 CLI Todo 프로젝트다. Codex는 항상 현재 단계에 맞춰 작업해야 한다.

## 현재 단계

- 현재 단계: Step 40. Query Lab 기여 후보 정리
- 현재 저장 방식: `GlueSqlTaskRepository`가 관리하는 GlueSQL `SledStorage`
- 현재 지원 명령: 기존 Task Management 명령과 `analyze`, `lab list`, `lab run`, `lab seed`
- 현재 CLI 구조: `std::env::args()`를 `src/cli.rs`의 `parse_args`가 `Command` enum으로 변환하고, `src/main.rs`가 `TaskService<GlueSqlTaskRepository>`를 통해 명령을 실행하며 실패는 `AppError`로 표현한다.
- 보존된 이전 저장소: `JsonTaskRepository`, `tasks.json`, MemoryStorage 기반 테스트 흐름은 삭제하지 않고 남긴다.
- 아직 도입하지 않는 것: 새 외부 crate, 웹 서버, async 앱 구조, 사용자-facing 동시성 제어 명령, GlueSQL upstream 수정

## 핵심 원칙

- `docs/prompt.md`의 단계 순서를 지킨다.
- `docs/review_docs.md`의 문서화 원칙을 지킨다.
- 구현을 변경할 때마다 `README.md`도 함께 확인하고, 실행 방법/지원 기능/현재 단계/주의점이 바뀌었으면 갱신한다.
- 다음 단계 기능을 미리 구현하지 않는다.
- 현재 단계에서 설명해야 할 Rust 문법만 코드에 남긴다.
- Step 12에서는 `serde`, `serde_json`, `gluesql`, `futures`만 외부 crate로 사용한다.
- GlueSQL API가 async이므로 `GlueSqlTaskRepository` 내부에서만 `futures::executor::block_on`을 사용한다. `main.rs`를 async 앱으로 바꾸지 않는다.
- 현재 활성 저장소는 `GlueSqlTaskRepository`의 GlueSQL `SledStorage`다.
- `JsonTaskRepository`와 `tasks.json`을 삭제하지 않는다.
- Step 9에서 SQL 직접 실행 명령을 도입했다.
- Step 10에서는 `repl` 명령과 `.schema`, `.exit`, `.quit`을 지원한다.
- Step 11에서는 새 기능보다 테스트 보강을 우선한다.
- Step 12에서는 `GlueSqlTaskRepository::persistent("data/rust-task-db")`를 사용해 CLI 실행 간 Todo가 유지되게 한다.
- Step 13에서는 새 CLI 명령이나 새 외부 crate를 추가하지 않고, 코드와 문서가 Step 12 최종 기능 상태를 일관되게 설명하는지 검증한다.
- Step 14에서는 새 CLI 명령이나 새 외부 crate를 추가하지 않고, `src/repository/gluesql_repository.rs` 테스트로 `MemoryStorage` transaction 미지원, `SledStorage` rollback, repeatable read snapshot, write lock 충돌을 관찰한다.
- 같은 Sled DB를 여러 repository에서 관찰할 때는 같은 path를 두 번 `SledStorage::new`로 열지 말고, 먼저 만든 `SledStorage`를 `clone()`해서 `Glue::new`에 넣는 테스트 helper를 사용한다.
- Step 15에서는 새 CLI 명령이나 새 외부 crate를 추가하지 않고, GlueSQL `Glue::execute` 내부 흐름(Parser -> Planner -> Executor -> Store), `GStore`/`GStoreMut`/`Planner` trait bound, Storage별 기능 차이를 문서와 테스트로 분석한다.
- Step 15에서도 GlueSQL upstream source를 복사하거나 수정하지 않는다. 현재 프로젝트 코드는 public API와 local crate source 관찰을 문서화하는 수준으로 유지한다.
- Step 16에서는 새 CLI 명령이나 새 외부 crate를 추가하지 않고, 실제 custom storage를 production code에 도입하지 않는다. 대신 최소 custom storage를 만들 때 필요한 GlueSQL Store trait 책임, 읽기 전용 storage와 쓰기 가능 storage의 차이, 현재 repository 구조와 연결되는 지점을 문서화한다.
- Step 17에서는 새 CLI 명령이나 새 외부 crate를 추가하지 않고, Todo 명령별 SQL 생성 흐름과 GlueSQL `Payload`가 `Task`, `TaskStats`, `SqlResult`로 변환되는 경로를 문서화한다.
- Step 18에서는 새 CLI 명령이나 새 외부 crate를 추가하지 않고, 현재 프로젝트의 `JsonTaskRepository`, `MemoryStorage`, `SledStorage`와 문서 비교 대상 storage의 기능 차이를 표로 고도화한다.
- Step 19~28에서는 Project 1:N Task, Task N:M Tag를 실제 기능으로 구현하고 Project/Task/Tag CLI, JOIN/aggregate, 삭제 정책, Seed와 문서를 완성한다.
- Step 29~40에서는 기존 관계형 SQL을 대상으로 planned Statement renderer, TracingStorage runtime metrics, scenario/seed, DBMS 비교와 GlueSQL 기여 후보를 제공한다.
- GlueSQL 0.19의 `glue.plan` 반환형은 `Vec<ast::Statement>`이며 별도 `StatementPlan` 타입이 아님을 문서와 UI에서 구분한다.
- operator별 actual row는 공개 hook으로 확인할 수 없으므로 Storage row consumption과 혼동하지 않는다.
- GlueSQL 0.19가 `task_tags` 복합 PK를 지원하지 않으므로 연결 중복은 `TaskManagementRepository` 구현에서 검사한다.
- ID는 `id_sequences` table로 할당하고 SledStorage에서는 sequence 갱신과 INSERT를 같은 transaction에 둔다.
- SledStorage 다중 변경은 repository `transaction` helper로 원자화하고, transaction 미지원 MemoryStorage는 같은 closure를 비transaction 방식으로 실행한다.
- Seed 완료 상태는 `app_metadata`의 `seed_version`으로 관리하고 부분 Seed 예약 데이터를 정리한 뒤 재생성한다.
- 구현을 변경할 때마다 `docs/beginner-codebase-guide/`의 초심자 가이드를 함께 업데이트한다.
- 초심자 가이드는 실제 코드 경로, 함수명, 타입명, 실행 흐름, 수정 포인트를 코드와 연결해서 설명해야 한다.
- Markdown view 모드에서 줄이 붙지 않도록 `파일 경로: ...`, `역할: ...` 같은 key-value 설명은 표 또는 bullet list로 작성한다.
- 코드에는 존재하지 않는 뒤 단계 기능을 현재 구현처럼 설명하지 않는다. 뒤 단계 내용은 반드시 "예정" 또는 "TODO"로 표시한다.
- 사용자가 명시적으로 커밋을 요청하기 전에는 `git commit`을 실행하지 않는다.
- 사용자가 명시적으로 푸시를 요청하기 전에는 `git push`를 실행하지 않는다.

## 현재 파일 역할

- `src/main.rs`: `Command` 실행 분기, `TaskService<GlueSqlTaskRepository>` 메서드 호출, 출력
- `src/repl.rs`: REPL 입력 루프, `.schema`, `.exit`, `.quit`, REPL SQL 결과 출력
- `src/error.rs`: `AppError`, `Display`, `Error`, `From` 구현
- `src/service/mod.rs`: `TaskService<R: TaskRepository>`, service 테스트
- `src/repository/mod.rs`: `TaskRepository` trait, `SqlResult`, `JsonTaskRepository`, `tasks.json` 읽기/쓰기, search/stats, SQL unsupported 처리, repository 테스트
- `src/repository/gluesql_repository.rs`: `GlueSqlTaskRepository<S>`, GlueSQL `MemoryStorage` 테스트 흐름, GlueSQL `SledStorage` 영속 저장소, `tasks` table 생성, SQL 기반 CRUD/search/stats/sql, rollback/snapshot/write lock, explicit commit, nested transaction 관찰 테스트
- `src/command.rs`: CLI 명령을 표현하는 `Command` enum. `Search`, `Stats`, `Sql`, `Repl` 포함
- `src/cli.rs`: `std::env::args()` 결과를 `Command`로 바꾸는 parser와 parser 테스트
- `src/task.rs`: `Task` struct, `TaskStats` struct, `Task::new`, serde derive, task 테스트
- `README.md`: GitHub 첫 화면용 프로젝트 소개, 실행 방법, 테스트 방법, 현재 저장소 주의점
- `Cargo.toml`: Rust package 설정. `serde`, `serde_json`, `gluesql`, `futures` dependency 포함
- `tasks.json`: Step 7까지 사용한 JSON 저장 파일. Step 12 현재 기본 실행 저장소는 아니지만 삭제하지 않는다.
- `data/rust-task-db`: Step 12부터 CLI 실행 데이터가 저장되는 SledStorage 디렉터리. `.gitignore`로 추적하지 않는다.
- `docs/todo/step-1-progress.md`: Step 1 진행 상태
- `docs/todo/step-2-progress.md`: Step 2 진행 상태
- `docs/todo/step-3-progress.md`: Step 3 진행 상태
- `docs/todo/step-4-progress.md`: Step 4 진행 상태
- `docs/todo/step-5-progress.md`: Step 5 진행 상태
- `docs/todo/step-6-progress.md`: Step 6 진행 상태
- `docs/todo/step-7-progress.md`: Step 7 진행 상태
- `docs/todo/step-8-progress.md`: Step 8 진행 상태
- `docs/todo/step-9-progress.md`: Step 9 진행 상태
- `docs/todo/step-10-progress.md`: Step 10 진행 상태
- `docs/todo/step-11-progress.md`: Step 11 진행 상태
- `docs/todo/step-12-progress.md`: Step 12 진행 상태
- `docs/todo/step-13-progress.md`: Step 13 진행 상태
- `docs/todo/step-14-progress.md`: Step 14 진행 상태
- `docs/todo/step-15-progress.md`: Step 15 진행 상태
- `docs/todo/step-16-progress.md`: Step 16 진행 상태
- `docs/todo/step-17-progress.md`: Step 17 진행 상태
- `docs/todo/step-18-progress.md`: Step 18 진행 상태
- `docs/todo/roadmap.md`: 이후 단계 작업 계획
- `docs/beginner-codebase-guide/`: 현재 단계 코드를 초심자가 읽을 수 있게 설명하는 문서 세트

## 초심자 가이드 업데이트 규칙

구현 작업을 하면 반드시 아래 문서를 함께 확인하고 필요한 부분을 고친다.

```text
docs/beginner-codebase-guide/99-index.md
docs/beginner-codebase-guide/00-overview.md
docs/beginner-codebase-guide/01-project-map.md
docs/beginner-codebase-guide/02-reading-order.md
docs/beginner-codebase-guide/03-runtime-flow.md
docs/beginner-codebase-guide/04-feature-flows.md
docs/beginner-codebase-guide/05-file-by-file/00-index.md
docs/beginner-codebase-guide/05-file-by-file/01-entrypoint.md
docs/beginner-codebase-guide/05-file-by-file/02-domain-or-feature-files.md
docs/beginner-codebase-guide/05-file-by-file/03-global-and-common-files.md
docs/beginner-codebase-guide/05-file-by-file/04-configuration-files.md
docs/beginner-codebase-guide/05-file-by-file/05-test-files.md
docs/beginner-codebase-guide/06-language-from-code.md
docs/beginner-codebase-guide/07-framework-and-libraries.md
docs/beginner-codebase-guide/08-data-model.md
docs/beginner-codebase-guide/09-configuration.md
docs/beginner-codebase-guide/10-error-handling.md
docs/beginner-codebase-guide/11-testing.md
docs/beginner-codebase-guide/12-practice-tasks.md
docs/beginner-codebase-guide/13-common-mistakes.md
docs/beginner-codebase-guide/14-glossary.md
docs/beginner-codebase-guide/15-beginner-faq.md
docs/beginner-codebase-guide/16-run-guide.md
docs/beginner-codebase-guide/17-gluesql-internals.md
docs/beginner-codebase-guide/18-custom-storage.md
docs/beginner-codebase-guide/19-query-execution.md
docs/beginner-codebase-guide/20-storage-comparison.md
```

업데이트 기준:

- 새 파일을 추가하면 `01-project-map.md`, `05-file-by-file/00-index.md`, `99-index.md`를 갱신한다.
- 실행 흐름이 바뀌면 `03-runtime-flow.md`와 `04-feature-flows.md`를 갱신한다.
- 함수나 핵심 로직이 바뀌면 관련 `05-file-by-file/*.md` 문서를 갱신한다.
- 새 Rust 문법이 등장하면 `06-language-from-code.md`, `14-glossary.md`, 필요하면 `15-beginner-faq.md`를 갱신한다.
- 테스트가 바뀌면 `05-file-by-file/05-test-files.md`와 `11-testing.md`를 갱신한다.
- 실습으로 연결할 만한 변경이면 `12-practice-tasks.md`를 갱신한다.
- 단계 진행 상태가 바뀌면 해당 `docs/todo/step-N-progress.md`와 `docs/todo/roadmap.md`를 갱신한다.

`docs/review_docs.md` 원칙상 문서에는 다음이 반드시 들어가야 한다.

- 실제 파일 경로
- 실제 함수명, 타입명, 변수명
- 코드 블록
- 코드 해석
- 프로젝트에서의 역할
- 초심자가 수정할 수 있는 지점
- 현재 코드에 없는 내용은 "코드에서 확인되지 않음" 또는 "이후 단계 예정"으로 표시

## Step 12 검증 명령

```bash
cargo fmt --check
cargo check
cargo test
cargo run -- add "Rust 공부"
cargo run -- list
cargo run -- search rust
cargo run -- stats
cargo run -- sql "SELECT * FROM tasks"
cargo run -- sql "INSERT INTO tasks VALUES (1, 'Rust 공부', FALSE); SELECT id, title, done FROM tasks;"
cargo run -- repl
```

주의: Step 12는 GlueSQL `SledStorage`를 사용하므로 `cargo run`을 여러 번 나눠 실행해도 `data/rust-task-db`에 데이터가 유지된다. 테스트에서는 빠른 단위 검증을 위해 `MemoryStorage`도 계속 사용한다.

## Step 13 검증 명령

```bash
cargo fmt --check
cargo check
cargo test
```

주의: Step 13은 새 기능 구현 단계가 아니라 최종 검증과 문서 정합성 점검 단계다. 기능 검증이 필요하면 Step 12 검증 명령을 그대로 사용한다.

## Step 14 검증 명령

```bash
cargo fmt --check
cargo check
cargo test
```

주의: Step 14는 CLI 기능을 늘리는 단계가 아니라 GlueSQL storage 동시성/트랜잭션 특성을 테스트로 관찰하는 단계다. `cargo test` 기준 62개 테스트가 통과해야 한다.

## Step 15 검증 명령

```bash
cargo fmt --check
cargo check
cargo test
```

주의: Step 15는 CLI 기능을 늘리는 단계가 아니라 GlueSQL engine/storage adapter 분석을 문서화하고 테스트 경계를 보강하는 단계다. `cargo test` 기준 65개 테스트가 통과해야 한다.

## Step 16 검증 명령

```bash
cargo fmt --check
cargo check
cargo test
```

주의: Step 16은 production custom storage를 도입하는 단계가 아니라 Minimal Custom Storage 구조를 문서로 분석하는 단계다. `cargo test` 기준 65개 테스트가 유지되어야 한다.

## Step 17 검증 명령

```bash
cargo fmt --check
cargo check
cargo test
```

주의: Step 17은 새 CLI 기능을 추가하는 단계가 아니라 현재 Todo 명령의 SQL 생성과 `Payload` 변환 흐름을 문서로 분석하는 단계다. `cargo test` 기준 65개 테스트가 유지되어야 한다.

## Step 18 검증 명령

```bash
cargo fmt --check
cargo check
cargo test
```

주의: Step 18은 새 storage를 코드에 도입하는 단계가 아니라 storage별 기능 비교표를 고도화하는 단계다. `cargo test` 기준 65개 테스트가 유지되어야 한다.

## 앞으로 작업 요청을 받았을 때

1. 현재 단계의 `docs/todo/step-N-progress.md`와 `docs/todo/roadmap.md`를 먼저 확인한다.
2. `docs/beginner-codebase-guide/99-index.md`를 확인해 현재 초심자 가이드 구조를 파악한다.
3. 사용자가 특정 단계를 요청하지 않으면 현재 단계 범위 안에서만 수정한다.
4. 다음 단계로 넘어가자는 요청이 있으면 roadmap의 순서대로 구현한다.
5. 구현 후 `docs/todo/` 진행 문서와 `docs/beginner-codebase-guide/` 초심자 가이드를 함께 갱신한다.
6. `README.md`가 현재 기능, 실행 방법, 테스트 방법과 어긋나지 않는지 확인하고 필요하면 갱신한다.
7. `cargo fmt --check`와 `cargo test`를 실행한다.
8. 최종 보고에 코드 변경과 문서 변경을 모두 적는다.

## Git 작업 규칙

- 사용자가 커밋을 요청하기 전에는 커밋하지 않는다.
- 사용자가 푸시를 요청하기 전에는 푸시하지 않는다.
- 커밋 전에는 `git status --short`로 포함 파일을 확인한다.
- 커밋 전에는 가능한 경우 `cargo fmt --check`와 `cargo test`를 통과시킨다.
- 커밋 메시지는 아래 템플릿을 기본으로 사용한다.

```text
<type>: <짧은 요약>

- <변경 사항 1>
- <변경 사항 2>
- <문서/테스트 변경 사항>
```

권장 type:

- `feat`: 새 기능
- `fix`: 버그 수정
- `test`: 테스트 추가/수정
- `docs`: 문서 수정
- `refactor`: 동작 변경 없는 구조 개선
- `chore`: 설정, 정리 작업

## 단계 전환 규칙

- Step 2에서만 `src/command.rs`, `src/cli.rs`를 추가한다. 이미 추가되어 있으므로 이후 단계에서는 이 구조를 유지한다.
- Step 3으로 넘어갈 때만 `serde`, `serde_json`, `tasks.json`을 추가한다.
- Step 4에서만 `TaskRepository` trait와 repository 디렉터리를 추가한다. 이미 추가되어 있으므로 이후 단계에서는 이 구조를 유지한다.
- Step 5에서만 `TaskService`를 추가한다. 이미 추가되어 있으므로 이후 단계에서는 이 구조를 유지한다.
- Step 6에서만 `AppError`를 추가한다. 이미 추가되어 있으므로 이후 단계에서는 이 구조를 유지한다.
- Step 7에서만 `search`, `stats`, `TaskStats`를 추가한다. 이미 추가되어 있으므로 이후 단계에서는 이 구조를 유지한다.
- Step 8에서만 `GlueSqlTaskRepository`와 GlueSQL `MemoryStorage`를 추가한다. 이미 추가되어 있으므로 이후 단계에서는 이 구조를 유지한다.
- Step 9에서만 `sql` 명령을 추가한다. 이미 추가되어 있으므로 이후 단계에서는 이 구조를 유지한다.
- Step 10에서만 `repl` 명령과 `.schema`, `.exit`, `.quit`을 추가한다. 이미 추가되어 있으므로 이후 단계에서는 이 구조를 유지한다.
- Step 11에서는 테스트를 보강한다. 새 명령이나 저장소 기능을 추가하지 않는다.
- Step 12에서는 GlueSQL `SledStorage`를 도입해 CLI 기본 저장소를 영속 저장소로 전환한다. 새 CLI 명령은 추가하지 않는다.
- Step 13에서는 새 기능을 추가하지 않고 최종 검증과 문서 정합성 점검을 수행한다.
- Step 14에서는 새 CLI 명령이나 새 외부 crate를 추가하지 않고 GlueSQL `SledStorage` transaction/snapshot/write lock 관찰 테스트와 문서를 추가한다.
- Step 15에서는 새 CLI 명령이나 새 외부 crate를 추가하지 않고 GlueSQL Parser/Planner/Executor/Store 흐름, Store trait 책임, Storage별 기능 차이, 기여 전략을 문서화하고 관찰 테스트를 보강한다.
- Step 16에서는 새 CLI 명령이나 새 외부 crate를 추가하지 않고 Minimal Custom Storage를 만들 때 필요한 trait 책임과 구현 순서를 문서화한다.
- Step 17에서는 새 CLI 명령이나 새 외부 crate를 추가하지 않고 Todo 명령별 SQL 생성과 query result 변환 흐름을 문서화한다.
- Step 18에서는 새 CLI 명령이나 새 외부 crate를 추가하지 않고 storage별 기능 비교표를 고도화한다.

단계를 전환할 때는 초심자 가이드도 같은 단계 기준으로 재구성한다. 예를 들어 Step 2로 넘어가면 `Command` enum과 CLI parser를 현재 구현으로 설명하고, Step 3 이후 기능은 계속 예정으로 남긴다.
