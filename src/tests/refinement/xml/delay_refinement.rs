#[cfg(test)]
mod delay_refinement {
    use crate::tests::refinement::Helper::xml_refinement_check;

    static PATH: &str = "samples/xml/delayRefinement.xml";
    static PATH_2: &str = "samples/xml/loop.xml";

    // Self Refinements
    #[test]
    fn LoopTest() {
        assert!(xml_refinement_check(
            PATH_2,
            "refinement: SelfloopNonZeno <= SelfloopNonZeno"
        ));
    }

    // Self Refinements
    #[test]
    fn T1RefinesSelf() {
        assert!(xml_refinement_check(PATH, "refinement: T1 <= T1"));
    }

    #[test]
    fn T2RefinesSelf() {
        assert!(xml_refinement_check(PATH, "refinement: T2 <= T2"));
    }

    #[test]
    fn T3RefinesSelf() {
        assert!(xml_refinement_check(PATH, "refinement: T3 <= T3"));
    }

    #[test]
    fn C1RefinesSelf() {
        assert!(xml_refinement_check(PATH, "refinement: C1 <= C1"));
    }

    #[test]
    fn C2RefinesSelf() {
        assert!(xml_refinement_check(PATH, "refinement: C2 <= C2"));
    }

    #[test]
    fn F1RefinesSelf() {
        assert!(xml_refinement_check(PATH, "refinement: F1 <= F1"));
    }

    #[test]
    fn F2RefinesSelf() {
        assert!(xml_refinement_check(PATH, "refinement: F2 <= F2"));
    }

    #[test]
    fn F3RefinesSelf() {
        assert!(xml_refinement_check(PATH, "refinement: F3 <= F3"));
    }

    #[test]
    fn T4RefinesSelf() {
        assert!(xml_refinement_check(PATH, "refinement: T4 <= T4"));
    }

    #[test]
    fn T0RefinesSelf() {
        assert!(xml_refinement_check(PATH, "refinement: T0 <= T0"));
    }

    #[test]
    fn T5RefinesSelf() {
        assert!(xml_refinement_check(PATH, "refinement: T5 <= T5"));
    }

    #[test]
    fn T6RefinesSelf() {
        assert!(xml_refinement_check(PATH, "refinement: T6 <= T6"));
    }

    #[test]
    fn T7RefinesSelf() {
        assert!(xml_refinement_check(PATH, "refinement: T7 <= T7"));
    }

    #[test]
    fn T8RefinesSelf() {
        assert!(xml_refinement_check(PATH, "refinement: T8 <= T8"));
    }

    #[test]
    fn T9RefinesSelf() {
        assert!(xml_refinement_check(PATH, "refinement: T9 <= T9"));
    }

    #[test]
    fn T10RefinesSelf() {
        assert!(xml_refinement_check(PATH, "refinement: T10 <= T10"));
    }

    #[test]
    fn T11RefinesSelf() {
        assert!(xml_refinement_check(PATH, "refinement: T11 <= T11"));
    }

    #[test]
    fn N1RefinesSelf() {
        assert!(xml_refinement_check(PATH, "refinement: N1 <= N1"));
    }

    #[test]
    fn N2RefinesSelf() {
        assert!(xml_refinement_check(PATH, "refinement: N2 <= N2"));
    }

    #[test]
    fn N3RefinesSelf() {
        assert!(xml_refinement_check(PATH, "refinement: N3 <= N3"));
    }

    #[test]
    fn N4RefinesSelf() {
        assert!(xml_refinement_check(PATH, "refinement: N4 <= N4"));
    }

    #[test]
    fn D1RefinesSelf() {
        assert!(xml_refinement_check(PATH, "refinement: D1 <= D1"));
    }

    #[test]
    fn D2RefinesSelf() {
        assert!(xml_refinement_check(PATH, "refinement: D2 <= D2"));
    }

    #[test]
    fn K1RefinesSelf() {
        assert!(xml_refinement_check(PATH, "refinement: K1 <= K1"));
    }

