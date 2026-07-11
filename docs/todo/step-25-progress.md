# Step 25 진행 상황: Aggregate 기반 통계 추가

## 현재 상태

Step 25에서는 `project stats <id>`의 단일 Project 통계와 `project stats`의 전체 Project 통계를 제공한다. 전체 통계는 `LEFT JOIN`, `GROUP BY`, 조건부 COUNT를 실제 GlueSQL query로 실행한다.

## Step 24에서 Step 25로 달라진 점

| 구분 | Step 24 | Step 25 |
| --- | --- | --- |
| 관계형 query | JOIN/filter/sort | aggregate 추가 |
| Project 조회 | 기본 정보와 Task filter | 완료 통계 제공 |
| 결과 타입 | `Project`, `TaskDetail` | `ProjectStats` |
| 계산 위치 | DB row 변환 | COUNT는 DB, 완료율은 Rust |

## 실제 실행 흐름

```text
project stats 1
-> show_project(1)로 존재 확인
-> SELECT COUNT(*) FROM tasks WHERE project_id = 1
-> SELECT COUNT(*) FROM tasks WHERE project_id = 1 AND done = TRUE
-> ProjectStats::new(project, total, done)
-> todo와 completion_rate 계산
```

`select_count`는 GlueSQL의 마지막 `Payload::Select`에서 첫 번째 `Value::I64`를 읽는다. 예상한 SELECT/COUNT 형태가 아니면 `AppError::GlueSql`을 반환한다.

## 계산 책임 분리

```rust
let completion_rate = if total == 0 {
    0.0
} else {
    done as f64 * 100.0 / total as f64
};
```

- SQL은 row 수 집계를 담당한다.
- `ProjectStats::new`는 `todo = total - done`과 사용자 출력용 완료율을 담당한다.
- Task가 없는 Project의 완료율은 0.0%다.

전체 Project 통계는 다음 query를 사용하며 Task가 없는 Project도 결과에 남는다.

```sql
SELECT p.id, p.name,
       COUNT(t.id),
       COUNT(CASE WHEN t.done = TRUE THEN 1 END)
FROM projects p
LEFT JOIN tasks t ON t.project_id = p.id
GROUP BY p.id, p.name
ORDER BY p.id;
```

## 테스트 증거

- `lists_project_tasks_by_priority_then_id_and_calculates_stats`: total 2, done 1
- `project::tests::calculates_rate`: 75.0% 계산
- `project::tests::empty_rate_is_zero`: 0건 정책
- `aggregates_all_projects_with_left_join`: 빈 Project를 포함한 전체 집계
- 기존 `calculates_stats_with_gluesql_count`: 전체 Task 통계 회귀 방지

## 완료 기준

- Project 존재 여부 확인
- 전체/완료 수가 aggregate SQL로 계산
- 미완료 수와 완료율 출력
- 빈 Project 통계 정책 테스트
- 최종 80개 테스트 통과

## 다음 단계

Step 26에서는 Project/Task/Tag 삭제 정책과 storage별 transaction 범위를 정리한다.
