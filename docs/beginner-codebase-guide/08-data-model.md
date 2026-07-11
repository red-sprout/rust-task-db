# 데이터 모델 해설

## Step 28 관계 모델

| Rust 타입 | GlueSQL table | 핵심 필드 |
| --- | --- | --- |
| `Project` | `projects` | `id`, `name` |
| `Task` | `tasks` | `id`, `project_id`, `title`, `done`, `priority` |
| `Tag` | `tags` | `id`, `name` |
| 공개 struct 없음 | `task_tags` | `task_id`, `tag_id` |

`TaskDetail`은 Task, 선택적 Project 이름, Tag 목록을 합친 조회 모델이다.

## 이 프로젝트의 데이터 흐름 큰 그림

```text
CLI 문자열
-> Command
-> TaskService
-> TaskRepository
-> GlueSqlTaskRepository
-> GlueSQL SledStorage
-> AppError
-> String title, String keyword, i64 id
-> Task
-> Vec<Task>
-> TaskStats
-> SqlResult
-> REPL line
-> println!
```

## Entity/Model 목록

| 항목 | 내용 |
| --- | --- |
| 이름 | `Task` |
| 파일 경로 | `src/task.rs` |
| 역할 | Todo 한 건 |
| 필드 목록 | `id: i64`, `title: String`, `done: bool` |
| 각 필드의 의미 | id는 식별자, title은 제목, done은 완료 여부 |
| 어디서 생성되는가 | `Task::new` |
| 어디서 사용되는가 | `src/repository/mod.rs`의 `add`, `mark_done`, `delete`, `src/main.rs`의 `print_task` |
| DB에 저장되는가 | Step 40 현재는 GlueSQL `tasks` table에 저장됨 |
| 실제 저장 위치 | `data/rust-task-db` |
| 외부 응답으로 노출되는가 | CLI 출력으로 노출 |
| 수정 시 영향받는 파일 | `src/task.rs`, `src/main.rs`, 테스트, 이 문서 세트 |

## Command 모델

| 항목 | 내용 |
| --- | --- |
| 이름 | `Command` |
| 파일 경로 | `src/command.rs` |
| 역할 | CLI 명령을 타입으로 표현 |
| variant 목록 | `Add`, `List`, `Done`, `Delete`, `Search`, `Stats`, `Sql`, `Repl`, `Help` |
| 어디서 생성되는가 | `src/cli.rs`의 `parse_args` |
| 어디서 사용되는가 | `src/main.rs`의 `match command` |
| DB에 저장되는가 | 저장되지 않음 |
| 수정 시 영향받는 파일 | `src/command.rs`, `src/cli.rs`, `src/main.rs`, parser 테스트, 이 문서 세트 |

## DTO/Request/Response 목록

웹 API DTO는 코드에서 확인되지 않음. 현재는 CLI 인자 `Vec<String>`이 입력이고 `Command`가 내부 request 모델 역할을 한다.

## Error 모델

| 항목 | 내용 |
| --- | --- |
| 이름 | `AppError` |
| 파일 경로 | `src/error.rs` |
| 역할 | 애플리케이션 실패 종류 표현 |
| variant 목록 | `Io`, `Json`, `GlueSql`, `NotFound`, `InvalidCommand`, `Unsupported` |
| 어디서 생성되는가 | `src/cli.rs`, `src/repository/mod.rs`, `src/repository/gluesql_repository.rs` |
| 어디서 사용되는가 | `src/main.rs`, `src/service/mod.rs`, `src/repository/mod.rs`, `src/repository/gluesql_repository.rs`, 테스트 |
| 수정 시 영향받는 파일 | `src/error.rs`, 실패를 반환하는 코드, 테스트, 이 문서 세트 |

## Service 모델

| 항목 | 내용 |
| --- | --- |
| 이름 | `TaskService<R: TaskRepository>` |
| 파일 경로 | `src/service/mod.rs` |
| 역할 | CLI 실행 계층과 repository 계층 사이의 service layer |
| 메서드 목록 | `new`, `add`, `list`, `done`, `delete`, `search`, `stats`, `execute_sql` |
| 어디서 생성되는가 | `src/main.rs`의 `TaskService::new(repository)` |
| 어디서 사용되는가 | `src/main.rs`의 `service.add`, `service.list`, `service.done`, `service.delete`, `service.search`, `service.stats`, `service.execute_sql` |
| 수정 시 영향받는 파일 | `src/service/mod.rs`, `src/main.rs`, 테스트, 이 문서 세트 |

