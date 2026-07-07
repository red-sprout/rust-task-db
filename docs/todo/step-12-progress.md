# Step 12 진행 상황: GlueSQL SledStorage 영속 저장소 전환

## 현재 상태

현재 코드는 `docs/prompt.md`의 Step 8 확장 후보였던 파일 기반 영속 저장소를 Step 12로 구현했다.

Step 12에서는 새 CLI 명령을 추가하지 않고, 기본 GlueSQL 저장소를 `MemoryStorage`에서 `SledStorage`로 갈아끼웠다.

## Step 11에서 Step 12로 달라진 점

| 구분 | Step 11 | Step 12 |
| --- | --- | --- |
| 활성 저장소 | GlueSQL `MemoryStorage` | GlueSQL `SledStorage` |
| CLI 데이터 유지 | 프로세스 종료 시 사라짐 | `data/rust-task-db`에 유지 |
| 새 명령 | 없음 | 없음 |
| 테스트 수 | 57개 | 58개 |
| 새 Rust/설계 개념 | `assert!(matches!(...))` | generic repository storage, 파일 기반 storage |

## 구현된 내용

| 파일 | 변경 |
| --- | --- |
| `Cargo.toml` | `gluesql_sled_storage` feature 추가 |
| `.gitignore` | `/data/` 추가 |
| `src/repository/gluesql_repository.rs` | `GlueSqlTaskRepository<S>`, `persistent(path)`, SledStorage 영속 테스트 추가 |
| `src/main.rs` | `GlueSqlTaskRepository::persistent("data/rust-task-db")` 사용 |
| `README.md` | Step 12 저장 방식과 실행 주의점 갱신 |

## 초심자가 이해해야 할 핵심

`MemoryStorage`는 프로그램이 끝나면 데이터가 사라진다. `SledStorage`는 디렉터리에 데이터를 저장하므로 `cargo run`을 여러 번 실행해도 Todo가 남는다.

```text
cargo run -- add "Rust 공부"
-> data/rust-task-db에 저장

cargo run -- list
-> data/rust-task-db에서 다시 읽어 출력
```

## 완료된 테스트

- SledStorage repository를 다시 만들어도 Todo가 남는지 확인하는 테스트 추가
- 총 58개 테스트 통과

## 다음 단계

현재 Step 12는 저장소 영속화까지 완료했다. 이후 작업은 명시적인 새 요구가 있을 때 별도 단계로 계획한다.
