# 프로젝트 파일 지도

## 전체 디렉터리 트리

```text
rust-task-db/
├── README.md
├── Cargo.toml
├── Cargo.lock
├── AGENTS.md
├── tasks.json
├── data/                  # git 추적 안 함
├── src/
│   ├── main.rs
│   ├── repl.rs
│   ├── error.rs
│   ├── service.rs
│   ├── command.rs
│   ├── cli.rs
│   ├── repository/
│   │   ├── mod.rs
│   │   └── gluesql_repository.rs
│   └── task.rs
└── docs/
    ├── prompt.md
    ├── review_docs.md
    ├── todo/
    │   ├── roadmap.md
    │   ├── step-1-progress.md
    │   ├── step-2-progress.md
    │   ├── step-3-progress.md
    │   ├── step-4-progress.md
    │   ├── step-5-progress.md
    │   ├── step-6-progress.md
    │   ├── step-7-progress.md
    │   ├── step-8-progress.md
    │   ├── step-9-progress.md
    │   ├── step-10-progress.md
    │   ├── step-11-progress.md
    │   ├── step-12-progress.md
    │   └── step-13-progress.md
    └── beginner-codebase-guide/
```

## 루트 파일 설명

| 항목 | 내용 |
| --- | --- |
| 파일 경로 | [README.md](../../README.md) |
| 역할 | GitHub 첫 화면용 프로젝트 소개 문서 |
| 이 파일이 필요한 이유 | 프로젝트 목적, 실행 방법, 테스트 방법, MemoryStorage 주의점을 빠르게 보여주기 위해 필요하다. |
| 연결된 파일 | [docs/beginner-codebase-guide/99-index.md](99-index.md), [docs/beginner-codebase-guide/16-run-guide.md](16-run-guide.md), [src/main.rs](../../src/main.rs) |
| 초심자가 봐야 할 핵심 | 현재 Step 13 상태, 지원 명령, `SledStorage` 저장 위치 |
| 설명 깊이 | 짧은 설명으로 충분 |

| 항목 | 내용 |
| --- | --- |
| 파일 경로 | `Cargo.toml` |
| 역할 | Rust package 설정과 dependency 선언 |
| 이 파일이 필요한 이유 | Cargo가 프로젝트 이름, 버전, edition, 외부 crate를 알기 위해 필요하다. |
| 연결된 파일 | `src/main.rs`, `src/error.rs`, `src/service.rs`, `src/task.rs` |
| 초심자가 봐야 할 핵심 | `serde`, `serde_json`, `gluesql`, `futures` dependency |
| 설명 깊이 | 중간 설명 필요 |

| 항목 | 내용 |
| --- | --- |
| 파일 경로 | `tasks.json` |
| 역할 | Step 7까지 사용한 Todo 데이터 저장 파일 |
| 이 파일이 필요한 이유 | JSON 저장소 구현을 보존하기 위해 남겨둔다. Step 12 기본 실행 경로에서는 사용하지 않는다. |
| 연결된 파일 | `src/repository/mod.rs`, `src/task.rs` |
| 초심자가 봐야 할 핵심 | 기존 파일을 삭제하지 않고 저장소를 갈아끼우는 구조 |
| 설명 깊이 | 상세 설명 필요 |

| 항목 | 내용 |
| --- | --- |
| 파일 경로 | `data/rust-task-db` |
| 역할 | Step 12 기본 실행 데이터가 저장되는 SledStorage 디렉터리 |
| 이 파일이 필요한 이유 | CLI 명령을 여러 번 나눠 실행해도 Todo가 유지되게 한다. |
| 연결된 파일 | `src/main.rs`, `src/repository/gluesql_repository.rs`, `.gitignore` |
| 초심자가 봐야 할 핵심 | 이 디렉터리는 실행 중 생성되며 git에 커밋하지 않는다. |
| 설명 깊이 | 중간 설명 필요 |

| 항목 | 내용 |
| --- | --- |
| 파일 경로 | [docs/todo/step-13-progress.md](../todo/step-13-progress.md) |
| 역할 | Step 13 최종 검증 및 문서 정합성 점검 진행 상태 |
| 이 파일이 필요한 이유 | 새 기능을 추가하지 않는 최종 점검 단계임을 명확히 남기기 위해 필요하다. |
| 연결된 파일 | [docs/todo/roadmap.md](../todo/roadmap.md), [README.md](../../README.md), [docs/project-plan.md](../project-plan.md), [docs/step-by-step-workflow.md](../step-by-step-workflow.md) |
| 초심자가 봐야 할 핵심 | Step 13은 기능 추가가 아니라 현재 코드와 문서가 일치하는지 확인하는 단계다. |
| 설명 깊이 | 짧은 설명으로 충분 |

