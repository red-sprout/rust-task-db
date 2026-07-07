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

별도 데이터베이스 설정 파일은 코드에서 확인되지 않음. 현재 Step 11은 `GlueSqlTaskRepository`가 GlueSQL `MemoryStorage`를 코드 안에서 직접 만든다.

```rust
let storage = MemoryStorage::default();
let glue = Glue::new(storage);
```

`MemoryStorage`는 파일 경로, 계정, 포트 같은 설정이 없다. 프로그램이 끝나면 데이터도 사라진다.

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

현재 Step 11에서는 `[dependencies]`에 `serde`, `serde_json`, `gluesql`, `futures`를 둔다. Step 11 테스트 보강도 새 dependency 없이 Rust 내장 test harness로 처리한다.

## 설정 오류 해결 가이드

- `cargo test`가 프로젝트를 못 찾으면 `Cargo.toml` 위치에서 실행했는지 확인한다.
- dependency 관련 오류가 나면 `serde`, `serde_json`, `gluesql`, `futures` 버전과 `Cargo.lock` 상태를 확인한다.
