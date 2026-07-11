# Step 19 진행 상황: Project 도메인과 테이블 추가

## 현재 상태

Step 19에서는 기존 Todo 중심 모델에 `Project`를 실제 사용자 도메인으로 추가했다. 새 외부 crate는 추가하지 않았고, 활성 저장소는 계속 `GlueSqlTaskRepository<SledStorage>`다. Project는 이후 Task가 선택적으로 소속될 상위 엔터티다.

## Step 18에서 Step 19로 달라진 점

| 구분 | Step 18 | Step 19 |
| --- | --- | --- |
| 핵심 작업 | Storage 기능 비교 문서 | Project 도메인과 `projects` table |
| 도메인 타입 | `Task`, `TaskStats` | `Project`, `ProjectStats` 추가 |
| GlueSQL table | `tasks` | `projects` 추가 |
| 사용자 기능 | 기존 Todo 명령 | Project repository 기능의 기반 추가 |
| 외부 crate | 기존 4개 | 추가 없음 |

## 완료한 일

| 파일 경로 | 실제 타입/함수 | 역할 |
| --- | --- | --- |
| `src/project.rs` | `Project` | `id`, `name`을 가진 Project 모델 |
| `src/project.rs` | `ProjectStats::new` | total/done에서 todo와 완료율 계산 |
| `src/repository/mod.rs` | `TaskManagementRepository` | 관계형 기능을 한 repository 경계로 묶음 |
| `src/repository/gluesql_repository.rs` | `create_tables` | `projects` table 생성 |
| `src/repository/gluesql_repository.rs` | `add_project`, `list_projects`, `show_project`, `delete_project` | Project CRUD SQL 실행 |

## 핵심 코드와 해석

```sql
CREATE TABLE IF NOT EXISTS projects (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL
);
```

- `id INTEGER PRIMARY KEY`는 GlueSQL이 지원하는 단일 열 PK다.
- `name TEXT NOT NULL`은 Project 이름이 NULL인 row를 막는다.
- 공백 문자열은 DB의 `NOT NULL`로 막을 수 없으므로 `required_name(name, "project")`가 애플리케이션 계층에서 검증한다.

```text
add_project(name)
-> required_name
-> select_max_id("projects") + 1
-> INSERT INTO projects
-> Project 반환
```

## Repository 구조 선택

Project마다 별도 trait을 만드는 방식 A 대신 방식 B인 `TaskManagementRepository`를 선택했다. 현재 프로젝트 규모가 작고, 이후 Project/Task/Tag를 함께 다루는 상세 조회와 삭제 정책이 하나의 GlueSQL 인스턴스를 공유하기 때문이다. 기존 `TaskRepository`는 `JsonTaskRepository` 하위 호환을 위해 삭제하지 않았다.

## 테스트 증거

- `manages_projects_and_rejects_delete_when_tasks_exist`: 생성과 단건 조회
- `validates_project_and_task_fields`: 공백 Project 이름 거부
- `deletes_empty_project`: Task가 없는 Project 삭제
- `project::tests::calculates_rate`, `empty_rate_is_zero`: 통계 값 계산

최종 `cargo test`에서는 전체 80개 테스트가 통과한다.

## 완료 기준

- `Project`, `ProjectStats`가 실제 Rust 타입으로 존재
- `projects` table이 repository 초기화 시 생성
- Project 생성/목록/단건/삭제가 SQL로 구현
- 이름 검증과 기본 테스트 존재
- `cargo fmt --check`, `cargo check`, `cargo test` 통과

## 다음 단계

Step 20에서는 `Task`에 `project_id`와 `priority`를 추가하고 기존 JSON/Sled 데이터를 호환한다.
