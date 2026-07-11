# GlueSQL 기여 후보

| 우선순위 | 후보 | 현재 문제 | 난이도 | 첫 PR |
| --- | --- | --- | --- | --- |
| 1 | Pretty Plan Renderer | planned AST Debug만으로 읽기 어려움 | 낮음 | 적합 |
| 2 | Public Explain API / EXPLAIN | plan과 실행을 사용자 API로 설명하기 어려움 | 중간 | 분리 가능 |
| 3 | Runtime Storage Trace | Store 접근/소비 row 표준 metric 없음 | 중간 | 적합 |
| 4 | Operator Metrics Hook | Join/Aggregate/Sort actual rows 없음 | 높음 | RFC 권장 |
| 5 | Statistics/Cost | cardinality/cost/선택도 기반 선택 없음 | 매우 높음 | RFC 필요 |

재현 SQL은 `lab run scan/index/join/aggregate` 시나리오다. Pretty renderer는 기존 `Statement`를 읽기만 하므로 하위 호환 위험이 가장 낮다. Operator hook은 executor API와 성능에 영향을 주므로 feature flag와 RFC가 필요하다.

근거 파일: `core/src/glue.rs`, `ast.rs`, `ast/query.rs`, `store/planner.rs`, `plan/join.rs`, `executor/fetch.rs`, `store.rs`, `store/index.rs`.

## 1. Pretty Plan Renderer

- 현재 문제: `Vec<Statement>` Debug는 access path와 executor 핵심을 찾기 어렵다.
- 재현 SQL: `SELECT * FROM tasks WHERE id = 10`.
- 코드 근거: `ast.rs`, `ast/query.rs`의 public serializable 타입.
- 기대 효과: API 변경 없이 plan 가독성 개선.
- 난이도/호환성: 낮음, read-only utility라 하위 호환 위험이 낮다.
- 판단: 첫 PR에 적합, RFC 불필요.

## 2. EXPLAIN SQL 또는 Public Explain API

- 현재 문제: `glue.plan`을 호출하는 Rust 사용자만 plan에 접근한다.
- 재현 SQL: Query Lab의 모든 SELECT/UPDATE/DELETE.
- 코드 근거: `glue.rs`의 `plan`, `execute_stmt`, parser/translator 경계.
- 기대 효과: SQL/CLI 사용자도 구조화된 plan 조회.
- 난이도/호환성: 중간, 새 Statement/Payload 또는 API 타입 설계 필요.
- 판단: renderer와 분리한 PR, 출력 계약에는 작은 RFC 권장.

## 3. Runtime Storage Trace

- 현재 문제: Store 호출과 iterator 소비가 표준 report로 노출되지 않는다.
- 재현 SQL: `lab run scan`, `lab run index`, mutation.
- 코드 근거: `store.rs`, `store/index.rs`, `executor/fetch.rs`.
- 기대 효과: storage adapter 개발과 access path 검증.
- 난이도/호환성: 중간, opt-in wrapper면 기존 trait 호환 가능.
- 판단: 첫 PR 후보, RFC 불필요하거나 짧은 design discussion.

## 4. Operator Metrics Hook

- 현재 문제: Filter/Join/Aggregate/Sort의 input/output actual rows가 없다.
- 재현 SQL: `lab run join`, `aggregate`, `sort`, `subquery`.
- 코드 근거: `executor/join.rs`, `executor/aggregate.rs`, `executor/sort.rs`, `executor/select.rs`.
- 기대 효과: EXPLAIN ANALYZE에 가까운 operator 관찰성.
- 난이도/호환성: 높음, stream 경계와 실행 overhead/API lifetime 영향.
- 판단: 첫 PR 부적합, feature/API RFC 필요.

## 5. Statistics와 Cost Estimation

- 현재 문제: default `Planner::plan`은 primary key와 join planning 중심이며 estimated cardinality/cost가 없다.
- 재현 SQL: `lab run selectivity`, secondary index predicate.
- 코드 근거: `store/planner.rs`, `plan/primary_key.rs`, `plan/join.rs`.
- 기대 효과: 선택도에 따른 access path/join 순서 결정.
- 난이도/호환성: 매우 높음, 통계 수집·저장·cost model·plan 계약 필요.
- 판단: 첫 PR 부적합, 장기 RFC 필수.

프로젝트 검증 테스트는 `query_lab::plan::tests::*`, `query_lab::tests::*`, `query_lab::runtime::tests::*`다.

## 권장 추진 순서

1. Pretty renderer를 독립 utility와 snapshot/unit test로 제안한다.
2. renderer 출력 계약을 바탕으로 Explain API의 최소 구조화 타입을 논의한다.
3. opt-in `TracingStorage`를 storage adapter 진단 도구로 분리한다.
4. operator metrics는 overhead, async stream lifetime, 중첩 query 식별자를 포함한 RFC부터 작성한다.
5. statistics/cost는 별도 장기 설계로 분리한다.

첫 PR은 기존 executor 동작과 public trait 계약을 바꾸지 않는 범위가 적절하다. 이 프로젝트의 renderer 코드는 proof of concept이며 그대로 upstream API라고 가정하지 않는다. 특히 문자열 label 대신 안정적인 enum/structured field가 필요한지 maintainer와 먼저 합의해야 한다.
