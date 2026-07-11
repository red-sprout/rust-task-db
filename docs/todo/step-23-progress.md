# Step 23 진행 상황: Task-Tag CLI 추가

## 현재 상태

Step 23에서는 `task_tags` table을 실제 CLI 기능에 연결했다. 사용자는 기존 Tag 이름으로 Task를 연결·해제하고, Task에 연결된 Tag를 조회할 수 있다.

## Step 22에서 Step 23으로 달라진 점

| 구분 | Step 22 | Step 23 |
| --- | --- | --- |
| Task-Tag 관계 | table/repository 기반 | CLI 사용 가능 |
| 명령 | Tag CRUD | tag/untag/tags 추가 |
| 무결성 | 정책 정의 | 존재/중복 검사 실행 |
| 조회 | Tag 목록 | Task별 Tag 목록 |

## 추가한 명령과 Command

```bash
cargo run -- task tag 1 backend
cargo run -- task untag 1 backend
cargo run -- task tags 1
cargo run -- task list --tag backend
cargo run -- task add --tag backend --tag sql "Planner 분석"
```

`src/command.rs`에는 다음 variant가 있다.

```rust
TaskTag { id: i64, tag: String },
TaskUntag { id: i64, tag: String },
TaskTags { id: i64 },
```

## 연결 실행 흐름

```text
task tag 1 backend
-> parse_task
-> Command::TaskTag
-> TaskService::tag_task
-> GlueSqlTaskRepository::tag_task
-> Task 존재 확인
-> find_tag("backend")
-> task_tags 중복 COUNT
-> INSERT INTO task_tags
```

동일한 Task와 Tag 연결이 이미 있으면 `task already has tag`를 반환한다. 없는 Task는 기존 `AppError::NotFound`, 없는 Tag는 `AppError::Domain("Tag not found: ...")`으로 구분한다.

## 해제와 목록

`untag_task`는 Task와 Tag 존재를 확인한 뒤 두 id가 일치하는 `task_tags` row를 삭제한다. `task_tags`는 다음 JOIN으로 Tag 정보를 반환한다.

`task add`는 반복 가능한 `--tag`를 받는다. `add_task_with_tags`가 모든 Tag 존재와 옵션 내 대소문자 무관 중복을 먼저 검증하고, Task INSERT와 `task_tags` INSERT를 같은 repository transaction에서 실행한다.

```sql
SELECT tag.id, tag.name
FROM tags tag
JOIN task_tags tt ON tt.tag_id = tag.id
WHERE tt.task_id = ?
ORDER BY tag.id;
```

## 테스트 증거

- `cli::tests::parses_task_relationship_commands`
- `tags_untags_and_lists_tasks_by_tag`
- `untag_requires_existing_task_and_tag`
- `creates_task_and_tags_together`
- `cli::tests::parses_task_add_tags`
- 같은 테스트에서 중복 연결 오류 확인

## 완료 기준

- tag/untag/tags parsing, Service, repository, 출력 연결
- 중복 Task-Tag 연결 방지
- Task에서 Tag 해제와 목록 조회
- 기존 Task 명령 하위 호환
- 최종 80개 테스트 통과

## 다음 단계

Step 24에서는 Project/Tag 관계를 실제 JOIN 기반 조회에 사용한다.
