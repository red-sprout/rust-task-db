# Codex 단계별 작업 절차

이 문서는 앞으로 Codex가 `rust-task`를 단계 순서에 맞춰 작업하기 위한 절차다.

## 현재 단계 확인

작업 전 항상 아래 파일을 확인한다.

```text
AGENTS.md
docs/todo/roadmap.md
docs/todo/step-11-progress.md
docs/prompt.md
docs/review_docs.md
docs/beginner-codebase-guide/99-index.md
```

## 절대 원칙

- 현재 단계보다 뒤 기능을 먼저 구현하지 않는다.
- 뒤 단계 문법을 현재 단계 코드에 섞지 않는다.
- 단계가 바뀌면 코드와 문서를 같이 갱신한다.
- 구현을 바꾸면 `docs/beginner-codebase-guide/`도 현재 코드 기준으로 갱신한다.

## 현재 Step 11 작업 절차

Step 11에서는 새 기능을 추가하지 않고 테스트를 보강한다. 아래 파일이 현재 구현 범위다.

```text
src/command.rs
src/cli.rs
src/main.rs
src/repl.rs
src/error.rs
src/service.rs
src/task.rs
src/repository/mod.rs
src/repository/gluesql_repository.rs
tasks.json
docs/todo/step-10-progress.md
docs/todo/step-11-progress.md
docs/todo/roadmap.md
docs/beginner-codebase-guide/
```

검증:

```bash
cargo fmt --check
cargo test
cargo run -- add "Rust 공부"
cargo run -- list
cargo run -- search rust
cargo run -- stats
cargo run -- sql "SELECT * FROM tasks"
cargo run -- repl
```

주의: Step 11도 GlueSQL `MemoryStorage`라서 `cargo run`을 여러 번 나눠 실행하면 데이터가 유지되지 않을 수 있다. `repl` 안에서는 같은 저장소 인스턴스를 계속 쓰므로 `INSERT` 후 `SELECT`가 이어진다.

## Step 2 구현 완료 상태

추가된 파일:

```text
src/command.rs
src/cli.rs
```

구현된 것:

- `Command` enum
- `std::env::args` 기반 parser
- `match command` 실행 구조
- 잘못된 명령어 처리

## Step 3 구현 완료 상태

추가된 것:

- `serde`
- `serde_json`
- `tasks.json`
- 파일 읽기/쓰기

## Step 4 구현 완료 상태

추가된 것:

- `src/repository/mod.rs`
- `TaskRepository` trait
- `JsonTaskRepository`
- repository 단위 테스트

## Step 5 구현 완료 상태

추가된 것:

- `src/service.rs`
- `TaskService<R: TaskRepository>`
- service 단위 테스트

## Step 6 구현 완료 상태

추가된 것:

- `src/error.rs`
- `AppError`
- `Display`, `Error`, `From`
- error 단위 테스트

## Step 7 구현 완료 상태

추가된 것:

- `Command::Search`
- `Command::Stats`
- `TaskStats`
- repository/service search와 stats
- search/stats 테스트

## Step 8 구현 완료 상태

추가된 것:

- `src/repository/gluesql_repository.rs`
- `GlueSqlTaskRepository`
- `gluesql`, `futures`
- GlueSQL `MemoryStorage`
- `AppError::GlueSql`
- GlueSQL repository 테스트

## Step 9 구현 완료 상태

추가된 것:

- `Command::Sql { sql }`
- `TaskRepository::execute_sql`
- `TaskService::execute_sql`
- `SqlResult`
- `print_sql_results`
- SQL 실행 테스트

## Step 10 구현 완료 상태

추가된 것:

- `src/repl.rs`
- `Command::Repl`
- `repl` 명령 parsing
- `.schema`, `.exit`, `.quit`
- REPL 테스트

주의: Step 11에서는 새 기능보다 테스트 보강이 중심이다.

## Step 11 구현 완료 상태

추가된 것:

- `src/task.rs` domain 테스트
- `src/cli.rs` parser 실패 테스트
- `src/repl.rs` REPL 빈 줄/SQL 실패 지속 테스트
- `src/repository/gluesql_repository.rs` next id/GlueSQL 에러 타입 테스트
- 총 57개 테스트

## 문서 작성 절차

문서를 작성할 때는 현재 코드에 실제로 존재하는 파일만 현재 구현으로 설명한다. 뒤 단계는 “예정” 또는 “TODO”로 표시한다.

## 완료 보고 형식

```text
변경:
- 현재 단계 기준으로 무엇을 바꿨는지

검증:
- 실행한 명령과 결과

다음:
- 다음 단계에서 할 일
```
