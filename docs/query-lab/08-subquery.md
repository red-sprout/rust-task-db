# Subquery

`TableFactor::Derived`는 plan tree에서 Derived Subquery child로 렌더링한다. expression 내부 scalar/correlated subquery는 AST Debug/SQL 표현에는 존재하지만 공개 API로 실행 횟수를 계측할 수 없다.

`lab run subquery`는 Project별 correlated COUNT SQL을 실제로 plan/execute한다. GlueSQL 0.19과 현재 schema에서 실행 성공을 확인했다. 다만 planned filter 안에 subquery expression이 남으며 반복 실행 횟수는 공개 metric으로 확인되지 않는다.

JOIN+GROUP BY 비교 query는 aggregate scenario에서 별도로 실행한다. subquery 반복 실행이나 JOIN rewrite는 operator hook 없이는 단정하지 않는다.

근거 파일은 `ast/expr.rs`, `ast/query.rs`, `executor/select.rs`이며 scenario registry 테스트는 `scenario_list_and_scan_queries_are_available`다.

## 비교 절차

1. correlated COUNT SQL의 raw planned Statement에 subquery가 남는지 본다.
2. 실행 성공/실패와 Storage calls, consumed rows, returned rows를 기록한다.
3. JOIN + GROUP BY + HAVING 비교 쿼리를 같은 seed에서 실행한다.
4. 두 결과 집합의 의미가 같은지 확인하되 elapsed 하나로 rewrite 여부를 결론 내리지 않는다.

현재 공개 trace에는 subquery execution counter가 없다. 따라서 outer project 수만큼 반복되었다거나 decorrelation되었다는 문장은 코드에서 확인되지 않음으로 취급한다. 정확한 반복 횟수에는 `executor/select.rs` expression evaluation 경계의 opt-in hook이 필요하다.
