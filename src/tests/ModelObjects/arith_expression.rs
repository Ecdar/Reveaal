#[cfg(test)]
mod test {
    use crate::ModelObjects::representations::ArithExpression as AE;
    use AE::*;
    #[test]
    fn simplify_test1() {
        let expr = AE::ADif(Int(10), Int(5)); //10 - 5
        assert_eq!(Ok(Int(5)), expr.simplify());

        let expr = AE::AAdd(Int(10), Int(5)); //10 + 5
        assert_eq!(Ok(Int(15)), expr.simplify());
    }

    #[test]
    fn simplify_test2() {
        let expr = Multiplication(
            //(10 - 5) * (5 + 3)
            Box::new(AE::ADif(Int(10), Int(5))),
            Box::new(AE::AAdd(Int(5), Int(3))),
        );
        assert_eq!(Ok(Int(40)), expr.simplify());

        let expr = Multiplication(
            //(10 + 5) * (5 - 3)
            Box::new(AE::AAdd(Int(10), Int(5))),
            Box::new(AE::ADif(Int(5), Int(3))),
        );
        assert_eq!(Ok(Int(30)), expr.simplify());
    }
    #[test]
    fn simplify_test3() {
        let expr = AE::ADif(
            Clock(1),
            AE::ADif(Int(5), AE::ADif(Int(4), AE::ADif(Int(3), Int(2)))), //5-(4-(3-2))
        );
        assert_eq!(Ok(AE::ADif(Clock(1), Int(2))), expr.simplify());

        let expr = AE::AAdd(
            Clock(1),
            AE::AAdd(Int(5), AE::AAdd(Int(4), AE::AAdd(Int(3), Int(2)))),
        );
        assert_eq!(Ok(AE::AAdd(Clock(1), Int(14))), expr.simplify());
    }
    #[test]
    fn simplify_test4() {
        //((5-4)-3)-2
        let expr = AE::ADif(
            Clock(1),
            AE::ADif(AE::ADif(AE::ADif(Int(5), Int(4)), Int(3)), Int(2)),
        );
        assert_eq!(Ok(AE::ADif(Clock(1), Int(-4))), expr.simplify());

        let expr = AE::AAdd(
            Clock(1),
            AE::AAdd(AE::AAdd(AE::AAdd(Int(5), Int(4)), Int(3)), Int(2)),
        );
        assert_eq!(Ok(AE::AAdd(Clock(1), Int(14))), expr.simplify());
    }

    #[test]
    fn simplify_test5() {
        //((5-4)-3)-2
        let expr = AE::ADif(AE::ADif(AE::ADif(Int(5), Clock(4)), Int(3)), Int(2));
        assert_eq!(Ok(AE::ADif(Int(0), Clock(4))), expr.simplify());

        let expr = AE::AAdd(AE::AAdd(AE::AAdd(Int(5), Clock(4)), Int(3)), Int(2));
        assert_eq!(Ok(AE::AAdd(Int(10), Clock(4))), expr.simplify());
    }

    #[test]
    fn simplify_test6() {
        //5-(4-(3-2))
        let expr = AE::ADif(Int(5), AE::ADif(Int(4), AE::ADif(Int(3), Clock(2))));
        assert_eq!(Ok(AE::ADif(Int(4), Clock(2))), expr.simplify());

        let expr = AE::AAdd(Int(5), AE::AAdd(Int(4), AE::AAdd(Int(3), Clock(2))));
        assert_eq!(Ok(AE::AAdd(Int(12), Clock(2))), expr.simplify());
    }

    #[test]
    fn simplify_test7() {
        //5-(4-(3-2))
        let expr = AE::ADif(
            Int(5),
            AE::ADif(Clock(4), AE::ADif(Int(3), AE::ADif(Int(2), Int(1)))),
        );
        assert_eq!(Ok(AE::ADif(Clock(4), Int(3))), expr.simplify());

        let expr = AE::AAdd(
            Int(5),
            AE::AAdd(Clock(4), AE::AAdd(Int(3), AE::AAdd(Int(2), Int(1)))),
        );
        assert_eq!(Ok(AE::AAdd(Clock(4), Int(11))), expr.simplify());
    }

    #[test]
    fn simplify_test_highoperators_ints() {
        let expr = AE::AMul(Int(10), Int(5));
        assert_eq!(Ok(Int(50)), expr.simplify());

        let expr = AE::ADiv(Int(10), Int(5));
        assert_eq!(Ok(Int(2)), expr.simplify());

        let expr = AE::AMod(Int(10), Int(5));
        assert_eq!(Ok(Int(0)), expr.simplify());
    }

    #[test]
    fn simplify_test_highoperators_clocks() {
        let expr = AE::AMul(Clock(10), Int(5));
        assert_eq!(expr.simplify().ok(), None);

        let expr = AE::AMul(Int(10), Clock(5));
        assert_eq!(expr.simplify().ok(), None);

        let expr = AE::ADiv(Clock(10), Int(5));
        assert_eq!(expr.simplify().ok(), None);

        let expr = AE::ADiv(Int(10), Clock(5));
        assert_eq!(expr.simplify().ok(), None);

        let expr = AE::AMod(Clock(10), Int(5));
        assert_eq!(expr.simplify().ok(), None);

        let expr = AE::AMod(Int(10), Clock(5));
        assert_eq!(expr.simplify().ok(), None);
    }
}
