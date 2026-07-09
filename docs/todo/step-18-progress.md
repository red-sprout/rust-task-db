# Step 18 진행 상황: Storage별 기능 비교표 고도화

## 현재 상태

Step 18에서는 새 CLI 명령이나 새 외부 crate를 추가하지 않는다. 현재 프로젝트에서 직접 사용하는 `JsonTaskRepository`, GlueSQL `MemoryStorage`, GlueSQL `SledStorage`와 문서 비교 대상으로 보는 `SharedMemoryStorage`, `JsonStorage`, `MongoStorage`, `CompositeStorage`의 차이를 정리한다.

## Step 17에서 Step 18로 달라진 점

| 구분 | Step 17 | Step 18 |
| --- | --- | --- |
| 핵심 작업 | Query Execution 상세 분석 | Storage별 기능 비교표 고도화 |
| 코드 변경 | 없음 | 없음 |
| 새 CLI 명령 | 없음 | 없음 |
| 새 외부 crate | 없음 | 없음 |
| 테스트 수 | 65개 | 65개 |

## 완료한 일

| 항목 | 내용 |
| --- | --- |
| 새 문서 | `docs/beginner-codebase-guide/20-storage-comparison.md` |
| 비교 대상 | `JsonTaskRepository`, `MemoryStorage`, `SledStorage`, `SharedMemoryStorage`, `JsonStorage`, `MongoStorage`, `CompositeStorage` |
| 비교 축 | 현재 사용 여부, 영속성, 동시 접근, transaction, SQL 실행, 학습 포인트 |
| 코드 증거 | `JsonTaskRepository::execute_sql`, `GlueSqlTaskRepository::new`, `GlueSqlTaskRepository::persistent` |
| 문서 갱신 | README, AGENTS, roadmap, 초심자 가이드 인덱스와 glossary 갱신 |

## 핵심 개념

현재 프로젝트에서 실제로 연결된 저장소와 분석 후보 storage를 구분해야 한다.

```text
실제 코드 사용
-> JsonTaskRepository
-> MemoryStorage 테스트
-> SledStorage CLI 기본 저장소

문서 비교 대상
-> SharedMemoryStorage
-> JsonStorage
-> MongoStorage
-> CompositeStorage
```

## 완료 기준

- `docs/beginner-codebase-guide/20-storage-comparison.md`가 storage별 차이를 표로 설명
- 현재 코드에 없는 storage는 "코드에서 확인되지 않음"으로 표시
- README, AGENTS, roadmap, 초심자 가이드가 Step 18과 65개 테스트를 일관되게 설명
- `cargo fmt --check` 통과
- `cargo check` 통과
- `cargo test` 통과

## 다음 단계

사용자가 16, 17, 18까지만 진행한다고 했으므로 다음 구현 단계는 여기서 멈춘다. 이후 단계는 별도 요청이 있을 때 새 계획으로 다룬다.
