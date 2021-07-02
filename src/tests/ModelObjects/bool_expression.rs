#[cfg(test)]
mod bool_expression {
    use crate::ModelObjects::representations::BoolExpression;
    use std::boxed::Box;

    #[test]
    fn empty_constraint() {
        let max_bounds = BoolExpression::Int(5).get_higest_constraint();

        assert_eq!(max_bounds.clock_count(), 0);
    }

    #[test]
    fn single_greateg_constraint() {
        let max_bounds = BoolExpression::GreatEQ(
            Box::new(BoolExpression::Clock(1)),
            Box::new(BoolExpression::Int(5)),
        )
        .get_higest_constraint();

        assert_eq!(max_bounds.clock_count(), 1);
        assert_eq!(max_bounds.get(1), 5);
    }

    #[test]
    fn single_eq_constraint() {
        let max_bounds = BoolExpression::EQ(
            Box::new(BoolExpression::Clock(4)),
            Box::new(BoolExpression::Int(3)),
        )
        .get_higest_constraint();

        assert_eq!(max_bounds.clock_count(), 1);
        assert_eq!(max_bounds.get(4), 3);
    }

    #[test]
    fn single_lessthen_constraint() {
        let max_bounds = BoolExpression::LessT(
            Box::new(BoolExpression::Clock(9)),
            Box::new(BoolExpression::Int(8)),
        )
        .get_higest_constraint();

        assert_eq!(max_bounds.clock_count(), 1);
        assert_eq!(max_bounds.get(9), 7);
    }

    #[test]
    fn disjunction_on_same_clock_lessthen_constraint() {
        let expr = BoolExpression::OrOp(
            Box::new(BoolExpression::LessT(
                Box::new(BoolExpression::Clock(3)),
                Box::new(BoolExpression::Int(8)),
            )),
            Box::new(BoolExpression::LessT(
                Box::new(BoolExpression::Clock(3)),
                Box::new(BoolExpression::Int(4)),
            )),
        );
        let max_bounds = expr.get_higest_constraint();

        assert_eq!(max_bounds.clock_count(), 1);
        assert_eq!(max_bounds.get(3), 7);
    }

    #[test]
    fn disjunction_on_same_clock_greatthen_constraint() {
        let expr = BoolExpression::OrOp(
            Box::new(BoolExpression::GreatT(
                Box::new(BoolExpression::Clock(5)),
                Box::new(BoolExpression::Int(11)),
            )),
            Box::new(BoolExpression::GreatT(
                Box::new(BoolExpression::Clock(5)),
                Box::new(BoolExpression::Int(7)),
            )),
        );
        let max_bounds = expr.get_higest_constraint();

        assert_eq!(max_bounds.clock_count(), 1);
        assert_eq!(max_bounds.get(5), 12);
    }

    #[test]
    fn conjunction_greatthen_constraint() {
        let expr = BoolExpression::AndOp(
            Box::new(BoolExpression::GreatT(
                Box::new(BoolExpression::Clock(2)),
                Box::new(BoolExpression::Int(19)),
            )),
            Box::new(BoolExpression::GreatT(
                Box::new(BoolExpression::Clock(4)),
                Box::new(BoolExpression::Int(1)),
            )),
        );
        let max_bounds = expr.get_higest_constraint();

        assert_eq!(max_bounds.clock_count(), 2);
        assert_eq!(max_bounds.get(2), 20);
        assert_eq!(max_bounds.get(4), 2);
    }

    #[test]
    fn wrapped_constraint() {
        let expr = BoolExpression::Parentheses(Box::new(BoolExpression::GreatT(
            Box::new(BoolExpression::Clock(2)),
            Box::new(BoolExpression::Int(19)),
        )));
        let max_bounds = expr.get_higest_constraint();

        assert_eq!(max_bounds.clock_count(), 1);
        assert_eq!(max_bounds.get(2), 20);
    }

    #[test]
    fn illegal_constraint() {
        let expr = BoolExpression::Parentheses(Box::new(BoolExpression::GreatT(
            Box::new(BoolExpression::VarName(String::from("A"))),
            Box::new(BoolExpression::Int(19)),
        )));
        let max_bounds = expr.get_higest_constraint();

        assert_eq!(max_bounds.clock_count(), 0);
    }

    #[test]
    fn disjunction_with_illegal_constraint() {
        let expr = BoolExpression::OrOp(
            Box::new(BoolExpression::GreatT(
                Box::new(BoolExpression::VarName(String::from("A"))),
                Box::new(BoolExpression::Int(19)),
            )),
            Box::new(BoolExpression::GreatT(
                Box::new(BoolExpression::Clock(4)),
                Box::new(BoolExpression::Int(1)),
            )),
        );
        let max_bounds = expr.get_higest_constraint();

        assert_eq!(max_bounds.clock_count(), 1);
        assert_eq!(max_bounds.get(4), 2);
    }
}
