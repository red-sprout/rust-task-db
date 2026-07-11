# Runtime Tracing

`src/query_lab/runtime.rs`의 `TracingStorage<S>`는 inner storage를 변경하지 않고 다음 trait을 위임한다.

- `Store`, `StoreMut`
- `Index`, `IndexMut`
- `Metadata`, `Transaction`
- `CustomFunction`, `CustomFunctionMut`
- `AlterTable`, `Planner`

`fetch_data`, `scan_data`, `scan_indexed_data` 호출 수를 세고 반환 `RowIter`를 `map`으로 감싸 실제 소비 row를 센다. mutation은 append/insert/delete 호출 수를 센다. `TraceMetrics::reset`은 query마다 초기화한다.

테스트: `runtime_counts_scan_and_iterator_rows`, `primary_key_plan_uses_fetch_data`, `metrics_reset_for_each_query`.

측정 불가: Filter 전후, Join 중간 결과, Hash build/probe, Aggregate input/group, Sort input, subquery 반복 횟수. 근거는 공개 executor API에 operator hook이 없기 때문이다.

## Operator trace 방식 조사

- 방식 A 공개 API: planned Statement와 최종 Payload를 사용한다. 현재 기본 기능이다.
- 방식 B Wrapper hook: `TracingStorage`로 Storage 호출/iterator 소비를 계측한다. 현재 기본 기능이다.
- 방식 C Patch/Fork: executor의 Filter/Join/Aggregate/Sort stream 사이에 metric hook을 넣어야 한다. upstream 수정 없이 불가능하므로 메인 프로젝트에는 넣지 않았다.

선택적 `gluesql-internal-trace` feature는 현재 코드에 없다. 실제 fork API가 정해지기 전에 빈 feature를 추가하면 측정 가능하다는 오해를 주므로 기여 RFC 이후로 남긴다.

## 계측 시점

| Counter | 증가 시점 | row 의미 |
| --- | --- | --- |
| `fetch_data_calls` | PK key fetch 진입 | `Some(row)`일 때 consumed +1 |
| `scan_data_calls` | table iterator 요청 | iterator가 `Ok(row)`를 yield할 때 +1 |
| `scan_indexed_data_calls` | index iterator 요청 | index iterator가 yield할 때 +1 |
| write counters | append/insert/delete API 진입 | 호출 횟수이며 변경 row 수가 아님 |

`AtomicU64`와 `Arc<Counters>`를 사용하므로 wrapper가 만든 iterator closure도 같은 counter를 갱신한다. Ordering은 통계 누적만 필요하므로 `Relaxed`다. `rows_consumed`는 Storage가 executor에 공급한 row이며 Filter 통과 row와 같지 않다. 최종 SELECT row는 `Payload::Select.rows.len()`, 변경 row는 Payload의 count로 별도 기록한다.

초심자가 새 counter를 추가하려면 `Counters`, `TraceMetrics::reset`, `snapshot`, 대상 trait 위임, `MetricsSnapshot`, report 출력과 JSON 테스트를 모두 수정해야 한다.
