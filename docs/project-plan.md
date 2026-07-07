# rust-task 기획 문서

## 한 줄 요약

`rust-task`는 Rust 문법을 단계별로 학습하기 위해 만드는 CLI Todo 앱이다.

## 현재 단계

현재 구현 단계는 `Step 12. GlueSQL SledStorage 영속 저장소 전환`이다.

현재 코드는 CLI 문자열을 `Command` enum으로 변환한 뒤, `TaskService<GlueSqlTaskRepository<SledStorage>>`를 통해 Todo 기능을 실행한다. `add`, `list`, `done`, `delete`, `search`, `stats`, `sql`, `repl`을 지원하며, 실패는 `AppError`로 표현한다. 기존 `JsonTaskRepository`, `tasks.json`, MemoryStorage 테스트 흐름은 삭제하지 않고 보존한다.

## 현재 Step 12 목표

지원 명령:

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

학습 개념:

- 변수와 기본 타입
- 함수
- `struct`
- `enum`
- `trait`
- `impl Trait for Type`
- `Vec`
- `Option`
- `Result`
- `match`
- ownership
- borrowing
- mutable reference
- iterator 기반 CLI parsing
- 파일 I/O
- serde / serde_json
- JSON parsing
- repository 책임 분리
- service layer 분리
- custom error
- `Display`, `Error`, `From`
- iterator
- closure
- `filter`
- `count`
- `TaskStats`
- GlueSQL
- `MemoryStorage`
- `Payload`
- `Value`
- `block_on`
- `Command::Sql`
- `TaskRepository::execute_sql`
- `SqlResult`
- SQL 결과 출력
- REPL 입력 루프
- `BufRead`
- `Write`
- 표준 입력/출력
- Rust 내장 test harness
- `matches!`
- parser 실패 테스트
- REPL 에러 지속 테스트
- GlueSQL 에러 타입 테스트
- SledStorage
- 파일 기반 영속 저장

## 현재 아키텍처

```text
터미널 입력
-> src/cli.rs
-> src/command.rs
-> src/main.rs
-> src/error.rs
-> src/service.rs
-> src/repository/mod.rs
-> src/repository/gluesql_repository.rs
-> GlueSQL SledStorage
-> src/task.rs
-> 터미널 출력
```

Step 12에서는 Step 11의 테스트 보강 흐름을 유지하면서 기본 저장소를 GlueSQL `MemoryStorage`에서 `SledStorage`로 전환한다. `main.rs`는 `GlueSqlTaskRepository::persistent("data/rust-task-db")`를 호출하고, `src/repository/gluesql_repository.rs`는 `GlueSqlTaskRepository<S>` generic 구조로 MemoryStorage 테스트와 SledStorage 실행을 함께 지원한다.

## 현재 파일 구조

```text
rust-task/
  Cargo.toml
  tasks.json
  src/
    main.rs
    repl.rs
    error.rs
    service.rs
    command.rs
    cli.rs
    task.rs
    repository/
      mod.rs
      gluesql_repository.rs
  docs/
    prompt.md
    todo/
      step-1-progress.md
      step-2-progress.md
      step-3-progress.md
      step-4-progress.md
      step-5-progress.md
      step-6-progress.md
      step-7-progress.md
      step-8-progress.md
      step-9-progress.md
      step-10-progress.md
      step-11-progress.md
      step-12-progress.md
      roadmap.md
```

## 현재 제한

Custom error는 `AppError`로 구현되어 있다.

Step 12의 활성 저장소는 GlueSQL `SledStorage`다. 아래 두 명령은 별도 프로세스지만 `data/rust-task-db`에 저장된 데이터를 공유한다.

```bash
cargo run -- add "Rust 공부"
cargo run -- list
```

같은 repository 인스턴스 안에서의 add/list 흐름은 `src/repository/gluesql_repository.rs` 테스트가 검증한다.

SQL로 데이터를 넣고 바로 조회하려면 한 SQL 문자열 안에 여러 statement를 함께 넣는다.

```bash
cargo run -- sql "INSERT INTO tasks VALUES (1, 'Rust 공부', FALSE); SELECT * FROM tasks;"
```

또는 REPL 안에서 같은 저장소 인스턴스를 유지하며 여러 SQL을 순서대로 실행한다.

```bash
cargo run -- repl
```

## 이후 단계 요약

현재 `docs/prompt.md` 기준 단계 구현과 Step 8의 영속 저장소 확장은 Step 12까지 완료되어 있다. 이후 작업은 새 요구가 있을 때 별도 단계로 계획한다.

## 완료 기준

- 현재 단계 범위를 넘지 않는다.
- `cargo fmt --check`가 통과한다.
- `cargo test`가 통과한다.
- 현재 단계 문서가 코드와 일치한다.
