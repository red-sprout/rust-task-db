# 초심자용 파일 읽기 순서

## 전체 읽기 전략

먼저 `16-run-guide.md`를 보고 프로젝트가 실행되는지 확인한다. 그 다음 `main()`이 `TaskService`를 만들고, service가 `TaskRepository` trait 메서드를 호출하는 흐름을 따라간다.

## 0단계: 먼저 실행해보기

- 읽을 파일: `16-run-guide.md`
- 읽는 이유: 코드 읽기 전에 `cargo run`, `cargo test`가 되는지 확인한다.
- 이 파일에서 봐야 할 코드: `cargo run -- add "Rust 공부"`, `cargo test`
- 이 파일을 읽고 나면 알아야 하는 것: Step 10은 GlueSQL 저장소 흐름 위에 `repl` 명령을 추가했다.
- 다음에 읽을 파일: `src/service.rs`

## 1단계: service 구조 이해

- 읽을 파일: `src/service.rs`
- 읽는 이유: `TaskService<R: TaskRepository>`가 `main.rs`와 repository 사이에 있다.
- 이 파일에서 봐야 할 코드: `pub struct TaskService<R: TaskRepository>`, `impl<R: TaskRepository> TaskService<R>`
- 이 파일을 읽고 나면 알아야 하는 것: service는 저장소 구현체를 직접 고정하지 않고 trait bound에 의존한다.
- 다음에 읽을 파일: `src/repository/mod.rs`

## 2단계: repository 구조 이해

- 읽을 파일: `src/repository/mod.rs`
- 읽는 이유: `TaskRepository` trait, 보존된 `JsonTaskRepository`, `GlueSqlTaskRepository` 연결이 있다.
- 이 파일에서 봐야 할 코드: `pub trait TaskRepository`, `mod gluesql_repository`, `pub use gluesql_repository::GlueSqlTaskRepository`
- 이 파일을 읽고 나면 알아야 하는 것: 저장소 책임이 `main.rs`에서 분리되어 구현체를 교체할 수 있다.
- 다음에 읽을 파일: `src/repository/gluesql_repository.rs`

## 2-1단계: GlueSQL repository 이해

- 읽을 파일: `src/repository/gluesql_repository.rs`
- 읽는 이유: 현재 Step 12의 활성 저장소 구현체다.
- 이 파일에서 봐야 할 코드: `GlueSqlTaskRepository::persistent`, `execute`, `execute_sql`, `payload_to_sql_result`, `select_tasks`, `row_to_task`, `select_count`
- 이 파일을 읽고 나면 알아야 하는 것: GlueSQL `Payload`와 `Value`를 프로젝트 타입인 `Task`, `TaskStats`, `SqlResult`로 바꾼다.
- 다음에 읽을 파일: `src/repl.rs`

## 2-2단계: REPL 입력 흐름 이해

- 읽을 파일: `src/repl.rs`
- 읽는 이유: Step 10에서 추가되고 Step 11 테스트로 보강된 `repl` 명령의 입력 루프가 있다. Step 12에서는 REPL도 SledStorage 기반 repository를 사용한다.
- 이 파일에서 봐야 할 코드: `run_repl`, `run_repl_with_io`, `.schema`, `.exit`, `.quit`, `service.execute_sql`
- 이 파일을 읽고 나면 알아야 하는 것: REPL 안에서는 같은 service와 repository 인스턴스를 계속 사용한다.
- 다음에 읽을 파일: `src/task.rs`

## 3단계: 데이터 구조 이해

- 읽을 파일: `src/task.rs`
- 읽는 이유: Todo 한 건의 구조와 JSON 변환 derive가 정의되어 있다.
- 이 파일에서 봐야 할 코드: `pub struct Task`, `Serialize`, `Deserialize`
- 이 파일을 읽고 나면 알아야 하는 것: `Task` 필드가 JSON key가 된다.
- 다음에 읽을 파일: `src/command.rs`

## 4단계: 명령 모델 이해

- 읽을 파일: `src/command.rs`
- 읽는 이유: CLI 명령을 표현하는 `Command` enum이 있다.
- 이 파일에서 봐야 할 코드: `pub enum Command`, `Add { title: String }`, `Search { keyword: String }`, `Stats`, `Sql { sql: String }`, `Repl`
- 이 파일을 읽고 나면 알아야 하는 것: CLI 명령은 문자열 그대로가 아니라 타입으로 표현된다.
- 다음에 읽을 파일: `src/cli.rs`

## 5단계: CLI parsing 이해

