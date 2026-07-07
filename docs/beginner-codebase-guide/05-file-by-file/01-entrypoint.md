# 실행 진입점

## 포함된 파일 목록

- `src/main.rs`
- `src/repl.rs`

## 이 파일 묶음의 역할

프로그램 시작, CLI parser 호출, `GlueSqlTaskRepository` 생성, `TaskService` 생성, `Command` 실행, REPL 실행, 출력을 담당한다.

## 전체 연결 관계

```text
src/main.rs
-> mod cli
-> mod command
-> mod repository
-> mod service
-> mod task
-> cli::parse_args
-> GlueSqlTaskRepository::new
-> TaskService::new
-> match Command
-> service.add/list/done/delete/search/stats/execute_sql 또는 repl::run_repl
```

## 파일별 상세 설명

## 파일 경로

`src/main.rs`

### 이 파일의 역할

Step 10의 실행 흐름을 담는다. CLI parsing은 `src/cli.rs`로 분리했고, 명령 실행은 `src/service.rs`의 `TaskService`로 분리했다. 실패 타입은 `src/error.rs`의 `AppError`로 모았다. 현재 활성 저장소와 SQL 실행 세부사항은 `src/repository/gluesql_repository.rs`에 있고, REPL 입력 루프는 `src/repl.rs`에 있다.

### 이 파일이 필요한 이유

Rust CLI 프로그램은 `main()`에서 시작한다. 현재 단계에서는 `main.rs`가 repository를 직접 호출하지 않고 service를 호출한다.

### 이 파일과 연결된 다른 파일

- `src/cli.rs`: `parse_args` 제공
- `src/command.rs`: `Command` enum 제공
- `src/error.rs`: `AppError` 제공
- `src/service.rs`: `TaskService` 제공
- `src/repl.rs`: `run_repl` 제공
- `src/repository/mod.rs`: `TaskRepository`, 보존된 `JsonTaskRepository`, `GlueSqlTaskRepository` re-export 제공
- `src/repository/gluesql_repository.rs`: `GlueSqlTaskRepository` 제공
- `src/task.rs`: `Task` 타입 제공
- `tasks.json`: 보존된 JSON 저장 데이터. Step 11 기본 실행 경로에서는 사용하지 않는다.

### 핵심 코드 블록

```rust
let command = match cli::parse_args(std::env::args().collect()) {
    Ok(command) => command,
    Err(message) => {
        eprintln!("{message}");
        print_help();
        return;
    }
};
```

### 코드 블록별 해설

- `std::env::args()`: CLI 인자 iterator를 만든다.
- `collect()`: iterator를 `Vec<String>`으로 모은다.
- `cli::parse_args(...)`: 문자열 목록을 `Command`로 바꾼다.
- `Ok(command)`: parsing 성공
- `Err(message)`: parsing 실패
- `return`: 실패하면 기능 실행으로 넘어가지 않고 종료한다.

### repository와 service 생성 코드

```rust
let repository = match GlueSqlTaskRepository::new() {
    Ok(repository) => repository,
    Err(message) => {
        eprintln!("{message}");
        return;
    }
};
let mut service = TaskService::new(repository);
```

코드 해석:

- `GlueSqlTaskRepository::new`: GlueSQL `MemoryStorage`를 만들고 `tasks` table을 준비한다.
- `TaskService::new(repository)`: repository를 service 안으로 이동시킨다.
- `let mut service`: Step 11에서는 add/done/delete/list/search/stats/sql/repl 모두 GlueSQL 실행을 위해 service 내부 repository를 mutable하게 빌리므로 `mut`가 필요하다.

### 실행 분기 코드

```rust
match command {
    Command::Add { title } => match service.add(title) {
        Ok(task) => {
            println!("Added:");
            print_task(&task);
        }
        Err(message) => eprintln!("{message}"),
    },
    Command::List => match service.list() {
        Ok(tasks) => {
            println!("List:");
            print_tasks(&tasks);
        }
        Err(message) => eprintln!("{message}"),
    },
    Command::Done { id } => match service.done(id) {
        Ok(()) => println!("Done:\n{id}"),
        Err(message) => eprintln!("{message}"),
    },
    Command::Delete { id } => match service.delete(id) {
        Ok(task) => {
            println!("Deleted:");
            print_task(&task);
        }
        Err(message) => eprintln!("{message}"),
    },
    Command::Sql { sql } => match service.execute_sql(sql) {
        Ok(results) => {
            println!("SQL:");
            print_sql_results(&results);
        }
        Err(message) => eprintln!("{message}"),
    },
    Command::Repl => {
        if let Err(message) = repl::run_repl(&mut service) {
            eprintln!("{message}");
        }
    }
    Command::Help => unreachable!("help command returns before loading tasks"),
}
```

