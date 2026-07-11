# Step 21 진행 상황: Project CLI와 Service 추가

## 현재 상태

Step 21에서는 Step 19의 Project repository 기능을 실제 사용자 CLI로 노출했다. `parse_args`는 문자열을 `Command`로 바꾸고, `main::run`은 `TaskService`를 통해 repository를 호출한다.

## Step 20에서 Step 21로 달라진 점

| 구분 | Step 20 | Step 21 |
| --- | --- | --- |
| Project 기능 | repository 내부 | CLI에서 직접 사용 가능 |
| Command | 기존 Task 계열 | Project variant 5개 추가 |
| Service | 기존 Todo 위임 | Project CRUD/통계 위임 추가 |
| 출력 | Task 중심 | Project와 완료율 출력 추가 |

## 추가한 명령

```bash
cargo run -- project add "GlueSQL 분석"
cargo run -- project list
cargo run -- project show 1
cargo run -- project delete 1
cargo run -- project stats 1
```

## 실제 실행 흐름

```text
"project stats 1"
-> cli::parse_args
-> parse_project
-> Command::ProjectStats { id: 1 }
-> main::run
-> TaskService::project_stats(1)
-> TaskManagementRepository::project_stats(1)
-> GlueSqlTaskRepository::project_stats(1)
-> ProjectStats 출력
```

## 코드 연결

| 파일 경로 | 실제 이름 | 역할 |
| --- | --- | --- |
| `src/command.rs` | `ProjectAdd`, `ProjectList`, `ProjectShow`, `ProjectDelete`, `ProjectStats` | 명령 데이터 표현 |
| `src/cli.rs` | `parse_project` | 하위 명령과 id parsing |
| `src/service/mod.rs` | Project 위임 메서드 5개 | CLI와 repository 분리 |
| `src/main.rs` | `run`의 Project match branch | 결과 출력과 오류 전달 |

`TaskService<R>` 자체에는 trait bound를 강제하지 않고, 기존 메서드는 `R: TaskRepository`, 관계형 메서드는 `R: TaskManagementRepository`인 별도 `impl` block에 둔다. 따라서 보존된 `JsonTaskRepository`도 기존 Service API를 계속 사용할 수 있다.

## Project 통계 출력

```text
1 | GlueSQL 분석
total: 4
done: 3
todo: 1
completion: 75.0%
```

완료율은 `ProjectStats::new`에서 total이 0이면 0.0, 아니면 `done / total * 100`으로 계산한다.

## 테스트 증거

- `cli::tests::parses_project_commands`
- `cli::tests::parses_project_delete`
- `project::tests::calculates_rate`
- 기존 `service::tests::*_delegates` 7개를 보존해 Service 하위 호환 확인

## 완료 기준

- Project 명령 5개 parsing과 실행 연결
- 잘못된 id와 알 수 없는 하위 명령 오류 처리
- Service가 repository 구현 세부사항을 노출하지 않음
- help와 README에 사용법 반영
- 최종 80개 테스트 통과

## 다음 단계

Step 22에서는 Tag와 Task-Tag 연결 table을 추가한다.
