# Step 1 진행 상황: 메모리 기반 Todo

## 현재 상태

Step 1은 완료되었다. 현재 코드는 이후 Step 2까지 진행되었지만, 이 문서는 Step 1에서 완성한 메모리 기반 Todo 범위를 기록한다.

Step 1에서는 DB, JSON 파일, Repository trait, Service layer, GlueSQL을 사용하지 않는다. 오직 메모리의 `Vec<Task>`만 사용한다.

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
| `src/main.rs` | `Vec<Task>` 조작, 출력, 메모리 로직 테스트 |
| `src/task.rs` | `Task` struct와 `Task::new` |
| `Cargo.toml` | package 설정. 현재 외부 dependency 없음 |

## 구현된 함수

| 함수 | 위치 | 역할 |
| --- | --- | --- |
| `Task::new` | `src/task.rs` | 새 Todo 생성 |
| `add_task` | `src/main.rs` | `Vec<Task>`에 Todo 추가 |
| `mark_done` | `src/main.rs` | 특정 id의 Todo 완료 처리 |
| `delete_task` | `src/main.rs` | 특정 id의 Todo 삭제 |
| `next_id` | `src/main.rs` | 다음 id 계산 |
| `parse_id` | `src/main.rs` | 문자열 id를 `i64`로 변환 |

## Step 1에서 배우는 Rust 개념

- `struct`: `Task`
- `impl`: `Task::new`
- `Vec`: `Vec<Task>`
- ownership: `title: String`을 `Task::new`로 전달
- borrowing: `print_task(task: &Task)`
- mutable reference: `add_task(tasks: &mut Vec<Task>, ...)`
- `Option`: `mark_done`, `delete_task`, `parse_id`
- `match`: CLI 명령 분기와 `Option` 처리
- iterator: `iter`, `iter_mut`, `map`, `max`, `position`, `find`
- closure: `|task| task.id == id`

## 현재 제한

메모리 저장만 사용하므로 명령을 한 번 실행하면 데이터가 사라진다.

```bash
cargo run -- add "Rust 공부"
cargo run -- list
```

위 두 명령은 서로 다른 프로세스다. 따라서 `list`는 앞의 `add` 결과를 볼 수 없다. 이 제한은 Step 3 JSON 파일 저장소에서 해결한다.

## 완료된 테스트

- `Task` 생성 테스트
- add 테스트
- next id 테스트
- done 테스트
- 없는 id done 처리 테스트
- delete 테스트
- 없는 id delete 처리 테스트

## Step 1 완료 기준

- `cargo fmt --check` 통과
- `cargo test` 통과
- `cargo run -- add "Rust 공부"` 실행 가능
- `cargo run -- list` 실행 가능
- `cargo run -- done 1` 실행 가능
- `cargo run -- delete 1` 실행 가능

## 다음 단계

다음 단계였던 Step 2는 완료되었다. 현재 진행 상태는 `docs/todo/step-2-progress.md`를 본다.
