# Step 3 진행 상황: JSON 파일 저장소

## 현재 상태

현재 코드는 `docs/prompt.md`의 `Step 3. JSON 파일 저장소`까지 구현되어 있다.

Step 3에서는 Todo 목록을 `tasks.json`에 저장한다. 이제 `cargo run -- add "Rust 공부"`로 추가한 Todo가 다음 `cargo run -- list`에서 다시 보인다.

## 현재 지원 명령

```bash
cargo run -- add "Rust 공부"
cargo run -- list
cargo run -- done 1
cargo run -- delete 1
```

## 현재 파일

| 파일 | 역할 |
| --- | --- |
| `src/main.rs` | `tasks.json` 읽기/쓰기, `Command` 실행, Todo 조작 |
| `src/task.rs` | `Task` struct, `Task::new`, `Serialize`, `Deserialize` derive |
| `src/command.rs` | CLI 명령을 표현하는 `Command` enum |
| `src/cli.rs` | CLI 인자를 `Result<Command, String>`으로 변환 |
| `tasks.json` | Todo 데이터 저장 파일 |
| `Cargo.toml` | `serde`, `serde_json` dependency 선언 |

## 구현된 함수와 타입

| 이름 | 위치 | 역할 |
| --- | --- | --- |
| `TASKS_FILE` | `src/main.rs` | 기본 저장 파일 경로 |
| `load_tasks` | `src/main.rs` | `tasks.json`을 읽어 `Vec<Task>`로 변환 |
| `save_tasks` | `src/main.rs` | `Vec<Task>`를 JSON 문자열로 바꿔 파일에 저장 |
| `save_tasks` 호출부 | `src/main.rs` | 저장 실패 시 메시지 출력 후 현재 명령 종료 |
| `Task` | `src/task.rs` | JSON으로 저장 가능한 Todo 데이터 |

## Step 3에서 배우는 Rust 개념

- `Result`: 파일 읽기/JSON parsing 실패 처리
- `?`: JSON 직렬화 실패 전파
- `std::fs`: `read_to_string`, `write`
- `std::io::ErrorKind::NotFound`: 파일이 없을 때 빈 Vec 반환
- `serde` derive: `Serialize`, `Deserialize`
- `impl AsRef<Path>`: 문자열과 `PathBuf`를 모두 받을 수 있는 path 인자

## 현재 제한

아직 Repository trait와 Service layer는 없다. Step 4에서 저장소 책임을 별도 trait로 분리한다.

## 완료된 테스트

- Step 1 메모리 로직 테스트 7개
- Step 2 CLI parser 테스트 7개
- Step 3 JSON 저장/로드 테스트 3개
- 총 17개 테스트 통과

## 다음 단계

다음은 Step 4다. Step 4에서는 `TaskRepository` trait와 repository 디렉터리를 추가한다.