## Repository 모델

| 항목 | 내용 |
| --- | --- |
| 이름 | `TaskRepository` |
| 파일 경로 | `src/repository/mod.rs` |
| 역할 | Todo 저장소가 제공해야 하는 동작 약속 |
| 메서드 목록 | `add`, `find_all`, `mark_done`, `delete`, `search`, `stats`, `execute_sql` |
| 구현체 | `GlueSqlTaskRepository`, `JsonTaskRepository` |
| 수정 시 영향받는 파일 | `src/repository/mod.rs`, `src/service/mod.rs`, 테스트, 이 문서 세트 |

| 항목 | 내용 |
| --- | --- |
| 이름 | `JsonTaskRepository` |
| 파일 경로 | `src/repository/mod.rs` |
| 역할 | `tasks.json` 기반 Todo 저장소. Step 9에서는 삭제하지 않고 보존된 구현체 |
| 내부 필드 | `path: PathBuf`, `tasks: Vec<Task>` |
| 어디서 생성되는가 | JSON repository 테스트에서 `JsonTaskRepository::new(&path)`로 생성 |
| 어디서 사용되는가 | `src/repository/mod.rs` 테스트, 보존된 교체 가능 구현체 |

| 항목 | 내용 |
| --- | --- |
| 이름 | `GlueSqlTaskRepository` |
| 파일 경로 | `src/repository/gluesql_repository.rs` |
| 역할 | GlueSQL storage 기반 Todo 저장소 |
| 내부 필드 | `glue: Glue<S>` |
| 어디서 생성되는가 | `src/main.rs`의 `GlueSqlTaskRepository::persistent("data/rust-task-db")` |
| 어디서 사용되는가 | `TaskService<GlueSqlTaskRepository<TracingStorage<SledStorage>>>`, GlueSQL repository 테스트 |

## SQL 결과 모델

| 항목 | 내용 |
| --- | --- |
| 이름 | `SqlResult` |
| 파일 경로 | `src/repository/mod.rs` |
| 역할 | GlueSQL 실행 결과를 CLI 출력에 맞게 표현 |
| variant 목록 | `Select`, `Affected`, `Message` |
| 어디서 생성되는가 | `src/repository/gluesql_repository.rs`의 `payload_to_sql_result` |
| 어디서 사용되는가 | `src/main.rs`의 `print_sql_results`, SQL 실행 테스트 |
| DB에 저장되는가 | 저장되지 않음 |

## REPL 입력 모델

| 항목 | 내용 |
| --- | --- |
| 이름 | REPL line |
| 파일 경로 | `src/repl.rs` |
| 역할 | 사용자가 REPL 안에서 입력한 한 줄 |
| 값 종류 | `.schema`, `.exit`, `.quit`, SQL 문자열 |
| 어디서 생성되는가 | `input.read_line(&mut line)` |
| 어디서 사용되는가 | `run_repl_with_io`의 `match command` |
| DB에 저장되는가 | 저장되지 않음 |

## 통계 모델

| 항목 | 내용 |
| --- | --- |
| 이름 | `TaskStats` |
| 파일 경로 | `src/task.rs` |
| 역할 | Todo 전체/완료/미완료 개수 |
| 필드 목록 | `total: usize`, `done: usize`, `todo: usize` |
| 어디서 생성되는가 | `TaskStats::new`, `GlueSqlTaskRepository::stats`, `JsonTaskRepository::stats` |
| 어디서 사용되는가 | `src/main.rs`의 `print_stats` |

## Schema/Table/Collection 목록

Step 40 현재도 GlueSQL table schema가 있다.

Step 12 저장 흐름:

```text
Task
-> INSERT INTO tasks ...
-> GlueSqlTaskRepository<TracingStorage<SledStorage>>
-> data/rust-task-db
```

```sql
CREATE TABLE tasks (
  id INTEGER,
  title TEXT,
  done BOOLEAN
);
```

`src/repository/gluesql_repository.rs`의 `GlueSqlTaskRepository::persistent`가 이 table을 준비한다.

주의: 현재 storage는 `SledStorage`라서 table과 데이터가 `data/rust-task-db`에 유지된다.

## 보존된 JSON 저장 구조