### 이 파일에서 사용된 언어 문법

함수, `let mut`, `match`, enum pattern, service method call, `Result`, borrowing, mutable borrowing, slice, `unreachable!`, 테스트 attribute

### 이 파일에서 사용된 프레임워크/라이브러리 기능

`src/main.rs`는 Rust 표준 라이브러리 `std::env`를 사용한다. GlueSQL API 사용은 `src/repository/gluesql_repository.rs`에 있고, 보존된 JSON 파일 I/O와 `serde_json` 사용은 `src/repository/mod.rs`에 있다.

### 초심자가 수정할 수 있는 부분

출력 메시지, help 문구, `print_tasks`의 빈 목록 처리

### 수정 전 코드

```rust
println!("List:");
print_tasks(&tasks);
```

### 수정 후 코드

```rust
println!("List:");
if tasks.is_empty() {
    println!("No tasks");
} else {
    print_tasks(&tasks);
}
```

### 수정 시 영향받는 파일

출력만 바꾸면 `src/main.rs`가 중심이다. 새 명령을 추가하면 `src/command.rs`, `src/cli.rs`, `src/service.rs`, `src/repository/mod.rs`, 테스트, 초심자 가이드를 함께 수정해야 한다.

### 이 파일을 이해한 뒤 알아야 하는 것

현재 Todo, SQL, REPL 실행 흐름은 `main.rs -> TaskService -> TaskRepository -> GlueSqlTaskRepository -> GlueSQL MemoryStorage`다. 실패 흐름은 `cli/repository/repl -> AppError -> main.rs 출력`이다. `Command`는 명령을 표현할 뿐 데이터를 저장하지 않는다.

## 파일 경로

`src/repl.rs`

### 이 파일의 역할

`cargo run -- repl` 실행 후 사용자가 입력하는 SQL line을 반복해서 읽고 실행한다.

### 핵심 코드 블록

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

### 코드 블록별 해설

- `.exit`, `.quit`: REPL loop를 끝낸다.
- `.schema`: 현재 `tasks` table schema를 출력한다.
- `sql`: 일반 SQL 문자열로 보고 `service.execute_sql`에 전달한다.
- `write_sql_results`: `SqlResult`를 REPL 출력으로 쓴다.

### REPL 시작 코드

```rust
pub fn run_repl<R: TaskRepository>(service: &mut TaskService<R>) -> Result<(), AppError> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    run_repl_with_io(service, stdin.lock(), &mut stdout)
}
```

읽는 법:

```text
service를 수정 가능하게 빌린다.
-> 터미널 입력 stdin을 준비한다.
-> 터미널 출력 stdout을 준비한다.
-> run_repl_with_io에 실제 입력/출력 처리를 맡긴다.
```

### REPL 반복 코드

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

읽는 법:

```text
프롬프트를 출력한다.
-> 한 줄을 읽는다.
-> 입력이 끝났으면 종료한다.
-> 줄바꿈과 공백을 제거한다.
-> 빈 줄이면 다시 프롬프트로 돌아간다.
```

### REPL 결과 출력 코드

```rust
fn write_sql_results(output: &mut impl Write, results: &[SqlResult]) -> Result<(), AppError>
```

읽는 법:

```text
SqlResult 목록을 받는다.
-> SELECT면 label과 row를 표처럼 출력한다.
-> INSERT/UPDATE/DELETE면 변경 건수를 출력한다.
-> Message면 메시지를 그대로 출력한다.
```

### 테스트하기 쉽게 나눈 이유

`run_repl`은 실제 stdin/stdout을 사용한다. 반면 `run_repl_with_io`는 입력과 출력을 매개변수로 받는다. 그래서 테스트에서는 `Cursor`로 가짜 입력을 만들고 `Vec<u8>`로 가짜 출력을 받을 수 있다.

### 초심자가 수정할 수 있는 부분

프롬프트 문구는 `PROMPT` 상수를 바꾸면 된다.

```rust
const PROMPT: &str = "rust-task> ";
```
