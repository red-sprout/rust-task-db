# Planned Statement 구조

직접 존재하는 정보:

- `Statement::Query/Update/Delete/Insert`
- `Query.order_by`, `limit`, `offset`
- `Select.selection`, `group_by`, `having`, `distinct`
- `TableFactor::Table.index`
- `Join.join_operator`, `Join.join_executor`

`TableFactor::index`는 `PrimaryKey` 또는 `NonClustered`다. `JoinExecutor`는 `NestedLoop` 또는 `Hash`다. 근거 파일은 `gluesql-core/src/ast.rs`, `ast/query.rs`다.

전통적인 Physical Operator Tree 타입은 코드에서 확인되지 않는다. Query Lab tree는 위 필드를 계층적으로 재배치한 `SafeInterpretation`이며, 노드별 evidence를 JSON에 기록한다.

## Evidence 규칙

| Evidence | 예 | 의미 |
| --- | --- | --- |
| `Direct` | `TableFactor::Table.index`, table 이름 | planned AST 필드에 직접 존재 |
| `SafeInterpretation` | Projection이 Scan의 부모라는 tree 배치 | 실행 의미를 읽기 좋게 재배치 |
| Runtime | `scan_data_calls`, `rows_consumed` | 실행 중 wrapper가 직접 측정 |
| 추정 | build/probe side, operator actual row | 기본 출력에서 제외 |

```rust
pub struct PlanNode {
    pub label: String,
    pub evidence: Evidence,
    pub children: Vec<PlanNode>,
}
```

`src/query_lab/plan.rs`의 `relation_node`, `query_node`가 변환 책임을 나눈다. 새 GlueSQL AST variant를 지원할 때는 catch-all Debug 표시를 실제 지원처럼 설명하지 말고, variant별 변환과 테스트를 함께 추가한다.
