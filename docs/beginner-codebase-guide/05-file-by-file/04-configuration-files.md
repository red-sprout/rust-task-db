# 설정 파일

## 포함된 파일 목록

- `Cargo.toml`
- `Cargo.lock`

## 이 파일 묶음의 역할

Cargo가 프로젝트를 빌드하고 테스트할 수 있게 한다.

## 전체 연결 관계

```text
Cargo.toml -> cargo run / cargo test -> src/main.rs / src/cli.rs
```

## 파일별 상세 설명

## 파일 경로

`Cargo.toml`

### 이 파일의 역할

package metadata와 dependency를 선언한다.

### 이 파일이 필요한 이유

Cargo가 이 디렉터리를 Rust 프로젝트로 인식한다.

### 이 파일과 연결된 다른 파일

`src/main.rs`, `src/cli.rs`, `src/command.rs`, `src/task.rs`, `tasks.json`

### 핵심 코드 블록

```toml
[package]
name = "rust-task"
version = "0.1.0"
edition = "2021"

futures = "0.3"
gluesql = { version = "0.19.0", default-features = false, features = ["gluesql_memory_storage", "gluesql_sled_storage"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

### 코드 블록별 해설

- `name`: binary 이름
- `edition`: Rust 문법 edition
- `serde`: `Task`에 `Serialize`, `Deserialize` derive를 붙이기 위해 사용한다.
- `serde_json`: `Vec<Task>`와 JSON 문자열을 서로 변환하기 위해 사용한다.
- `futures`: GlueSQL의 async `execute`를 동기 CLI에서 기다리기 위해 사용한다.
- `gluesql`: Step 8부터 GlueSQL `MemoryStorage`를 사용하기 위해 추가했다.
- `default-features = false`: 필요한 기능만 켜기 위해 기본 feature를 끈다.
- `features = ["gluesql_memory_storage", "gluesql_sled_storage"]`: 테스트용 MemoryStorage와 CLI 기본 실행용 SledStorage 기능을 함께 켠다.

### 이 파일에서 사용된 언어 문법

TOML 문법이다. Rust 문법이 아니다.

### 이 파일에서 사용된 프레임워크/라이브러리 기능

Cargo 기능이다.

### 초심자가 수정할 수 있는 부분

`version`은 바꿔볼 수 있다.

### 수정 전 코드

```toml
version = "0.1.0"
```

### 수정 후 코드

```toml
version = "0.1.1"
```

### 수정 시 영향받는 파일

보통 코드에는 영향이 없다.

### 이 파일을 이해한 뒤 알아야 하는 것

Step 16 현재는 `serde`, `serde_json`, `gluesql`, `futures`를 dependency로 사용한다. Step 12에서 새 crate 이름을 직접 추가하지는 않았지만, `gluesql` feature에 `gluesql_sled_storage`를 추가해 SledStorage를 사용한다. Step 16에서도 새 외부 crate는 추가하지 않았다.

현재 `Cargo.toml` 핵심:

```toml
gluesql = { version = "0.19.0", default-features = false, features = ["gluesql_memory_storage", "gluesql_sled_storage"] }
```

## 파일 경로

`Cargo.lock`

### 이 파일의 역할

Cargo가 자동 생성한 lock 파일이다.

### 이 파일이 필요한 이유

빌드 재현성을 보장한다.

### 이 파일과 연결된 다른 파일

`Cargo.toml`

### 핵심 코드 블록

```toml
[[package]]
name = "rust-task"
version = "0.1.0"
```

### 코드 블록별 해설

현재는 `serde`, `serde_json`, `gluesql`, `futures`와 그 하위 dependency가 함께 기록된다.

### 이 파일에서 사용된 언어 문법

TOML 형식.

### 이 파일에서 사용된 프레임워크/라이브러리 기능

Cargo lock 기능.

### 초심자가 수정할 수 있는 부분

직접 수정하지 않는다.

### 수정 전 코드

직접 수정하지 않는다.

### 수정 후 코드

직접 수정하지 않는다.

### 수정 시 영향받는 파일

직접 수정하면 Cargo 상태와 어긋날 수 있다.

### 이 파일을 이해한 뒤 알아야 하는 것

Cargo가 관리하는 파일이다.
