# Step 5 진행 상황: Service layer 도입

## 현재 상태

현재 코드는 `docs/prompt.md`의 `Step 5. Service layer 도입`까지 구현되어 있다.

Step 5에서는 `src/main.rs`가 repository를 직접 호출하지 않고 `src/service.rs`의 `TaskService<R: TaskRepository>`를 호출하도록 바꿨다.

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
| `src/main.rs` | CLI 명령을 실행하고 service 메서드를 호출한다. |
| `src/service.rs` | `TaskService<R: TaskRepository>`와 service 테스트 |
| `src/repository/mod.rs` | `TaskRepository` trait, `JsonTaskRepository`, JSON 파일 저장/로드 |
| `src/task.rs` | `Task` struct와 serde derive |
| `src/command.rs` | CLI 명령을 표현하는 `Command` enum |
| `src/cli.rs` | CLI 인자를 `Command`로 변환 |
| `tasks.json` | Todo 데이터 저장 파일 |

## 구현된 함수와 타입

| 이름 | 위치 | 역할 |
| --- | --- | --- |
| `TaskService<R: TaskRepository>` | `src/service.rs` | repository에 의존하는 service layer |
| `TaskService::new` | `src/service.rs` | repository를 받아 service 생성 |
| `TaskService::add` | `src/service.rs` | Todo 추가 요청을 repository에 위임 |
| `TaskService::list` | `src/service.rs` | Todo 목록 조회를 repository에 위임 |
| `TaskService::done` | `src/service.rs` | Todo 완료 처리를 repository에 위임 |
| `TaskService::delete` | `src/service.rs` | Todo 삭제를 repository에 위임 |

## Step 5에서 배우는 Rust 개념

- generic struct: `TaskService<R: TaskRepository>`
- trait bound: `R: TaskRepository`
- 의존성 분리: `main.rs`가 repository 세부사항 대신 service를 호출
- 테스트용 fake repository 구현
- service layer와 repository layer의 역할 구분

## 현재 제한

아직 custom error는 없다. Step 6에서 `AppError`를 추가한다.

아직 `search`, `stats`는 없다. Step 7에서 추가한다.

## 완료된 테스트

- CLI parser 테스트 7개
- main 흐름 보조 테스트 1개
- repository 테스트 7개
- service 테스트 4개
- 총 19개 테스트 통과

## 다음 단계

다음은 Step 6이다. Step 6에서는 `src/error.rs`와 `AppError`를 추가한다.
