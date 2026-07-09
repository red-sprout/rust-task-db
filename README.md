# rust-task-db

Rust + GlueSQL 미니 프로젝트: CLI Todo List

`rust-task-db`는 Rust 문법과 간단한 아키텍처 분리를 단계별로 학습하기 위한 CLI Todo 프로젝트다. 현재는 `Command` enum, CLI parser, service layer, repository trait, custom error, GlueSQL `SledStorage`, SQL 실행 모드, REPL 모드, 테스트 보강, GlueSQL `SledStorage` 트랜잭션/동시성 관찰 테스트, GlueSQL Engine/Storage Adapter 분석, Minimal Custom Storage 분석, Query Execution 상세 분석, Storage별 기능 비교표 문서까지 구현되어 있다.

## 현재 상태

| 항목 | 내용 |
| --- | --- |
| 현재 단계 | Step 18. Storage별 기능 비교표 고도화 |
| 실행 방식 | Cargo로 실행하는 CLI 앱 |
| 활성 저장소 | `GlueSqlTaskRepository` + GlueSQL `SledStorage` |
| 저장 위치 | `data/rust-task-db` |
| 보존된 저장소 | `JsonTaskRepository` + `tasks.json`, MemoryStorage 테스트 흐름 |
| 테스트 | `cargo test` 기준 65개 |

## 주요 기능

```bash
cargo run -- add "Rust 공부"
cargo run -- list
cargo run -- done 1
cargo run -- delete 1
cargo run -- search rust
cargo run -- stats
cargo run -- sql "SELECT * FROM tasks"
cargo run -- repl
```

지원 기능:

- Todo 추가, 목록 조회, 완료 처리, 삭제
- 제목 검색
- 전체/완료/미완료 통계
- GlueSQL에 SQL 직접 실행
- 같은 실행 안에서 여러 SQL을 입력하는 REPL 모드

## 실행하기

```bash
cargo check
cargo run -- help
```

Todo 추가:

```bash
cargo run -- add "Rust 공부"
```

SQL 직접 실행:

```bash
cargo run -- sql "INSERT INTO tasks VALUES (1, 'Rust 공부', FALSE); SELECT id, title, done FROM tasks;"
```

REPL 실행:

```bash
cargo run -- repl
```

REPL 안에서는 아래 명령을 사용할 수 있다.

```text
.schema
.exit
.quit
```

## 테스트

```bash
cargo fmt --check
cargo test
```

현재 테스트는 domain, CLI parser, custom error, service, JSON repository, GlueSQL repository, SQL 실행, REPL 흐름, GlueSQL `SledStorage` rollback/snapshot/write lock/commit/nested transaction 흐름을 검증한다.

## 주의할 점

현재 기본 실행 경로는 GlueSQL `SledStorage`를 사용한다. 그래서 아래처럼 명령을 서로 다른 프로세스로 나눠 실행해도 데이터가 `data/rust-task-db`에 유지된다.

```bash
cargo run -- add "Rust 공부"
cargo run -- list
```

REPL 안에서도 같은 저장소 인스턴스를 사용하므로 입력한 SQL 결과가 이어진다.

```bash
cargo run -- repl
```

`tasks.json`은 이전 단계의 `JsonTaskRepository`를 보존하기 위해 남아 있다. 현재 기본 실행 저장소는 `tasks.json`이 아니라 `data/rust-task-db`다.

GlueSQL의 동시성 제어는 core가 하나의 방식으로 강제하기보다 storage 구현체에 달려 있다. 이 프로젝트는 새 CLI 명령을 추가하지 않고 `src/repository/gluesql_repository.rs` 테스트에서 다음을 관찰한다.

- `MemoryStorage`는 명시적 transaction을 지원하지 않는다.
- `SledStorage`는 `BEGIN`, `COMMIT`, `ROLLBACK`을 SQL로 실행할 수 있다.
- 열린 transaction 안에서 읽은 repository는 commit 전 snapshot을 계속 본다.
- 같은 `SledStorage`를 여러 `Glue` 인스턴스에서 보려면 같은 경로를 두 번 여는 대신 `SledStorage::clone()`으로 나눈다.
- transaction이 write lock을 잡은 동안 다른 writer는 `database is locked` 에러를 받는다.

Step 15에서는 Notion의 GlueSQL 분석 리포트 기준으로 SQL 실행 흐름과 Storage Adapter 구조도 문서화했다. 현재 프로젝트는 GlueSQL 내부 Parser/Planner/Executor를 직접 호출하지 않고 `Glue::execute`로 간접 관찰한다.

