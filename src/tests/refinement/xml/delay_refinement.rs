#[cfg(test)]
mod test {
    use crate::tests::refinement::helper::xml_refinement_check;

    const PATH: &str = "samples/xml/delayRefinement.xml";
    const PATH_2: &str = "samples/xml/loop.xml";

    // Self Refinements
    #[test]
    fn loop_test() {
        assert!(xml_refinement_check(
            PATH_2,
            "refinement: SelfloopNonZeno <= SelfloopNonZeno"
        ));
    }

    // Self Refinements
    #[test]
    fn t1refines_self() {
        assert!(xml_refinement_check(PATH, "refinement: T1 <= T1"));
    }

    #[test]
    fn t2refines_self() {
        assert!(xml_refinement_check(PATH, "refinement: T2 <= T2"));
    }

    #[test]
    fn t3refines_self() {
        assert!(xml_refinement_check(PATH, "refinement: T3 <= T3"));
    }

    #[test]
    fn c1refines_self() {
        assert!(xml_refinement_check(PATH, "refinement: C1 <= C1"));
    }

    #[test]
    fn c2refines_self() {
        assert!(xml_refinement_check(PATH, "refinement: C2 <= C2"));
    }

    #[test]
    fn f1refines_self() {
        assert!(xml_refinement_check(PATH, "refinement: F1 <= F1"));
    }

    #[test]
    fn f2refines_self() {
        assert!(xml_refinement_check(PATH, "refinement: F2 <= F2"));
    }

    #[test]
    fn f3refines_self() {
        assert!(xml_refinement_check(PATH, "refinement: F3 <= F3"));
    }

    #[test]
    fn t4refines_self() {
        assert!(xml_refinement_check(PATH, "refinement: T4 <= T4"));
    }

    #[test]
    fn t0refines_self() {
        assert!(xml_refinement_check(PATH, "refinement: T0 <= T0"));
    }

    #[test]
    fn t5refines_self() {
        assert!(xml_refinement_check(PATH, "refinement: T5 <= T5"));
    }

    #[test]
    fn t6refines_self() {
        assert!(xml_refinement_check(PATH, "refinement: T6 <= T6"));
    }

    #[test]
    fn t7refines_self() {
        assert!(xml_refinement_check(PATH, "refinement: T7 <= T7"));
    }

    #[test]
    fn t8refines_self() {
        assert!(xml_refinement_check(PATH, "refinement: T8 <= T8"));
    }

    #[test]
    fn t9refines_self() {
        assert!(xml_refinement_check(PATH, "refinement: T9 <= T9"));
    }

    #[test]
    fn t10refines_self() {
        assert!(xml_refinement_check(PATH, "refinement: T10 <= T10"));
    }

    #[test]
    fn t11refines_self() {
        assert!(xml_refinement_check(PATH, "refinement: T11 <= T11"));
    }

    #[test]
    fn n1refines_self() {
        assert!(xml_refinement_check(PATH, "refinement: N1 <= N1"));
    }

    #[test]
    fn n2refines_self() {
        assert!(xml_refinement_check(PATH, "refinement: N2 <= N2"));
    }

    #[test]
    fn n3refines_self() {
        assert!(xml_refinement_check(PATH, "refinement: N3 <= N3"));
    }

    #[test]
    fn n4refines_self() {
        assert!(xml_refinement_check(PATH, "refinement: N4 <= N4"));
    }

    #[test]
    fn d1refines_self() {
        assert!(xml_refinement_check(PATH, "refinement: D1 <= D1"));
    }

    #[test]
    fn d2refines_self() {
        assert!(xml_refinement_check(PATH, "refinement: D2 <= D2"));
    }

    #[test]
    fn k1refines_self() {
        assert!(xml_refinement_check(PATH, "refinement: K1 <= K1"));
    }

    #[test]
    fn k2refines_self() {
        assert!(xml_refinement_check(PATH, "refinement: K2 <= K2"));
    }

