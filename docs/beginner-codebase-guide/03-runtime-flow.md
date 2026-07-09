# 프로젝트 실행 흐름

## 실행 명령어

```bash
cargo run -- add "Rust 공부"
cargo run -- list
cargo run -- done 1
cargo run -- delete 1
cargo run -- search rust
cargo run -- stats
cargo run -- sql "SELECT * FROM tasks"
cargo run -- repl
```

## 실행 진입점

진입점은 `src/main.rs`의 `fn main()`이다.

```rust
fn main() {
    let command = match cli::parse_args(std::env::args().collect()) {
        Ok(command) => command,
        Err(message) => {
            eprintln!("{message}");
            print_help();
            return;
        }
    };
}
```

## 프로그램이 시작될 때 일어나는 일

1. 새 프로세스가 시작된다.
2. `main()`이 호출된다.
3. CLI 인자가 `Vec<String>`으로 모인다.
4. `cli::parse_args(...)`가 인자를 `Command`로 변환한다.
5. help 명령이면 파일을 읽지 않고 help를 출력한다.
6. 그 외 명령이면 `GlueSqlTaskRepository::persistent("data/rust-task-db")`로 repository를 만든다.
7. repository가 내부에서 GlueSQL `SledStorage`를 열고 `CREATE TABLE IF NOT EXISTS tasks (...)`를 실행한다.
8. `TaskService::new(repository)`로 service를 만든다.
9. `main()`이 `match command`로 service 메서드를 호출한다.
10. service가 repository 메서드에 위임한다.
11. repository가 `execute(sql)`을 통해 GlueSQL SQL을 실행한다.
12. Todo 명령이면 `Payload`와 `Value`를 `Task` 또는 `TaskStats`로 바꾼 뒤 결과를 출력한다.
13. `sql` 명령이면 `Payload`를 `SqlResult`로 바꾼 뒤 SQL 결과를 출력한다.
14. `repl` 명령이면 `src/repl.rs`의 입력 루프 안에서 여러 SQL을 반복 실행한다.

## Step 9에서 Step 10으로 달라진 점

| 구분 | Step 9 | Step 10 |
| --- | --- | --- |
| SQL 입력 방식 | CLI 인자로 SQL 문자열 1개 전달 | REPL에서 여러 줄 입력 |
| 명령 enum | `Command::Sql { sql }` | `Command::Repl` 추가 |
| 새 파일 | 없음 | `src/repl.rs` |
| 종료 방식 | SQL 실행 후 프로그램 종료 | `.exit` 또는 `.quit` 입력 |
| schema 확인 | 문서나 코드 확인 | `.schema` 입력 |

초심자용으로 줄이면 아래와 같다.

```text
Step 9:
cargo run -- sql "SELECT * FROM tasks" 한 번 실행

Step 10:
cargo run -- repl 실행 후 rust-task> 프롬프트에서 여러 SQL 실행
```

## Step 8에서 Step 9로 달라진 점

| 구분 | Step 8 | Step 9 |
| --- | --- | --- |
| 직접 SQL 입력 | 코드에서 확인되지 않음 | `cargo run -- sql "SELECT * FROM tasks"` |
| 명령 enum | `Command::Sql` 없음 | `Command::Sql { sql }` |
| service 메서드 | Todo 전용 메서드 | `TaskService::execute_sql` 추가 |
| repository trait | Todo 전용 메서드 | `TaskRepository::execute_sql` 추가 |
| SQL 결과 표현 | 내부 `Payload`를 `Task`, `TaskStats`로 변환 | `Payload`를 `SqlResult`로 변환 |

초심자용으로 줄이면 아래와 같다.

```text
Step 8:
코드가 정해둔 SQL만 repository 내부에서 실행했다.

Step 9:
사용자가 입력한 SQL 문자열도 GlueSQL에 직접 전달할 수 있다.
```

주의: Step 9도 GlueSQL `MemoryStorage`를 사용한다. 프로그램이 끝나면 데이터가 사라지므로, SQL에서 입력 후 조회를 바로 확인하려면 한 SQL 문자열 안에 `INSERT ...; SELECT ...;`를 함께 넣는다.

## Step 7에서 Step 8으로 달라진 점