    #[test]
    fn K2RefinesSelf() {
        assert!(xml_refinement_check(PATH, "refinement: K2 <= K2"));
    }

    #[test]
    fn K3RefinesSelf() {
        assert!(xml_refinement_check(PATH, "refinement: K3 <= K3"));
    }

    #[test]
    fn K4RefinesSelf() {
        assert!(xml_refinement_check(PATH, "refinement: K4 <= K4"));
    }

    #[test]
    fn K5RefinesSelf() {
        assert!(xml_refinement_check(PATH, "refinement: K5 <= K5"));
    }

    #[test]
    fn K6RefinesSelf() {
        assert!(xml_refinement_check(PATH, "refinement: K6 <= K6"));
    }

    #[test]
    fn P0RefinesSelf() {
        assert!(xml_refinement_check(PATH, "refinement: P0 <= P0"));
    }

    #[test]
    fn P1RefinesSelf() {
        assert!(xml_refinement_check(PATH, "refinement: P1 <= P1"));
    }

    #[test]
    fn P2RefinesSelf() {
        assert!(xml_refinement_check(PATH, "refinement: P2 <= P2"));
    }

    #[test]
    fn P3RefinesSelf() {
        assert!(xml_refinement_check(PATH, "refinement: P3 <= P3"));
    }

    #[test]
    fn P4RefinesSelf() {
        assert!(xml_refinement_check(PATH, "refinement: P4 <= P4"));
    }

    #[test]
    fn P5RefinesSelf() {
        assert!(xml_refinement_check(PATH, "refinement: P5 <= P5"));
    }

    #[test]
    fn P6RefinesSelf() {
        assert!(xml_refinement_check(PATH, "refinement: P6 <= P6"));
    }

    #[test]
    fn P7RefinesSelf() {
        assert!(xml_refinement_check(PATH, "refinement: P7 <= P7"));
    }

    #[test]
    fn L1RefinesSelf() {
        assert!(xml_refinement_check(PATH, "refinement: L1 <= L1"));
    }

    #[test]
    fn L2RefinesSelf() {
        assert!(xml_refinement_check(PATH, "refinement: L2 <= L2"));
    }

    #[test]
    fn L3RefinesSelf() {
        assert!(xml_refinement_check(PATH, "refinement: L3 <= L3"));
    }

    #[test]
    fn L4RefinesSelf() {
        assert!(xml_refinement_check(PATH, "refinement: L4 <= L4"));
    }

    #[test]
    fn L5RefinesSelf() {
        assert!(xml_refinement_check(PATH, "refinement: L5 <= L5"));
    }

    #[test]
    fn L6RefinesSelf() {
        assert!(xml_refinement_check(PATH, "refinement: L6 <= L6"));
    }

    #[test]
    fn L7RefinesSelf() {
        assert!(xml_refinement_check(PATH, "refinement: L7 <= L7"));
    }

    #[test]
    fn Z1RefinesSelf() {
        assert!(xml_refinement_check(PATH, "refinement: Z1 <= Z1"));
    }

    #[test]
    fn Z2RefinesSelf() {
        assert!(xml_refinement_check(PATH, "refinement: Z2 <= Z2"));
    }

    #[test]
    fn Z3RefinesSelf() {
        assert!(xml_refinement_check(PATH, "refinement: Z3 <= Z3"));
    }

    #[test]
    fn Z4RefinesSelf() {
        assert!(xml_refinement_check(PATH, "refinement: Z4 <= Z4"));
    }

    #[test]
    fn Z5RefinesSelf() {
        assert!(xml_refinement_check(PATH, "refinement: Z5 <= Z5"));
    }

    #[test]
    fn Z6RefinesSelf() {
        assert!(xml_refinement_check(PATH, "refinement: Z6 <= Z6"));
    }

    #[test]
    fn Z7RefinesSelf() {
        assert!(xml_refinement_check(PATH, "refinement: Z7 <= Z7"));
    }

    //     // Rest of the tests

    #[test]
    fn T1T2RefinesT3() {
        assert!(xml_refinement_check(PATH, "refinement: T1||T2 <= T3"));
    }

