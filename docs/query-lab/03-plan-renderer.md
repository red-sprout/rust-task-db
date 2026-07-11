# Plan Renderer

`src/query_lab/plan.rs`의 `statement_to_node`가 planned Statement를 `PlanNode`로 변환한다. `report::render_tree`는 `└─`, `├─` tree를 만든다.

지원 노드:

- Table Scan, Primary Key Lookup, Index Scan
- Join 종류와 NestedLoop/Hash executor
- Filter, Aggregate/Group By/Having
- Distinct, Projection, Sort, Limit
- Insert, Update, Delete
- Derived Subquery

테스트: `renders_primary_key_access_path`, `renders_non_clustered_index_access_path`, `renders_full_scan_access_path`, `renders_join_plan_from_planned_statement`, `renders_aggregate_sort_and_limit`.

Projection 표현식은 planned AST Debug를 사용한다. 존재하지 않는 cardinality, cost, join build/probe 추정은 출력하지 않는다.

## 출력 예와 해석

```text
└─ Limit [10]
   └─ Sort [priority DESC, id ASC]
      └─ Projection [...]
         └─ Filter [project_id = 1]
            └─ Table Scan [tasks]
```

이 tree는 전통적인 physical operator 객체를 직렬화한 결과가 아니다. `query_node`가 `selection`, `projection`, `order_by`, `limit` 필드를 실행 의미에 맞는 읽기 순서로 감싼 표현이다. 실제 access path는 최하단 `TableFactor.index`에서만 판정한다.

Tree 형식은 사람이 읽는 UI이고 JSON 형식은 `PlanNode.evidence`와 runtime 필드를 보존하는 자동화용 출력이다. 원본 확인이 필요하면 `--raw-plan`을 사용한다. `tree_report_contains_required_sections`, `json_report_contains_plan_and_metrics`, `tree_node_serializes_to_json`이 세 계약을 검증한다.