| 구분 | Step 7 | Step 8 |
| --- | --- | --- |
| 활성 repository | `JsonTaskRepository` | `GlueSqlTaskRepository` |
| 저장 위치 | `tasks.json` 파일 | GlueSQL `SledStorage`의 `data/rust-task-db` |
| repository 생성 | JSON 파일 읽기 | `CREATE TABLE tasks (...)` 실행 |
| 데이터 변경 | `Vec<Task>` 수정 후 JSON 저장 | SQL `INSERT`, `UPDATE`, `DELETE` 실행 |
| 데이터 조회 | `Vec<Task>` clone/filter/count | SQL `SELECT`, `COUNT` 실행 |
| 기존 JSON 코드 | 기본 실행 경로 | 삭제하지 않고 보존 |

초심자용으로 줄이면 아래와 같다.

```text
Step 7:
JsonTaskRepository가 tasks.json을 읽고 썼다.

Step 8:
GlueSqlTaskRepository가 GlueSQL SledStorage에 SQL을 실행한다.
```

주의: Step 12의 `SledStorage`는 `data/rust-task-db`에 데이터를 저장한다. 그래서 `cargo run -- add`와 `cargo run -- list`를 별도 실행해도 데이터가 이어진다.

## Step 6에서 Step 7으로 달라진 점

| 구분 | Step 6 | Step 7 |
| --- | --- | --- |
| 지원 명령 | `add`, `list`, `done`, `delete` | `search`, `stats` 추가 |
| 데이터 변경 | add/done/delete 중심 | search/stats는 데이터 변경 없음 |
| 핵심 문법 | custom error, `From`, `Display` | iterator, `filter`, `count`, closure |
| 새 모델 | 없음 | `TaskStats` |
| 출력 | Task 목록 중심 | 검색 결과와 통계 출력 |

초심자용으로 줄이면 아래와 같다.

```text
Step 6:
실패를 AppError로 정리했다.

Step 7:
Todo를 찾고 개수를 세는 읽기 기능이 추가됐다.
```

## Step 5에서 Step 6으로 달라진 점

실행 성공 흐름은 거의 같다. 달라진 부분은 실패가 지나가는 길이다.

| 상황 | Step 5 | Step 6 |
| --- | --- | --- |
| CLI 명령이 잘못됨 | `Err(String)` | `Err(AppError::InvalidCommand(message))` |
| 없는 id | `Err(String)` | `Err(AppError::NotFound(id))` |
| 파일 읽기 실패 | `Err(String)` | `Err(AppError::Io(error))` |
| JSON parsing 실패 | `Err(String)` | `Err(AppError::Json(error))` |
| 화면 출력 | 문자열을 그대로 출력 | `AppError`의 `Display` 결과를 출력 |

초심자용으로 줄이면 아래와 같다.

```text
Step 5:
실패 메시지를 그냥 String으로 들고 다님

Step 6:
실패 종류를 AppError enum으로 나눠 들고 다님
```

## Repository 생성 흐름

```rust
let repository = match GlueSqlTaskRepository::persistent("data/rust-task-db") {
    Ok(repository) => repository,
    Err(message) => {
        eprintln!("{message}");
        return;
    }
};
let mut service = TaskService::new(repository);
```

코드 해석:

- `GlueSqlTaskRepository::persistent("data/rust-task-db")`: GlueSQL `SledStorage`를 열고 `tasks` table을 준비한다.
- `Ok(repository)`: 저장소 생성 성공
- `Err(message)`: GlueSQL table 생성 실패
- `let repository`: 생성 직후에는 repository 변수를 직접 바꾸지 않는다.
- `TaskService::new(repository)`: repository 소유권을 service로 넘긴다.
- `let mut service`: Step 9에서는 add/done/delete/list/search/stats/sql 모두 GlueSQL 실행을 위해 내부 repository를 mutable로 빌리므로 `mut`가 필요하다.

## Service 호출 흐름

```rust
Command::Add { title } => match service.add(title) {
    Ok(task) => {
        println!("Added:");
        print_task(&task);
    }
    Err(message) => eprintln!("{message}"),
},
```

코드 해석:

- `service.add(title)`: Todo 추가 요청을 service에 보낸다.
- `Ok(task)`: 추가된 Task를 받는다.
- `Err(message)`: service나 repository에서 올라온 실패 메시지를 출력한다.

`src/service.rs`의 실제 구현은 아래처럼 repository에 위임한다.

