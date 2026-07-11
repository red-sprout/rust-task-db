# Query Lab 개요

Query Lab은 기존 `projects`, `tasks`, `tags`, `task_tags` SQL을 대상으로 다음 경계를 관찰한다.

```text
SQL -> planned ast::Statement -> Access Path/JoinExecutor
-> Executor -> TracingStorage -> Row Iterator -> Payload
```

실행 명령:

```bash
cargo run -- analyze "SELECT * FROM tasks"
cargo run -- analyze --plan "SELECT * FROM tasks WHERE id = 1"
cargo run -- analyze --runtime "SELECT * FROM tasks"
cargo run -- analyze --raw-plan "SELECT ..."
cargo run -- analyze --format json "SELECT ..."
cargo run -- lab list
cargo run -- lab run join
```

GlueSQL 근거는 `gluesql-core-0.19.0/src/glue.rs`, `ast.rs`, `ast/query.rs`, `executor/fetch.rs`, `store.rs`, `store/index.rs`다. 프로젝트 테스트 `query_lab::tests::*`가 plan/runtime 경계를 검증한다.

공개 API로 Storage 호출과 iterator 소비 row는 측정 가능하다. Filter, Join, Aggregate, Sort 내부 input/output row는 공개 hook이 없어 측정 불가능하다.

## 독자와 사용 목적

이 문서는 GlueSQL을 처음 읽는 개발자가 SQL 결과만 확인하던 단계에서, 계획과 실제 저장소 접근을 분리해 관찰하도록 돕는다. 메인 Project/Task/Tag 기능은 바꾸지 않으며 `src/query_lab/`은 관찰 계층으로만 동작한다.

| 경계 | 코드에서 확인하는 값 | 확인할 수 없는 값 |
| --- | --- | --- |
| Plan | planned `Statement`, `IndexItem`, `JoinExecutor` | estimated rows, cost |
| Runtime | Store API 호출, iterator row, Payload row | operator별 input/output |
| Mutation | affected row, write API 호출 | row별 변경 전/후 값 |

## 읽는 순서와 수정 지점

1. `src/query_lab/mod.rs::analyze`에서 plan과 runtime의 경계를 확인한다.
2. `src/query_lab/plan.rs::statement_to_node`에서 AST를 표시 모델로 바꾸는 규칙을 확인한다.
3. `src/query_lab/runtime.rs::TracingStorage`에서 Storage 위임과 계측을 확인한다.
4. 새 실험 SQL은 `src/query_lab/scenarios.rs::scenario_sql`에 추가한다.
5. 새 출력 필드는 `AnalysisReport`와 Tree/JSON 테스트를 함께 수정한다.

`lab run mutation`과 mutation을 포함하는 `lab run all`은 현재 Sled 데이터에 실제 UPDATE/DELETE를 수행한다. 안전한 관찰만 원하면 `analyze --plan`을 사용하거나 별도 DB 복사본에서 실행해야 한다.
