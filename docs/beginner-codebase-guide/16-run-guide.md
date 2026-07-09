# 실행 가이드

## 이 문서의 목적

이 문서는 초심자가 `rust-task`를 터미널에서 직접 실행하고, 현재 Step 18의 Storage별 기능 비교표 상태와 Step 12에서 완성된 저장 흐름, SQL 실행 흐름, REPL 흐름, 테스트 상태를 이해하게 돕는다.

## 실행 전 확인

프로젝트 루트에서 명령을 실행한다.

```bash
pwd
```

기대 위치:

```text
/Users/jujaewan/1_Projects/rust-task-db
```

이 위치에 `Cargo.toml`이 있어야 한다. `tasks.json`은 Step 7까지 사용한 JSON 저장소 파일로 남아 있지만, Step 12 기본 실행 경로에서는 사용하지 않는다. Step 12 기본 실행 데이터는 `data/rust-task-db`에 저장된다.

```bash
ls Cargo.toml
```

## 컴파일 확인

```bash
cargo check
```

의미:

```text
코드를 실행하지 않고 컴파일 가능한지 확인한다.
```

## 테스트 실행

```bash
cargo test
```

Step 18 현재도 테스트는 `src/task.rs`, `src/cli.rs`, `src/error.rs`, `src/service.rs`, `src/repl.rs`, `src/repository/mod.rs`, `src/repository/gluesql_repository.rs`, `src/main.rs` 안에 있다.

정상 결과 예시:

```text
running 65 tests
test result: ok. 65 passed
```

Step 12는 GlueSQL `SledStorage`를 사용한다. 이 저장소는 `data/rust-task-db`에 데이터를 저장하므로, 여러 CLI 명령을 나눠 실행해도 이전 데이터가 유지된다.

Step 17은 CLI 명령을 추가하지 않는다. 대신 Todo 명령별 SQL 생성과 GlueSQL `Payload` 변환 흐름을 문서로 분석한다. `cargo test` 안에서는 Step 15에서 보강한 `MemoryStorage` transaction 미지원, `SledStorage` rollback/commit/snapshot/write lock/nested transaction, `JsonTaskRepository` SQL 미지원을 계속 확인한다.

## CLI 실행: add

```bash
cargo run -- add "Rust 공부"
```

예상 출력:

```text
Added:
1 | Rust 공부 | false
```

코드 흐름:

```text
src/main.rs main()
-> src/cli.rs parse_args()
-> Command::Add { title }
-> GlueSqlTaskRepository::persistent("data/rust-task-db")
-> CREATE TABLE tasks (...)
-> TaskService::new(repository)
-> service.add(title)
-> repository 내부에서 SELECT로 다음 id 계산
-> repository 내부에서 INSERT 실행
-> print_task(&task)
```

## Step 5와 달라진 점

Step 5:

```text
실패가 생기면 Err(String)으로 전달
```

Step 6:

```text
실패가 생기면 Err(AppError)로 전달
```

구체적으로 달라진 흐름:

```text
잘못된 명령
-> AppError::InvalidCommand

없는 Todo id
-> AppError::NotFound(id)

파일 읽기/쓰기 실패
-> AppError::Io(error)

JSON parsing 실패
-> AppError::Json(error)
```

사용자가 치는 명령은 그대로다. 달라진 것은 코드 내부에서 실패를 분류하는 방식이다.

## Step 7과 달라진 점

Step 7:

```text
JsonTaskRepository가 tasks.json을 읽고 썼다.
```

Step 8:

```text
GlueSqlTaskRepository가 GlueSQL SledStorage에 SQL을 실행한다.
```

구체적으로 달라진 흐름:

```text
add
-> INSERT INTO tasks VALUES (...)

list/search
-> SELECT id, title, done FROM tasks ...

stats
-> SELECT COUNT(*) FROM tasks
```

## CLI 실행: list

```bash
cargo run -- list
```

Step 12의 `SledStorage`는 실행이 끝나도 데이터가 남는다. 그래서 `cargo run -- add "Rust 공부"`를 실행한 뒤 별도 명령으로 `cargo run -- list`를 실행해도 저장된 Todo가 보인다.

```text
List:
1 | Rust 공부 | false
```

프로세스를 나눠도 `add -> list`가 이어지는지는 `src/repository/gluesql_repository.rs`의 `persists_tasks_with_sled_storage` 테스트가 검증한다.

## Step 4에서 Step 5로 달라진 점

Step 4:

```text
main.rs
-> JsonTaskRepository::new("tasks.json")
-> repository.add / find_all / mark_done / delete
```

Step 5:

```text
main.rs
-> JsonTaskRepository::new("tasks.json")
-> TaskService::new(repository)
-> service.add / list / done / delete
-> service가 repository에 위임
-> repository가 tasks.json을 읽고 씀
```