```rust
pub fn add(&mut self, title: String) -> Result<Task, AppError> {
    self.repository.add(title)
}
```

## GlueSQL 실행 흐름은 어디에 있나?

Step 18 현재 GlueSQL 세부사항과 SQL 결과 변환은 `src/repository/gluesql_repository.rs` 안에 있다. 명령별 SQL과 `Payload` 변환 상세는 [19-query-execution.md](19-query-execution.md)에서 다루고, storage별 기능 차이는 [20-storage-comparison.md](20-storage-comparison.md)에서 다룬다.

```text
src/main.rs
-> GlueSqlTaskRepository::persistent("data/rust-task-db")
-> src/repository/gluesql_repository.rs
-> Glue::new(SledStorage::new(...))
-> CREATE TABLE tasks (...)
-> TaskService::new
-> service.add/list/done/delete
-> service.search/stats
-> service.execute_sql
-> repl::run_repl
-> src/service.rs TaskService
```

`main.rs`는 이제 GlueSQL API 세부사항을 모른다.

## SQL 명령 실행 흐름

```text
cargo run -- sql "SELECT * FROM tasks"
-> src/cli.rs parse_args
-> Command::Sql { sql }
-> src/main.rs match command
-> TaskService::execute_sql(sql)
-> TaskRepository::execute_sql(sql)
-> GlueSqlTaskRepository::execute_sql(sql)
-> GlueSQL execute
-> Payload를 SqlResult로 변환
-> print_sql_results
```

핵심은 `main.rs`가 SQL 엔진을 직접 만지지 않는다는 점이다. `main.rs`는 `SqlResult`만 받아서 출력하고, 실제 GlueSQL 실행은 repository가 담당한다.

## REPL 명령 실행 흐름

```text
cargo run -- repl
-> src/cli.rs parse_args
-> Command::Repl
-> src/main.rs match command
-> repl::run_repl(&mut service)
-> rust-task> 프롬프트 출력
-> 한 줄 입력 읽기
-> .schema면 schema 출력
-> .exit 또는 .quit이면 종료
-> SQL이면 service.execute_sql
-> 결과 출력 후 다시 프롬프트
```

REPL은 같은 `TaskService<GlueSqlTaskRepository>`를 계속 빌려 쓴다. 그래서 REPL 안에서는 `INSERT` 다음 `SELECT`가 이어진다.

## REPL 코드 한 줄씩 읽기

REPL은 `src/repl.rs`의 `run_repl`에서 시작한다.

```rust
pub fn run_repl<R: TaskRepository>(service: &mut TaskService<R>) -> Result<(), AppError> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    run_repl_with_io(service, stdin.lock(), &mut stdout)
}
```

코드 해석:

- `R: TaskRepository`: REPL이 특정 저장소 구현체에 고정되지 않고 repository trait에 의존한다.
- `service: &mut TaskService<R>`: REPL 안에서 SQL을 여러 번 실행해야 하므로 service를 수정 가능하게 빌린다.
- `io::stdin()`: 터미널 입력을 준비한다.
- `io::stdout()`: 터미널 출력을 준비한다.
- `stdin.lock()`: 표준 입력을 잠가서 줄 단위로 안정적으로 읽는다.
- `run_repl_with_io(...)`: 실제 반복 입력 처리를 맡긴다.

입력 루프는 아래 코드가 담당한다.

```rust
loop {
    write!(output, "{PROMPT}")?;
    output.flush()?;

    let mut line = String::new();
    let bytes_read = input.read_line(&mut line)?;
    if bytes_read == 0 {
        break;
    }

    let command = line.trim();
    if command.is_empty() {
        continue;
    }
}
```

코드 해석:

- `loop`: `.exit`, `.quit`, EOF가 나오기 전까지 계속 반복한다.
- `write!(output, "{PROMPT}")`: `rust-task> ` 프롬프트를 출력한다.
- `flush()`: 줄바꿈이 없어도 프롬프트가 즉시 보이게 한다.
- `String::new()`: 입력 한 줄을 담을 빈 문자열을 만든다.
- `read_line(&mut line)`: 사용자가 입력한 한 줄을 `line`에 넣는다.
- `bytes_read == 0`: 더 읽을 입력이 없으면 REPL을 종료한다.
- `trim()`: 마지막의 엔터(`\n`)와 앞뒤 공백을 제거한다.
- `continue`: 빈 줄이면 아무 일도 하지 않고 다음 프롬프트로 넘어간다.

