# 용어 사전

| 용어 | 종류 | 현재 프로젝트 의미 | 실제 코드 |
| --- | --- | --- | --- |
| Project | domain | Task를 묶는 상위 항목 | `src/project.rs` |
| TaskTag | relation | Task와 Tag의 N:M 연결 | `task_tags` table |
| aggregate | SQL | COUNT로 통계를 만듦 | `project_stats` |
| application constraint | 설계 | DB 미지원 규칙을 repository가 검사 | `tag_task` |

## 언어 문법

| 용어 | 분류 | 한 줄 설명 | 프로젝트 코드 예시 | 관련 파일 | 주의할 점 |
| --- | --- | --- | --- | --- | --- |
| `struct` | Rust 문법 | 여러 필드를 묶어 타입을 만든다. | `pub struct Task` | `src/task.rs` | 메서드는 `impl`에 작성한다. |
| `enum` | Rust 문법 | 여러 형태 중 하나를 표현한다. | `pub enum Command` | `src/command.rs` | variant마다 필요한 값을 다르게 가질 수 있다. |
| struct-like enum variant | Rust 문법 | enum variant가 이름 붙은 필드를 가진다. | `Add { title: String }` | `src/command.rs` | `Command::Add { title }`처럼 꺼낸다. |
| `impl` | Rust 문법 | 타입에 함수를 붙이는 블록이다. | `impl Task` | `src/task.rs` | Java처럼 struct 안에 메서드를 직접 넣지 않는다. |
| `trait` | Rust 문법 | 어떤 타입이 제공해야 하는 동작의 약속이다. | `pub trait TaskRepository` | `src/repository/mod.rs` | Java interface와 비슷하게 볼 수 있다. |
| generic | Rust 문법 | 타입을 나중에 정할 수 있게 한다. | `TaskService<R>` | `src/service/mod.rs` | 아무 타입이나 받는다는 뜻은 아니다. |
| trait bound | Rust 문법 | generic 타입에 필요한 조건을 붙인다. | `R: TaskRepository` | `src/service/mod.rs` | `R`은 `TaskRepository`를 구현해야 한다. |
| `impl Trait for Type` | Rust 문법 | 특정 타입이 trait 약속을 구현한다. | `impl TaskRepository for JsonTaskRepository` | `src/repository/mod.rs` | trait에 선언된 메서드를 실제 코드로 작성해야 한다. |
| `Vec` | Rust 표준 컬렉션 | 크기가 변할 수 있는 목록이다. | `Vec<Task>` | `src/main.rs` | 실행 중 Todo 목록 역할을 한다. |
| `Option` | Rust 표준 타입 | 값이 있음 `Some` 또는 없음 `None`을 표현한다. | `Option<Task>` | `src/main.rs` | null 대신 사용한다. |
| `Result` | Rust 표준 타입 | 성공 `Ok` 또는 실패 `Err`를 표현한다. | `Result<Command, AppError>` | `src/cli.rs` | 사용자 입력 실패 처리에 사용한다. |
| custom error | Rust 패턴 | 앱 전용 실패 타입을 만든다. | `AppError` | `src/error.rs` | 실패 종류를 enum variant로 나눈다. |
| `Display` | Rust trait | `{}` 출력 형식을 정한다. | `impl fmt::Display for AppError` | `src/error.rs` | `eprintln!("{message}")` 출력에 쓰인다. |
| `From` | Rust trait | 다른 에러 타입을 앱 에러로 바꾼다. | `impl From<std::io::Error> for AppError` | `src/error.rs` | `?`와 함께 자주 쓰인다. |
| `?` | Rust 문법 | `Err`이면 현재 함수에서 바로 반환한다. | `require_next(...)?` | `src/cli.rs` | 반환 타입이 맞아야 사용할 수 있다. |
| iterator | Rust 문법 | 여러 값을 하나씩 처리하는 흐름이다. | `tasks.iter()` | `src/main.rs` | 반복문 자체가 아니라 반복 가능한 흐름이다. |
| `iter()` | Rust 문법 | 값을 읽기용 참조로 하나씩 본다. | `tasks.iter()` | `src/main.rs` | 값을 수정할 수 없다. |
| `iter_mut()` | Rust 문법 | 값을 수정 가능한 참조로 하나씩 본다. | `tasks.iter_mut()` | `src/main.rs` | `task.done = true` 같은 수정에 필요하다. |
| `into_iter()` | Rust 문법 | 값을 소유권째 하나씩 꺼낸다. | `args.into_iter()` | `src/cli.rs` | 원래 컬렉션을 소비한다. |
| closure | Rust 문법 | 이름 없는 작은 함수다. | `|task| task.done` | `src/repository/mod.rs` | iterator 메서드와 자주 같이 쓴다. |
| `filter` | Rust iterator 메서드 | 조건에 맞는 값만 남긴다. | `.filter(|task| ...)` | `src/repository/mod.rs` | 검색에 사용한다. |
| `count` | Rust iterator 메서드 | iterator의 항목 개수를 센다. | `.count()` | `src/repository/mod.rs` | 통계 계산에 사용한다. |
| borrowing | Rust 문법 | 소유권을 가져오지 않고 값을 빌려 사용한다. | `print_task(task: &Task)` | `src/main.rs` | 수정하려면 `&mut`가 필요하다. |
| `matches!` | Rust macro | 값이 특정 패턴과 맞는지 확인한다. | `matches!(result, Err(AppError::GlueSql(_)))` | `src/repository/gluesql_repository.rs` | `_`는 안쪽 값은 검사하지 않는다는 뜻이다. |
| `impl AsRef<Path>` | Rust 문법 | path처럼 볼 수 있는 여러 타입을 함수 인자로 받는다. | `persistent(path: impl AsRef<Path>)` | `src/repository/gluesql_repository.rs` | `&str`과 `PathBuf`를 모두 받을 수 있다. |

