# Sort, Limit, Distinct

`Query.order_by`, `Query.limit`, `Select.distinct`는 planned Statement에 직접 존재한다. renderer는 Projection 위에 Distinct, Sort, Limit 순서로 표현한다.

Storage iterator rows consumed와 최종 rows returned 차이로 LIMIT 이전에 storage가 몇 row를 공급했는지는 관찰할 수 있다. 그러나 executor가 전체 정렬했는지, top-N을 사용했는지 나타내는 별도 plan/operator 정보는 없다.

테스트 `renders_aggregate_sort_and_limit`는 node 존재를 검증하며 실행 시간은 assertion으로 사용하지 않는다.

GlueSQL 근거 파일은 `ast/query.rs`, `executor/sort.rs`, `executor/select.rs`다.

## 실험 SQL

```sql
SELECT * FROM tasks
WHERE project_id = 1
ORDER BY priority DESC, id ASC
LIMIT 10;
```

| 관찰 | 해석 한계 |
| --- | --- |
| Sort/Limit node 존재 | 정렬 알고리즘은 알 수 없음 |
| consumed > returned | Filter/Sort/Limit 전 공급량 차이는 보임 |
| returned <= 10 | 최종 LIMIT 효과는 확인 가능 |
| indexed calls = 0 | index order 활용을 확인하지 못함 |

`DISTINCT project_id`는 `Select.distinct`를 직접 읽어 별도 node로 표시한다. distinct 전후 row를 operator별로 측정하지 못하므로 최종 row와 Storage consumed만 비교한다.
