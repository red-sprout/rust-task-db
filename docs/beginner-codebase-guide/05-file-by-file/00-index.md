# 파일별 상세 해설 인덱스

## 이 디렉터리의 목적

현재 Step 11 코드 파일을 하나씩 읽을 수 있게 나눈다.

## 파일별 해설 문서 목록

- `01-entrypoint.md`: `src/main.rs`, `src/repl.rs`
- `02-domain-or-feature-files.md`: `src/error.rs`, `src/service.rs`, `src/repository/mod.rs`, `src/repository/gluesql_repository.rs`, `src/command.rs`, `src/cli.rs`, `src/task.rs`, `tasks.json`
- `03-global-and-common-files.md`: 현재 공통 파일 없음
- `04-configuration-files.md`: `Cargo.toml`, `Cargo.lock`
- `05-test-files.md`: `src/main.rs`, `src/cli.rs` 내부 테스트

## 도메인별 파일 묶음

- 실행: `src/main.rs`
- REPL: `src/repl.rs`
- 에러: `src/error.rs`
- 서비스: `src/service.rs`
- 저장소 trait와 JSON 보존 구현체: `src/repository/mod.rs`
- 현재 활성 GlueSQL 저장소와 SQL 실행: `src/repository/gluesql_repository.rs`
- 명령 모델: `src/command.rs`
- CLI parser: `src/cli.rs`
- Todo 데이터 모델: `src/task.rs`
- 보존된 JSON 저장 데이터: `tasks.json`
- GitHub 첫 화면 소개: `README.md`
- 설정: `Cargo.toml`
- 테스트: `src/main.rs`, `src/error.rs`, `src/cli.rs`, `src/service.rs`, `src/repository/mod.rs`, `src/repository/gluesql_repository.rs`의 `#[cfg(test)] mod tests`

## 초심자가 먼저 읽을 파일

1. `src/main.rs`
2. `src/error.rs`
3. `src/service.rs`
4. `src/repository/mod.rs`
5. `src/repository/gluesql_repository.rs`

## 상세 해설 생략 기준

`target/`은 빌드 산출물이라 설명하지 않는다. `Cargo.lock`은 자동 생성 파일이므로 짧게만 설명한다.

## 다음에 읽을 문서

`06-language-from-code.md`
