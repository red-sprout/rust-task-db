# Selectivity

선택도 실험은 같은 SQL shape에서 데이터 분포만 바꾸고 다음을 비교한다.

```text
Plan -> Access Path -> Storage Calls -> Rows Consumed -> Rows Returned -> Elapsed
```

GlueSQL 0.19 planned Statement에는 estimated cardinality와 cost가 없다. SledStorage의 `plan_index`는 secondary index를 계획하지만 table statistics를 읽어 선택도 비용을 비교하는 코드는 확인되지 않는다. 따라서 선택도가 낮은 `project_id = 1`과 높은 `done = TRUE`가 모두 index scan이어도 비용 기반 선택이라고 부르지 않는다.

skewed seed는 특정 Project/Tag/done/priority에 데이터를 집중시킨다. 실행하지 않은 profile의 측정값은 현재 결과처럼 설명하지 않는다.

현재 `lab seed --skewed`는 10 Project/10,000 Task/20 Tag 중 약 80%를 첫 Project와 첫 Tag에 집중한다. 근거 코드는 `GlueSqlTaskRepository::seed_lab_profile`, scenario 검증은 `scenario_list_and_scan_queries_are_available`다.

## Seed profile

| Profile | Project | Task | Tag | 목적 |
| --- | ---: | ---: | ---: | --- |
| small | 10 | 1,000 | 20 | 빠른 기능 확인 |
| medium | 100 | 100,000 | 100 | 분포 차이 관찰 |
| large | 250 | 250,000 | 200 | 선택 실행 부하 실험 |
| skewed | 10 | 10,000 | 20 | 첫 Project/Tag 약 80% 집중 |

이 숫자는 코드에 설정된 생성량이며 이 문서 작성 시 medium/large 성능 측정 결과가 아니다. `execute_batches`는 500 statement 단위로 실행하고 `reserve_ids`로 ID 범위를 먼저 확보한다. profile별 metadata key로 같은 profile의 재실행을 막는다.

Task INSERT는 `(id, project_id, title, done, priority)` column 목록을 명시한다. Step 18에서 3열 `tasks`를 ALTER한 DB는 물리 column 순서가 신규 schema와 다를 수 있으므로, column 없는 `INSERT INTO tasks VALUES (...)`를 사용하면 숫자 `project_id`가 TEXT `title` 위치로 들어갈 수 있기 때문이다. 회귀 테스트 `inserts_task_after_migrating_step_18_column_order`가 이 경계를 검증한다.

`large`는 전체 profile을 하나의 Sled transaction으로 보호한다. 비정상 종료로 lock이 남으면 `query_lab::persistent_traced`가 시작 시 60초 stale timeout으로 복구를 유도하고 초기화 완료 후 GlueSQL 기본값과 같은 1시간 timeout으로 되돌린다.

선택도 비교 보고에는 profile, 실제 row count, predicate 결과 row, plan, 세 Storage call, consumed/returned, elapsed를 함께 남겨야 한다. elapsed 단독 비교는 캐시와 시스템 부하 영향을 분리하지 못한다.
