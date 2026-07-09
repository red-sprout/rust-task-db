# Step 16 진행 상황: Minimal Custom Storage 분석

## 현재 상태

Step 16에서는 새 CLI 명령이나 새 외부 crate를 추가하지 않는다. 실제 custom storage를 production code에 붙이지 않고, GlueSQL Storage Adapter를 직접 만들 때 필요한 최소 책임을 현재 코드와 연결해 문서화한다.

## Step 15에서 Step 16으로 달라진 점

| 구분 | Step 15 | Step 16 |
| --- | --- | --- |
| 활성 저장소 | GlueSQL `SledStorage` | GlueSQL `SledStorage` |
| 새 CLI 명령 | 없음 | 없음 |
| 새 외부 crate | 없음 | 없음 |
| 핵심 작업 | Engine/Storage Adapter 분석 | Minimal Custom Storage 구현 책임 분석 |
| 테스트 수 | 65개 | 65개 |

## 완료된 일

| 항목 | 내용 |
| --- | --- |
| 새 문서 | `docs/beginner-codebase-guide/18-custom-storage.md` |
| 읽기 책임 | SELECT를 위해 필요한 schema/data 조회 책임을 설명 |
| 쓰기 책임 | INSERT/UPDATE/DELETE를 위해 필요한 schema/data 변경 책임을 설명 |
| 현재 코드 연결 | `GlueSqlTaskRepository<S>`의 `GStore + GStoreMut + Planner` 조건과 연결 |
| 범위 제한 | production custom storage와 새 crate는 이후 단계 예정으로 표시 |

## 초심자가 이해해야 할 핵심

GlueSQL custom storage는 SQL 문법을 새로 만드는 일이 아니다. SQL parsing, planning, expression 평가, executor 흐름은 GlueSQL이 맡고, custom storage는 executor가 요구하는 schema와 row 읽기/쓰기 인터페이스를 제공한다.

```text
GlueSQL Engine
-> Store trait 계열 호출
-> Custom Storage
-> schema/data 읽기 또는 쓰기
```

현재 프로젝트의 `TaskRepository`는 앱 입장에서의 저장소 추상화이고, GlueSQL의 `Store`/`StoreMut`은 SQL engine 입장에서의 storage adapter 계약이다. 두 trait 계층은 역할이 다르다.

## 완료 기준

- `cargo fmt --check` 통과
- `cargo check` 통과
- `cargo test` 통과
- README, AGENTS, roadmap, 초심자 가이드가 Step 16과 65개 테스트를 일관되게 설명

## 다음 단계

Step 17에서는 현재 Todo SQL을 기준으로 Query Execution 흐름을 상세 분석한다.
