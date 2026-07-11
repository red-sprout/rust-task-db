# Scan과 Index

Full scan SQL은 `scan_data = 1`, rows consumed가 실제 table row 수와 같음을 `runtime_counts_scan_and_iterator_rows`에서 확인한다.

새 schema의 PK predicate는 planned `IndexItem::PrimaryKey`와 `fetch_data`를 사용한다. Step 18에서 migration한 기존 Sled `tasks` table은 사후 PK 추가가 없어 같은 SQL이 Table Scan일 수 있다. Query Lab은 이 차이를 그대로 출력한다.

SledStorage에는 다음 인덱스를 생성한다.

```sql
CREATE INDEX idx_tasks_project_id ON tasks(project_id);
CREATE INDEX idx_tasks_done ON tasks(done);
CREATE INDEX idx_task_tags_tag_id ON task_tags(tag_id);
```

그러나 GlueSQL 0.19 기본 `Planner::plan`은 `plan_primary_key`, `plan_join`만 호출한다(`store/planner.rs`). 일반 SQL predicate가 `NonClustered`로 자동 계획되지 않아 현재 실험에서는 `scan_indexed_data = 0`이 관찰된다. 인덱스 존재를 인덱스 선택으로 해석하면 안 된다.

## 판정 표

| SQL 형태 | Plan에서 볼 항목 | Runtime 기대 | 현재 결론 |
| --- | --- | --- | --- |
| 조건 없음 | `index: None` | `scan_data` | Full scan 확인 가능 |
| `id = literal` | `PrimaryKey` | `fetch_data` | 새 schema에서 직접 확인 |
| `project_id = 1` | `NonClustered` 여부 | indexed 또는 scan | 기본 planner는 자동 선택하지 않음 |
| `done = TRUE` | 동일 | 동일 | index 생성만으로 사용을 보장하지 않음 |

테스트 `runtime_counts_scan_and_iterator_rows`는 2개 task에 대해 scan 1회, consumed 2, returned 2를 정확히 검증한다. `primary_key_plan_uses_fetch_data`는 새 Memory schema에서 PK plan과 fetch 1회를 검증한다. `renders_non_clustered_index_access_path`는 planner 선택 테스트가 아니라 AST variant renderer 단위 테스트라는 점을 구분해야 한다.
