# Step 39 진행 상황: DBMS 비교 문서

## 현재 상태

Step 39 완료. GlueSQL, PostgreSQL, MySQL InnoDB, Oracle의 explain 관찰 범위를 비교했다. 현재 활성 CLI 저장소는 계속 `GlueSqlTaskRepository<TracingStorage<SledStorage>>`이며 기존 Project/Task/Tag 기능과 JSON/MemoryStorage 테스트 흐름을 보존한다.

## 이전 단계에서 달라진 점

| 구분 | 이전 단계 | Step 39 |
| --- | --- | --- |
| 중심 작업 | 앞 단계의 Query Lab 기반 | DBMS 비교 문서 |
| 분석 근거 | 앞 단계에서 확보한 plan/runtime 정보 | 해당 단계의 직접 근거와 한계를 문서화 |
| 회귀 정책 | 기존 관계형 기능 유지 | 기존 명령과 테스트를 삭제하지 않음 |

## 실제 구현 위치

- `docs/query-lab/12-dbms-comparison.md`; GlueSQL 근거 `glue.rs`, `store/planner.rs`, executor/store 파일
- 공통 진입점: `src/main.rs::run`, `src/query_lab/mod.rs::analyze`
- 공통 보고서: `src/query_lab/report.rs::AnalysisReport`
- 상세 해설: `docs/query-lab/12-dbms-comparison.md`

## 구현과 해석

Plan 표현, access, join, aggregate, estimate/cost, actual rows, storage 경계를 공식 문서 근거로 비교했다.

```bash
cargo run -- analyze --plan "SELECT * FROM tasks WHERE id = 1"
cargo run -- analyze --runtime "SELECT * FROM tasks"
```

위 명령은 plan-only와 runtime의 차이를 재현하는 공통 최소 예다. 시나리오 전용 SQL과 기대 관찰 항목은 연결된 Query Lab 문서에 기록했다.

## 테스트 근거

- `json_report_contains_plan_and_metrics`, `tree_report_contains_required_sections`
- 실행 시간은 assertion으로 사용하지 않는다.
- plan 종류, Storage 호출, consumed/returned/affected row와 결과값을 검증 대상으로 사용한다.

## 제약과 주의점

타 DBMS를 이 저장소에서 실행한 결과가 아니라 공식 문서 기반 비교다. GlueSQL에 없는 cost/actual operator row를 억지로 대응시키지 않는다.

문서에서 직접 측정한 값, planned AST의 직접 필드, 안전한 UI 해석, 코드에서 확인되지 않는 추정을 구분한다. 확인 불가능한 값은 0으로 가장하지 않고 limitations에 남긴다.

## 완료 기준

- 실제 파일 경로·함수명·테스트 이름 기록
- 메인 관계형 기능 회귀 없음
- Tree/JSON 또는 runtime 결과에서 단계 핵심을 관찰 가능
- README, roadmap, 초심자 가이드와 현재 단계 정합성 유지