| 항목 | 내용 |
| --- | --- |
| 파일 경로 | `Cargo.lock` |
| 역할 | Cargo가 생성한 lock 파일 |
| 이 파일이 필요한 이유 | 빌드 재현성을 위해 필요하다. |
| 연결된 파일 | `Cargo.toml` |
| 초심자가 봐야 할 핵심 | 직접 수정하지 않는다. |
| 설명 깊이 | 짧은 설명으로 충분 |

## 소스 코드 디렉터리 설명

| 항목 | 내용 |
| --- | --- |
| 파일 경로 | `src/main.rs` |
| 역할 | 프로그램 시작점, `Command` 실행, SledStorage 기반 GlueSQL repository 생성, service 메서드 호출, Todo/SQL 결과 출력 |
| 이 파일이 필요한 이유 | CLI 명령이 service 호출로 이어지는 곳이다. |
| 연결된 파일 | `src/cli.rs`, `src/command.rs`, `src/service.rs`, `src/repository/mod.rs`, `src/task.rs` |
| 초심자가 봐야 할 핵심 | `main`, `GlueSqlTaskRepository::persistent`, `TaskService::new`, `service.add`, `service.execute_sql`, `repl::run_repl` |
| 설명 깊이 | 상세 설명 필요 |

| 항목 | 내용 |
| --- | --- |
| 파일 경로 | `src/repl.rs` |
| 역할 | REPL 입력 루프, `.schema`, `.exit`, `.quit`, SQL 결과 출력 |
| 이 파일이 필요한 이유 | `cargo run -- repl` 실행 중 여러 SQL을 같은 저장소 인스턴스에서 실행하기 위해 필요하다. |
| 연결된 파일 | `src/main.rs`, `src/service.rs`, `src/repository/mod.rs` |
| 초심자가 봐야 할 핵심 | `run_repl`, `run_repl_with_io`, `BufRead`, `Write`, `service.execute_sql` |
| 설명 깊이 | 상세 설명 필요 |

| 항목 | 내용 |
| --- | --- |
| 파일 경로 | `src/error.rs` |
| 역할 | `AppError` custom error 정의 |
| 이 파일이 필요한 이유 | CLI, service, repository의 실패 타입을 하나로 모으기 위해 필요하다. |
| 연결된 파일 | `src/cli.rs`, `src/service.rs`, `src/repository/mod.rs`, `src/main.rs` |
| 초심자가 봐야 할 핵심 | `AppError`, `Display`, `Error`, `From` |
| 설명 깊이 | 상세 설명 필요 |

| 항목 | 내용 |
| --- | --- |
| 파일 경로 | `src/service.rs` |
| 역할 | `TaskService<R: TaskRepository>` 구현 |
| 이 파일이 필요한 이유 | `main.rs`와 repository 사이에 비즈니스 로직 계층을 둔다. |
| 연결된 파일 | `src/main.rs`, `src/repository/mod.rs`, `src/task.rs` |
| 초심자가 봐야 할 핵심 | `TaskService`, `R: TaskRepository`, `service.add`, `service.done` |
| 설명 깊이 | 상세 설명 필요 |

| 항목 | 내용 |
| --- | --- |
| 파일 경로 | `src/repository/mod.rs` |
| 역할 | `TaskRepository` trait, `SqlResult`, 기존 `JsonTaskRepository`, `GlueSqlTaskRepository` module 연결 |
| 이 파일이 필요한 이유 | 저장소 책임을 `main.rs`에서 분리하고 구현체를 교체할 수 있게 한다. |
| 연결된 파일 | `src/main.rs`, `src/task.rs`, `tasks.json`, `src/repository/gluesql_repository.rs` |
| 초심자가 봐야 할 핵심 | `TaskRepository`, `execute_sql`, `SqlResult`, `JsonTaskRepository`, `pub use gluesql_repository::GlueSqlTaskRepository` |
| 설명 깊이 | 상세 설명 필요 |

