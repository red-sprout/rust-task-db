# 프로젝트 전체 개요

## 프로젝트 한 줄 요약

`rust-task`는 Rust 문법을 단계별로 배우기 위한 CLI Todo 앱이며, 현재는 Step 18로 Storage별 기능 차이를 현재 코드와 연결해 분석한다. CLI 기능 구현은 Step 12에서 GlueSQL SQL 실행 모드, REPL 모드, 테스트 보강, SledStorage 영속 저장까지 완료되어 있다.

## 이 프로젝트가 해결하는 문제

Rust 초심자가 `struct`, `enum`, `trait`, generic, `impl`, `Vec`, ownership, borrowing, `Option`, `Result`, `match`, custom error, 파일 I/O, serde JSON 변환, GlueSQL 저장소 교체, SQL 결과 매핑, storage별 transaction 차이, GlueSQL Parser/Planner/Executor/Store 흐름, Minimal Custom Storage 책임, Query Execution 변환 흐름, Storage별 기능 차이를 실제 코드 안에서 볼 수 있게 한다.

## 핵심 기능 목록

현재 구현:

- `cargo run -- add "Rust 공부"`: `TaskService`를 통해 Todo를 GlueSQL `SledStorage`에 추가
- `cargo run -- list`: `TaskService`를 통해 현재 프로세스의 Todo 목록 출력
- `cargo run -- done 1`: `TaskService`를 통해 Todo를 완료 처리
- `cargo run -- delete 1`: `TaskService`를 통해 Todo를 삭제
- `cargo run -- search rust`: 제목에 keyword가 포함된 Todo 검색
- `cargo run -- stats`: 전체/완료/미완료 개수 출력
- `cargo run -- sql "SELECT * FROM tasks"`: GlueSQL에 SQL 문자열 직접 전달
- `cargo run -- repl`: 작은 SQL 콘솔 실행

이후 단계 예정:

- 코드에서 확인되지 않음

## 사용 기술 스택

- Rust 2021 edition
- Cargo
- Rust standard library: `std::env`, `std::fs`, `std::path`, `std::io`
- 외부 crate: `serde`, `serde_json`, `gluesql`, `futures`

## 전체 실행 구조

```text
cargo run -- add "Rust 공부"
-> src/main.rs main()
-> std::env::args().collect()
-> src/cli.rs parse_args(args)
-> GlueSqlTaskRepository::persistent("data/rust-task-db")
-> CREATE TABLE tasks (...)
-> TaskService::new(repository)
-> src/command.rs Command::Add { title }
-> service.add(title)
-> TaskRepository::add(&mut repository, title)
-> GlueSQL INSERT 실행
-> print_task(&task)
```

## 초심자가 알아야 할 큰 그림

Step 10의 핵심 변화는 `sql` 명령을 한 번 실행하는 것에서 나아가, `repl` 안에서 여러 SQL을 순서대로 실행할 수 있게 된 것이다.
Step 11의 핵심 변화는 새 기능 추가가 아니라 기존 구조를 테스트로 더 단단하게 묶은 것이다.
Step 12의 핵심 변화는 `MemoryStorage` 대신 `SledStorage`를 사용해서 CLI 실행 간 Todo가 유지되게 한 것이다.
Step 13의 핵심 변화는 새 기능 추가가 아니라 현재 코드와 문서가 같은 상태를 설명하는지 점검하는 것이다.
Step 14의 핵심 변화는 새 CLI 명령 없이 `src/repository/gluesql_repository.rs` 테스트에서 `MemoryStorage` transaction 미지원, `SledStorage` rollback, repeatable read snapshot, write lock 충돌을 관찰하는 것이다.
Step 15의 핵심 변화는 새 CLI 명령 없이 `Glue::execute` 뒤의 Parser, Planner, Executor, Store 흐름과 Storage Adapter 책임을 문서화하고, commit/nested transaction/SQL 미지원 경계 테스트를 추가한 것이다.
Step 16의 핵심 변화는 production custom storage를 추가하지 않고, custom storage를 만들 때 필요한 읽기/쓰기/transaction/index 책임을 문서화한 것이다.
Step 17의 핵심 변화는 새 CLI 명령 없이 Todo 명령별 SQL 생성과 `Payload`가 `Task`, `TaskStats`, `SqlResult`로 변환되는 경로를 문서화한 것이다.
Step 18의 핵심 변화는 새 storage를 도입하지 않고, 현재 코드에서 실제 사용하는 storage와 문서 비교 대상 storage를 표로 구분한 것이다.

```text
src/main.rs
-> TaskService
-> TaskRepository trait
-> GlueSqlTaskRepository
-> GlueSQL SledStorage
```

Step 7까지는 JSON 파일과 iterator 검색/통계가 중심이었다. Step 8에서는 `Glue::new`, `MemoryStorage`, `execute(...).await`, `block_on`, `Payload`, `Value`가 새로 중요해진다.

Step 9에서는 `Command::Sql`, `TaskRepository::execute_sql`, `SqlResult`, SQL 결과 출력이 새로 중요해졌다.
Step 10에서는 `Command::Repl`, `src/repl.rs`, `BufRead`, `Write`, `.schema`, `.exit`, `.quit`이 새로 중요해진다.

주의: Step 12의 GlueSQL 저장소는 `SledStorage`라서 프로그램이 끝나도 `data/rust-task-db`에 데이터가 남는다. REPL 안에서는 같은 실행도 유지된다.

## 대표 요청 흐름 요약

```text
사용자 CLI
-> src/cli.rs
-> src/command.rs
-> src/main.rs
-> src/service.rs
-> TaskRepository
-> GlueSqlTaskRepository
-> src/error.rs AppError
-> src/task.rs
-> 터미널 출력
```

## 문서에서 사용할 용어 기준

- Todo: `Task` 값 한 개
- 목록: `Vec<Task>`
- 현재 활성 저장소: GlueSQL `SledStorage`
- 현재 저장 위치: `data/rust-task-db`
- 현재 transaction 관찰 위치: `src/repository/gluesql_repository.rs` 테스트
- 현재 GlueSQL 내부 흐름 해설 위치: `docs/beginner-codebase-guide/17-gluesql-internals.md`
- 현재 custom storage 분석 위치: `docs/beginner-codebase-guide/18-custom-storage.md`
- 현재 query execution 분석 위치: `docs/beginner-codebase-guide/19-query-execution.md`
- 현재 storage 비교표 위치: `docs/beginner-codebase-guide/20-storage-comparison.md`
- 보존된 저장 파일: `tasks.json`
- SQL 결과 모델: `SqlResult`
- REPL 모듈: `src/repl.rs`
- 테스트 보강: `src/task.rs`, `src/cli.rs`, `src/repl.rs`, `src/repository/gluesql_repository.rs`
- CLI 인자: `std::env::args()`로 들어온 문자열들
- 명령 모델: `Command`
- 현재 실행: 한 번의 `cargo run -- ...`
