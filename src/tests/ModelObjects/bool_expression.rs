#[cfg(test)]
mod bool_expression {
    use crate::ModelObjects::representations::ArithExpression as AE;
    use crate::ModelObjects::representations::BoolExpression as BE;
    use AE::Int;
    use BE::Bool;
    #[test]
    fn simplify_test1() {
        let mut expr = (Bool(false) & BE::BLessEQ(Int(3), Int(2))) | Bool(true);
        expr.simplify();
        assert_eq!(Bool(true), expr);
    }

    #[test]
    fn simplify_test2() {
        let mut expr = BE::BLessEQ(Int(2), Int(3));
        expr.simplify();
        assert_eq!(Bool(true), expr);
    }
}
