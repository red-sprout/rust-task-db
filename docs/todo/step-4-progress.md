# Step 4 진행 상황: Repository trait 도입

## 현재 상태

현재 코드는 `docs/prompt.md`의 `Step 4. Repository trait 도입`까지 구현되어 있다.

Step 4에서는 `tasks.json` 파일 읽기/쓰기 책임을 `src/main.rs`에서 `src/repository/mod.rs`의 `JsonTaskRepository`로 옮겼다.

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
| `src/main.rs` | CLI 명령을 실행하고 repository 메서드를 호출한다. |
| `src/repository/mod.rs` | `TaskRepository` trait, `JsonTaskRepository`, JSON 파일 저장/로드 |
| `src/task.rs` | `Task` struct와 serde derive |
| `src/command.rs` | CLI 명령을 표현하는 `Command` enum |
| `src/cli.rs` | CLI 인자를 `Command`로 변환 |
| `tasks.json` | Todo 데이터 저장 파일 |

## 구현된 함수와 타입

| 이름 | 위치 | 역할 |
| --- | --- | --- |
| `TaskRepository` | `src/repository/mod.rs` | 저장소가 제공해야 하는 동작 목록 |
| `JsonTaskRepository` | `src/repository/mod.rs` | `tasks.json` 기반 저장소 구현체 |
| `JsonTaskRepository::new` | `src/repository/mod.rs` | 파일을 읽어 repository 생성 |
| `TaskRepository::add` | `src/repository/mod.rs` | Todo 추가 후 저장 |
| `TaskRepository::find_all` | `src/repository/mod.rs` | Todo 목록 조회 |
| `TaskRepository::mark_done` | `src/repository/mod.rs` | Todo 완료 처리 후 저장 |
| `TaskRepository::delete` | `src/repository/mod.rs` | Todo 삭제 후 저장 |

## Step 4에서 배우는 Rust 개념

- `trait`: 공통 동작 약속
- `impl Trait for Type`: `JsonTaskRepository`가 `TaskRepository`를 구현
- `&mut self`: 저장소 내부 상태를 바꾸는 메서드
- 의존성 분리: `main.rs`가 파일 I/O 세부사항을 직접 다루지 않음
- `PathBuf`: repository가 저장 파일 경로를 소유

## 현재 제한

아직 Service layer는 없다. `main.rs`가 repository를 직접 호출한다. Step 5에서 `TaskService<R: TaskRepository>`를 추가한다.

Custom error도 아직 없다. Step 6에서 `AppError`를 추가한다.

## 완료된 테스트

- CLI parser 테스트 7개
- main 흐름 보조 테스트 1개
- repository 테스트 7개
- 총 15개 테스트 통과

## 다음 단계

다음은 Step 5다. Step 5에서는 `TaskService<R: TaskRepository>`를 추가한다.