## 프레임워크

| 용어 | 분류 | 한 줄 설명 | 프로젝트 코드 예시 | 관련 파일 | 주의할 점 |
| --- | --- | --- | --- | --- | --- |
| 웹 프레임워크 | 프레임워크 | HTTP 서버를 만드는 도구 | 코드에서 확인되지 않음 | 없음 | 현재 프로젝트는 CLI 앱이다. |

## 라이브러리

| 용어 | 분류 | 한 줄 설명 | 프로젝트 코드 예시 | 관련 파일 | 주의할 점 |
| --- | --- | --- | --- | --- | --- |
| 외부 crate | 라이브러리 | Cargo dependency로 추가하는 Rust 패키지 | `serde`, `serde_json`, `gluesql`, `futures`, `async-trait` | `Cargo.toml` | Step 12는 새 crate 이름 대신 `gluesql_sled_storage` feature를 추가했고, Step 16도 이를 유지한다. |
| serde | 라이브러리 | Rust 값을 다른 형식으로 바꾸는 기반 crate | `Serialize`, `Deserialize` | `src/task.rs` | derive feature가 필요하다. |
| serde_json | 라이브러리 | Rust 값과 JSON 문자열을 서로 변환한다. | `serde_json::from_str` | `src/repository/mod.rs` | JSON 문법 오류는 parsing 실패가 된다. |
| gluesql | 라이브러리 | Rust 코드 안에서 SQL 엔진과 storage를 제공한다. | `Glue::new` | `src/repository/gluesql_repository.rs` | Step 40 현재도 repository 내부 구현, `sql` 명령, REPL SQL 실행, transaction 관찰 테스트에 사용한다. |
| futures | 라이브러리 | async Future를 실행하거나 조합하는 도구를 제공한다. | `block_on` | `src/repository/gluesql_repository.rs` | `main.rs`를 async로 바꾸지 않기 위해 repository 내부에서만 사용한다. |

## 빌드 도구

| 용어 | 분류 | 한 줄 설명 | 프로젝트 코드 예시 | 관련 파일 | 주의할 점 |
| --- | --- | --- | --- | --- | --- |
| Cargo | 빌드 도구 | Rust 프로젝트 빌드/실행/테스트 도구 | `cargo test` | `Cargo.toml` | 프로젝트 루트에서 실행한다. |

## 데이터베이스

