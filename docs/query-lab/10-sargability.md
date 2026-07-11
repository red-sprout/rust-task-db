# Sargability

비교 대상:

```sql
id = 10
id + 1 = 11
project_id = 1
title = 'Rust'
LOWER(title) = 'rust'
title LIKE 'Rust%'
title LIKE '%Rust%'
```

판단 순서는 plan의 `TableFactor.index`, runtime의 fetch/scan/indexed 호출, rows consumed, rows returned다. 시간만으로 sargable 여부를 결론 내리지 않는다.

GlueSQL 0.19 core 기본 planner에서는 PK equality가 변환되고, SledStorage 전용 planner는 `plan_index`를 추가로 호출해 단일 table의 secondary-index equality도 변환한다. 계산식과 함수/LIKE predicate, JOIN 내부 filter의 index pushdown 여부는 각각 실제 plan과 runtime 결과로 판정한다.

근거 파일은 `store/planner.rs`, `plan/primary_key.rs`, `executor/fetch.rs`다. 테스트 `primary_key_plan_uses_fetch_data`, `renders_non_clustered_index_access_path`가 구분을 검증한다.

## 결과 기록 형식

| Predicate | Planned access | Storage API | 판단 |
| --- | --- | --- | --- |
| `id = 10` | PK 또는 Scan | fetch/scan | schema 상태와 함께 기록 |
| `id + 1 = 11` | 실제 plan 확인 | 실제 call 확인 | 동치식이라고 PK 사용을 추정하지 않음 |
| `LOWER(title)` | 실제 plan 확인 | 실제 call 확인 | 함수식 residual 여부를 추정하지 않음 |
| prefix/suffix LIKE | 실제 plan 확인 | 실제 call 확인 | 문자열 패턴만으로 range scan이라 부르지 않음 |

Sargable이라는 용어는 “DBMS가 access path 조건으로 변환할 가능성이 있는 표현”을 뜻한다. Query Lab의 최종 판정은 반드시 planned `IndexItem`과 runtime Store call이 일치할 때만 확정한다.
