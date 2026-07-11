# Step 22 진행 상황: Tag와 task_tags 추가

## 현재 상태

Step 22에서는 Task N:M Tag 관계의 두 번째 도메인인 `Tag`와 교차 table `task_tags`를 추가했다. Tag는 Query Lab 전용 더미가 아니라 CLI에서 생성·조회·삭제하고 Task에 연결할 실제 사용자 기능이다.

## Step 21에서 Step 22로 달라진 점

| 구분 | Step 21 | Step 22 |
| --- | --- | --- |
| 도메인 관계 | Project 1:N Task | Task N:M Tag 추가 |
| table | `projects`, `tasks` | `tags`, `task_tags` 추가 |
| 중복 정책 | 없음 | Tag 이름 대소문자 무관 중복 금지 |
| GlueSQL 제약 | 단일 PK/FK 사용 | 복합 PK 미지원 조정 필요 |

## 완료한 일

| 파일 경로 | 실제 타입/함수 | 역할 |
| --- | --- | --- |
| `src/tag.rs` | `Tag` | `id`, `name` 모델 |
| `src/repository/gluesql_repository.rs` | `add_tag`, `list_tags`, `delete_tag` | Tag CRUD |
| 같은 파일 | `find_tag` | `ILIKE`로 대소문자 무관 검색 |
| 같은 파일 | `create_tables` | `tags`, `task_tags` 생성 |

## table과 GlueSQL 지원 범위

```sql
CREATE TABLE IF NOT EXISTS tags (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS task_tags (
    task_id INTEGER,
    tag_id INTEGER
);
```

목표 schema의 `PRIMARY KEY (task_id, tag_id)`는 GlueSQL 0.19에서 `unsupported constraint`로 거부됐다. `task_tags`에 FK를 적용한 실험에서는 연결 row를 먼저 삭제한 뒤에도 부모 Task/Tag 삭제가 `referencing column exists`로 막히는 동작을 확인했다. 따라서 현재 지원 가능한 형태로 table을 단순화하고 다음 책임을 repository로 옮겼다.

- Task와 Tag 존재 확인
- 동일 `(task_id, tag_id)` COUNT 후 중복 거부
- Task/Tag 삭제 전 연결 row 명시적 삭제

## Tag 이름 중복 정책

```text
add_tag(" Backend ")
-> required_name으로 trim
-> find_tag("Backend")
-> WHERE name ILIKE 'Backend'
-> 이미 있으면 "tag name already exists"
```

UNIQUE 제약만으로는 대소문자 무관 중복을 표현하지 못하므로 애플리케이션 계층 검증을 선택했다.

## 테스트 증거

- `prevents_case_insensitive_duplicate_tags`
- `tag_names_are_trimmed`
- `deleting_task_or_tag_cleans_join_rows`
- `validates_project_and_task_fields`

## 완료 기준

- `Tag` 타입과 두 table 존재
- Tag 이름 공백/대소문자 중복 정책 구현
- GlueSQL 복합 PK/FK 조정 이유 문서화
- 새 외부 crate 없음
- 최종 80개 테스트 통과

## 다음 단계

Step 23에서는 Tag 관계를 `task tag`, `task untag`, `task tags` CLI로 노출한다.
