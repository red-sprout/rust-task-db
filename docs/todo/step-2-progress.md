# Step 2 진행 상황: Command enum 도입

## 현재 상태

현재 코드는 `docs/prompt.md`의 `Step 2. Command enum 도입`까지 구현되어 있다.

Step 2에서는 CLI 문자열을 `src/main.rs`에서 바로 처리하지 않고, `src/cli.rs`가 `Command` enum으로 변환한다. 저장 방식은 아직 Step 1과 같아서 프로세스 안의 `Vec<Task>`만 사용한다.

## 현재 지원 명령

```bash
cargo run -- add "Rust 공부"
cargo run -- list
cargo run -- done 1
cargo run -- delete 1
```

잘못된 입력 확인:

```bash
cargo run -- done abc
cargo run -- unknown
```

## 현재 파일

| 파일 | 역할 |
| --- | --- |
| `src/main.rs` | `Command`를 실행하고 `Vec<Task>`를 조작한다. |
| `src/command.rs` | `Command` enum으로 CLI 명령의 종류와 필요한 값을 표현한다. |
| `src/cli.rs` | `Vec<String>` CLI 인자를 `Result<Command, String>`으로 변환한다. |
| `src/task.rs` | `Task` struct와 `Task::new`를 정의한다. |
| `Cargo.toml` | package 설정. 현재 외부 dependency 없음 |

## 구현된 함수와 타입

| 이름 | 위치 | 역할 |
| --- | --- | --- |
| `Command` | `src/command.rs` | `Add`, `List`, `Done`, `Delete`, `Help` 명령 표현 |
| `parse_args` | `src/cli.rs` | CLI 인자를 `Command`로 변환 |
| `require_next` | `src/cli.rs` | 다음 CLI 인자가 없으면 사용법 문자열을 에러로 반환 |
| `parse_id` | `src/cli.rs` | 문자열 id를 `i64`로 변환 |
| `add_task` | `src/main.rs` | `Vec<Task>`에 Todo 추가 |
| `mark_done` | `src/main.rs` | 특정 id의 Todo 완료 처리 |
| `delete_task` | `src/main.rs` | 특정 id의 Todo 삭제 |

## Step 2에서 배우는 Rust 개념

- `enum`: `Command`
- struct-like enum variant: `Command::Add { title }`
- `Result`: `parse_args`의 성공/실패
- `?`: 실패를 만나면 현재 함수에서 바로 반환
- iterator: `args.into_iter()`, `iter.next()`
- ownership: CLI 인자 `String`을 `Command` 안으로 이동
- `match`: `Command` 종류별 실행 분기

## 현재 제한

Step 2도 메모리 저장만 사용한다. `cargo run -- add "Rust 공부"`로 추가한 Todo는 다음 `cargo run -- list`에서 남아 있지 않다.

이 제한은 Step 3 JSON 파일 저장소에서 해결한다.

## 완료된 테스트

- Step 1 메모리 로직 테스트 7개
- Step 2 CLI parser 테스트 7개
- 총 14개 테스트 통과

## Step 2 완료 기준

- `cargo fmt --check` 통과
- `cargo check` 통과
- `cargo test` 통과
- `cargo run -- add "Rust 공부"` 실행 가능
- `cargo run -- list` 실행 가능
- `cargo run -- done abc`에서 id 에러 출력
- `cargo run -- unknown`에서 unknown command 에러 출력

## 다음 단계

다음은 Step 3이다. Step 3에서는 `serde`, `serde_json`, `tasks.json`을 추가해서 실행 사이에 Todo가 유지되게 만든다.
