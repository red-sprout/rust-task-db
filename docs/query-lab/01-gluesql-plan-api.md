# GlueSQL Plan API

GlueSQL 0.19의 실제 API는 다음과 같다.

```rust
pub async fn plan(&mut self, sql: impl AsRef<str>) -> Result<Vec<Statement>>;
pub async fn execute_stmt(&mut self, statement: &Statement) -> Result<Payload>;
```

요청서의 `Vec<StatementPlan>`과 달리 실제 반환형은 planned `ast::Statement`다. `core/src/glue.rs`에서 parse -> translate -> `storage.plan`을 수행하고, `execute_stmt`가 같은 statement를 executor에 전달한다.

`GlueSqlTaskRepository::plan_sql`과 `execute_statement`가 async 경계를 repository 내부 `block_on`으로 유지한다. 테스트 `primary_key_plan_uses_fetch_data`는 계획과 실행이 같은 statement를 공유하는지 검증한다.

## 실제 호출 흐름

```rust
let statements = repository.plan_sql(sql)?;
for statement in &statements {
    repository.execute_statement(statement)?;
}
```

`src/query_lab/mod.rs::analyze`는 먼저 모든 statement를 계획하고 raw Debug와 `PlanNode`를 만든 뒤, runtime 요청일 때만 같은 statement를 실행한다. 따라서 `--plan`은 INSERT/UPDATE/DELETE를 실행하지 않는다. 테스트 `plan_only_does_not_execute_mutation`이 이 안전 경계를 검증한다.

요청서의 `StatementPlan`이라는 이름은 GlueSQL 0.19 public API에 존재하지 않는다. 이 프로젝트 문서에서 “계획”은 parser 결과 그대로가 아니라 storage planner가 반영된 `ast::Statement`를 뜻한다. 버전 업그레이드 시 `glue.plan` 반환형과 `Planner::plan` 호출 순서를 가장 먼저 재확인해야 한다.