    #[test]
    fn C1RefinesC2() {
        assert!(xml_refinement_check(PATH, "refinement: C1 <= C2"));
    }

    #[test]
    fn C2RefinesC1() {
        assert!(xml_refinement_check(PATH, "refinement: C2 <= C1"));
    }

    #[test]
    fn T0T1T2RefinesT3() {
        assert!(xml_refinement_check(PATH, "refinement: T0||T1||T2 <= T3"));
    }

    #[test]
    fn F1F2RefinesF3() {
        assert!(xml_refinement_check(PATH, "refinement: F1||F2 <= F3"));
    }

    #[test]
    fn T4RefinesT3() {
        assert!(xml_refinement_check(PATH, "refinement: T4 <= T3"));
    }

    #[test]
    fn T6RefinesT5() {
        assert!(xml_refinement_check(PATH, "refinement: T6 <= T5"));
    }

    #[test]
    fn T7NotRefinesT8() {
        //Refinement passes, tho should fail ! same symbols
        assert!(!xml_refinement_check(PATH, "refinement: T7 <= T8"));
    }

    #[test]
    fn T9NotRefinesT8() {
        //Refinement passes, tho should fail ! same symbols
        assert!(!xml_refinement_check(PATH, "refinement: T9 <= T8"));
    }

    #[test]
    fn T10NotRefinesT11() {
        //Refinement passes, tho should fail !
        assert!(!xml_refinement_check(PATH, "refinement: T10 <= T11"));
    }

    #[test]
    fn N1RefinesN2() {
        assert!(xml_refinement_check(PATH, "refinement: N1 <= N2"));
    }

    #[test]
    fn D2RefinesD1() {
        assert!(xml_refinement_check(PATH, "refinement: D2 <= D1"));
    }

    #[test]
    fn D1NotRefinesD2() {
        assert!(!xml_refinement_check(PATH, "refinement: D1 <= D2"));
    }

    #[test]
    fn K1NotRefinesK2() {
        //Should fail, but passes ?
        assert!(!xml_refinement_check(PATH, "refinement: K1 <= K2"));
    }

    #[test]
    fn K3NotRefinesK4() {
        //should fail, tho passes ?!
        assert!(!xml_refinement_check(PATH, "refinement: K3 <= K4"));
    }

    #[test]
    fn K5NotRefinesK6() {
        //Should fail, tho passes ?!?
        assert!(!xml_refinement_check(PATH, "refinement: K5 <= K6"));
    }

    #[test]
    fn P0RefinesP1() {
        assert!(xml_refinement_check(PATH, "refinement: P0 <= P1"));
    }

    #[test]
    fn P2NotRefinesP3() {
        assert!(!xml_refinement_check(PATH, "refinement: P2 <= P3"));
    }

    #[test]
    fn P4RefinesP5() {
        assert!(xml_refinement_check(PATH, "refinement: P4 <= P5"));
    }

    #[test]
    fn P6RefinesP7() {
        assert!(xml_refinement_check(PATH, "refinement: P6 <= P7"));
    }

    #[test]
    fn L1L2NotRefinesL3() {
        assert!(!xml_refinement_check(PATH, "refinement: L1||L2 <= L3"));
    }

    #[test]
    fn L4RefinesL5() {
        //should pass tho fails
        assert!(xml_refinement_check(PATH, "refinement: L5 <= L5"));
    }

    #[test]
    fn Z1RefinesZ2() {
        assert!(xml_refinement_check(PATH, "refinement: Z1 <= Z2"));
    }

    #[test]
    fn Z3RefinesZ4() {
        assert!(xml_refinement_check(PATH, "refinement: Z3 <= Z4"));
    }

    #[test]
    fn Q1NotRefinesQ2() {
        //refinement should not hold tho it holds ?
        assert!(!xml_refinement_check(PATH, "refinement: Q1 <= Q2"));
    }

    #[test]
    fn Q2NotRefinesQ1() {
        assert!(!xml_refinement_check(PATH, "refinement: Q2 <= Q1"));
    }
}
