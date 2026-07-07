# 대표 기능별 실행 흐름

## 공통 흐름

모든 명령은 먼저 같은 parser를 지난다.

```text
cargo run -- ...
-> src/main.rs main()
-> std::env::args().collect()
-> src/cli.rs parse_args(args)
-> src/command.rs Command
```

help가 아닌 명령은 그 다음 GlueSQL repository를 만든다.

```text
Command
-> GlueSqlTaskRepository::new()
-> GlueSQL MemoryStorage
-> CREATE TABLE tasks (...)
-> TaskService::new(repository)
```

## Step 7에서 Step 8으로 달라진 점

| 구분 | Step 7 | Step 8 |
| --- | --- | --- |
| 명령 | add/list/done/delete/search/stats | 명령은 그대로 유지 |
| 활성 저장소 | `JsonTaskRepository` | `GlueSqlTaskRepository` |
| 데이터 보관 | `tasks.json` | GlueSQL `MemoryStorage` |
| repository 내부 로직 | `Vec<Task>` 수정 | SQL `INSERT`, `SELECT`, `UPDATE`, `DELETE`, `COUNT` |
| 주의점 | 실행 사이 데이터 유지 | `MemoryStorage`라 실행이 끝나면 데이터 사라짐 |

## Step 8에서 Step 9로 달라진 점

| 구분 | Step 8 | Step 9 |
| --- | --- | --- |
| 사용자가 입력하는 명령 | Todo 전용 명령 | Todo 전용 명령 + `sql` |
| SQL 실행 위치 | repository 내부 구현 | repository 내부 구현 + 사용자 SQL 직접 실행 |
| 결과 타입 | `Task`, `TaskStats` | `Task`, `TaskStats`, `SqlResult` |
| 대표 흐름 | `service.stats -> repository.stats` | `service.execute_sql -> repository.execute_sql` |

## Step 9에서 Step 10으로 달라진 점

| 구분 | Step 9 | Step 10 |
| --- | --- | --- |
| 명령 | `sql` | `repl` 추가 |
| 입력 | CLI 인자 | 표준 입력 line |
| 반복 실행 | 코드에서 확인되지 않음 | `.exit` 또는 `.quit` 전까지 반복 |
| schema 출력 | 코드에서 확인되지 않음 | `.schema` |

## 기능명: add

### 실행 명령

```bash
cargo run -- add "Rust 공부"
```

### 전체 실행 흐름

1. `parse_args`가 `Ok(Command::Add { title })`를 반환한다.
2. `GlueSqlTaskRepository::new`가 GlueSQL `MemoryStorage`와 `tasks` table을 만든다.
3. `TaskService::new(repository)`가 service를 만든다.
4. `service.add(title)`가 Todo 추가 요청을 repository에 위임한다.
5. 추가된 Task를 출력한다.

### 핵심 코드 블록

```rust
Command::Add { title } => {
    match service.add(title) {
        Ok(task) => {
            println!("Added:");
            print_task(&task);
        }
        Err(message) => eprintln!("{message}"),
    }
}
```

### 데이터 변화

```text
TaskService
-> GlueSqlTaskRepository::add
-> SELECT id, title, done FROM tasks ORDER BY id
-> INSERT INTO tasks VALUES (...)
-> 추가된 Task 출력
```

## 기능명: list

### 실행 명령

```bash
cargo run -- list
```

### 전체 실행 흐름

1. `parse_args`가 `Ok(Command::List)`를 반환한다.
2. `GlueSqlTaskRepository::new`가 GlueSQL `MemoryStorage`와 `tasks` table을 만든다.
3. `TaskService::new(repository)`가 service를 만든다.
4. `service.list()`가 목록 조회를 repository에 위임한다.
5. `print_tasks(&tasks)`가 목록을 출력한다.

### 핵심 코드 블록

```rust
Command::List => match service.list() {
    Ok(tasks) => {
        println!("List:");
        print_tasks(&tasks);
    }
    Err(message) => eprintln!("{message}"),
},
```

`list`는 데이터를 바꾸지 않지만, Step 9에서는 GlueSQL `SELECT id, title, done FROM tasks ORDER BY id`를 실행한다.

## 기능명: done

### 실행 명령

```bash
cargo run -- done 1
```

### 전체 실행 흐름

1. `parse_args`가 `Ok(Command::Done { id })`를 반환한다.
2. `GlueSqlTaskRepository::new`가 GlueSQL `MemoryStorage`와 `tasks` table을 만든다.
3. `TaskService::new(repository)`가 service를 만든다.
4. `service.done(id)`가 완료 처리를 repository에 위임한다.

### 핵심 코드 블록

```rust
Command::Done { id } => match service.done(id) {
    Ok(()) => {
        println!("Done:\n{id}");
    }
    Err(message) => eprintln!("{message}"),
},
```

없는 id면 `SELECT ... WHERE id = ...` 결과가 비어 있고 `AppError::NotFound(id)`를 반환한다.

## 기능명: delete

### 실행 명령

```bash
cargo run -- delete 1
```

### 전체 실행 흐름

1. `parse_args`가 `Ok(Command::Delete { id })`를 반환한다.
2. `GlueSqlTaskRepository::new`가 GlueSQL `MemoryStorage`와 `tasks` table을 만든다.
3. `TaskService::new(repository)`가 service를 만든다.
4. `service.delete(id)`가 삭제 처리를 repository에 위임한다.

### 핵심 코드 블록

