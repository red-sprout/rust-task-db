# Step 26 진행 상황: 삭제 및 Transaction 정책 구현

## 현재 상태

Step 26에서는 관계형 데이터가 고아 row를 남기지 않도록 삭제 순서와 실패 정책을 명시하고, storage capability에 맞춘 공통 `transaction` helper를 구현했다.

## Step 25에서 Step 26으로 달라진 점

| 구분 | Step 25 | Step 26 |
| --- | --- | --- |
| 핵심 작업 | 통계 조회 | 관계형 삭제 정합성 |
| Project 삭제 | 기본 CRUD 기반 | 하위 Task가 있으면 거부 |
| Task/Tag 삭제 | 부모 row 삭제 | `task_tags` 선행 정리 |
| transaction | 기존 관찰 테스트 | 실제 CRUD 적용 범위 결정 |

## 삭제 정책

### Project

```text
show_project(id)
-> tasks WHERE project_id = id COUNT
-> 1개 이상이면 "project has tasks"
-> 0개면 projects row 삭제
```

`--force`는 현재 코드에 없다. 하위 Task를 자동 삭제하지 않는 restrict 정책을 선택했다.

### Task와 Tag

```sql
DELETE FROM task_tags WHERE task_id = ?;
DELETE FROM tasks WHERE id = ?;

DELETE FROM task_tags WHERE tag_id = ?;
DELETE FROM tags WHERE id = ?;
```

두 문장은 한 `Glue::execute` 호출에 순서대로 전달한다. GlueSQL 0.19의 `task_tags` FK 삭제 동작 제한 때문에 DB cascade는 사용하지 않는다.

## Transaction 적용 범위

| 작업 | 현재 처리 | 이유 |
| --- | --- | --- |
| Task 삭제 + 연결 삭제 | 한 execute의 연속 SQL | MemoryStorage에서도 동작해야 함 |
| Tag 삭제 + 연결 삭제 | 한 execute의 연속 SQL | 같은 이유 |
| Project 삭제 + 하위 검증 | COUNT 후 조건부 DELETE | 기본 restrict 정책 |
| Task 생성 + Tag 연결 | 코드에서 확인되지 않음 | 하나의 CLI 작업으로 아직 제공하지 않음 |

`GlueSqlTaskRepository`는 생성 시 Sled면 `transactional = true`, Memory면 `false`를 저장한다. 최상위 `transaction` 호출은 Sled에서 `BEGIN/COMMIT/ROLLBACK`을 실행하고, 내부 repository 메서드는 `in_transaction`을 확인해 nested BEGIN을 만들지 않는다. MemoryStorage에서는 동일 closure를 그대로 실행해 기존 빠른 테스트 흐름을 유지한다.

이 경계는 Task/Tag 삭제, Project 검증+삭제, sequence 할당+INSERT, Task+Tag 생성, Tag 중복 검사, Seed 전체 생성에 적용된다.

## 테스트 증거

- `manages_projects_and_rejects_delete_when_tasks_exist`
- `deletes_empty_project`
- `deleting_task_or_tag_cleans_join_rows`
- 기존 Memory/Sled rollback, commit, snapshot, write-lock 테스트 유지

## 완료 기준

- Project restrict 정책과 오류 메시지 구현
- Task/Tag 연결 row 선행 삭제
- FK cascade 미사용 이유 문서화
- storage별 transaction capability와 nested 방지 구현
- 최종 80개 테스트 통과

## 다음 단계

Step 27에서는 관계형 query 실험에 재사용할 현실적인 Seed 기능을 추가한다.
