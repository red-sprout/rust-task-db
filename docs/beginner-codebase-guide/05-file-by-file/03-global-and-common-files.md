# 공통 파일

## 포함된 파일 목록

Step 17 현재도 공통 에러 파일 `src/error.rs`가 있다. 유틸, 설정 모듈 파일은 없다.

[src/error.rs](../../../src/error.rs)는 여러 계층에서 함께 쓰는 공통 에러 파일이다. repository 설명은 [02-domain-or-feature-files.md](02-domain-or-feature-files.md)에서 다룬다.

## 이 파일 묶음의 역할

`src/error.rs`가 `AppError`를 제공하고, `src/cli.rs`, `src/service.rs`, `src/repository/mod.rs`, `src/repository/gluesql_repository.rs`, `src/main.rs`가 이 타입을 사용한다.

## 전체 연결 관계

```text
src/cli.rs
src/service.rs
src/repository/mod.rs
src/repository/gluesql_repository.rs
-> AppError
-> src/main.rs 출력
```

## 파일별 상세 설명

## 파일 경로

`src/error.rs`

### 이 파일의 역할

`AppError` custom error를 정의한다.

### 이 파일이 필요한 이유

여러 파일에서 실패 타입을 `AppError`로 통일하기 위해 필요하다.

### 이 파일과 연결된 다른 파일

`src/cli.rs`, `src/service.rs`, `src/repository/mod.rs`, `src/main.rs`

### 핵심 코드 블록

```rust
pub enum AppError {
    Io(std::io::Error),
    Json(serde_json::Error),
    GlueSql(String),
    NotFound(i64),
    InvalidCommand(String),
    Unsupported(String),
}
```

### 코드 블록별 해설

각 variant는 실패 종류를 나타낸다. `Unsupported`는 예를 들어 보존된 `JsonTaskRepository`에 SQL 직접 실행을 요청했을 때 사용한다.

### 이 파일에서 사용된 언어 문법

enum, trait impl, `Display`, `From`, `std::error::Error`

### 이 파일에서 사용된 프레임워크/라이브러리 기능

코드에서 확인되지 않음.

### 초심자가 수정할 수 있는 부분

`Display`의 메시지 문구를 바꿀 수 있다.

### 수정 전 코드

```rust
Self::NotFound(id) => write!(formatter, "Task not found: {id}"),
```

### 수정 후 코드

```rust
Self::NotFound(id) => write!(formatter, "No task with id: {id}"),
```

### 수정 시 영향받는 파일

`src/error.rs` 테스트와 에러 메시지를 기대하는 문서

### 이 파일을 이해한 뒤 알아야 하는 것

현재는 `AppError`가 공통 실패 타입이다. Step 9에서는 GlueSQL 실패를 `AppError::GlueSql`로, 지원하지 않는 repository 기능을 `AppError::Unsupported`로 표현한다.