| 항목 | 내용 |
| --- | --- |
| 파일 경로 | `src/repository/gluesql_repository.rs` |
| 역할 | `GlueSqlTaskRepository<S>` 구현과 SQL 실행 결과 변환 |
| 이 파일이 필요한 이유 | GlueSQL `SledStorage`로 Todo를 저장하고, Step 9의 `sql` 명령을 실행하는 repository 구현체다. 테스트에서는 `MemoryStorage`도 사용한다. |
| 연결된 파일 | `src/repository/mod.rs`, `src/main.rs`, `src/task.rs`, `src/error.rs` |
| 초심자가 봐야 할 핵심 | `Glue::new`, `SledStorage`, `MemoryStorage`, `persistent`, `execute`, `execute_sql`, `payload_to_sql_result`, `Payload`, `Value`, `block_on` |
| 설명 깊이 | 상세 설명 필요 |

| 항목 | 내용 |
| --- | --- |
| 파일 경로 | `src/task.rs` |
| 역할 | `Task` struct, `Task::new`, serde derive |
| 이 파일이 필요한 이유 | Todo 데이터 모양, JSON 변환 가능 여부, Task/TaskStats 테스트를 정의한다. |
| 연결된 파일 | `src/main.rs`, `tasks.json` |
| 초심자가 봐야 할 핵심 | `Serialize`, `Deserialize`, `id`, `title`, `done` |
| 설명 깊이 | 상세 설명 필요 |

| 항목 | 내용 |
| --- | --- |
| 파일 경로 | `src/command.rs` |
| 역할 | CLI 명령을 `Command` enum으로 표현 |
| 이 파일이 필요한 이유 | 문자열 명령과 실제 실행 로직 사이에 명확한 타입을 둔다. |
| 연결된 파일 | `src/cli.rs`, `src/main.rs` |
| 초심자가 봐야 할 핵심 | `Command::Add { title }`, `Command::Done { id }`, `Command::Sql { sql }`, `Command::Repl` |
| 설명 깊이 | 상세 설명 필요 |

| 항목 | 내용 |
| --- | --- |
| 파일 경로 | `src/cli.rs` |
| 역할 | CLI 인자 `Vec<String>`을 `Result<Command, AppError>`로 변환 |
| 이 파일이 필요한 이유 | `main.rs`가 문자열 parsing 세부사항을 직접 알지 않아도 된다. |
| 연결된 파일 | `src/command.rs`, `src/main.rs` |
| 초심자가 봐야 할 핵심 | `parse_args`, `require_next`, `parse_id`, parser tests |
| 설명 깊이 | 상세 설명 필요 |

## 설정 파일 설명

별도 런타임 설정 파일은 코드에서 확인되지 않음. Step 12 기본 저장소는 GlueSQL `SledStorage`이며 `src/main.rs`에서 `"data/rust-task-db"` 경로를 직접 넘긴다.

## 테스트 디렉터리 설명

별도 `tests/` 디렉터리는 없다. 테스트는 `src/main.rs`, `src/error.rs`, `src/cli.rs`, `src/service.rs`, `src/repository/mod.rs`, `src/repository/gluesql_repository.rs`의 `#[cfg(test)] mod tests`에 있다.

## 배포/인프라 파일 설명

Docker, CI/CD, 배포 설정은 코드에서 확인되지 않음.

## 초심자가 몰라도 되는 파일

- `target/`: Cargo 빌드 결과물
- `Cargo.lock`: 처음에는 직접 읽지 않아도 된다.

## 파일 간 연결 관계 요약

```text
src/main.rs
-> cli::parse_args(...)
-> GlueSqlTaskRepository::persistent("data/rust-task-db")
-> TaskService::new(repository)
-> match Command
-> service.add/list/done/delete/search/stats/execute_sql 또는 repl::run_repl

src/service.rs
-> TaskRepository trait
-> repository.add/find_all/mark_done/delete/search/stats/execute_sql

src/repository/mod.rs
-> JsonTaskRepository 보존
-> GlueSqlTaskRepository re-export

src/repository/gluesql_repository.rs
-> GlueSQL SledStorage
-> CREATE TABLE tasks
-> INSERT/SELECT/UPDATE/DELETE/COUNT/user SQL
-> Payload를 SqlResult로 변환

src/task.rs
-> Serialize / Deserialize
-> Task / TaskStats
```
