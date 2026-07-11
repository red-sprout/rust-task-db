# Step 27 진행 상황: Seed 기능 추가

## 현재 상태

Step 27에서는 Project/Task/Tag가 실제 기능인 상태에서 JOIN, aggregate, sort, filter 실험에 사용할 Seed를 추가했다. 별도 crate 없이 기존 repository 메서드를 재사용한다.

## Step 26에서 Step 27로 달라진 점

| 구분 | Step 26 | Step 27 |
| --- | --- | --- |
| 데이터 생성 | 사용자가 개별 명령 실행 | `seed` 한 번으로 대량 생성 |
| Project | 수동 | 10개 |
| Task | 수동 | 1,000개 |
| Tag | 수동 | 20개와 다수 연결 |
| 재실행 | 해당 없음 | version metadata 기반 idempotency |

## CLI와 실행 흐름

```bash
cargo run -- seed
```

```text
Command::Seed
-> main::run
-> TaskService::seed
-> TaskManagementRepository::seed
-> GlueSqlTaskRepository::seed
```

## 생성 규칙

```text
Project: Query Lab 01 ~ Query Lab 10
Tag: tag-01 ~ tag-20
Task: Seed task 0001 ~ Seed task 1000
Project 배분: Task 번호 modulo 10
priority: modulo 규칙으로 1~5
done: 3의 배수 Task 완료
Tag: 모든 Task 1개 이상, 4의 배수는 추가 Tag 연결
```

이 분포는 Project filter, priority sort, done aggregate, Tag JOIN을 한 데이터셋에서 관찰하게 한다.

## Idempotency 정책

`app_metadata`의 `seed_version = 1`이 완료 marker다. marker가 없으면 `cleanup_partial_seed`가 `Seed task %`, `tag-%`, `Query Lab %` 예약 데이터와 연결을 순서대로 정리한 뒤 다시 생성한다. SledStorage에서는 정리, 생성, 완료 marker 기록 전체가 하나의 transaction이므로 중간 실패 시 rollback된다. 예약 Project에 사용자가 만든 비-Seed Task가 있으면 삭제하지 않고 `seed project contains non-seed tasks`로 중단한다.

## ID와 성능

각 생성은 `allocate_id`가 `id_sequences`의 다음 값을 할당한다. 저장된 sequence가 raw SQL로 추가된 최대 id보다 작으면 `MAX(id) + 1` 이상으로 보정한다. Sled에서는 할당과 INSERT가 Seed transaction 안에 함께 있다.

## 테스트 증거

`seed_creates_expected_counts_and_is_idempotent`는 MemoryStorage에서 `seed()`를 두 번 호출한 뒤 다음을 확인한다.

- Project 10개
- Task 1,000개
- Tag 20개
- `task_tags` 1,000개 초과
- 두 번째 실행 후 개수 불변
- `seed_recovers_reserved_partial_data_before_rebuilding`에서 불완전한 예약 데이터를 정확한 10/1,000/20 상태로 재구성

## 완료 기준

- `seed` CLI 연결
- 최소 요구 개수와 다양한 priority/done/관계 생성
- 성공 후 재실행 중복 방지
- 대규모 확장 지점과 한계 문서화
- 최종 80개 테스트 통과

## 다음 단계

Step 28에서는 전체 코드, 테스트, README, 진행 문서와 초심자 가이드 정합성을 검증한다.
