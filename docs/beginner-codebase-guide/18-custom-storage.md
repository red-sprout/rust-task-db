# Minimal Custom Storage 분석

## 이 문서의 목적

이 문서는 GlueSQL에 custom storage를 붙인다는 말이 무엇인지 현재 `rust-task` 코드와 연결해 설명한다.

중요한 범위 제한:

- Step 16에서는 production custom storage를 구현하지 않는다.
- 새 CLI 명령을 추가하지 않는다.
- 새 외부 crate를 추가하지 않는다.
- GlueSQL upstream source를 수정하지 않는다.
- 이 문서는 최소 custom storage를 만들 때 필요한 책임과 구현 순서를 이해하기 위한 분석 문서다.

## 두 종류의 저장소 추상화

현재 프로젝트에는 저장소 추상화가 두 층 있다.

| 층 | trait 또는 타입 | 보는 관점 | 역할 |
| --- | --- | --- | --- |
| 앱 계층 | `TaskRepository` | Todo 앱 | `add`, `list`, `done`, `delete`, `search`, `stats`, `execute_sql` |
| GlueSQL 계층 | `Store`, `StoreMut`, `GStore`, `GStoreMut`, `Planner` | SQL engine | schema와 row를 읽고 쓰는 storage adapter 계약 |

현재 코드에서 앱 계층은 `src/repository/mod.rs`에 있다.

```rust
pub trait TaskRepository {
    fn add(&mut self, title: String) -> Result<Task, AppError>;
    fn find_all(&mut self) -> Result<Vec<Task>, AppError>;
    fn mark_done(&mut self, id: i64) -> Result<(), AppError>;
    fn delete(&mut self, id: i64) -> Result<Task, AppError>;
    fn search(&mut self, keyword: &str) -> Result<Vec<Task>, AppError>;
    fn stats(&mut self) -> Result<TaskStats, AppError>;
    fn execute_sql(&mut self, sql: String) -> Result<Vec<SqlResult>, AppError>;
}
```

GlueSQL 계층은 `src/repository/gluesql_repository.rs`의 generic bound에 드러난다.

```rust
pub struct GlueSqlTaskRepository<S = MemoryStorage>
where
    S: GStore + GStoreMut + Planner,
{
    glue: Glue<S>,
}
```

코드 해석:

- `TaskRepository`: Todo 앱이 원하는 기능 이름을 가진다.
- `GStore`, `GStoreMut`, `Planner`: GlueSQL이 SQL을 실행하기 위해 storage에 요구하는 기능 묶음이다.
- `GlueSqlTaskRepository<S>`: 두 세계를 이어준다. 앱에는 `TaskRepository`로 보이고, 내부에서는 `Glue<S>`로 SQL을 실행한다.

## Custom Storage는 무엇을 구현하는가

GlueSQL custom storage는 SQL parser를 새로 만드는 일이 아니다.

GlueSQL이 담당하는 것:

- SQL 문자열 parsing
- AST와 plan 생성
- expression 평가
- projection, filter, aggregation 실행
- `Payload` 결과 생성

Storage가 담당하는 것:

- schema 조회
- schema 저장 또는 삭제
- row 조회
- row 추가
- row 수정
- row 삭제
- 필요한 경우 index, transaction, metadata 지원

```text
SQL String
-> GlueSQL Parser / Planner / Executor
-> Store trait 계열 호출
-> Custom Storage
-> 실제 데이터 구조
```

## 읽기 전용 storage와 쓰기 가능 storage

최소 custom storage를 생각할 때는 먼저 읽기 전용과 쓰기 가능 storage를 나눠야 한다.

| 구분 | 필요한 책임 | 가능한 SQL |
| --- | --- | --- |
| 읽기 전용 storage | schema 조회, row scan, row fetch | `SELECT` 중심 |
| 쓰기 가능 storage | schema 변경, row append/insert/delete/update | `CREATE TABLE`, `INSERT`, `UPDATE`, `DELETE` |
| transaction storage | begin/commit/rollback 상태 관리 | `BEGIN`, `COMMIT`, `ROLLBACK` |
| index storage | index 생성, index lookup, index 갱신 | index가 필요한 최적화 |

