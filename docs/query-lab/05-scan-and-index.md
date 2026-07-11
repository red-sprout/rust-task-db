# Scan과 Index

Full scan SQL은 `scan_data = 1`, rows consumed가 실제 table row 수와 같음을 `runtime_counts_scan_and_iterator_rows`에서 확인한다.

새 schema의 PK predicate는 planned `IndexItem::PrimaryKey`와 `fetch_data`를 사용한다. Step 18에서 migration한 기존 Sled `tasks` table은 사후 PK 추가가 없어 같은 SQL이 Table Scan일 수 있다. Query Lab은 이 차이를 그대로 출력한다.

SledStorage에는 다음 인덱스를 생성한다.

```sql
CREATE INDEX idx_tasks_project_id ON tasks(project_id);
CREATE INDEX idx_tasks_done ON tasks(done);
CREATE INDEX idx_task_tags_tag_id ON task_tags(tag_id);
```

GlueSQL 0.19 core의 `Planner` 기본 구현은 `plan_primary_key`, `plan_join`만 호출하지만, `gluesql_sled_storage/src/planner.rs`의 `SledStorage` 전용 구현은 그 사이에 `plan_index`를 호출한다. `TracingStorage<S>`는 반드시 `self.inner.plan(statement)`를 위임해야 이 전용 구현이 보존된다. 초기 구현은 빈 `Planner` impl로 core 기본 구현을 선택해 secondary index를 가렸으며, 현재는 위임하도록 수정했다.

수정 후 `project_id = 1`은 `idx_tasks_project_id`, `done = TRUE`는 `idx_tasks_done`을 `IndexItem::NonClustered`로 계획하고 runtime에서 `scan_indexed_data = 1`이 확인된다. 다만 Sled planner에 cardinality/cost 비교는 확인되지 않으므로 높은 선택도의 `done = TRUE`도 인덱스를 선택한다.

## 판정 표

| SQL 형태 | Plan에서 볼 항목 | Runtime 기대 | 현재 결론 |
| --- | --- | --- | --- |
| 조건 없음 | `index: None` | `scan_data` | Full scan 확인 가능 |
| `id = literal` | `PrimaryKey` | `fetch_data` | 새 schema에서 직접 확인 |
| `project_id = 1` | `NonClustered` | `scan_indexed_data` | Sled `plan_index`가 인덱스 선택 |
| `done = TRUE` | `NonClustered` | `scan_indexed_data` | 선택도 비용 비교 없이 인덱스 선택 |

테스트 `runtime_counts_scan_and_iterator_rows`는 2개 task에 대해 scan 1회, consumed 2, returned 2를 정확히 검증한다. `primary_key_plan_uses_fetch_data`는 새 Memory schema에서 PK plan과 fetch 1회를 검증한다. `renders_non_clustered_index_access_path`는 AST renderer 단위 테스트이며, 실제 Sled planner 위임과 index scan은 `tracing_storage_delegates_sled_secondary_index_planning`이 검증한다.