사용자가 실행하는 명령은 같지만, 코드 책임이 달라졌다. `main.rs`는 repository 메서드를 직접 호출하지 않고 `TaskService`를 호출한다.

## Step 8에서 달라진 점

```text
main.rs
-> GlueSqlTaskRepository::persistent("data/rust-task-db")
-> TaskService::new(repository)
-> service.add / list / done / delete / search / stats
-> repository 내부에서 SQL 실행
```

## CLI 실행: done

```bash
cargo run -- done 1
```

예상 출력:

```text
Done:
1
```

주의: Step 12에서는 `done`도 `data/rust-task-db`에 남은 Todo를 대상으로 실행된다.

## CLI 실행: delete

```bash
cargo run -- delete 1
```

예상 출력:

```text
Deleted:
1 | Rust 공부 | true
```

삭제도 같은 이유로 repository 테스트에서 주요 흐름을 확인한다.

## CLI 실행: search

```bash
cargo run -- search rust
```

예상 출력 예시:

```text
Search:
```

## CLI 실행: stats

```bash
cargo run -- stats
```

예상 출력 예시:

```text
Stats:
total: 0
done: 0
todo: 0
```

## CLI 실행: sql

```bash
cargo run -- sql "SELECT * FROM tasks"
```

예상 출력 예시:

```text
SQL:
id | title | done
```

데이터를 넣고 바로 조회하려면 한 SQL 문자열 안에 여러 statement를 함께 넣는다.

```bash
cargo run -- sql "INSERT INTO tasks VALUES (1, 'Rust 공부', FALSE); SELECT id, title, done FROM tasks;"
```

예상 출력 예시:

```text
SQL:
insert: 1
id | title | done
1 | Rust 공부 | false
```

## CLI 실행: repl

```bash
cargo run -- repl
```

예상 시작 출력:

```text
rust-task SQL REPL
Type .schema, .exit, or .quit
rust-task>
```

REPL 안에서 입력:

```text
rust-task> INSERT INTO tasks VALUES (1, 'Rust 공부', FALSE);
insert: 1
rust-task> SELECT id, title, done FROM tasks;
id | title | done
1 | Rust 공부 | false
rust-task> .schema
CREATE TABLE tasks (
  id INTEGER,
  title TEXT,
  done BOOLEAN
);
rust-task> .exit
```

REPL 안에서는 같은 `GlueSqlTaskRepository` 인스턴스를 계속 쓰기 때문에 `INSERT` 뒤에 `SELECT`가 이어진다.

## tasks.json 직접 보기

```bash
cat tasks.json
```

Step 12 기본 실행 경로는 `tasks.json`을 사용하지 않는다. 이 파일은 `JsonTaskRepository` 구현을 보존하기 위해 남아 있다.

기존 JSON 저장소를 설명할 때의 예시는 아래와 같다.

```json
[
  {
    "id": 1,
    "title": "Rust 공부",
    "done": false
  }
]
```

보존된 JSON 저장소에서 Todo를 모두 삭제하면 `tasks.json`은 빈 배열이 된다.

```json
[]
```

## 도움말 보기

아무 명령도 주지 않으면 help가 출력된다.

```bash
cargo run
```

예상 출력:

```text
rust-task Step 12: Persistent GlueSQL Todo CLI

Usage:
  rust-task add "Rust 공부"
  rust-task list
  rust-task done 1
  rust-task delete 1
  rust-task search rust
  rust-task stats
  rust-task sql "SELECT * FROM tasks"
  rust-task repl

Note: Step 12 stores GlueSQL data under data/rust-task-db.
```

## 자주 보는 문제

| 증상 | 원인 | 해결 |
| --- | --- | --- |
| `could not find Cargo.toml` | 프로젝트 루트가 아닌 곳에서 실행 | `/Users/jujaewan/1_Projects/rust-task-db`로 이동 |
| `list`가 예상과 다름 | Step 12의 `SledStorage`는 이전 실행 데이터를 유지함 | 깨끗하게 확인하려면 `data/rust-task-db`를 지운 뒤 다시 실행 |
| `Task not found: 1` | `data/rust-task-db`에 id 1 Todo가 없음 | 먼저 Todo를 추가하거나 저장 디렉터리 상태를 확인 |
| `GlueSQL error: ...` | GlueSQL SQL 실행 또는 row 변환 실패 | [src/repository/gluesql_repository.rs](../../src/repository/gluesql_repository.rs)의 SQL 문자열 확인 |
| `id must be an integer` | `parse_id`가 숫자 변환에 실패 | 숫자 id를 입력 |

## 다음에 읽을 문서

실행을 확인했다면 [03-runtime-flow.md](03-runtime-flow.md)에서 service, repository, AppError 흐름을 읽는다.