| 용어 | 분류 | 한 줄 설명 | 프로젝트 코드 예시 | 관련 파일 | 주의할 점 |
| --- | --- | --- | --- | --- | --- |
| 메모리 저장 | 저장 방식 | 프로그램 실행 중 RAM에만 데이터를 둔다. | `let mut tasks = Vec::new();` | `src/main.rs` | 프로그램 종료 시 데이터가 사라진다. |
| JSON 파일 저장 | 저장 방식 | 데이터를 JSON 파일에 저장한다. | `tasks.json` | `tasks.json`, `src/repository/mod.rs` | 프로그램 종료 후에도 데이터가 남는다. |
| Repository | 저장소 패턴 | 데이터를 어디에 저장하는지 감싸는 역할이다. | `TaskRepository` | `src/repository/mod.rs` | 현재는 JSON 구현체와 GlueSQL 구현체가 함께 있다. |
| Service layer | 애플리케이션 계층 | 명령 실행 흐름과 저장소 사이에 있는 계층이다. | `TaskService` | `src/service/mod.rs` | 현재는 repository에 위임하는 역할이 중심이다. |
| JsonTaskRepository | 저장소 구현체 | `tasks.json`을 사용하는 Todo 저장소다. | `JsonTaskRepository::new` | `src/repository/mod.rs` | Step 40 현재도 삭제하지 않고 보존된 구현체이며 SQL은 지원하지 않는다. |
| GlueSqlTaskRepository | 저장소 구현체 | GlueSQL storage를 사용하는 Todo 저장소다. | `GlueSqlTaskRepository::persistent` | `src/repository/gluesql_repository.rs` | Step 40 현재 `main.rs`가 사용하는 활성 구현체다. |
| MemoryStorage | GlueSQL 저장 방식 | 프로그램 실행 중 메모리에만 SQL table을 둔다. | `MemoryStorage::default()` | `src/repository/gluesql_repository.rs` | 현재는 테스트에서 주로 사용한다. |
| SledStorage | GlueSQL 저장 방식 | 디렉터리에 SQL table 데이터를 저장한다. | `SledStorage::new(path)` | `src/repository/gluesql_repository.rs` | Step 12 기본 실행 저장소이며 Step 14에서 transaction 관찰 대상이다. |
| SharedMemoryStorage | GlueSQL 저장 방식 | `MemoryStorage`를 여러 thread에서 공유하는 패턴을 볼 수 있는 storage다. | 코드에서 확인되지 않음 | `docs/beginner-codebase-guide/20-storage-comparison.md` | 현재 dependency와 코드에는 직접 도입하지 않았다. |
| JsonStorage | GlueSQL 저장 방식 | GlueSQL storage 계층의 JSON storage다. | 코드에서 확인되지 않음 | `docs/beginner-codebase-guide/20-storage-comparison.md` | 프로젝트의 `JsonTaskRepository`와 다르다. |
| MongoStorage | GlueSQL 저장 방식 | MongoDB 위에 GlueSQL storage adapter를 붙이는 분석 후보이다. | 코드에서 확인되지 않음 | `docs/beginner-codebase-guide/20-storage-comparison.md` | 현재 외부 DB 설정을 추가하지 않는다. |
| CompositeStorage | GlueSQL 저장 방식 | 여러 storage를 조합하는 구조를 볼 수 있는 분석 후보이다. | 코드에서 확인되지 않음 | `docs/beginner-codebase-guide/20-storage-comparison.md` | 현재 코드에 도입하지 않는다. |
| Transaction | DB 동작 단위 | 여러 SQL을 하나의 작업 단위로 묶는다. | `BEGIN`, `COMMIT`, `ROLLBACK` | `src/repository/gluesql_repository.rs` | `MemoryStorage`는 명시적 transaction을 지원하지 않고, `SledStorage`에서 관찰한다. |
| Snapshot | DB 읽기 시점 | transaction이 시작된 시점의 데이터를 계속 보게 하는 관찰 대상이다. | `sled_storage_keeps_repeatable_read_snapshot_until_commit` | `src/repository/gluesql_repository.rs` | 현재 프로젝트는 GlueSQL 내부 구현체를 직접 수정하지 않고 테스트로 결과를 확인한다. |
| Write lock | DB 쓰기 제어 | 한 writer transaction이 열려 있을 때 다른 writer를 막는 동작이다. | `database is locked` | `src/repository/gluesql_repository.rs` | 현재 테스트는 에러 메시지에 `database is locked`가 포함되는지 확인한다. |
| Parser | GlueSQL 내부 계층 | SQL 문자열을 구문 구조로 해석한다. | `Glue::execute` 내부 | `docs/beginner-codebase-guide/17-gluesql-internals.md` | 현재 프로젝트는 직접 호출하지 않는다. |
| Planner | GlueSQL 내부 계층 | AST를 실행 가능한 plan으로 준비한다. | `Planner` trait bound | `src/repository/gluesql_repository.rs` | 현재 코드에서는 storage trait bound로 이름이 보인다. |
| Executor | GlueSQL 내부 계층 | plan을 실행하고 storage를 호출한다. | `Glue::execute` 내부 | `docs/beginner-codebase-guide/17-gluesql-internals.md` | 현재 프로젝트는 `Payload` 결과로 간접 관찰한다. |
| Store Adapter | GlueSQL 구조 | SQL engine과 실제 storage를 연결하는 trait 구현이다. | `GStore + GStoreMut + Planner` | `src/repository/gluesql_repository.rs` | storage별 지원 기능이 다르다. |
| Minimal Custom Storage | GlueSQL 학습 주제 | GlueSQL engine이 호출할 최소 storage trait 책임을 직접 구현한다고 가정하고 읽는 분석 주제다. | `GStore + GStoreMut + Planner` | `docs/beginner-codebase-guide/18-custom-storage.md` | Step 16에서는 production code로 구현하지 않고 문서로 책임만 정리한다. |
| Read-only storage | GlueSQL storage 책임 | schema/data 조회만 제공하는 storage 형태다. | `Store` 책임 | `docs/beginner-codebase-guide/18-custom-storage.md` | Todo 앱의 `add`, `done`, `delete`까지 처리하려면 쓰기 책임도 필요하다. |
| Writable storage | GlueSQL storage 책임 | schema/data 생성, 삽입, 수정, 삭제를 제공하는 storage 형태다. | `StoreMut` 책임 | `docs/beginner-codebase-guide/18-custom-storage.md` | transaction과 index는 별도 확장 책임으로 봐야 한다. |
| SqlResult | 프로젝트 결과 타입 | SQL 실행 결과를 CLI 출력용으로 표현한다. | `SqlResult::Select` | `src/repository/mod.rs` | GlueSQL `Payload`를 그대로 main에 노출하지 않기 위해 사용한다. |
| Query Execution | GlueSQL 실행 흐름 | SQL 문자열이 실행되고 결과 `Payload`가 프로젝트 타입으로 바뀌는 흐름이다. | `execute(sql)` | `docs/beginner-codebase-guide/19-query-execution.md` | 현재 프로젝트는 `Glue::execute` public API로 간접 관찰한다. |
| `row_to_task` | 변환 함수 | GlueSQL row를 `Task`로 바꾼다. | `[Value::I64, Value::Str, Value::Bool]` | `src/repository/gluesql_repository.rs` | column 순서와 타입이 맞아야 한다. |
| `select_count` | 변환 함수 | `COUNT` query 결과를 `usize`로 바꾼다. | `Value::I64(value)` | `src/repository/gluesql_repository.rs` | COUNT payload가 아니면 `AppError::GlueSql`이 된다. |
| `payload_to_sql_result` | 변환 함수 | GlueSQL `Payload`를 CLI 출력용 `SqlResult`로 바꾼다. | `Payload::Select` | `src/repository/gluesql_repository.rs` | Todo domain model 변환과 다르다. |
| REPL | 실행 모드 | 프로그램을 종료하지 않고 입력을 반복해서 받는 모드다. | `run_repl` | `src/repl.rs` | `.exit` 또는 `.quit`으로 종료한다. |

