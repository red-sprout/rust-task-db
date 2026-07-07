# rust-task 단계별 로드맵

## Step 1. 메모리 기반 Todo

상태: 완료

범위:

- `Vec<Task>`
- `add`
- `list`
- `done`
- `delete`
- 기본 테스트

사용하지 않는 것:

- JSON
- Repository trait
- Service layer
- custom error
- GlueSQL

## Step 2. Command enum 도입

상태: 완료

완료된 일:

- `src/command.rs` 추가
- `Command` enum 추가
- `src/cli.rs` 추가
- CLI parsing을 `main.rs`에서 분리
- 잘못된 명령 처리 개선
- CLI parsing 단위 테스트 추가

## Step 3. JSON 파일 저장소

상태: 완료

완료된 일:

- `serde`, `serde_json` 추가
- `tasks.json` 추가
- 앱 시작 시 파일 읽기
- 명령 실행 후 파일 저장
- JSON/file I/O 실패 처리
- JSON 저장/로드 테스트 추가

## Step 4. Repository trait 도입

상태: 완료

완료된 일:

- `src/repository/mod.rs`
- `TaskRepository` trait
- `JsonTaskRepository`
- `main.rs`에서 repository trait 메서드 호출
- repository 단위 테스트 추가

## Step 5. Service layer 도입

상태: 완료

완료된 일:

- `src/service.rs`
- `TaskService<R: TaskRepository>`
- `main.rs`에서 service 메서드 호출
- service 단위 테스트 추가

## Step 6. Custom Error 도입

상태: 완료

완료된 일:

- `src/error.rs`
- `AppError`
- `Display`, `Error`, `From`
- CLI, service, repository 반환 타입을 `AppError`로 변경
- `AppError` 단위 테스트 추가

## Step 7. search와 stats 구현

상태: 완료

완료된 일:

- `search`
- `stats`
- `TaskStats`
- iterator/closure 기반 검색과 집계
- iterator/closure 설명

## Step 8. GlueSQL 저장소 추가

상태: 완료

완료된 일:

- GlueSQL dependency 추가
- `GlueSqlTaskRepository`
- `tasks` table 생성
- 기존 `JsonTaskRepository` 삭제 없이 보존
- `main.rs`에서 활성 repository를 GlueSQL 구현체로 교체
- GlueSQL `MemoryStorage` 기반 add/list/done/delete/search/stats 구현
- GlueSQL repository 단위 테스트 추가

## Step 9. SQL 실행 모드

상태: 완료

완료된 일:

- `sql` 명령
- GlueSQL 결과 출력
- `Command::Sql { sql }`
- `TaskRepository::execute_sql`
- `SqlResult`
- SELECT / INSERT / UPDATE / DELETE 결과 출력 구분
- SQL 실행 테스트 추가

## Step 10. REPL 모드

상태: 완료

완료된 일:

- `repl` 명령
- `.schema`, `.exit`, `.quit`
- 같은 REPL 세션 안에서 SQL 실행 결과 유지
- `src/repl.rs`
- REPL 단위 테스트 추가

## Step 11. 테스트 추가

상태: 완료

완료된 일:

- `Task::new` 기본값 테스트
- `TaskStats::new` 계산 테스트
- CLI parser help alias 테스트
- CLI parser 인자 부족 테스트
- REPL 빈 줄 무시 테스트
- REPL SQL 에러 후 계속 실행 테스트
- GlueSQL repository id 재사용 흐름 테스트
- GlueSQL invalid SQL 에러 타입 테스트

현재 총 57개 테스트가 존재한다.

## Step 12. GlueSQL SledStorage 영속 저장소 전환

상태: 완료

완료된 일:

- `gluesql_sled_storage` feature 활성화
- `GlueSqlTaskRepository<S>` generic 구조로 정리
- `GlueSqlTaskRepository::persistent(path)` 추가
- `main.rs` 기본 저장소를 `data/rust-task-db` 기반 SledStorage로 전환
- 기존 MemoryStorage 테스트 흐름 보존
- SledStorage 재실행 후 데이터 유지 테스트 추가

현재 총 58개 테스트가 존재한다.

## Step 13. 최종 검증 및 문서 정합성 점검

상태: 완료

범위:

- 새 CLI 명령 추가 없음
- 새 외부 crate 추가 없음
- Step 12의 GlueSQL `SledStorage` 활성 저장소 유지
- `JsonTaskRepository`, `tasks.json`, MemoryStorage 테스트 흐름 보존
- README, 단계 문서, 초심자 가이드 시작 문서가 현재 코드와 같은 상태를 설명하는지 점검

완료 기준:

- `cargo fmt --check` 통과
- `cargo check` 통과
- `cargo test` 통과
- 현재 총 58개 테스트 유지