```text
사용자 SQL
-> GlueSqlTaskRepository::execute
-> Glue::execute
-> Parser / Planner / Executor
-> GStore / GStoreMut / Planner trait bound를 만족하는 Storage
-> Payload
-> SqlResult
```

Step 16에서는 실제 custom storage를 production code에 붙이지 않고, 최소 custom storage를 만들 때 어떤 trait 책임이 필요한지 문서화했다.

- 읽기 전용 storage는 schema/data 조회 계열 Store 책임이 중심이다.
- 쓰기 가능 storage는 schema/data 변경 계열 StoreMut 책임이 추가된다.
- 현재 `GlueSqlTaskRepository<S>`의 `S: GStore + GStoreMut + Planner` 조건은 Todo CRUD와 SQL 실행에 필요한 최소 경계를 보여준다.
- 실제 custom storage 도입은 이후 단계 예정이다.

Step 17에서는 Todo 명령이 실제로 어떤 SQL을 만들고, GlueSQL `Payload`를 어떤 프로젝트 타입으로 바꾸는지 문서화했다.

- Todo 전용 명령은 `Payload::Select`를 `Task` 또는 `TaskStats`로 변환한다.
- `sql`과 `repl`은 `Payload`를 CLI 출력용 `SqlResult`로 변환한다.
- 핵심 변환 지점은 `execute`, `select_tasks`, `row_to_task`, `select_count`, `payload_to_sql_result`, `value_to_string`이다.

Step 18에서는 storage별 기능 비교표를 고도화했다.

- 현재 코드에서 실제 사용하는 저장소는 `JsonTaskRepository`, `MemoryStorage` 테스트 흐름, `SledStorage` 기본 실행 흐름이다.
- `SharedMemoryStorage`, `JsonStorage`, `MongoStorage`, `CompositeStorage`는 문서 비교 대상으로만 다룬다.
- GlueSQL `JsonStorage`와 프로젝트의 `JsonTaskRepository`는 서로 다른 개념으로 구분한다.

## 프로젝트 구조

```text
src/main.rs
-> src/cli.rs
-> src/command.rs
-> src/service.rs
-> src/repository/mod.rs
-> src/repository/gluesql_repository.rs
-> src/task.rs
```

핵심 파일:

- `src/main.rs`: CLI 명령 실행 분기와 출력
- `src/cli.rs`: `std::env::args()`를 `Command`로 변환
- `src/command.rs`: CLI 명령을 표현하는 enum
- `src/service.rs`: `TaskService<R: TaskRepository>`
- `src/repository/mod.rs`: `TaskRepository`, `SqlResult`, 보존된 `JsonTaskRepository`
- `src/repository/gluesql_repository.rs`: 현재 활성 SledStorage 저장소와 MemoryStorage 테스트 흐름
- `src/repl.rs`: SQL REPL 입력 루프
- `src/task.rs`: `Task`, `TaskStats`

## 학습 문서

초심자용 상세 문서는 아래에서 시작하면 된다.

- [초심자용 코드베이스 완전 해설서](docs/beginner-codebase-guide/99-index.md)
- [실행 가이드](docs/beginner-codebase-guide/16-run-guide.md)
- [프로젝트 전체 개요](docs/beginner-codebase-guide/00-overview.md)
- [테스트 해설](docs/beginner-codebase-guide/11-testing.md)

작업 단계와 진행 상태는 아래 문서에 정리되어 있다.

- [구현 프롬프트](docs/prompt.md)
- [단계별 로드맵](docs/todo/roadmap.md)
- [Step 11 진행 상황](docs/todo/step-11-progress.md)
- [Step 12 진행 상황](docs/todo/step-12-progress.md)
- [Step 13 진행 상황](docs/todo/step-13-progress.md)
- [Step 14 진행 상황](docs/todo/step-14-progress.md)
- [Step 15 진행 상황](docs/todo/step-15-progress.md)
- [Step 16 진행 상황](docs/todo/step-16-progress.md)
- [Step 17 진행 상황](docs/todo/step-17-progress.md)
- [Step 18 진행 상황](docs/todo/step-18-progress.md)

## 기술 스택

- Rust 2021 edition
- Cargo
- serde
- serde_json
- GlueSQL
- futures

## 저장소 설명

GitHub repository description으로는 아래 문구를 권장한다.

```text
Rust + GlueSQL 미니 프로젝트: CLI Todo List
```
