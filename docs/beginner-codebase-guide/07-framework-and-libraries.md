# 프레임워크와 라이브러리 해설

## 이 문서의 목적

현재 코드에서 Rust 언어 기능, 표준 라이브러리, Cargo 기능, 프로젝트 자체 규칙을 구분한다.

## 언어 자체 기능과 외부 기능 구분 기준

- Rust 문법: `struct`, `enum`, `impl`, `match`, `Option`, `Result`, borrow
- 표준 라이브러리: `std::env::args`, `std::fs`, `std::path`, `std::io`
- 빌드 도구: Cargo
- 외부 라이브러리: `serde`, `serde_json`, `gluesql`, `futures`

## 프레임워크 기능 목록

코드에서 확인되지 않음. 현재 프로젝트는 웹 프레임워크를 사용하지 않는다.

## 외부 라이브러리 목록

현재 외부 라이브러리는 `serde`, `serde_json`, `gluesql`, `futures`다.

| 이름 | 역할 | 등장 위치 |
| --- | --- | --- |
| `serde` | `Task`에 `Serialize`, `Deserialize` derive 제공 | `Cargo.toml`, `src/task.rs` |
| `serde_json` | JSON 문자열과 `Vec<Task>` 변환. 보존된 `JsonTaskRepository`에서 사용 | `Cargo.toml`, `src/repository/mod.rs` |
| `gluesql` | Rust 코드 안에서 SQL 엔진, `MemoryStorage`, `SledStorage` 제공 | `Cargo.toml`, `src/repository/gluesql_repository.rs` |
| `futures` | GlueSQL의 async `execute`를 동기 코드에서 기다리는 `block_on` 제공 | `Cargo.toml`, `src/repository/gluesql_repository.rs` |

## 빌드 도구 기능 목록

| 항목 | 내용 |
| --- | --- |
| 이름 | Cargo |
| 분류 | 빌드 도구 |
| 등장 위치 | `Cargo.toml`, `Cargo.lock` |
| 역할 | 빌드, 실행, 테스트 |
| 언어 자체 기능인지 여부 | Rust 언어 문법이 아님 |
| 프로젝트에서 쓰인 이유 | Rust 프로젝트 표준 도구 |
| 초심자가 주의할 점 | `Cargo.lock`은 직접 수정하지 않는다. |

## 프로젝트 자체 규칙 목록

| 항목 | 내용 |
| --- | --- |
| 이름 | Step 기반 구현 |
| 분류 | 프로젝트 자체 규칙 |
| 등장 위치 | [docs/prompt.md](../prompt.md), [docs/todo/roadmap.md](../todo/roadmap.md) |
| 역할 | 초심자가 문법을 순서대로 배우도록 구현 범위를 제한 |
| 언어 자체 기능인지 여부 | Rust 문법이 아님 |
| 프로젝트에서 쓰인 이유 | 학습 난이도 조절 |
| 초심자가 주의할 점 | 뒤 단계 기능을 먼저 넣지 않는다. |

## 초심자가 헷갈리기 쉬운 구분

- `Vec`는 Rust 표준 컬렉션이지 외부 라이브러리가 아니다.
- `Result`와 `Option`은 Rust 표준 타입이지 외부 라이브러리가 아니다.
- `std::fs`는 표준 라이브러리이고, `serde_json`은 외부 crate다.
- `gluesql`은 웹 프레임워크가 아니라 Rust 코드 안에서 쓰는 SQL 엔진 라이브러리다. Step 12에서는 `SledStorage`로 데이터를 디렉터리에 유지한다.
- GlueSQL의 transaction/동시성 특성은 core 하나로 고정되는 것이 아니라 storage 구현체별로 달라진다. Step 15 테스트는 `MemoryStorage`, `SledStorage`, `JsonTaskRepository`의 지원 경계를 현재 Todo table과 repository trait으로 관찰한다.
- Step 15 문서는 `Glue::execute`가 Parser, Planner, Executor, Store 호출 흐름을 감싼다는 점을 설명한다. 현재 프로젝트는 이 내부 계층을 직접 호출하지 않는다.
- `SledStorage`를 같은 DB에서 여러 `Glue` 인스턴스와 함께 쓰려면 같은 path를 동시에 두 번 열지 말고 `SledStorage::clone()`으로 handle을 나눠 쓰는 패턴을 사용한다.
- `futures::executor::block_on`은 `main.rs`를 async로 바꾸지 않고 repository 내부에서 async 작업을 기다리기 위해 쓴다.
- `cargo test`는 Rust 문법이 아니라 Cargo 명령이다.
- `Task`는 Rust 내장 타입이 아니라 이 프로젝트가 만든 타입이다.
- `Command`도 Rust 내장 타입이 아니라 이 프로젝트가 만든 enum이다.
