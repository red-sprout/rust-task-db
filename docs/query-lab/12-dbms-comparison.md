# DBMS 비교

| 기준 | GlueSQL 0.19 | PostgreSQL | MySQL InnoDB | Oracle |
| --- | --- | --- | --- | --- |
| Plan 표현 | planned AST Statement | logical/physical plan tree | EXPLAIN iterator/tree | execution plan rows/tree |
| Access Path | PK/NonClustered AST slot | Seq/Index/Bitmap | ALL/ref/range/index | TABLE/INDEX access |
| Join Strategy | NestedLoop/Hash enum | Nested/Hash/Merge | Nested loop 계열, hash 일부 | Nested/Hash/Merge |
| Cardinality/Cost | 공개 plan에 없음 | rows/cost | rows/cost | cardinality/cost |
| Actual Rows | 공개 operator metric 없음 | EXPLAIN ANALYZE | EXPLAIN ANALYZE | DBMS_XPLAN/AUTOTRACE |
| Storage 경계 | Store/Index trait | heap/index access methods 내부 | InnoDB engine | kernel/storage 내부 |

다른 DBMS 항목은 일반적인 EXPLAIN 기능 비교이며 이 프로젝트가 해당 DBMS를 실행해 검증한 결과는 아니다. GlueSQL은 `glue.plan`, AST, Store wrapper의 실제 코드로 확인했다.

## 공식 문서 근거

- [PostgreSQL EXPLAIN](https://www.postgresql.org/docs/16/sql-explain.html): planner plan, startup/total cost, `EXPLAIN ANALYZE`
- [PostgreSQL ANALYZE](https://www.postgresql.org/docs/current/sql-analyze.html): sampled statistics와 estimate 변화
- [MySQL 8.4 EXPLAIN](https://dev.mysql.com/doc/refman/8.4/en/explain.html): TRADITIONAL/JSON/TREE와 `EXPLAIN ANALYZE FORMAT=TREE`
- [Oracle DBMS_XPLAN](https://docs.oracle.com/en/database/oracle/oracle-database/26/arpls/DBMS_XPLAN.html): DISPLAY_CURSOR와 ALLSTATS/LAST/IOSTATS

GlueSQL 비교의 프로젝트 증거는 `query_lab::tests::json_report_contains_plan_and_metrics`와 `tree_report_contains_required_sections`다.

## 핵심 차이 해석

PostgreSQL, MySQL, Oracle의 EXPLAIN 계열은 cardinality/cost 또는 actual row를 operator 단위로 제공하는 성숙한 진단 인터페이스다. GlueSQL 0.19의 `glue.plan`은 storage planner가 반영된 실행용 AST를 반환하지만 costed physical-plan report 계약은 아니다. 따라서 이름이 비슷해도 동일 수준의 정보로 대응시키지 않는다.

| 질문 | GlueSQL Query Lab 답변 | 다른 DBMS의 대표 답변 |
| --- | --- | --- |
| 어떤 table access인가 | `IndexItem` + Store call | Scan node/access type |
| join 전략은 무엇인가 | `JoinExecutor` | join operator node |
| 예상 row/cost는 | 코드에서 확인되지 않음 | optimizer estimate |
| 실제 operator row는 | 공개 API로 확인 불가 | ANALYZE/ALLSTATS 계열 |
| storage 책임 경계는 | public Store/Index traits | DBMS 내부 access method/engine |

공식 문서는 버전에 따라 동작이 바뀔 수 있으므로 링크의 대상 버전을 명시했다. 이 저장소 테스트는 타 DBMS를 실행하지 않으며 비교표는 공식 문서 기반 개념 비교다.
