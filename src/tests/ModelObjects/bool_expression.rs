#[cfg(test)]
mod bool_expression {
    use crate::ModelObjects::representations::BoolExpression;
    use std::boxed::Box;

    #[test]
    fn empty_constraint() {
        let max_bounds = BoolExpression::Int(5).get_highest_constraints(0);

        assert_eq!(max_bounds.clock_count(), 0);
    }

    #[test]
    fn single_greateg_constraint() {
        let max_bounds = BoolExpression::GreatEQ(
            Box::new(BoolExpression::Clock(1)),
            Box::new(BoolExpression::Int(5)),
        )
        .get_highest_constraints(2);

        assert_eq!(max_bounds.clock_bounds[1], 5);
    }

    #[test]
    fn single_eq_constraint() {
        let max_bounds = BoolExpression::EQ(
            Box::new(BoolExpression::Clock(4)),
            Box::new(BoolExpression::Int(3)),
        )
        .get_highest_constraints(5);
        
        assert_eq!(max_bounds.clock_bounds[4], 3);
    }

    #[test]
    fn single_lessthen_constraint() {
        let max_bounds = BoolExpression::LessT(
            Box::new(BoolExpression::Clock(9)),
            Box::new(BoolExpression::Int(8)),
        )
        .get_highest_constraints(10);
        
        assert_eq!(max_bounds.clock_bounds[9], 7);
    }

    //This test case might be wrong max bound should potentially be 8 here?
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
        let max_bounds = expr.get_highest_constraints(4);
        
        assert_eq!(max_bounds.clock_bounds[3], 7);
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
        let max_bounds = expr.get_highest_constraints(6);
        
        assert_eq!(max_bounds.clock_bounds[5], 12);
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
        let max_bounds = expr.get_highest_constraints(5);
        
        assert_eq!(max_bounds.clock_bounds[2], 20);
        assert_eq!(max_bounds.clock_bounds[4], 2);
    }

    #[test]
    fn wrapped_constraint() {
        let expr = BoolExpression::Parentheses(Box::new(BoolExpression::GreatT(
            Box::new(BoolExpression::Clock(2)),
            Box::new(BoolExpression::Int(19)),
        )));
        let max_bounds = expr.get_highest_constraints(3);
        
        assert_eq!(max_bounds.clock_bounds[0], 0);
        assert_eq!(max_bounds.clock_bounds[1], 0);
        assert_eq!(max_bounds.clock_bounds[2], 20);
    }

    #[test]
    fn illegal_constraint() {
        let expr = BoolExpression::Parentheses(Box::new(BoolExpression::GreatT(
            Box::new(BoolExpression::VarName(String::from("A"))),
            Box::new(BoolExpression::Int(19)),
        )));
        let max_bounds = expr.get_highest_constraints(2);

        assert_eq!(max_bounds.clock_bounds[0], 0);
        assert_eq!(max_bounds.clock_bounds[1], 0);
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
        let max_bounds = expr.get_highest_constraints(5);
        
        assert_eq!(max_bounds.clock_bounds[4], 2);
    }
}