    #[test]
    fn k3refines_self() {
        assert!(xml_refinement_check(PATH, "refinement: K3 <= K3"));
    }

    #[test]
    fn k4refines_self() {
        assert!(xml_refinement_check(PATH, "refinement: K4 <= K4"));
    }

    #[test]
    fn k5refines_self() {
        assert!(xml_refinement_check(PATH, "refinement: K5 <= K5"));
    }

    #[test]
    fn k6refines_self() {
        assert!(xml_refinement_check(PATH, "refinement: K6 <= K6"));
    }

    #[test]
    fn p0refines_self() {
        assert!(xml_refinement_check(PATH, "refinement: P0 <= P0"));
    }

    #[test]
    fn p1refines_self() {
        assert!(xml_refinement_check(PATH, "refinement: P1 <= P1"));
    }

    #[test]
    fn p2refines_self() {
        assert!(xml_refinement_check(PATH, "refinement: P2 <= P2"));
    }

    #[test]
    fn p3refines_self() {
        assert!(xml_refinement_check(PATH, "refinement: P3 <= P3"));
    }

    #[test]
    fn p4refines_self() {
        assert!(xml_refinement_check(PATH, "refinement: P4 <= P4"));
    }

    #[test]
    fn p5refines_self() {
        assert!(xml_refinement_check(PATH, "refinement: P5 <= P5"));
    }

    #[test]
    fn p6refines_self() {
        assert!(xml_refinement_check(PATH, "refinement: P6 <= P6"));
    }

    #[test]
    fn p7refines_self() {
        assert!(xml_refinement_check(PATH, "refinement: P7 <= P7"));
    }

    #[test]
    fn l1refines_self() {
        assert!(xml_refinement_check(PATH, "refinement: L1 <= L1"));
    }

    #[test]
    fn l2refines_self() {
        assert!(xml_refinement_check(PATH, "refinement: L2 <= L2"));
    }

    #[test]
    fn l3refines_self() {
        assert!(xml_refinement_check(PATH, "refinement: L3 <= L3"));
    }

    #[test]
    fn l4refines_self() {
        assert!(xml_refinement_check(PATH, "refinement: L4 <= L4"));
    }

    #[test]
    fn l5refines_self() {
        assert!(xml_refinement_check(PATH, "refinement: L5 <= L5"));
    }

    #[test]
    fn l6refines_self() {
        assert!(xml_refinement_check(PATH, "refinement: L6 <= L6"));
    }

    #[test]
    fn l7refines_self() {
        assert!(xml_refinement_check(PATH, "refinement: L7 <= L7"));
    }

    #[test]
    fn z1refines_self() {
        assert!(xml_refinement_check(PATH, "refinement: Z1 <= Z1"));
    }

    #[test]
    fn z2refines_self() {
        assert!(xml_refinement_check(PATH, "refinement: Z2 <= Z2"));
    }

    #[test]
    fn z3refines_self() {
        assert!(xml_refinement_check(PATH, "refinement: Z3 <= Z3"));
    }

    #[test]
    fn z4refines_self() {
        assert!(xml_refinement_check(PATH, "refinement: Z4 <= Z4"));
    }

    #[test]
    fn z5refines_self() {
        assert!(xml_refinement_check(PATH, "refinement: Z5 <= Z5"));
    }

    #[test]
    fn z6refines_self() {
        assert!(xml_refinement_check(PATH, "refinement: Z6 <= Z6"));
    }

    #[test]
    fn z7refines_self() {
        assert!(xml_refinement_check(PATH, "refinement: Z7 <= Z7"));
    }

    //     // Rest of the tests

    #[test]
    fn t1t2refines_t3() {
        assert!(xml_refinement_check(PATH, "refinement: T1||T2 <= T3"));
    }

