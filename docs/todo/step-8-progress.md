# Step 8 진행 상황: GlueSQL 저장소 추가

## 현재 상태

현재 코드는 `docs/prompt.md`의 `Step 8. GlueSQL 저장소 추가`까지 구현되어 있다.

Step 8에서는 기존 JSON 저장소를 지우지 않고, 같은 `TaskRepository` trait를 구현하는 `GlueSqlTaskRepository`를 추가했다.

## Step 7에서 Step 8으로 달라진 점

| 구분 | Step 7 | Step 8 |
| --- | --- | --- |
| 활성 저장소 | `JsonTaskRepository` | `GlueSqlTaskRepository` |
| 저장 방식 | `tasks.json` 파일 | GlueSQL `MemoryStorage` |
| 기존 JSON 코드 | 기본 실행 경로 | 보존된 교체 가능 구현체 |
| service 구조 | `TaskService<JsonTaskRepository>` | `TaskService<GlueSqlTaskRepository>` |
| repository 조회 메서드 | `&self`로 조회 | GlueSQL 실행 때문에 `&mut self`로 조회 |
| 새 crate | 없음 | `gluesql`, `futures` |
| SQL 직접 입력 | 코드에서 확인되지 않음 | 이후 Step 9 예정 |

## 현재 파일

| 파일 | 역할 |
| --- | --- |
| `Cargo.toml` | `gluesql`, `futures` dependency 추가 |
| `src/main.rs` | `GlueSqlTaskRepository::new()`를 service에 주입 |
| `src/error.rs` | `AppError::GlueSql(String)` 추가 |
| `src/repository/mod.rs` | `TaskRepository` trait, 기존 `JsonTaskRepository` 보존, `GlueSqlTaskRepository` re-export |
| `src/repository/gluesql_repository.rs` | GlueSQL `MemoryStorage`, `tasks` table, SQL 기반 Todo 기능 |
| `src/service.rs` | 조회 메서드도 `&mut self` 흐름으로 조정 |

## 현재 동작

```bash
cargo run -- add "Rust 공부"
cargo run -- list
cargo run -- search rust
cargo run -- stats
```

주의: Step 8의 GlueSQL 저장소는 `MemoryStorage`라서 프로세스가 끝나면 데이터가 사라진다. 그래서 `cargo run -- add` 다음에 별도 프로세스로 `cargo run -- list`를 실행하면 방금 추가한 Todo가 남아 있지 않다.

같은 저장소 인스턴스 안에서의 흐름은 `src/repository/gluesql_repository.rs`의 테스트로 검증한다.

## 완료된 테스트

- CLI parser 테스트 9개
- error 테스트 4개
- main 흐름 보조 테스트 1개
- JSON repository 테스트 9개
- GlueSQL repository 테스트 8개
- service 테스트 6개
- 총 37개 테스트 통과

## 다음 단계

다음은 Step 9이다. Step 9에서는 `rust-task sql "SELECT * FROM tasks"`처럼 SQL 문자열을 직접 실행하는 명령을 추가한다.
