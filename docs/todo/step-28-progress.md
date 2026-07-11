# Step 28 진행 상황: 테스트와 문서 정리

## 현재 상태

Step 28에서는 Step 19~27에서 추가한 Project/Task/Tag 기능을 요구사항별로 감사하고, 기존 Todo/JSON/SQL/REPL/Storage 테스트를 유지하면서 문서를 현재 코드와 맞췄다. 현재 활성 저장소는 계속 `GlueSqlTaskRepository<SledStorage>`이며 새 외부 crate는 없다.

## Step 27에서 Step 28로 달라진 점

| 구분 | Step 27 | Step 28 |
| --- | --- | --- |
| 핵심 작업 | Seed 구현 | 전체 테스트와 문서 정합성 |
| 기능 범위 | 관계형 기능 완성 | 회귀/정책/제약 증거 정리 |
| 테스트 | Seed 테스트 추가 중 | 최종 80개 통과 |
| 문서 | 단계별 일부 반영 | README, roadmap, progress, 초심자 가이드 갱신 |

## 최종 코드 구조

```text
src/
├── project.rs
├── task.rs
├── tag.rs
├── command.rs
├── cli.rs
├── service/
│   └── mod.rs
└── repository/
    ├── mod.rs
    └── gluesql_repository.rs
```

작은 코드베이스에 맞춰 domain별 Service/Repository 파일을 무조건 나누지 않았다. `TaskManagementRepository` 하나가 Project/Task/Tag 교차 작업을 담당하고, `TaskService<R>`가 CLI와 저장소를 분리한다. 기존 `TaskRepository`와 `JsonTaskRepository`는 보존했다.

## 요구사항별 완료 증거

| 요구사항 | 실제 코드/문서 | 테스트 증거 |
| --- | --- | --- |
| Project CRUD/검증 | `add_project`, `show_project`, `delete_project` | `manages_projects...`, `deletes_empty_project`, `validates...` |
| Project 소속 Task | `Task::with_project`, `add_task` | `lists_project_tasks...` |
| priority 정렬 | `list_tasks`의 ORDER BY | 같은 테스트에서 `[5, 1]` 확인 |
| Project 통계 | `project_stats`, `ProjectStats::new` | 통계/빈 완료율 테스트 |
| Tag 중복 | `find_tag` + ILIKE | `prevents_case_insensitive_duplicate_tags` |
| 연결/해제/filter | `tag_task`, `untag_task`, `list_tasks` | `tags_untags_and_lists_tasks_by_tag` |
| Task 상세 | `TaskDetail`, `show_task` | `task_detail_includes_project_and_tags` |
| 삭제 정리 | `delete`, `delete_tag` | `deleting_task_or_tag_cleans_join_rows` |
| Project 삭제 제한 | `delete_project` | `project has tasks` 검증 |
| Seed | `seed` | 개수와 idempotency 테스트 |
| 부분 Seed 복구 | `cleanup_partial_seed`, `app_metadata` | 예약 데이터 재구성 테스트 |
| sequence rollback | `allocate_id`, `transaction` | rollback 후 id 재할당 테스트 |
| 기존 기능 | `TaskRepository`, 기존 CLI variant | JSON/REPL/SQL/Service/Storage 회귀 테스트 |

## 문서 변경

| 문서 | 반영 내용 |
| --- | --- |
| `README.md` | 도메인 관계, table, 전체 CLI, 삭제/중복/ID/transaction/Seed 정책 |
| `AGENTS.md` | 현재 단계를 Step 28로 전환하고 GlueSQL 제약 기록 |
| `docs/todo/roadmap.md` | Step 19~28 완료 상태 |
| `docs/todo/step-19-progress.md` ~ `step-28-progress.md` | 단계별 상세 진행 기록 |
| `docs/beginner-codebase-guide/21-relational-task-management.md` | 실제 함수와 SQL 중심 종합 해설 |
| 기존 초심자 가이드 | 경로, 데이터 모델, 실행 흐름, 테스트, FAQ 갱신 |

## 최종 정책 요약

- 기존 `add`는 `project_id = NULL`, priority 3으로 유지한다.
- Tag 이름은 trim 후 대소문자 무관 중복 금지다.
- Task가 있는 Project는 삭제하지 않는다.
- Task/Tag 삭제 전에 `task_tags`를 정리한다.
- ID는 `id_sequences`로 할당하며 Sled에서는 할당과 INSERT가 같은 transaction이다.
- GlueSQL 0.19의 복합 PK 및 교차 table FK 삭제 제약은 애플리케이션 검증으로 조정했다.
- MemoryStorage는 비transaction closure, SledStorage는 최상위 `BEGIN/COMMIT/ROLLBACK` 경계를 사용한다.
- Seed는 `app_metadata.seed_version`과 부분 데이터 정리로 재시작 가능하다.
- `project stats`는 전체 Project LEFT JOIN aggregate를 지원한다.
- `task add --tag ...`는 Task와 Tag 연결을 원자적으로 생성한다.

## 최종 검증 결과

```text
cargo fmt --check -> 통과
cargo check       -> 통과
cargo test        -> 80 passed, 0 failed
git diff --check  -> 통과
```

Seed 테스트는 1,000 Task를 실제 MemoryStorage에 생성해 개수와 재실행 불변성을 확인한다. 기존 SledStorage rollback/commit/snapshot/write-lock 테스트도 유지된다.

## 완료 기준

- Step 19~28 명시 기능과 정책 구현
- 기존 Todo, JSON repository, SQL, REPL, storage 테스트 유지
- 실제 코드 경로·함수명·SQL이 문서와 일치
- README, roadmap, progress, 초심자 가이드 현재 단계 일치
- 세 Cargo 검증과 diff 검사 통과

## GlueSQL upstream 의존 제약

- GlueSQL 0.19는 `PRIMARY KEY (task_id, tag_id)` 복합 PK를 거부한다.
- 교차 table FK는 연결 삭제 후에도 부모 삭제를 막는 동작이 관찰되어 현재 애플리케이션 제약을 유지한다.
- 이 두 항목은 프로젝트 내부 우회 구현이 완료되어 있으며, DB 제약 자체의 재도입은 GlueSQL upstream 지원이 생길 때 검토한다.

## 다음 단계

Step 28로 sequence, transaction, atomic Task-Tag 생성, 전체 aggregate, versioned Seed까지 포함한 관계형 Task Management 확장을 완료한다.
