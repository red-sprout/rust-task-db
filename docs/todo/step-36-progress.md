# Step 36 진행 상황: Subquery 실험

## 현재 상태

Step 36 완료. Project별 correlated COUNT subquery를 실제 plan/execute 시나리오로 추가했다. 현재 활성 CLI 저장소는 계속 `GlueSqlTaskRepository<TracingStorage<SledStorage>>`이며 기존 Project/Task/Tag 기능과 JSON/MemoryStorage 테스트 흐름을 보존한다.

## 이전 단계에서 달라진 점

| 구분 | 이전 단계 | Step 36 |
| --- | --- | --- |
| 중심 작업 | 앞 단계의 Query Lab 기반 | Subquery 실험 |
| 분석 근거 | 앞 단계에서 확보한 plan/runtime 정보 | 해당 단계의 직접 근거와 한계를 문서화 |
| 회귀 정책 | 기존 관계형 기능 유지 | 기존 명령과 테스트를 삭제하지 않음 |

## 실제 구현 위치

- `scenario_sql("subquery")`, `plan.rs::relation_node`의 `TableFactor::Derived`
- 공통 진입점: `src/main.rs::run`, `src/query_lab/mod.rs::analyze`
- 공통 보고서: `src/query_lab/report.rs::AnalysisReport`
- 상세 해설: `docs/query-lab/08-subquery.md`

## 구현과 해석

현재 GlueSQL 0.19에서 대상 correlated query 실행 성공을 확인했으며 expression 내부 subquery 반복 횟수는 별도 metric이 없다.

```bash
cargo run -- analyze --plan "SELECT * FROM tasks WHERE id = 1"
cargo run -- analyze --runtime "SELECT * FROM tasks"
```

위 명령은 plan-only와 runtime의 차이를 재현하는 공통 최소 예다. 시나리오 전용 SQL과 기대 관찰 항목은 연결된 Query Lab 문서에 기록했다.

## 테스트 근거

- `scenario_list_and_scan_queries_are_available`와 전체 `cargo test`
- 실행 시간은 assertion으로 사용하지 않는다.
- plan 종류, Storage 호출, consumed/returned/affected row와 결과값을 검증 대상으로 사용한다.

## 제약과 주의점

Derived table은 child tree로 표시하지만 scalar/correlated subquery의 실행 횟수와 JOIN rewrite 여부는 확정할 수 없다.

문서에서 직접 측정한 값, planned AST의 직접 필드, 안전한 UI 해석, 코드에서 확인되지 않는 추정을 구분한다. 확인 불가능한 값은 0으로 가장하지 않고 limitations에 남긴다.

## 완료 기준

- 실제 파일 경로·함수명·테스트 이름 기록
- 메인 관계형 기능 회귀 없음
- Tree/JSON 또는 runtime 결과에서 단계 핵심을 관찰 가능
- README, roadmap, 초심자 가이드와 현재 단계 정합성 유지
