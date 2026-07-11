# Aggregate, Group By, HAVING

Query Lab은 projection의 aggregate expression, `Select.group_by`, `Select.having`을 직접 읽어 Aggregate 노드로 묶는다.

시나리오에는 Project LEFT JOIN COUNT, Task project별 COUNT, MAX/MIN/COUNT, HAVING이 포함된다. `renders_aggregate_sort_and_limit`가 Group By와 상위 Sort/Limit 렌더링을 검증한다.

Storage rows consumed는 aggregate 입력으로 사용된 storage row의 합계에 가깝지만 JOIN 중간 row나 aggregate group input과 동일하다고 단정할 수 없다. 최종 `Payload::Select.rows.len()`만 결과 group 수로 확정한다.

현재 `GROUP BY project_id` 직접 실행에서는 tasks scan 1회, 2 row 소비, 결과 group 1개가 관찰됐다.

## 결과 읽기

```sql
SELECT project_id, COUNT(*)
FROM tasks
GROUP BY project_id
HAVING COUNT(*) >= 10;
```

Plan의 `group_by`와 `having`은 직접 근거다. `result_rows`는 HAVING 이후 최종 group 수다. 반면 `rows_consumed`는 Storage iterator가 공급한 row 총합이므로 Aggregate 입력 row나 JOIN 출력 row로 이름을 바꾸면 안 된다.

| 값 | 확정 가능한 해석 |
| --- | --- |
| `scan_data_calls = 1` | tasks scan iterator를 한 번 요청 |
| `rows_consumed = N` | Storage에서 N rows를 소비 |
| `result_rows = G` | 최종 Payload에 G개 group row |
| elapsed | 해당 실행의 참고 시간, 테스트 기준 아님 |

MAX/MIN/COUNT와 LEFT JOIN COUNT도 `scenario_sql("aggregate")`에 포함된다. 새 aggregate 함수를 추가하면 계획 node 존재뿐 아니라 반환 Payload 값도 별도 테스트하는 것이 권장된다.
