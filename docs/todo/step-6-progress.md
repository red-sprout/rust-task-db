# Step 6 진행 상황: Custom Error 도입

## 현재 상태

현재 코드는 `docs/prompt.md`의 `Step 6. Custom Error 도입`까지 구현되어 있다.

Step 6에서는 실패를 단순 `String`으로 표현하던 흐름을 `src/error.rs`의 `AppError`로 바꿨다.

## Step 5에서 Step 6으로 달라진 점

| 구분 | Step 5 | Step 6 |
| --- | --- | --- |
| 실패 타입 | `Err(String)` | `Err(AppError)` |
| CLI parsing 실패 | 문자열 메시지 | `AppError::InvalidCommand` |
| 없는 id | 문자열 `"Task not found: ..."` | `AppError::NotFound(id)` |
| 파일/JSON 실패 | 문자열 메시지 | `AppError::Io`, `AppError::Json` |
| 출력 | `eprintln!("{message}")` | `Display` 구현 덕분에 동일하게 출력 |

## 현재 파일

| 파일 | 역할 |
| --- | --- |
| `src/error.rs` | `AppError`, `Display`, `Error`, `From` 구현 |
| `src/cli.rs` | CLI parsing 실패를 `AppError::InvalidCommand`로 반환 |
| `src/service/mod.rs` | service 메서드가 `Result<_, AppError>` 반환 |
| `src/repository/mod.rs` | 파일/JSON/id 실패를 `AppError`로 반환 |
| `src/main.rs` | `AppError`를 출력 |

## 완료된 테스트

- CLI parser 테스트 7개
- error 테스트 3개
- main 흐름 보조 테스트 1개
- repository 테스트 7개
- service 테스트 4개
- 총 22개 테스트 통과

## 다음 단계

다음은 Step 7이다. Step 7에서는 `search`, `stats`를 추가한다.
