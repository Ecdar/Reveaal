#[cfg(test)]
mod bool_expression {
    use crate::ModelObjects::representations::BoolExpression as BE;
    use BE::{Bool, Int};
    #[test]
    fn simplify_test1() {
        let mut expr = (Bool(false) & BE::BLessEQ(Int(3), Int(2))) | Bool(true);
        expr.simplify();
        assert_eq!(Bool(true), expr);
    }
}


