# Step 11 진행 상황: 테스트 추가

## 현재 상태

현재 코드는 `docs/prompt.md`의 `Step 11. 테스트 추가`까지 구현되어 있다.

Step 11에서는 새 기능을 추가하지 않고, 기존 Step 10 코드의 테스트를 촘촘히 보강했다.

## Step 10에서 Step 11로 달라진 점

| 구분 | Step 10 | Step 11 |
| --- | --- | --- |
| 기능 | REPL 모드 구현 | 새 기능 없음 |
| 테스트 수 | 46개 | 57개 |
| 보강 영역 | REPL 기본 테스트 | Task, CLI parser, REPL 에러/빈 줄, GlueSQL edge case |
| 새 Rust 개념 | `BufRead`, `Write`, `Cursor` | `assert!(matches!(...))` |

## 추가된 테스트

| 파일 | 추가된 검증 |
| --- | --- |
| `src/task.rs` | `Task::new`, `TaskStats::new` |
| `src/cli.rs` | help alias, add/done/search/sql 인자 부족 |
| `src/repl.rs` | 빈 줄 무시, SQL 에러 후 계속 실행 |
| `src/repository/gluesql_repository.rs` | delete 후 next id, invalid SQL이 `AppError::GlueSql`인지 |

## 완료된 테스트

- 총 57개 테스트 통과

## 다음 단계

현재 `docs/prompt.md`의 단계 구현은 Step 11까지 완료되어 있다. 이후 작업은 리팩터링, 문서 보강, 저장소 영속화 같은 새 요구가 있을 때 별도 단계로 계획한다.