입력값 분기는 아래 코드가 담당한다.

```rust
match command {
    ".exit" | ".quit" => break,
    ".schema" => writeln!(output, "{SCHEMA}")?,
    sql => match service.execute_sql(sql.to_string()) {
        Ok(results) => write_sql_results(output, &results)?,
        Err(message) => writeln!(output, "{message}")?,
    },
}
```

코드 해석:

- `.exit` 또는 `.quit`: `break`로 REPL loop를 끝낸다.
- `.schema`: `tasks` table의 schema 문자열을 출력한다.
- 그 외 입력: SQL 문자열로 보고 `service.execute_sql`에 넘긴다.
- `Ok(results)`: SQL 실행 성공 결과를 `write_sql_results`가 출력한다.
- `Err(message)`: SQL 실행 실패를 REPL 화면에 출력하고 다음 입력을 기다린다.

REPL 안에서 데이터가 이어지는 이유는 `main.rs`가 service를 한 번 만들고, 그 같은 service를 REPL loop 전체에 넘기기 때문이다.

```text
GlueSqlTaskRepository::persistent("data/rust-task-db")
-> TaskService::new(repository)
-> repl::run_repl(&mut service)
-> service.execute_sql 반복 호출
```

## GlueSQL repository 안의 SQL 실행 흐름

```rust
fn execute(&mut self, sql: impl AsRef<str>) -> Result<Vec<Payload>, AppError> {
    block_on(self.glue.execute(sql)).map_err(|error| AppError::GlueSql(error.to_string()))
}
```

코드 해석:

- `&mut self`: GlueSQL의 `execute`가 내부 storage를 사용하므로 mutable borrow가 필요하다.
- `sql: impl AsRef<str>`: `&str`과 `String`을 모두 받을 수 있게 한다.
- `block_on(...)`: GlueSQL의 async 실행을 현재 동기 CLI 흐름 안에서 끝까지 기다린다.
- `self.glue.execute(sql)`: SQL 문자열을 GlueSQL 엔진에 전달한다.
- `map_err(...)`: GlueSQL 에러를 `AppError::GlueSql(String)`으로 바꾼다.

## repository 안의 파일 읽기 흐름

이 흐름은 현재 기본 실행 경로가 아니라, 보존된 `JsonTaskRepository`의 흐름이다.

```rust
fn load_tasks(path: impl AsRef<Path>) -> Result<Vec<Task>, AppError> {
    let path = path.as_ref();

    match fs::read_to_string(path) {
        Ok(contents) => serde_json::from_str(&contents).map_err(AppError::from),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(Vec::new()),
        Err(error) => Err(AppError::from(error)),
    }
}
```

코드 해석:

- `path: impl AsRef<Path>`: `"tasks.json"` 같은 문자열도 받고, 테스트에서 만든 `PathBuf`도 받을 수 있게 한다.
- `Result<Vec<Task>, AppError>`: 성공하면 Todo 목록, 실패하면 `AppError`를 반환한다.
- `let path = path.as_ref();`: 들어온 경로 값을 `Path`처럼 다룰 수 있는 참조로 바꾼다.
- `fs::read_to_string(path)`: 파일 내용을 문자열로 읽는다.
- `Ok(contents)`: 파일 읽기에 성공한 경우다. 이때 `contents`는 JSON 문자열이다.
- `serde_json::from_str(&contents)`: JSON 문자열을 `Vec<Task>`로 바꾼다.
- `map_err(AppError::from)`: serde_json 에러를 `AppError::Json`으로 바꾼다.
- `Err(error) if error.kind() == ErrorKind::NotFound`: 파일이 없는 경우다. 처음 실행일 수 있으므로 빈 Vec를 반환한다.
- `Err(error)`: 파일은 있는데 권한 문제 등으로 읽지 못한 경우다. 이 경우는 실패로 처리한다.

### `load_tasks`를 한 줄씩 읽기

함수 시작:

```rust
fn load_tasks(path: impl AsRef<Path>) -> Result<Vec<Task>, AppError>
```

읽는 법:

```text
경로를 하나 받는다.
성공하면 Vec<Task>를 돌려준다.
실패하면 AppError를 돌려준다.
```

