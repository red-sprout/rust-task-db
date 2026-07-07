# 초심자가 자주 하는 실수

## 언어 문법 관련 실수

## 실수 이름: `mut` 없이 Vec를 수정하려 함

### 왜 발생하는가

Rust 변수는 기본적으로 immutable이라는 점을 놓쳐서 발생한다.

### 문제가 되는 이유

`tasks.push(...)` 같은 수정이 불가능하다.

### 잘못된 코드

```rust
let tasks = Vec::new();
add_task(&mut tasks, "Rust".to_string());
```

### 올바른 코드

```rust
let mut tasks = Vec::new();
add_task(&mut tasks, "Rust".to_string());
```

### 관련 파일

`src/main.rs`

### 예방 방법

값을 수정해야 하면 변수 선언에 `mut`가 있는지 확인한다.

## 프로젝트 구조 관련 실수

## 실수 이름: Step 5에 웹/API 개념을 찾음

### 왜 발생하는가

백엔드 프로젝트라는 말 때문에 Controller, Service, Repository를 기대해서 발생한다.

### 문제가 되는 이유

현재 단계에서는 웹 Controller나 HTTP API가 의도적으로 없다. Service layer는 있지만 웹 계층은 아니다.

### 잘못된 코드

```text
src/controller.rs를 만들거나 HTTP server crate를 추가하려고 함
```

### 올바른 코드

```text
Step 7에서는 `src/command.rs`, `src/cli.rs`, `src/error.rs`, `src/main.rs`, `src/service.rs`, `src/task.rs`, `src/repository/mod.rs`, `tasks.json`을 본다.
```

### 관련 파일

`docs/todo/roadmap.md`

### 예방 방법

현재 단계와 로드맵을 먼저 확인한다.

## 실수 이름: 새 명령을 `main.rs`에만 추가

### 왜 발생하는가

Step 1 방식처럼 문자열 분기를 `main.rs`에 바로 추가하려고 해서 발생한다.

### 문제가 되는 이유

Step 2에서는 명령 추가 흐름이 `Command` enum, parser, 실행 분기로 나뉜다.

### 잘못된 코드

```text
src/main.rs의 match command에만 새 분기를 추가함
```

### 올바른 코드

```text
src/command.rs에 variant 추가
src/cli.rs에 parsing 추가
src/main.rs에 실행 분기 추가
src/cli.rs에 parser 테스트 추가
```

### 관련 파일

`src/command.rs`, `src/cli.rs`, `src/main.rs`

### 예방 방법

`08-data-model.md`의 `Command` 변경 체크리스트를 따른다.

## 데이터 모델 관련 실수

## 실수 이름: `Task` 필드를 추가하고 테스트를 안 바꿈

### 왜 발생하는가

테스트 expected 값도 데이터 모델을 기준으로 작성된다는 점을 놓쳐서 발생한다.

### 문제가 되는 이유

`assert_eq!`가 실패한다.

### 잘못된 코드

```rust
pub priority: i64,
```

필드만 추가하고 `Task::new`와 테스트를 그대로 둔다.

### 올바른 코드

```rust
pub priority: i64,
```

그리고 `Task::new`, `print_task`, 테스트 expected 값을 함께 수정한다.

### 관련 파일

`src/task.rs`, `src/main.rs`

### 예방 방법

`08-data-model.md`의 체크리스트를 따른다.

## 설정 관련 실수

## 실수 이름: 현재 단계에 허용되지 않은 dependency 추가

### 왜 발생하는가

미리 CLI crate나 DB crate를 쓰고 싶어서 발생한다.

### 문제가 되는 이유

학습 단계가 꼬인다. Step 10은 `serde`, `serde_json`, `gluesql`, `futures`까지만 사용하면서 GlueSQL 저장소, SQL 실행 모드, REPL 모드를 배우는 단계다.

### 잘못된 코드

```toml
clap = "4"
anyhow = "1"
```

### 올바른 코드

```toml
serde = { version = "1", features = ["derive"] }
serde_json = "1"
gluesql = { version = "0.19.0", default-features = false, features = ["gluesql_memory_storage"] }
futures = "0.3"
```

### 관련 파일

`Cargo.toml`

### 예방 방법

현재 Step 11에서는 `serde`, `serde_json`, `gluesql`, `futures`만 둔다. `clap`은 현재 로드맵에서 사용하지 않는다.

## 테스트 관련 실수

## 실수 이름: Step 9에서 `cargo run -- add` 다음 `cargo run -- list`가 이어진다고 생각함

### 왜 발생하는가

Step 9의 활성 저장소는 GlueSQL `MemoryStorage`다. 이 저장소는 프로그램이 끝나면 데이터가 사라진다.

### 문제가 되는 이유

별도 `cargo run` 명령은 별도 프로세스이므로 이전 명령의 데이터가 남아 있지 않을 수 있다.

### 잘못된 코드

```bash
cargo run -- add "Rust 공부"
cargo run -- list
```

### 올바른 코드

```bash
cargo run -- sql "INSERT INTO tasks VALUES (1, 'Rust 공부', FALSE); SELECT * FROM tasks;"
```

### 관련 파일

`src/repository/mod.rs`, `03-runtime-flow.md`

### 예방 방법

실행 결과가 이상하면 현재 명령이 새 프로세스에서 새 `MemoryStorage`를 만드는지 먼저 생각한다. 같은 실행 안에서 확인하려면 SQL 여러 statement를 한 문자열에 넣는다.

## 실수 이름: `sql` 명령과 `repl` 명령을 섞어서 생각함

### 왜 발생하는가

`sql` 명령과 `repl` 명령의 실행 방식이 다르다는 점을 놓쳐서 발생한다.

### 문제가 되는 이유

`sql`은 명령 한 번에 SQL 문자열 하나를 실행하고 종료한다. 여러 SQL을 차례로 입력하려면 Step 10의 `repl`을 사용해야 한다.

### 잘못된 코드

```bash
cargo run -- sql
```

### 올바른 코드

```bash
cargo run -- sql "SELECT * FROM tasks"
cargo run -- repl
```

### 관련 파일

`src/cli.rs`, `src/main.rs`, `src/repository/gluesql_repository.rs`

### 예방 방법

한 번 실행은 `Command::Sql { sql }`, 반복 입력은 `Command::Repl`로 구분한다.

## 에러 처리 관련 실수

## 실수 이름: `unwrap()`으로 id를 파싱

### 왜 발생하는가

빨리 숫자만 꺼내고 싶어서 발생한다.

### 문제가 되는 이유

잘못된 입력에서 panic이 발생한다.

### 잘못된 코드

```rust
value.parse::<i64>().unwrap()
```

### 올바른 코드

```rust
value
    .parse::<i64>()
    .map_err(|_| format!("id must be an integer: {value}"))
```

### 관련 파일

`src/cli.rs`

### 예방 방법

사용자 입력은 실패 가능성을 `Result`로 처리한다.