```rust
Command::Delete { id } => match service.delete(id) {
    Ok(task) => {
        println!("Deleted:");
        print_task(&task);
    }
    Err(message) => eprintln!("{message}"),
},
```

## 기능명: search

### 실행 명령

```bash
cargo run -- search rust
```

### 전체 실행 흐름

1. `parse_args`가 `Ok(Command::Search { keyword })`를 반환한다.
2. `GlueSqlTaskRepository::new`가 GlueSQL `MemoryStorage`와 `tasks` table을 만든다.
3. `TaskService::new(repository)`가 service를 만든다.
4. `service.search(&keyword)`가 검색을 repository에 위임한다.
5. `print_tasks(&tasks)`가 검색 결과를 출력한다.

### 핵심 코드 블록

```rust
Command::Search { keyword } => match service.search(&keyword) {
    Ok(tasks) => {
        println!("Search:");
        print_tasks(&tasks);
    }
    Err(message) => eprintln!("{message}"),
},
```

검색은 GlueSQL의 `ILIKE`를 사용하므로 대소문자를 구분하지 않는다.

## 기능명: stats

### 실행 명령

```bash
cargo run -- stats
```

### 전체 실행 흐름

1. `parse_args`가 `Ok(Command::Stats)`를 반환한다.
2. `GlueSqlTaskRepository::new`가 GlueSQL `MemoryStorage`와 `tasks` table을 만든다.
3. `TaskService::new(repository)`가 service를 만든다.
4. `service.stats()`가 통계 계산을 repository에 위임한다.
5. `print_stats(&stats)`가 total/done/todo를 출력한다.

### 핵심 코드 블록

```rust
Command::Stats => match service.stats() {
    Ok(stats) => {
        println!("Stats:");
        print_stats(&stats);
    }
    Err(message) => eprintln!("{message}"),
},
```

`stats`는 데이터를 바꾸지 않고 `SELECT COUNT(*) FROM tasks`와 `SELECT COUNT(*) FROM tasks WHERE done = TRUE`를 실행한다.

## 기능명: sql

### 실행 명령

```bash
cargo run -- sql "SELECT * FROM tasks"
```

데이터를 넣고 바로 조회하려면 한 SQL 문자열 안에 여러 statement를 함께 넣는다.

```bash
cargo run -- sql "INSERT INTO tasks VALUES (1, 'Rust 공부', FALSE); SELECT * FROM tasks;"
```

### 전체 실행 흐름

1. `parse_args`가 `Ok(Command::Sql { sql })`를 반환한다.
2. `GlueSqlTaskRepository::new`가 GlueSQL `MemoryStorage`와 `tasks` table을 만든다.
3. `TaskService::new(repository)`가 service를 만든다.
4. `service.execute_sql(sql)`이 SQL 실행을 repository에 위임한다.
5. `GlueSqlTaskRepository::execute_sql`이 GlueSQL `Payload`를 `SqlResult`로 바꾼다.
6. `print_sql_results(&results)`가 SELECT 결과 또는 변경 건수를 출력한다.

### 핵심 코드 블록

```rust
Command::Sql { sql } => match service.execute_sql(sql) {
    Ok(results) => {
        println!("SQL:");
        print_sql_results(&results);
    }
    Err(message) => eprintln!("{message}"),
},
```

### 데이터 변화

```text
TaskService
-> GlueSqlTaskRepository::execute_sql
-> GlueSQL execute
-> Payload::Select 또는 Payload::Insert/Update/Delete
-> SqlResult::Select 또는 SqlResult::Affected
-> print_sql_results
```

주의: 이 기능은 REPL이 아니다. 명령 한 번에 SQL 문자열 하나를 실행하고 프로그램이 종료된다.

## 기능명: repl

### 실행 명령

```bash
cargo run -- repl
```

### REPL 안에서 입력

```text
rust-task> INSERT INTO tasks VALUES (1, 'Rust 공부', FALSE);
rust-task> SELECT id, title, done FROM tasks;
rust-task> .schema
rust-task> .exit
```

### 전체 실행 흐름

1. `parse_args`가 `Ok(Command::Repl)`을 반환한다.
2. `GlueSqlTaskRepository::new`가 GlueSQL `MemoryStorage`와 `tasks` table을 만든다.
3. `TaskService::new(repository)`가 service를 만든다.
4. `main.rs`가 `repl::run_repl(&mut service)`를 호출한다.
5. `src/repl.rs`가 한 줄씩 입력을 읽는다.
6. SQL이면 `service.execute_sql(sql)`을 호출한다.
7. `.schema`면 현재 `tasks` table schema를 출력한다.
8. `.exit` 또는 `.quit`이면 REPL을 끝낸다.

### 핵심 코드 블록

```rust
Command::Repl => {
    if let Err(message) = repl::run_repl(&mut service) {
        eprintln!("{message}");
    }
}
```

### 데이터 변화

```text
REPL 입력
-> repl::run_repl
-> service.execute_sql
-> GlueSqlTaskRepository::execute_sql
-> 같은 MemoryStorage에 SQL 실행
-> 결과 출력
-> 다음 입력 대기
```

## 기능명: help와 unknown command

### 실행 명령

```bash
cargo run
cargo run -- unknown
```

명령이 없으면 `Command::Help`가 되고, help는 GlueSQL repository를 만들지 않는다.

모르는 명령은 `parse_args`가 `Err(AppError::InvalidCommand(...))`를 반환하고 help를 출력한다.
