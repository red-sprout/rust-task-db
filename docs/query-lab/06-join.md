# JOIN 분석

대상은 Project 1:N Task와 Task N:M Tag의 실제 JOIN이다. planned `Join`에서 다음을 직접 읽는다.

- `JoinOperator::Inner` / `LeftOuter`
- `JoinExecutor::NestedLoop` / `Hash`
- Hash의 key/value expression과 residual where clause
- `TableWithJoins`에 기록된 좌우 순서

`renders_join_plan_from_planned_statement`가 JOIN 노드를 검증한다. Storage trace는 양쪽 table scan과 총 소비 row를 보여주지만 중간 JOIN row 수는 보여주지 않는다. 근거 파일은 `ast/query.rs`, `plan/join.rs`, `executor/join.rs`다.

현재 Project-Task SQL의 직접 실행에서는 `Hash` executor, projects/tasks 각각의 scan으로 총 2개 storage row 소비, 최종 0 row가 관찰됐다. 이 숫자는 현재 로컬 데이터 분포의 결과이며 일반적인 비용 결론이 아니다.

## 시나리오와 관찰 기준

| 시나리오 | 관계 | 직접 확인 | 확인 불가 |
| --- | --- | --- | --- |
| Project-Task | 1:N INNER | join 종류, executor, 양쪽 access | 중간 결과 row |
| Project-Task 통계 | LEFT | LeftOuter, aggregate | null 확장 row 수 |
| Task-TaskTag-Tag | N:M 3-table | AST에 기록된 join 순서 | optimizer 탐색 후보 |
| Filter+Join | PK/filter 조합 | planned predicate와 access path | 정확한 pushdown row 효과 |

```sql
SELECT t.title, tag.name
FROM tasks t
JOIN task_tags tt ON tt.task_id = t.id
JOIN tags tag ON tag.id = tt.tag_id;
```

출력 순서는 planned `TableWithJoins` 순서이며 “가장 싼 join 순서를 optimizer가 선택했다”는 뜻이 아니다. `JoinExecutor::Hash`가 표시되어도 Query Lab은 build/probe side를 추정하지 않는다.
