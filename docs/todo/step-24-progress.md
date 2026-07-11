# Step 24 진행 상황: JOIN 기반 조회 기능 추가

## 현재 상태

Step 24에서는 Project와 Tag 관계를 단순 저장에 그치지 않고 Task 목록 filter와 Task 상세 화면에서 사용한다. 이후 Optimizer 분석에 재사용할 실제 JOIN query가 repository에 존재한다.

## Step 23에서 Step 24로 달라진 점

| 구분 | Step 23 | Step 24 |
| --- | --- | --- |
| 목록 | 전체/Project 기반 | Tag JOIN filter 추가 |
| 상세 | 기본 Task | Project 이름과 Tag 목록 포함 |
| 조회 모델 | `Task` | `TaskDetail` 추가 |
| SQL 학습 | CRUD 중심 | JOIN, filter, sort 사용 |

## 주요 SQL

Project별 Task:

```sql
SELECT id, project_id, title, done, priority
FROM tasks
WHERE project_id = ?
ORDER BY priority DESC, id ASC;
```

Tag별 Task:

```sql
SELECT t.id, t.project_id, t.title, t.done, t.priority
FROM tasks t
JOIN task_tags tt ON tt.task_id = t.id
JOIN tags tag ON tag.id = tt.tag_id
WHERE tag.name ILIKE ?
ORDER BY t.priority DESC, t.id ASC;
```

GlueSQL에 bind parameter API를 직접 사용하지 않으므로 `sql_string`이 작은따옴표를 escape한 SQL literal을 만든다.

## Task 상세 조회

`TaskDetail`은 저장 table이 아니라 query 결과를 조합한 모델이다.

```rust
pub struct TaskDetail {
    pub task: Task,
    pub project_name: Option<String>,
    pub tags: Vec<Tag>,
}
```

`show_task`는 `find_one`으로 기본 Task를 읽고, `project_id`가 있으면 `show_project`, 마지막으로 `task_tags`를 호출한다. CLI는 Project가 없으면 `(none)`, Tag가 여러 개면 쉼표 목록으로 출력한다.

## 테스트 증거

- `lists_project_tasks_by_priority_then_id_and_calculates_stats`
- `tags_untags_and_lists_tasks_by_tag`
- `task_detail_includes_project_and_tags`
- `Task::with_project` 모델 테스트

## 완료 기준

- Project filter에서 priority/id 정렬 적용
- Tag filter가 3-table JOIN 사용
- Task 상세에 Project와 Tag 포함
- 코드에 없는 기능을 현재 기능처럼 문서화하지 않음
- 최종 80개 테스트 통과

## 다음 단계

Step 25에서는 Project별 Task 집계를 `COUNT` 기반 통계로 제공한다.