## 프로젝트 규칙

| 용어 | 분류 | 한 줄 설명 | 프로젝트 코드 예시 | 관련 파일 | 주의할 점 |
| --- | --- | --- | --- | --- | --- |
| Step 3 | 프로젝트 학습 단계 | Todo를 `tasks.json`에 저장하기 시작한 단계 | `load_tasks`, `save_tasks` | 이전 단계의 `src/main.rs` | 현재는 완료된 단계다. |
| Step 4 | 프로젝트 학습 단계 | Repository trait와 JSON repository를 도입한 단계 | `TaskRepository`, `JsonTaskRepository` | `src/repository/mod.rs` | Service layer와 DB는 아직 넣지 않는다. |
| Step 5 | 프로젝트 학습 단계 | Service layer를 도입한 단계 | `TaskService<R: TaskRepository>` | `src/service/mod.rs` | 현재는 완료된 단계다. |
| Step 6 | 프로젝트 학습 단계 | Custom error를 도입한 단계 | `AppError` | `src/error.rs` | DB는 아직 넣지 않는다. |
| Step 7 | 프로젝트 학습 단계 | 검색과 통계를 도입한 단계 | `search`, `stats`, `TaskStats` | `src/repository/mod.rs`, `src/task.rs` | 완료된 단계다. |
| Step 8 | 프로젝트 학습 단계 | GlueSQL 저장소를 추가한 단계 | `GlueSqlTaskRepository` | `src/repository/gluesql_repository.rs` | 완료된 단계다. |
| Step 9 | 프로젝트 학습 단계 | SQL 실행 모드를 추가한 단계 | `Command::Sql`, `SqlResult` | `src/command.rs`, `src/repository/mod.rs` | 완료된 단계다. |
| Step 10 | 프로젝트 학습 단계 | REPL 모드를 추가한 단계 | `Command::Repl`, `run_repl` | `src/command.rs`, `src/repl.rs` | `.schema`, `.exit`, `.quit`을 지원한다. |
| Step 11 | 프로젝트 학습 단계 | 테스트를 보강한 단계 | `missing_add_title_returns_error`, `invalid_sql_returns_gluesql_error` | `src/cli.rs`, `src/repository/gluesql_repository.rs` | 새 기능을 추가하지 않고 총 57개 테스트로 기존 흐름을 확인한다. |
| Step 12 | 프로젝트 학습 단계 | GlueSQL 저장소를 영속 저장소로 전환한 단계 | `GlueSqlTaskRepository::persistent` | `src/main.rs`, `src/repository/gluesql_repository.rs` | 데이터가 `data/rust-task-db`에 유지된다. |
| Step 14 | 프로젝트 학습 단계 | GlueSQL SledStorage transaction과 동시성 특성을 테스트로 관찰한 단계 | `sled_storage_keeps_repeatable_read_snapshot_until_commit` | `src/repository/gluesql_repository.rs` | 새 CLI 명령이나 새 외부 crate는 추가하지 않는다. |
| Step 15 | 프로젝트 학습 단계 | GlueSQL Engine/Storage Adapter 구조를 문서와 테스트로 분석한 단계 | `17-gluesql-internals.md` | `docs/beginner-codebase-guide/17-gluesql-internals.md` | Parser/Planner/Executor를 직접 구현하지 않는다. |
| Step 16 | 프로젝트 학습 단계 | Minimal Custom Storage 책임과 구현 순서를 문서로 분석한 단계 | `18-custom-storage.md` | `docs/beginner-codebase-guide/18-custom-storage.md` | 실제 custom storage를 production code에 도입하지 않는다. |
| Step 17 | 프로젝트 학습 단계 | Todo 명령별 SQL 생성과 `Payload` 변환 흐름을 문서로 분석한 단계 | `19-query-execution.md` | `docs/beginner-codebase-guide/19-query-execution.md` | 새 CLI 명령이나 새 외부 crate를 추가하지 않는다. |
| Step 18 | 프로젝트 학습 단계 | Storage별 기능 차이와 현재 코드 도입 여부를 문서로 분석한 단계 | `20-storage-comparison.md` | `docs/beginner-codebase-guide/20-storage-comparison.md` | 새 storage를 코드에 도입하지 않는다. |

## 테스트

| 용어 | 분류 | 한 줄 설명 | 프로젝트 코드 예시 | 관련 파일 | 주의할 점 |
| --- | --- | --- | --- | --- | --- |
| `#[test]` | Rust 테스트 attribute | 함수를 테스트로 표시한다. | `#[test] fn parses_add_command()` | `src/cli.rs` | `cargo test`로 실행한다. |
| `assert_eq!` | 테스트 macro | 두 값이 같은지 확인한다. | `assert_eq!(command, Ok(Command::List))` | `src/cli.rs` | 비교 타입이 `PartialEq`를 구현해야 한다. |
| `assert!` | 테스트 macro | 조건이 `true`인지 확인한다. | `assert!(matches!(result, Err(AppError::GlueSql(_))))` | `src/repository/gluesql_repository.rs` | 조건이 `false`면 테스트가 실패한다. |

## 배포/인프라

| 용어 | 분류 | 한 줄 설명 | 프로젝트 코드 예시 | 관련 파일 | 주의할 점 |
| --- | --- | --- | --- | --- | --- |
| CI/CD | 배포/인프라 | 자동 빌드와 테스트 파이프라인 | 코드에서 확인되지 않음 | 없음 | 현재 저장소에는 CI 설정이 없다. |
