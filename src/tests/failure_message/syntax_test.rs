#[cfg(test)]

mod test {
    use crate::{
        tests::refinement::Helper::json_run_query,
        System::query_failures::{SyntaxFailure, SyntaxResult, QueryResult},
    };

    const PATH: &str = "samples/json/SyntaxTest";

    #[test]
    fn syntax_failure_test() {
        let actual = json_run_query(PATH, "syntax: syntaxFailure").unwrap();
        assert!(matches!(
            actual,
            QueryResult::Syntax(SyntaxResult::Err(
                SyntaxFailure::Unparsable { .. }
            ))
        ));
    }
}
