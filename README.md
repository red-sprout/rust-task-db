# rust-task-db

Rust와 GlueSQL로 만든 관계형 Task Management CLI다. Step 28 현재 Project 1:N Task, Task N:M Tag 관계를 실제 사용자 기능으로 제공하며, 기존 Todo 명령과 JSON repository 학습 코드도 보존한다.

## 현재 상태

| 항목 | 내용 |
| --- | --- |
| 현재 단계 | Step 28. 테스트와 문서 정리 |
| 활성 저장소 | `GlueSqlTaskRepository<SledStorage>` |
| 저장 위치 | `data/rust-task-db` |
| 테스트 저장소 | GlueSQL `MemoryStorage` |
| 보존 코드 | `JsonTaskRepository`, `tasks.json`, 기존 `add/list/done/delete/search/stats` |
| 외부 crate | `serde`, `serde_json`, `gluesql`, `futures` |

## 도메인과 테이블

```text
Project 1 --- N Task
Task    N --- M Tag (task_tags)
```

```sql
projects(id PRIMARY KEY, name NOT NULL)
tasks(id PRIMARY KEY, project_id, title NOT NULL, done NOT NULL, priority NOT NULL)
tags(id PRIMARY KEY, name NOT NULL)
task_tags(task_id, tag_id)
id_sequences(entity PRIMARY KEY, next_id NOT NULL)
app_metadata(key PRIMARY KEY, value NOT NULL)
```

`tasks.project_id`에는 GlueSQL FK를 사용한다. GlueSQL 0.19가 table-level 복합 `PRIMARY KEY (task_id, tag_id)`를 거부하고, `task_tags` FK를 둔 상태에서는 선행 join 행 삭제 뒤에도 부모 삭제가 막히는 동작이 있어 `task_tags` 무결성은 repository가 검사한다. 중복 연결은 `tag_task`, 고아 참조는 Task/Tag 존재 확인, 정리는 삭제 전 `task_tags` 삭제로 막는다.

Step 18의 기존 3열 `tasks` 테이블은 시작 시 `project_id`와 기본 priority 3을 추가하는 migration을 수행한다.

## CLI

```bash
cargo run -- project add "GlueSQL 분석"
cargo run -- project list
cargo run -- project show 1
cargo run -- project stats 1
cargo run -- project stats
cargo run -- project delete 1

cargo run -- task add --project 1 --priority 5 "Planner 분석"
cargo run -- task add --project 1 --priority 5 --tag backend --tag sql "Planner 분석"
cargo run -- task list
cargo run -- task list --project 1
cargo run -- task list --tag backend
cargo run -- task show 1
cargo run -- task done 1
cargo run -- task delete 1
cargo run -- task search rust

cargo run -- tag add backend
cargo run -- tag list
cargo run -- task tag 1 backend
cargo run -- task tags 1
cargo run -- task untag 1 backend
cargo run -- tag delete 1

cargo run -- seed
cargo run -- sql "SELECT * FROM tasks"
cargo run -- repl
```

기존 `add`, `list`, `done`, `delete`, `search`, `stats`는 계속 지원한다. 기존 `add`는 `project_id = NULL`, priority 3으로 생성한다.

## 정책

- Project 이름과 Task 제목은 공백일 수 없고 priority는 1~5다.
- Tag 이름은 앞뒤 공백을 제거하고 `ILIKE` 검사로 대소문자 무관 중복을 막는다.
- Project에 Task가 있으면 `project has tasks`로 삭제를 거부한다.
- Task/Tag 삭제는 `task_tags`를 먼저 지운다.
- ID는 `id_sequences` table에서 entity별 `next_id`를 할당한다. SledStorage에서는 sequence 갱신과 INSERT가 같은 transaction에 있어 동시 writer가 같은 ID를 확정하지 못한다. 기존 raw SQL row가 있으면 `MAX(id) + 1` 이상으로 sequence를 자동 보정한다.
- repository의 `transaction` helper는 SledStorage에서 최상위 작업만 `BEGIN/COMMIT/ROLLBACK`으로 감싸고 MemoryStorage에서는 같은 closure를 transaction 없이 실행한다. 삭제, Project 하위 검증, Task+Tag 생성, Tag 중복 검사, Seed에 적용된다.
- Seed는 `app_metadata.seed_version = 1`로 완료를 기록한다. 완료 전 예약 이름의 부분 Seed 데이터를 정리하고 Sled transaction 안에서 다시 생성하므로 실패 시 rollback할 수 있다.
- `project stats`는 전체 Project를 `LEFT JOIN ... GROUP BY`로 집계하며, id를 주면 기존 단일 Project 통계를 반환한다.

Repository는 작은 코드베이스에 맞춰 방식 B인 `TaskManagementRepository` 하나가 관계형 기능을 담당한다. Project/Task/Tag를 넘나드는 삭제와 상세 조회의 경계를 한곳에 두기 위해서다. 이전 학습 흐름의 `TaskRepository`는 JSON 하위 호환을 위해 보존한다. Service는 [src/service/mod.rs](src/service/mod.rs)의 `TaskService`가 repository 호출을 CLI와 분리한다.

## 검증

```bash
cargo fmt --check
cargo check
cargo test
```

상세 설계와 GlueSQL 조정 사항은 [관계형 Task Management 가이드](docs/beginner-codebase-guide/21-relational-task-management.md), 단계 기록은 [로드맵](docs/todo/roadmap.md)에서 확인할 수 있다.
