#[cfg(test)]
mod test {
    use crate::ModelObjects::Expressions::ArithExpression as AE;
    use crate::ModelObjects::Expressions::BoolExpression as BE;
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
