# Step 9 진행 상황: SQL 실행 모드

## 현재 상태

현재 코드는 `docs/prompt.md`의 `Step 9. SQL 실행 모드`까지 구현되어 있다.

Step 9에서는 `rust-task sql "SELECT * FROM tasks"`처럼 사용자가 입력한 SQL 문자열을 GlueSQL에 직접 전달한다.

## Step 8에서 Step 9로 달라진 점

| 구분 | Step 8 | Step 9 |
| --- | --- | --- |
| 지원 명령 | `add`, `list`, `done`, `delete`, `search`, `stats` | `sql` 추가 |
| 명령 enum | `Command::Sql` 없음 | `Command::Sql { sql }` 추가 |
| repository trait | Todo 전용 메서드 | `execute_sql` 추가 |
| SQL 사용 위치 | repository 내부 구현 | 사용자가 CLI로 SQL 직접 입력 |
| 출력 | Todo/Stats 전용 출력 | `SqlResult`로 SELECT/변경 결과 구분 |

## 현재 파일

| 파일 | 역할 |
| --- | --- |
| `src/command.rs` | `Command::Sql { sql }` 추가 |
| `src/cli.rs` | `sql` 명령 parsing |
| `src/repository/mod.rs` | `SqlResult`, `TaskRepository::execute_sql` 추가 |
| `src/repository/gluesql_repository.rs` | GlueSQL `execute_sql`, `Payload`를 `SqlResult`로 변환 |
| `src/service.rs` | `TaskService::execute_sql` 추가 |
| `src/main.rs` | SQL 결과 출력 |

## 현재 동작

```bash
cargo run -- sql "SELECT * FROM tasks"
cargo run -- sql "INSERT INTO tasks VALUES (1, 'Rust 공부', FALSE); SELECT id, title, done FROM tasks;"
```

주의: Step 9도 GlueSQL `MemoryStorage`를 사용하므로 실행이 끝나면 데이터가 사라진다. SQL 입력 안에서 `INSERT`와 `SELECT`를 함께 쓰면 같은 실행 안에서 결과를 확인할 수 있다.

## 완료된 테스트

- 총 42개 테스트 통과

## 다음 단계

Step 10으로 넘어가 `repl` 명령과 `.schema`, `.exit`, `.quit` 입력을 추가했다. 현재 진행 상태는 `docs/todo/step-10-progress.md`를 본다.
