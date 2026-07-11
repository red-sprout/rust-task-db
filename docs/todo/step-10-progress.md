# Step 10 진행 상황: REPL 모드

## 현재 상태

현재 코드는 `docs/prompt.md`의 `Step 10. REPL 모드`까지 구현되어 있다.

Step 10에서는 `cargo run -- repl`로 작은 SQL 콘솔을 실행한다. REPL 안에서는 같은 `GlueSqlTaskRepository` 인스턴스를 계속 사용하므로 `INSERT` 후 `SELECT`로 바로 결과를 확인할 수 있다.

## Step 9에서 Step 10으로 달라진 점

| 구분 | Step 9 | Step 10 |
| --- | --- | --- |
| 지원 명령 | `sql`로 SQL 문자열 1회 실행 | `repl`로 여러 SQL 입력 |
| 명령 enum | `Command::Sql { sql }` | `Command::Repl` 추가 |
| 입력 방식 | CLI 인자 하나 | 표준 입력 line 반복 |
| 종료 방식 | 명령 실행 후 종료 | `.exit` 또는 `.quit` |
| schema 확인 | 코드/문서 확인 | `.schema` 명령으로 출력 |

## 현재 파일

| 파일 | 역할 |
| --- | --- |
| `src/command.rs` | `Command::Repl` 추가 |
| `src/cli.rs` | `repl` 명령 parsing |
| `src/main.rs` | `Command::Repl`에서 `repl::run_repl` 호출 |
| `src/repl.rs` | REPL 입력 루프, `.schema`, `.exit`, `.quit`, SQL 실행 결과 출력 |
| `src/service/mod.rs` | 기존 `TaskService::execute_sql`을 REPL에서도 사용 |
| `src/repository/gluesql_repository.rs` | 기존 `execute_sql`로 REPL SQL 실행 |

## 현재 동작

```bash
cargo run -- repl
```

REPL 안에서:

```text
rust-task> INSERT INTO tasks VALUES (1, 'Rust 공부', FALSE);
rust-task> SELECT id, title, done FROM tasks;
rust-task> .schema
rust-task> .exit
```

주의: Step 10도 GlueSQL `MemoryStorage`를 사용한다. REPL을 종료하면 데이터는 사라진다.

## 완료된 테스트

- 총 46개 테스트 통과

## 다음 단계

다음은 Step 11이다. Step 11에서는 필요한 테스트를 더 촘촘히 보강한다.