    #[test]
    fn c1refines_c2() {
        assert!(xml_refinement_check(PATH, "refinement: C1 <= C2"));
    }

    #[test]
    fn c2refines_c1() {
        assert!(xml_refinement_check(PATH, "refinement: C2 <= C1"));
    }

    #[test]
    fn t0t1t2refines_t3() {
        assert!(xml_refinement_check(PATH, "refinement: T0||T1||T2 <= T3"));
    }

    #[test]
    fn f1f2refines_f3() {
        assert!(xml_refinement_check(PATH, "refinement: F1||F2 <= F3"));
    }

    #[test]
    fn t4refines_t3() {
        assert!(xml_refinement_check(PATH, "refinement: T4 <= T3"));
    }

    #[test]
    fn t6refines_t5() {
        assert!(xml_refinement_check(PATH, "refinement: T6 <= T5"));
    }

    #[test]
    fn t7not_refines_t8() {
        //Refinement passes, tho should fail ! same symbols
        assert!(!xml_refinement_check(PATH, "refinement: T7 <= T8"));
    }

    #[test]
    fn t9not_refines_t8() {
        //Refinement passes, tho should fail ! same symbols
        assert!(!xml_refinement_check(PATH, "refinement: T9 <= T8"));
    }

    #[test]
    fn t10not_refines_t11() {
        //Refinement passes, tho should fail !
        assert!(!xml_refinement_check(PATH, "refinement: T10 <= T11"));
    }

    #[test]
    fn n1refines_n2() {
        assert!(xml_refinement_check(PATH, "refinement: N1 <= N2"));
    }

    #[test]
    fn d2refines_d1() {
        assert!(xml_refinement_check(PATH, "refinement: D2 <= D1"));
    }

    #[test]
    fn d1not_refines_d2() {
        assert!(!xml_refinement_check(PATH, "refinement: D1 <= D2"));
    }

    #[test]
    fn k1not_refines_k2() {
        //Should fail, but passes ?
        assert!(!xml_refinement_check(PATH, "refinement: K1 <= K2"));
    }

    #[test]
    fn k3not_refines_k4() {
        //should fail, tho passes ?!
        assert!(!xml_refinement_check(PATH, "refinement: K3 <= K4"));
    }

    #[test]
    fn k5not_refines_k6() {
        //Should fail, tho passes ?!?
        assert!(!xml_refinement_check(PATH, "refinement: K5 <= K6"));
    }

    #[test]
    fn p0refines_p1() {
        assert!(xml_refinement_check(PATH, "refinement: P0 <= P1"));
    }

    #[test]
    fn p2not_refines_p3() {
        assert!(!xml_refinement_check(PATH, "refinement: P2 <= P3"));
    }

    #[test]
    fn p4refines_p5() {
        assert!(xml_refinement_check(PATH, "refinement: P4 <= P5"));
    }

    #[test]
    fn p6refines_p7() {
        assert!(xml_refinement_check(PATH, "refinement: P6 <= P7"));
    }

    #[test]
    fn l1l2not_refines_l3() {
        assert!(!xml_refinement_check(PATH, "refinement: L1||L2 <= L3"));
    }

    #[test]
    fn l4refines_l5() {
        //should pass tho fails
        assert!(xml_refinement_check(PATH, "refinement: L5 <= L5"));
    }

    #[test]
    fn z1refines_z2() {
        assert!(xml_refinement_check(PATH, "refinement: Z1 <= Z2"));
    }

    #[test]
    fn z3refines_z4() {
        assert!(xml_refinement_check(PATH, "refinement: Z3 <= Z4"));
    }

    #[test]
    fn q1not_refines_q2() {
        //refinement should not hold tho it holds ?
        assert!(!xml_refinement_check(PATH, "refinement: Q1 <= Q2"));
    }

    #[test]
    fn q2not_refines_q1() {
        assert!(!xml_refinement_check(PATH, "refinement: Q2 <= Q1"));
    }
}
