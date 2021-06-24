#[cfg(test)]
mod Conjunction_refinement {
    use crate::tests::refinement::Helper::setup;
    use crate::ModelObjects::representations::SystemRepresentation;
    use crate::System::refine;
    use std::borrow::Borrow;

    static PATH: &str = "samples/json/Conjunction";

    #[test]
    fn T1RefinesSelf() {
        let (automataList, decl) = setup(PATH.to_string());
        assert!(refine::check_refinement(
            SystemRepresentation::Component(automataList.get("Test1").unwrap().clone()),
            SystemRepresentation::Component(automataList.get("Test1").unwrap().clone()),
            decl.borrow()
        )
        .unwrap());
    }

    #[test]
    fn T2RefinesSelf() {
        let (automataList, decl) = setup(PATH.to_string());
        assert!(refine::check_refinement(
            SystemRepresentation::Component(automataList.get("Test2").unwrap().clone()),
            SystemRepresentation::Component(automataList.get("Test2").unwrap().clone()),
            decl.borrow()
        )
        .unwrap());
    }

    #[test]
    fn T3RefinesSelf() {
        let (automataList, decl) = setup(PATH.to_string());
        assert!(refine::check_refinement(
            SystemRepresentation::Component(automataList.get("Test3").unwrap().clone()),
            SystemRepresentation::Component(automataList.get("Test3").unwrap().clone()),
            decl.borrow()
        )
        .unwrap());
    }

    #[test]
    fn T4RefinesSelf() {
        let (automataList, decl) = setup(PATH.to_string());
        assert!(refine::check_refinement(
            SystemRepresentation::Component(automataList.get("Test4").unwrap().clone()),
            SystemRepresentation::Component(automataList.get("Test4").unwrap().clone()),
            decl.borrow()
        )
        .unwrap());
    }

    #[test]
    fn T5RefinesSelf() {
        let (automataList, decl) = setup(PATH.to_string());
        assert!(refine::check_refinement(
            SystemRepresentation::Component(automataList.get("Test5").unwrap().clone()),
            SystemRepresentation::Component(automataList.get("Test5").unwrap().clone()),
            decl.borrow()
        )
        .unwrap());
    }

    #[test]
    fn T1ConjT2RefinesT3() {
        let (automataList, decl) = setup(PATH.to_string());
        assert!(refine::check_refinement(
            SystemRepresentation::Conjunction(
                Box::from(SystemRepresentation::Component(
                    automataList.get("Test1").unwrap().clone()
                )),
                Box::from(SystemRepresentation::Component(
                    automataList.get("Test2").unwrap().clone()
                ))
            ),
            SystemRepresentation::Component(automataList.get("Test3").unwrap().clone()),
            decl.borrow()
        )
        .unwrap());
    }

    #[test]
    fn T2ConjT3RefinesT1() {
        let (automataList, decl) = setup(PATH.to_string());
        assert!(refine::check_refinement(
            SystemRepresentation::Conjunction(
                Box::from(SystemRepresentation::Component(
                    automataList.get("Test2").unwrap().clone()
                )),
                Box::from(SystemRepresentation::Component(
                    automataList.get("Test3").unwrap().clone()
                ))
            ),
            SystemRepresentation::Component(automataList.get("Test1").unwrap().clone()),
            decl.borrow()
        )
        .unwrap());
    }

    #[test]
    fn T1ConjT3RefinesT2() {
        let (automataList, decl) = setup(PATH.to_string());
        assert!(refine::check_refinement(
            SystemRepresentation::Conjunction(
                Box::from(SystemRepresentation::Component(
                    automataList.get("Test1").unwrap().clone()
                )),
                Box::from(SystemRepresentation::Component(
                    automataList.get("Test3").unwrap().clone()
                ))
            ),
            SystemRepresentation::Component(automataList.get("Test2").unwrap().clone()),
            decl.borrow()
        )
        .unwrap());
    }

    #[test]
    fn T1ConjT2ConjT4RefinesT5() {
        let (automataList, decl) = setup(PATH.to_string());
        assert!(refine::check_refinement(
            SystemRepresentation::Conjunction(
                Box::from(SystemRepresentation::Conjunction(
                    Box::from(SystemRepresentation::Component(
                        automataList.get("Test1").unwrap().clone()
                    )),
                    Box::from(SystemRepresentation::Component(
                        automataList.get("Test2").unwrap().clone()
                    ))
                )),
                Box::from(SystemRepresentation::Component(
                    automataList.get("Test4").unwrap().clone()
                ))
            ),
            SystemRepresentation::Component(automataList.get("Test5").unwrap().clone()),
            decl.borrow()
        )
        .unwrap());
    }

    #[test]
    fn T3ConjT4RefinesT5() {
        let (automataList, decl) = setup(PATH.to_string());
        assert!(refine::check_refinement(
            SystemRepresentation::Conjunction(
                Box::from(SystemRepresentation::Component(
                    automataList.get("Test3").unwrap().clone()
                )),
                Box::from(SystemRepresentation::Component(
                    automataList.get("Test4").unwrap().clone()
                ))
            ),
            SystemRepresentation::Component(automataList.get("Test5").unwrap().clone()),
            decl.borrow()
        )
        .unwrap());
    }

    #[test]
    fn test1NestedConjRefinesT5() {
        let (automataList, decl) = setup(PATH.to_string());
        let ts1 = SystemRepresentation::Conjunction(
            Box::from(SystemRepresentation::Component(
                automataList.get("Test1").unwrap().clone(),
            )),
            Box::from(SystemRepresentation::Component(
                automataList.get("Test2").unwrap().clone(),
            )),
        );
        let ts2: SystemRepresentation = SystemRepresentation::Conjunction(
            Box::from(ts1),
            Box::from(SystemRepresentation::Component(
                automataList.get("Test4").unwrap().clone(),
            )),
        );

        assert!(refine::check_refinement(
            ts2,
            SystemRepresentation::Component(automataList.get("Test5").unwrap().clone()),
            decl.borrow()
        )
        .unwrap());
    }

    #[test]
    fn T6ConjT7RefinesT8() {
        let (automataList, decl) = setup(PATH.to_string());
        assert!(refine::check_refinement(
            SystemRepresentation::Conjunction(
                Box::from(SystemRepresentation::Component(
                    automataList.get("Test6").unwrap().clone()
                )),
                Box::from(SystemRepresentation::Component(
                    automataList.get("Test7").unwrap().clone()
                ))
            ),
            SystemRepresentation::Component(automataList.get("Test8").unwrap().clone()),
            decl.borrow()
        )
        .unwrap());
    }

    #[test]
    fn test1NestedConjRefinesT12() {
        let (automataList, decl) = setup(PATH.to_string());
        let ts1 = SystemRepresentation::Conjunction(
            Box::from(SystemRepresentation::Component(
                automataList.get("Test9").unwrap().clone(),
            )),
            Box::from(SystemRepresentation::Component(
                automataList.get("Test10").unwrap().clone(),
            )),
        );
        let ts2: SystemRepresentation = SystemRepresentation::Conjunction(
            Box::from(ts1),
            Box::from(SystemRepresentation::Component(
                automataList.get("Test11").unwrap().clone(),
            )),
        );

        assert!(refine::check_refinement(
            ts2,
            SystemRepresentation::Component(automataList.get("Test12").unwrap().clone()),
            decl.borrow()
        )
        .unwrap());
    }
}