현재 Todo CLI는 add/list/done/delete/search/stats/sql/repl을 지원한다. 그래서 현재 활성 storage인 `SledStorage`는 읽기와 쓰기 모두 가능해야 한다.

## 현재 Todo 기능과 storage 책임 연결

| Todo 기능 | repository 메서드 | SQL | 필요한 storage 책임 |
| --- | --- | --- | --- |
| 추가 | `add` | `INSERT INTO tasks ...` | row insert |
| 목록 | `find_all` | `SELECT ... ORDER BY id` | row scan |
| 완료 | `mark_done` | `UPDATE tasks SET done = TRUE ...` | row update |
| 삭제 | `delete` | `DELETE FROM tasks ...` | row delete |
| 검색 | `search` | `SELECT ... WHERE title ILIKE ...` | row scan + executor filter |
| 통계 | `stats` | `SELECT COUNT(*) ...` | row scan + aggregation |
| SQL 직접 실행 | `execute_sql` | 사용자 입력 SQL | SQL별 필요한 storage 책임 |

이 표에서 중요한 점:

`search`와 `stats`는 앱 코드에서 직접 iterator로 처리하지 않는다. GlueSQL에 SQL을 넘기고, GlueSQL executor가 필요한 평가를 수행한다.

## Minimal Custom Storage를 만든다면 순서

Step 16에서는 실제 구현하지 않지만, 구현한다면 순서는 아래가 안전하다.

1. 저장할 row 자료구조를 정한다.
2. schema를 어디에 보관할지 정한다.
3. `SELECT`만 되는 읽기 경로를 먼저 만든다.
4. `CREATE TABLE`과 `INSERT`를 붙인다.
5. `UPDATE`, `DELETE`를 붙인다.
6. `COUNT`, `WHERE`, `ORDER BY`가 GlueSQL executor를 통해 기대대로 동작하는지 테스트한다.
7. transaction이나 index는 마지막에 검토한다.

```text
읽기 가능
-> 생성/삽입 가능
-> 수정/삭제 가능
-> transaction 가능
-> index 가능
```

## 현재 프로젝트에서 바로 구현하지 않는 이유

현재 프로젝트의 학습 목표는 Rust CLI Todo와 GlueSQL 구조 이해다. production custom storage를 지금 넣으면 다음 부담이 생긴다.

- GlueSQL storage trait 구현 세부사항이 Rust 초심자 범위를 크게 넘는다.
- async trait, row iterator, schema 타입, key 타입을 한 번에 설명해야 한다.
- 기본 실행 저장소가 `SledStorage`인 현재 단계와 역할이 겹친다.
- 새 custom storage가 Todo 기능보다 GlueSQL 구현 자체로 학습 초점을 옮긴다.

그래서 Step 16은 분석 문서로 고정하고, 실제 custom storage 구현은 이후 단계 예정으로 둔다.

## 초심자가 수정할 수 있는 지점

문서 실습으로는 `src/repository/gluesql_repository.rs`의 SQL 문자열과 `18-custom-storage.md`의 책임 표를 함께 보는 것이 좋다.

예를 들어 `stats`를 보면:

```rust
fn stats(&mut self) -> Result<TaskStats, AppError> {
    let total = select_count(self, "SELECT COUNT(*) FROM tasks;")?;
    let done = select_count(self, "SELECT COUNT(*) FROM tasks WHERE done = TRUE;")?;

    Ok(TaskStats::new(total, done))
}
```

코드 해석:

- 앱은 `TaskStats`가 필요하다.
- repository는 `COUNT` SQL을 만든다.
- GlueSQL은 SQL 실행 흐름을 처리한다.
- storage는 row를 읽을 수 있어야 한다.

이처럼 앱 기능 하나를 storage 책임으로 바꿔 생각하는 연습이 Step 16의 핵심이다.

## 이후 단계 예정

코드에서 확인되지 않음:

- 직접 custom storage struct 추가
- `Store`와 `StoreMut` 직접 구현
- transaction 직접 구현
- index 직접 구현
- 기본 CLI 저장소를 custom storage로 교체

이 항목들은 이후 단계에서 별도 요구가 있을 때 다룬다.
