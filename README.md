# rust-task-db

Rust + GlueSQL 미니 프로젝트: CLI Todo List

`rust-task-db`는 Rust 문법과 간단한 아키텍처 분리를 단계별로 학습하기 위한 CLI Todo 프로젝트다. 현재는 `Command` enum, CLI parser, service layer, repository trait, custom error, GlueSQL `SledStorage`, SQL 실행 모드, REPL 모드, 테스트 보강까지 구현되어 있다.

## 현재 상태

| 항목 | 내용 |
| --- | --- |
| 현재 단계 | Step 13. 최종 검증 및 문서 정합성 점검 |
| 실행 방식 | Cargo로 실행하는 CLI 앱 |
| 활성 저장소 | `GlueSqlTaskRepository` + GlueSQL `SledStorage` |
| 저장 위치 | `data/rust-task-db` |
| 보존된 저장소 | `JsonTaskRepository` + `tasks.json`, MemoryStorage 테스트 흐름 |
| 테스트 | `cargo test` 기준 58개 |

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

현재 테스트는 domain, CLI parser, custom error, service, JSON repository, GlueSQL repository, SQL 실행, REPL 흐름을 검증한다.

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

- `docs/beginner-codebase-guide/99-index.md`
- `docs/beginner-codebase-guide/16-run-guide.md`
- `docs/beginner-codebase-guide/00-overview.md`
- `docs/beginner-codebase-guide/11-testing.md`

작업 단계와 진행 상태는 아래 문서에 정리되어 있다.

- `docs/prompt.md`
- `docs/todo/roadmap.md`
- `docs/todo/step-11-progress.md`
- `docs/todo/step-12-progress.md`
- `docs/todo/step-13-progress.md`

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