경로 변환:

```rust
let path = path.as_ref();
```

읽는 법:

```text
path를 Path처럼 다룰 수 있는 참조로 바꾼다.
이후 path.display() 같은 메서드를 쓸 수 있다.
```

파일 읽기:

```rust
match fs::read_to_string(path) {
```

읽는 법:

```text
파일을 문자열로 읽어본다.
성공과 실패를 match로 나눠 처리한다.
```

파일 읽기 성공:

```rust
Ok(contents) => serde_json::from_str(&contents)
    .map_err(|error| format!("Failed to parse {}: {error}", path.display())),
```

읽는 법:

```text
파일 읽기에 성공하면 contents에 파일 내용이 들어온다.
그 문자열을 serde_json::from_str로 Vec<Task>로 바꾼다.
JSON 문법이 잘못되면 그 에러를 `AppError::Json`으로 바꾼다.
```

파일 없음:

```rust
Err(error) if error.kind() == ErrorKind::NotFound => Ok(Vec::new()),
```

읽는 법:

```text
tasks.json이 없으면 실패로 끝내지 않는다.
처음 실행한 상황으로 보고 빈 Vec<Task>를 반환한다.
```

그 외 파일 읽기 실패:

```rust
Err(error) => Err(AppError::from(error)),
```

읽는 법:

```text
파일이 없는 경우가 아닌 다른 읽기 실패는 `AppError::Io`로 반환한다.
```

### `load_tasks`의 세 가지 결과

| 상황 | 예시 | 반환 |
| --- | --- | --- |
| 파일이 있고 JSON도 정상 | `tasks.json`에 `[]` 또는 Task 배열 있음 | `Ok(Vec<Task>)` |
| 파일이 없음 | 처음 실행해서 `tasks.json`이 없음 | `Ok(Vec::new())` |
| 파일이 있지만 JSON이 깨짐 | `{ invalid json` | `Err(AppError::Json(...))` |

## repository 안의 파일 저장 흐름

```rust
fn save_tasks(path: impl AsRef<Path>, tasks: &[Task]) -> Result<(), AppError> {
    let path = path.as_ref();
    let contents = serde_json::to_string_pretty(tasks)?;

    fs::write(path, contents)?;

    Ok(())
}
```

코드 해석:

- `serde_json::to_string_pretty(tasks)`: `Vec<Task>`를 보기 좋은 JSON 문자열로 바꾼다.
- `?`: JSON 변환이나 파일 쓰기에 실패하면 `AppError`로 바뀌어 바로 반환된다.
- `fs::write(path, contents)`: JSON 문자열을 파일에 쓴다.

## 명령 실행 흐름

```text
Command::Add
-> service.add
-> repository.add
-> GlueSqlTaskRepository가 INSERT SQL 실행

Command::List
-> service.list
-> repository.find_all
-> print_tasks

Command::Done
-> service.done
-> repository.mark_done
-> GlueSqlTaskRepository가 UPDATE SQL 실행

Command::Delete
-> service.delete
-> repository.delete
-> GlueSqlTaskRepository가 DELETE SQL 실행

Command::Search
-> service.search
-> repository.search
-> print_tasks

Command::Stats
-> service.stats
-> repository.stats
-> print_stats

Command::Sql
-> service.execute_sql
-> repository.execute_sql
-> GlueSqlTaskRepository가 사용자 SQL 실행
-> print_sql_results

Command::Repl
-> repl::run_repl
-> service.execute_sql 반복 호출
-> REPL output에 SQL 결과 출력
```

`list`는 데이터를 바꾸지 않으므로 파일을 다시 저장하지 않는다.
`search`와 `stats`도 데이터를 바꾸지 않는다. Step 9에서는 파일 저장 대신 GlueSQL `SELECT`와 `COUNT`를 실행한다.

## 초심자용 실행 흐름 요약

```text
프로그램 시작
-> CLI 인자 읽기
-> parse_args로 Command 만들기
-> GlueSqlTaskRepository 생성
-> TaskService 생성
-> service 메서드 호출
-> service가 repository 메서드 호출
-> repository가 SQL 실행
-> Payload/Value를 Task, TaskStats, SqlResult로 변환
-> repl이면 같은 실행 안에서 다음 입력 대기
-> 프로그램 종료
```