- 읽을 파일: `src/cli.rs`
- 읽는 이유: `std::env::args()` 결과가 `Command`로 바뀌는 곳이다.
- 이 파일에서 봐야 할 코드: `parse_args`, `require_next`, `parse_id`
- 이 파일을 읽고 나면 알아야 하는 것: 성공하면 `Ok(Command)`, 실패하면 `Err(AppError)`가 나온다.
- 다음에 읽을 파일: `src/main.rs`의 `match command`

## 6단계: 프로그램 실행 구조 이해

- 읽을 파일: `src/main.rs`
- 읽는 이유: Rust 프로그램의 시작점인 `main()`이 있고, `Command`를 실제 기능으로 실행한다.
- 이 파일에서 봐야 할 코드: `GlueSqlTaskRepository::persistent`, `TaskService::new`, `service.add`, `service.search`, `service.stats`, `service.execute_sql`, `repl::run_repl`
- 이 파일을 읽고 나면 알아야 하는 것: `main.rs`는 GlueSQL API를 직접 호출하지 않고 service를 호출한다.
- 다음에 읽을 파일: `03-runtime-flow.md`

## 7단계: 비즈니스 로직 이해

- 읽을 파일: `src/service.rs`, `src/repository/mod.rs`
- 읽는 이유: Step 10은 service layer가 Todo 명령, SQL 직접 실행, REPL SQL 실행 요청을 받고 GlueSQL repository가 SQL로 저장/검색/집계/직접 실행을 담당한다.
- 이 파일에서 봐야 할 코드: `TaskService::search`, `TaskService::stats`, `TaskService::execute_sql`, `GlueSqlTaskRepository::search`, `GlueSqlTaskRepository::stats`, `GlueSqlTaskRepository::execute_sql`
- 이 파일을 읽고 나면 알아야 하는 것: service는 현재 repository에 위임하고, repository가 SQL을 실행한다.
- 다음에 읽을 파일: 테스트 모듈

## 8단계: 에러 처리 이해

- 읽을 파일: `src/error.rs`, `src/cli.rs`, `src/service.rs`, `src/repository/mod.rs`, `src/main.rs`
- 읽는 이유: CLI parsing 실패, 파일 I/O 실패, JSON parsing 실패, GlueSQL 실패를 `Result`로 표현한다.
- 이 파일에서 봐야 할 코드: `AppError`, `Result<Command, AppError>`, `Result<Task, AppError>`, `?`
- 이 파일을 읽고 나면 알아야 하는 것: custom error가 생겨 실패 종류가 enum variant로 나뉜다.
- 다음에 읽을 파일: `10-error-handling.md`

## 9단계: 테스트 이해

- 읽을 파일: `src/cli.rs`, `src/error.rs`, `src/service.rs`, `src/repository/mod.rs`, `src/repository/gluesql_repository.rs`, `src/main.rs`의 `mod tests`
- 읽는 이유: parser 테스트, service 테스트, repository 저장소 테스트가 분리되어 있다.
- 이 파일에서 봐야 할 코드: `parses_add_command`, `parses_sql_command`, `add_delegates_to_repository`, `executes_select_sql_with_gluesql`
- 이 파일을 읽고 나면 알아야 하는 것: Step 12 테스트는 총 58개다.
- 다음에 읽을 파일: `12-practice-tasks.md`

## 최종 체크리스트

- `Command::Add { title }`가 무엇을 담는지 설명할 수 있다.
- `parse_args`가 왜 `Result<Command, AppError>`를 반환하는지 설명할 수 있다.
- `main()`에서 왜 다시 문자열이 아니라 `Command`를 `match`하는지 설명할 수 있다.
- Step 10에서 `cargo run -- add` 다음 별도 `cargo run -- list`가 이어지지 않는 이유를 설명할 수 있다.
- `TaskService<R: TaskRepository>`가 왜 generic을 쓰는지 설명할 수 있다.
- `TaskRepository` trait, 보존된 `JsonTaskRepository`, 현재 활성 `GlueSqlTaskRepository`의 차이를 설명할 수 있다.
- 없는 id가 `AppError::NotFound(id)`로 표현되는 위치를 찾을 수 있다.
- Step 7의 `search`가 `filter`와 closure로 구현된 위치를 찾을 수 있다.
- Step 8의 `search`가 GlueSQL `ILIKE`로 구현된 위치를 찾을 수 있다.
- Step 8의 `stats`가 GlueSQL `COUNT`로 완료 개수를 계산하는 위치를 찾을 수 있다.
- Step 9의 `sql` 명령이 `Command::Sql -> TaskService::execute_sql -> TaskRepository::execute_sql -> GlueSqlTaskRepository::execute_sql -> SqlResult`로 이어지는 위치를 찾을 수 있다.
- Step 10의 `repl` 명령이 `Command::Repl -> repl::run_repl -> service.execute_sql`로 이어지는 위치를 찾을 수 있다.
