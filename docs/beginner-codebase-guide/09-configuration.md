# 설정 파일 해설

## 설정 파일 목록

- `Cargo.toml`
- `Cargo.lock`
- `tasks.json`

## 빌드 설정

| 항목 | 내용 |
| --- | --- |
| 파일 경로 | `Cargo.toml` |
| 역할 | Rust package 설정 |
| 이 파일이 없으면 생기는 문제 | Cargo가 프로젝트를 빌드할 수 없다. |
| 주요 설정 항목 | `name`, `version`, `edition`, `[dependencies]` |
| 설정값별 의미 | `edition = "2021"`은 Rust 2021 edition 사용 |
| 초심자가 수정해도 되는 값 | `version` |
| 수정하면 위험한 값 | `edition`, dependency 임의 추가 |
| 관련 실행 명령어 | `cargo run`, `cargo test`, `cargo fmt --check` |

## 실행 설정

별도 실행 설정 파일은 코드에서 확인되지 않음.

## 환경변수

환경변수 사용은 코드에서 확인되지 않음.

## 데이터베이스 설정

별도 데이터베이스 설정 파일은 코드에서 확인되지 않음. Step 15 현재도 `src/main.rs`가 `GlueSqlTaskRepository::persistent("data/rust-task-db")`를 호출한다.

```rust
let repository = GlueSqlTaskRepository::persistent("data/rust-task-db");
```

`SledStorage`는 별도 설정 파일 없이 코드에서 넘긴 경로를 사용한다. 현재 저장 위치는 `data/rust-task-db`다.

## 외부 서비스 설정

코드에서 확인되지 않음.

## Docker 설정

코드에서 확인되지 않음.

## 테스트 설정

별도 테스트 설정 파일은 없다. `cargo test`가 `src/cli.rs`, `src/error.rs`, `src/service.rs`, `src/repository/mod.rs`, `src/repository/gluesql_repository.rs`, `src/main.rs` 안의 테스트를 실행한다.

## CI/CD 설정

코드에서 확인되지 않음.

## 초심자가 수정해도 되는 값

`Cargo.toml`의 `version`

## 수정하면 위험한 값

Step 15 현재 `[dependencies]`에는 `serde`, `serde_json`, `gluesql`, `futures`를 둔다. `gluesql` feature에는 `gluesql_memory_storage`와 `gluesql_sled_storage`가 포함된다.

## 설정 오류 해결 가이드

- `cargo test`가 프로젝트를 못 찾으면 `Cargo.toml` 위치에서 실행했는지 확인한다.
- dependency 관련 오류가 나면 `serde`, `serde_json`, `gluesql`, `futures` 버전과 `Cargo.lock` 상태를 확인한다.
