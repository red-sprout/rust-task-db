# 파일별 상세 해설 인덱스

Step 28에서 추가된 `src/project.rs`, `src/tag.rs`와 이동된 `src/service/mod.rs`는 [21-relational-task-management.md](../21-relational-task-management.md)가 함수명과 수정 지점을 설명한다.

Step 40의 `src/query_lab/` 파일은 [Query Lab 개요](../../query-lab/00-overview.md)와 연결해 읽는다.

## 이 디렉터리의 목적

현재 Step 40 코드와 문서 파일을 하나씩 읽을 수 있게 나눈다. 기능 코드는 Step 12의 GlueSQL `SledStorage` 상태를 유지하고, Step 18 문서는 Storage별 기능 비교표를 추가한다.

## 파일별 해설 문서 목록

- [01-entrypoint.md](01-entrypoint.md): `src/main.rs`, `src/repl.rs`
- [02-domain-or-feature-files.md](02-domain-or-feature-files.md): `src/error.rs`, `src/service/mod.rs`, `src/repository/mod.rs`, `src/repository/gluesql_repository.rs`, `src/command.rs`, `src/cli.rs`, `src/task.rs`, `tasks.json`
- [03-global-and-common-files.md](03-global-and-common-files.md): 현재 공통 파일 없음
- [04-configuration-files.md](04-configuration-files.md): `Cargo.toml`, `Cargo.lock`
- [05-test-files.md](05-test-files.md): `src/main.rs`, `src/cli.rs` 내부 테스트
- [../17-gluesql-internals.md](../17-gluesql-internals.md): GlueSQL 내부 실행 흐름과 Storage Adapter 해설
- [../18-custom-storage.md](../18-custom-storage.md): Minimal Custom Storage trait 책임과 구현 순서
- [../19-query-execution.md](../19-query-execution.md): Todo 명령별 SQL 생성과 `Payload` 변환 흐름
- [../20-storage-comparison.md](../20-storage-comparison.md): Storage별 기능 차이와 현재 코드 도입 여부 비교

## 도메인별 파일 묶음

- 실행: [src/main.rs](../../../src/main.rs)
- REPL: [src/repl.rs](../../../src/repl.rs)
- 에러: [src/error.rs](../../../src/error.rs)
- 서비스: [src/service/mod.rs](../../../src/service/mod.rs)
- 저장소 trait와 JSON 보존 구현체: [src/repository/mod.rs](../../../src/repository/mod.rs)
- 현재 활성 SledStorage 저장소, MemoryStorage 테스트 흐름, SQL 실행: [src/repository/gluesql_repository.rs](../../../src/repository/gluesql_repository.rs)
- 명령 모델: [src/command.rs](../../../src/command.rs)
- CLI parser: [src/cli.rs](../../../src/cli.rs)
- Todo 데이터 모델: [src/task.rs](../../../src/task.rs)
- 보존된 JSON 저장 데이터: [tasks.json](../../../tasks.json)
- 실행 중 생성되는 SledStorage 데이터: `data/rust-task-db`
- GitHub 첫 화면 소개: [README.md](../../../README.md)
- 단계 진행 문서: [docs/todo/step-18-progress.md](../../todo/step-18-progress.md), [docs/todo/step-17-progress.md](../../todo/step-17-progress.md), [docs/todo/step-16-progress.md](../../todo/step-16-progress.md), [docs/todo/step-15-progress.md](../../todo/step-15-progress.md), [docs/todo/roadmap.md](../../todo/roadmap.md)
- 설정: [Cargo.toml](../../../Cargo.toml)
- 테스트: `src/main.rs`, `src/task.rs`, `src/error.rs`, `src/cli.rs`, `src/service/mod.rs`, `src/repl.rs`, `src/repository/mod.rs`, `src/repository/gluesql_repository.rs`의 `#[cfg(test)] mod tests`

## 초심자가 먼저 읽을 파일

1. [src/main.rs](../../../src/main.rs)
2. [src/error.rs](../../../src/error.rs)
3. [src/service/mod.rs](../../../src/service/mod.rs)
4. [src/repository/mod.rs](../../../src/repository/mod.rs)
5. [src/repository/gluesql_repository.rs](../../../src/repository/gluesql_repository.rs)

## 상세 해설 생략 기준

`target/`은 빌드 산출물이라 설명하지 않는다. `Cargo.lock`은 자동 생성 파일이므로 짧게만 설명한다.

## 다음에 읽을 문서

[06-language-from-code.md](../06-language-from-code.md)
