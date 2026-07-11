# Step 20 진행 상황: Task에 project_id와 priority 추가

## 현재 상태

Step 20에서는 기존 Task 명령을 유지하면서 Task가 Project에 선택적으로 속하고 priority 1~5를 가지도록 모델과 table을 확장했다. 기존 `add "Rust 공부"`는 Project가 없는 Task와 기본 priority 3을 만든다.

## Step 19에서 Step 20으로 달라진 점

| 구분 | Step 19 | Step 20 |
| --- | --- | --- |
| `Task` 필드 | `id`, `title`, `done` | `project_id`, `priority` 추가 |
| Project 관계 | 타입만 존재 | Task에서 선택적으로 참조 |
| 정렬 | 주로 `id` | Project 목록에서 priority DESC, id ASC |
| 이전 데이터 | 3열 Task | JSON 기본값과 Sled migration 적용 |

## 완료한 일

| 파일 경로 | 실제 코드 | 역할 |
| --- | --- | --- |
| `src/task.rs` | `project_id: Option<i64>` | Project 미지정은 `None`/SQL NULL |
| `src/task.rs` | `priority: i64` | Task 우선순위 |
| `src/task.rs` | `Task::with_project` | 관계와 priority를 지정해 생성 |
| `src/repository/gluesql_repository.rs` | `add_task` | Project 존재와 priority 검증 후 INSERT |
| `src/repository/gluesql_repository.rs` | `list_tasks` | Project/Tag filter와 정렬 |
| `src/repository/gluesql_repository.rs` | `row_to_task` | 5열 GlueSQL row를 `Task`로 변환 |

## 데이터 모델

```rust
pub struct Task {
    pub id: i64,
    pub project_id: Option<i64>,
    pub title: String,
    pub done: bool,
    pub priority: i64,
}
```

`Option<i64>`를 사용해 기존 독립 Todo와 Project 소속 Task를 모두 표현한다. `Task::new`는 `project_id = None`, priority 3을 사용하므로 기존 코드 호출 방식도 유지된다.

## 저장 데이터 호환

JSON에는 serde 기본값을 지정했다.

```rust
#[serde(default)]
pub project_id: Option<i64>,

#[serde(default = "default_priority")]
pub priority: i64,
```

기존 Sled DB의 3열 `tasks` table은 `create_tables`에서 새 열 조회가 실패할 때 다음 migration을 실행한다.

```sql
ALTER TABLE tasks ADD COLUMN project_id INTEGER;
ALTER TABLE tasks ADD COLUMN priority INTEGER DEFAULT 3;
```

새 DB에서는 `tasks.project_id`가 `projects(id)`를 참조하는 FK를 갖는다. 기존 table은 GlueSQL ALTER 제약상 같은 방식으로 FK를 사후 추가하지 않으며, `add_task`가 Project 존재를 검사한다.

## 검증과 정렬

```sql
SELECT id, project_id, title, done, priority
FROM tasks
WHERE project_id = 1
ORDER BY priority DESC, id ASC;
```

- priority 범위 밖이면 `priority must be between 1 and 5`
- 없는 Project면 `Project not found: <id>`
- 제목이 공백이면 `task title must not be empty`

관련 테스트는 `validates_project_and_task_fields`, `lists_project_tasks_by_priority_then_id_and_calculates_stats`, `task_new_uses_unassigned_project_and_default_priority`, `task_with_project_keeps_relationship_and_priority`다.

## ID sequence 보강

`id_sequences(entity PRIMARY KEY, next_id)` table을 추가했다. `allocate_id`는 저장된 `next_id`와 실제 table의 `MAX(id) + 1` 중 큰 값을 할당한 뒤 sequence를 증가시킨다. SledStorage에서는 이 갱신과 INSERT가 하나의 transaction이므로 삭제된 id를 재사용하지 않고 경쟁 writer가 같은 id를 commit하지 못한다. `sequence_does_not_reuse_id_after_delete`, `sled_transaction_rolls_back_sequence_allocation`이 이를 검증한다.

## 완료 기준

- 기존 Task 명령과 JSON repository 동작 유지
- Project 지정/미지정 Task 모두 생성 가능
- priority 1~5 검증과 정렬 구현
- 기존 Sled schema migration 구현
- 최종 80개 테스트와 세 검증 명령 통과

## 다음 단계

Step 21에서는 Project 명령을 CLI와 Service에 연결한다.
