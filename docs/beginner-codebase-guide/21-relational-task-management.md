# Step 28 관계형 Task Management 해설

## 실제 파일과 역할

| 파일 경로 | 실제 타입/함수 | 프로젝트 역할 |
| --- | --- | --- |
| `src/project.rs` | `Project`, `ProjectStats::new` | Project와 완료율 모델 |
| `src/task.rs` | `Task`, `TaskDetail`, `Task::with_project` | Project/priority/Tag 상세를 포함한 Task 모델 |
| `src/tag.rs` | `Tag` | Tag 모델 |
| `src/command.rs` | `Command` | Project/Task/Tag/Seed 명령 표현 |
| `src/cli.rs` | `parse_args`, `parse_project`, `parse_task`, `parse_tag` | 문자열 옵션 parsing |
| `src/service/mod.rs` | `TaskService<R>` | CLI와 repository 사이 위임 계층 |
| `src/repository/mod.rs` | `TaskManagementRepository` | 관계형 기능의 애플리케이션 저장소 경계 |
| `src/repository/gluesql_repository.rs` | `create_tables`, `add_task`, `list_tasks`, `tag_task`, `seed` | SQL, row 변환, 관계 무결성 |

## 실행 흐름

```text
task list --tag backend
-> parse_task
-> Command::TaskList
-> main::run
-> TaskService::list_tasks
-> GlueSqlTaskRepository::list_tasks
-> tasks JOIN task_tags JOIN tags
-> row_to_task
-> print_tasks
```

Project 통계는 `project_stats`가 Project 존재를 확인하고 두 `COUNT` query로 전체/완료 수를 얻은 뒤 `ProjectStats::new`가 미완료 수와 완료율을 계산한다.

## 테이블 생성과 GlueSQL 제약

```rust
CREATE TABLE IF NOT EXISTS tasks (
    id INTEGER PRIMARY KEY,
    project_id INTEGER,
    title TEXT NOT NULL,
    done BOOLEAN NOT NULL,
    priority INTEGER NOT NULL,
    FOREIGN KEY (project_id) REFERENCES projects(id)
);
```

GlueSQL 0.19에서 단일 PK와 `tasks.project_id` FK는 테스트로 동작을 확인했다. 반면 `PRIMARY KEY (task_id, tag_id)`는 `unsupported constraint`가 발생한다. `task_tags` FK까지 사용하면 join 행 선행 삭제 뒤 부모 삭제가 막히는 동작도 관찰했다. 따라서 현재 `task_tags`는 두 INTEGER 열을 두고 다음 코드를 애플리케이션 제약으로 사용한다.

```rust
let count = select_count(self, "... task_id = ... AND tag_id = ...")?;
if count > 0 {
    return Err(AppError::Domain("task already has tag".to_string()));
}
```

## ID와 동시성

도메인 table 외에 `id_sequences(entity, next_id)`와 `app_metadata(key, value)` 내부 상태 table을 사용한다. 전자는 id 할당, 후자는 Seed version과 복구 판단을 담당한다.

`allocate_id`는 `id_sequences(entity, next_id)`에서 entity별 id를 할당한다. raw SQL insert로 sequence보다 큰 id가 생겼을 때는 실제 `MAX(id) + 1`로 보정한다. SledStorage에서는 sequence UPDATE와 domain INSERT가 같은 transaction에 있으므로 rollback 시 id도 소비되지 않는다.

## 삭제와 transaction

```text
Task 삭제 -> task_tags 삭제 -> tasks 삭제
Tag 삭제  -> task_tags 삭제 -> tags 삭제
Project 삭제 -> 하위 Task COUNT -> 0일 때만 projects 삭제
```

MemoryStorage는 명시적 transaction을 거부하고 SledStorage는 `BEGIN/COMMIT/ROLLBACK`을 지원한다. repository의 `transaction` helper는 `transactional`과 `in_transaction` 상태로 최상위 Sled 작업만 transaction으로 감싼다. MemoryStorage에서는 같은 closure가 비transaction 방식으로 실행된다. 삭제, sequence, Project 검증, Task+Tag 생성, Tag 중복, Seed가 이 경계를 사용한다.

## Seed 정책

`seed`는 `app_metadata.seed_version`으로 완료 상태를 확인한다. 완료 marker가 없으면 예약된 부분 Seed 데이터를 정리한 뒤 Project 10개, Tag 20개, Task 1,000개를 생성하고 마지막에 version을 기록한다. SledStorage에서는 전체가 한 transaction이어서 실패 시 rollback된다.

## 초심자가 수정할 지점

- 출력 형식: `src/main.rs`의 `print_task`, `Command::TaskShow` branch
- validation: `required_name`, `add_task`, `add_tag`
- 정렬: `GlueSqlTaskRepository::list_tasks`의 `ORDER BY`
- Seed 분포: `GlueSqlTaskRepository::seed`
- 새 명령: `Command` -> `parse_args` -> `main::run` -> `TaskService` -> repository 순서
