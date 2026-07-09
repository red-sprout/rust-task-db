# Step 17 진행 상황: Query Execution 상세 분석

## 현재 상태

Step 17에서는 새 CLI 명령이나 새 외부 crate를 추가하지 않는다. 현재 Todo 명령이 `GlueSqlTaskRepository` 안에서 어떤 SQL로 바뀌고, GlueSQL `Payload`가 `Task`, `TaskStats`, `SqlResult`로 변환되는 흐름을 문서화한다.

## Step 16에서 Step 17로 달라진 점

| 구분 | Step 16 | Step 17 |
| --- | --- | --- |
| 핵심 작업 | Minimal Custom Storage 책임 분석 | Query Execution 상세 분석 |
| 코드 변경 | 없음 | 없음 |
| 새 CLI 명령 | 없음 | 없음 |
| 새 외부 crate | 없음 | 없음 |
| 테스트 수 | 65개 | 65개 |

## 완료한 일

| 항목 | 내용 |
| --- | --- |
| 새 문서 | `docs/beginner-codebase-guide/19-query-execution.md` |
| Todo 명령별 SQL | `add`, `list`, `done`, `delete`, `search`, `stats`, `sql`, `repl`이 실행하는 SQL 흐름 정리 |
| 변환 흐름 | `execute`, `select_tasks`, `row_to_task`, `select_count`, `payload_to_sql_result`, `value_to_string` 설명 |
| 실패 경계 | `AppError::GlueSql`, `AppError::NotFound`, `AppError::Unsupported`가 올라오는 지점 정리 |
| 문서 갱신 | README, AGENTS, roadmap, 초심자 가이드 인덱스와 실행 흐름 문서 갱신 |

## 핵심 개념

현재 프로젝트의 query execution은 `src/repository/gluesql_repository.rs`의 `execute`에 모인다.

```text
TaskService
-> TaskRepository
-> GlueSqlTaskRepository
-> execute(sql)
-> Glue::execute(sql)
-> Payload
-> Task / TaskStats / SqlResult
```

Todo 전용 명령은 `Payload`를 domain model로 바꾸고, `sql`/`repl`은 `Payload`를 CLI 출력용 `SqlResult`로 바꾼다.

## 완료 기준

- `docs/beginner-codebase-guide/19-query-execution.md`가 실제 함수명과 SQL을 설명
- README, AGENTS, roadmap, 초심자 가이드가 Step 17과 65개 테스트를 일관되게 설명
- `cargo fmt --check` 통과
- `cargo check` 통과
- `cargo test` 통과

## 다음 단계

Step 18에서는 Storage별 기능 비교 표를 고도화한다.