| 항목 | 내용 |
| --- | --- |
| 파일 경로 | `tasks.json` |
| 역할 | Step 7까지 사용한 Todo 목록 저장 |
| JSON 최상위 타입 | 배열 |
| 배열 원소 | `Task` 객체 |

예시:

```json
[
  {
    "id": 1,
    "title": "Rust 공부",
    "done": false
  }
]
```

## 데이터 변환 흐름

add:

```text
Vec<String>
-> parse_args
-> Command::Add { title }
-> GlueSqlTaskRepository::persistent("data/rust-task-db")
-> TaskService::new
-> service.add
-> repository.add
-> SELECT id, title, done FROM tasks ORDER BY id
-> Task::new
-> Task
-> INSERT INTO tasks VALUES (...)
```

done/delete:

```text
Vec<String>
-> parse_args
-> parse_id
-> Command::Done { id } 또는 Command::Delete { id }
-> GlueSqlTaskRepository::persistent("data/rust-task-db")
-> TaskService::new
-> service.done 또는 service.delete
-> repository.mark_done 또는 repository.delete
-> SELECT ... WHERE id = ...
-> UPDATE 또는 DELETE SQL
```

search:

```text
Vec<String>
-> parse_args
-> Command::Search { keyword }
-> GlueSqlTaskRepository::persistent("data/rust-task-db")
-> TaskService::new
-> service.search
-> repository.search
-> SELECT id, title, done FROM tasks WHERE title ILIKE ...
-> Vec<Task>
-> print_tasks
```

stats:

```text
Vec<String>
-> parse_args
-> Command::Stats
-> GlueSqlTaskRepository::persistent("data/rust-task-db")
-> TaskService::new
-> service.stats
-> repository.stats
-> SELECT COUNT(*) FROM tasks
-> TaskStats
-> print_stats
```

sql:

```text
Vec<String>
-> parse_args
-> Command::Sql { sql }
-> GlueSqlTaskRepository::persistent("data/rust-task-db")
-> TaskService::new
-> service.execute_sql
-> repository.execute_sql
-> GlueSQL execute
-> Payload
-> SqlResult
-> print_sql_results
```

repl:

```text
Vec<String>
-> parse_args
-> Command::Repl
-> GlueSqlTaskRepository::persistent("data/rust-task-db")
-> TaskService::new
-> repl::run_repl
-> read_line
-> .schema 또는 .exit/.quit 또는 SQL
-> SQL이면 service.execute_sql
-> SqlResult
-> REPL output
```

## 필드 추가 시 수정해야 하는 파일

`Task`에 필드를 추가하면 아래를 같이 수정한다.

- [src/task.rs](../../src/task.rs): struct와 `Task::new`
- [src/repository/mod.rs](../../src/repository/mod.rs): 저장/로드 테스트 기대값
- [src/repository/gluesql_repository.rs](../../src/repository/gluesql_repository.rs): `CREATE TABLE`, `row_to_task`, SQL projection
- [src/service/mod.rs](../../src/service/mod.rs): service 테스트 기대값
- [src/main.rs](../../src/main.rs): `print_task`
- [docs/beginner-codebase-guide/08-data-model.md](08-data-model.md)
- [docs/beginner-codebase-guide/04-feature-flows.md](04-feature-flows.md)

`Command`에 명령을 추가하면 아래를 같이 수정한다.

- `src/command.rs`: enum variant
- `src/cli.rs`: 문자열 parsing
- `src/main.rs`: 실행 분기
- `src/repl.rs`: REPL에서 처리할 입력이면 분기 추가
- `src/service/mod.rs`: 필요한 service 메서드
- `src/repository/mod.rs`: 필요한 저장소 메서드
- `src/repository/gluesql_repository.rs`: 필요한 SQL 구현
- `src/cli.rs` 테스트
- 초심자 가이드 문서

## 민감 정보 또는 노출 주의 필드

현재 민감 정보는 코드에서 확인되지 않음. 모든 `Task` 필드는 CLI에 출력된다.

## 데이터 모델 변경 체크리스트

- struct 또는 enum 수정
- service 수정
- error variant 수정
- repository trait 수정
- repository 구현체 수정
- parser 수정
- 실행 분기 수정
- 출력 함수 수정
- `SqlResult` 수정
- 테스트 expected 값 수정
- 초심자 가이드 수정
