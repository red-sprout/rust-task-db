# Step 7 진행 상황: search와 stats 구현

## 현재 상태

현재 코드는 `docs/prompt.md`의 `Step 7. search와 stats 구현`까지 구현되어 있다.

Step 7에서는 CLI 명령에 `search`, `stats`가 추가되었다.

## Step 6에서 Step 7으로 달라진 점

| 구분 | Step 6 | Step 7 |
| --- | --- | --- |
| 지원 명령 | `add`, `list`, `done`, `delete` | `add`, `list`, `done`, `delete`, `search`, `stats` |
| 명령 enum | `Add`, `List`, `Done`, `Delete`, `Help` | `Search`, `Stats` 추가 |
| 데이터 모델 | `Task` | `Task`, `TaskStats` |
| repository | CRUD 중심 | `search`, `stats` 추가 |
| iterator 사용 | id 찾기, 출력 | `filter`, `count`, `collect`가 핵심 |

## 현재 파일

| 파일 | 역할 |
| --- | --- |
| `src/command.rs` | `Command::Search`, `Command::Stats` 추가 |
| `src/cli.rs` | `search`, `stats` parsing |
| `src/task.rs` | `TaskStats` 추가 |
| `src/service.rs` | `search`, `stats` service 메서드 |
| `src/repository/mod.rs` | 제목 검색과 통계 계산 |
| `src/main.rs` | search/stats 출력 |

## 현재 동작

```bash
cargo run -- search rust
cargo run -- stats
```

검색은 대소문자를 구분하지 않는다.

## 완료된 테스트

- CLI parser 테스트 9개
- error 테스트 3개
- main 흐름 보조 테스트 1개
- repository 테스트 9개
- service 테스트 6개
- 총 28개 테스트 통과

## 다음 단계

다음은 Step 8이다. Step 8에서는 GlueSQL 저장소를 추가한다.
