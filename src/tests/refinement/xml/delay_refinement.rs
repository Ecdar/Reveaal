#[cfg(test)]
mod delay_refinement {
    use crate::ModelObjects::xml_parser;
    use crate::System::{refine, input_enabler};
    use crate::ModelObjects::representations::SystemRepresentation;
    use std::borrow::Borrow;
    use crate::ModelObjects::component::Component;
    use crate::ModelObjects::system_declarations::SystemDeclarations;
    use crate::tests::refinement::Helper::optimize_components;

    static PATH: &str = "samples/xml/delayRefinement.xml";

    // Self Refinements
    #[test]
    fn T1RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        assert!(refine::check_refinement(SystemRepresentation::Component(optimized_components.get(0).unwrap().clone()),
                                         SystemRepresentation::Component(optimized_components.get(0).unwrap().clone()), &decl).unwrap());
    }

    #[test]
    fn T2RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        assert!(refine::check_refinement(SystemRepresentation::Component(optimized_components.get(1).unwrap().clone()), SystemRepresentation::Component(optimized_components.get(1).unwrap().clone()), decl.borrow()).unwrap());
    }

    #[test]
    fn T3RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        assert!(refine::check_refinement(SystemRepresentation::Component(optimized_components.get(2).unwrap().clone()), SystemRepresentation::Component(optimized_components.get(2).unwrap().clone()), decl.borrow()).unwrap());
    }

    #[test]
    fn C1RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        assert!(refine::check_refinement(SystemRepresentation::Component(optimized_components.get(3).unwrap().clone()), SystemRepresentation::Component(optimized_components.get(3).unwrap().clone()), decl.borrow()).unwrap());
    }

    #[test]
    fn C2RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        assert!(refine::check_refinement(SystemRepresentation::Component(optimized_components.get(4).unwrap().clone()), SystemRepresentation::Component(optimized_components.get(4).unwrap().clone()), decl.borrow()).unwrap());
    }

    #[test]
    fn F1RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        assert!(refine::check_refinement(SystemRepresentation::Component(optimized_components.get(7).unwrap().clone()), SystemRepresentation::Component(optimized_components.get(7).unwrap().clone()), decl.borrow()).unwrap());
    }

    #[test]
    fn F2RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        assert!(refine::check_refinement(SystemRepresentation::Component(optimized_components.get(8).unwrap().clone()), SystemRepresentation::Component(optimized_components.get(8).unwrap().clone()), decl.borrow()).unwrap());
    }

    #[test]
    fn F3RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        assert!(refine::check_refinement(SystemRepresentation::Component(optimized_components.get(9).unwrap().clone()), SystemRepresentation::Component(optimized_components.get(9).unwrap().clone()), decl.borrow()).unwrap());
    }

    #[test]
    fn T4RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        assert!(refine::check_refinement(SystemRepresentation::Component(optimized_components.get(10).unwrap().clone()), SystemRepresentation::Component(optimized_components.get(10).unwrap().clone()), decl.borrow()).unwrap());
    }

    #[test]
    fn T0RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        assert!(refine::check_refinement(SystemRepresentation::Component(optimized_components.get(11).unwrap().clone()), SystemRepresentation::Component(optimized_components.get(11).unwrap().clone()), decl.borrow()).unwrap());
    }

    #[test]
    fn T5RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        assert!(refine::check_refinement(SystemRepresentation::Component(optimized_components.get(12).unwrap().clone()), SystemRepresentation::Component(optimized_components.get(12).unwrap().clone()), decl.borrow()).unwrap());
    }

    #[test]
    fn T6RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        assert!(refine::check_refinement(SystemRepresentation::Component(optimized_components.get(13).unwrap().clone()), SystemRepresentation::Component(optimized_components.get(13).unwrap().clone()), decl.borrow()).unwrap());
    }

    #[test]
    fn T7RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        assert!(refine::check_refinement(SystemRepresentation::Component(optimized_components.get(14).unwrap().clone()), SystemRepresentation::Component(optimized_components.get(14).unwrap().clone()), decl.borrow()).unwrap());
    }

    #[test]
    fn T8RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        assert!(refine::check_refinement(SystemRepresentation::Component(optimized_components.get(15).unwrap().clone()), SystemRepresentation::Component(optimized_components.get(15).unwrap().clone()), decl.borrow()).unwrap());
    }

    #[test]
    fn T9RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        assert!(refine::check_refinement(SystemRepresentation::Component(optimized_components.get(16).unwrap().clone()), SystemRepresentation::Component(optimized_components.get(16).unwrap().clone()), decl.borrow()).unwrap());
    }

    #[test]
    fn T10RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        assert!(refine::check_refinement(SystemRepresentation::Component(optimized_components.get(17).unwrap().clone()), SystemRepresentation::Component(optimized_components.get(17).unwrap().clone()), decl.borrow()).unwrap());
    }

    #[test]
    fn T11RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        assert!(refine::check_refinement(SystemRepresentation::Component(optimized_components.get(18).unwrap().clone()), SystemRepresentation::Component(optimized_components.get(18).unwrap().clone()), decl.borrow()).unwrap());
    }

    #[test]
    fn N1RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        assert!(refine::check_refinement(SystemRepresentation::Component(optimized_components.get(19).unwrap().clone()), SystemRepresentation::Component(optimized_components.get(19).unwrap().clone()), decl.borrow()).unwrap());
    }

    #[test]
    fn N2RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        assert!(refine::check_refinement(SystemRepresentation::Component(optimized_components.get(20).unwrap().clone()), SystemRepresentation::Component(optimized_components.get(20).unwrap().clone()), decl.borrow()).unwrap());
    }

    #[test]
    fn N3RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        assert!(refine::check_refinement(SystemRepresentation::Component(optimized_components.get(21).unwrap().clone()), SystemRepresentation::Component(optimized_components.get(21).unwrap().clone()), decl.borrow()).unwrap());
    }

    #[test]
    fn N4RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        assert!(refine::check_refinement(SystemRepresentation::Component(optimized_components.get(22).unwrap().clone()), SystemRepresentation::Component(optimized_components.get(22).unwrap().clone()), decl.borrow()).unwrap());
    }

    #[test]
    fn D1RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        assert!(refine::check_refinement(SystemRepresentation::Component(optimized_components.get(23).unwrap().clone()), SystemRepresentation::Component(optimized_components.get(23).unwrap().clone()), decl.borrow()).unwrap());
    }

    #[test]
    fn D2RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        assert!(refine::check_refinement(SystemRepresentation::Component(optimized_components.get(24).unwrap().clone()), SystemRepresentation::Component(optimized_components.get(24).unwrap().clone()), decl.borrow()).unwrap());
    }

    #[test]
    fn K1RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        assert!(refine::check_refinement(SystemRepresentation::Component(optimized_components.get(25).unwrap().clone()), SystemRepresentation::Component(optimized_components.get(25).unwrap().clone()), decl.borrow()).unwrap());
    }

    #[test]
    fn K2RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        assert!(refine::check_refinement(SystemRepresentation::Component(optimized_components.get(26).unwrap().clone()), SystemRepresentation::Component(optimized_components.get(26).unwrap().clone()), decl.borrow()).unwrap());
    }

    #[test]
    fn K3RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        assert!(refine::check_refinement(SystemRepresentation::Component(optimized_components.get(27).unwrap().clone()), SystemRepresentation::Component(optimized_components.get(27).unwrap().clone()), decl.borrow()).unwrap());
    }

    #[test]
    fn K4RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        assert!(refine::check_refinement(SystemRepresentation::Component(optimized_components.get(28).unwrap().clone()), SystemRepresentation::Component(optimized_components.get(28).unwrap().clone()), decl.borrow()).unwrap());
    }

    #[test]
    fn K5RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        assert!(refine::check_refinement(SystemRepresentation::Component(optimized_components.get(29).unwrap().clone()), SystemRepresentation::Component(optimized_components.get(29).unwrap().clone()), decl.borrow()).unwrap());
    }

    #[test]
    fn K6RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        assert!(refine::check_refinement(SystemRepresentation::Component(optimized_components.get(30).unwrap().clone()), SystemRepresentation::Component(optimized_components.get(30).unwrap().clone()), decl.borrow()).unwrap());
    }

    #[test]
    fn P0RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        assert!(refine::check_refinement(SystemRepresentation::Component(optimized_components.get(31).unwrap().clone()), SystemRepresentation::Component(optimized_components.get(31).unwrap().clone()), decl.borrow()).unwrap());
    }

    #[test]
    fn P1RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        assert!(refine::check_refinement(SystemRepresentation::Component(optimized_components.get(32).unwrap().clone()), SystemRepresentation::Component(optimized_components.get(32).unwrap().clone()), decl.borrow()).unwrap());
    }

    #[test]
    fn P2RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        assert!(refine::check_refinement(SystemRepresentation::Component(optimized_components.get(33).unwrap().clone()), SystemRepresentation::Component(optimized_components.get(33).unwrap().clone()), decl.borrow()).unwrap());
    }

    #[test]
    fn P3RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        assert!(refine::check_refinement(SystemRepresentation::Component(optimized_components.get(34).unwrap().clone()), SystemRepresentation::Component(optimized_components.get(34).unwrap().clone()), decl.borrow()).unwrap());
    }

    #[test]
    fn P4RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        assert!(refine::check_refinement(SystemRepresentation::Component(optimized_components.get(35).unwrap().clone()), SystemRepresentation::Component(optimized_components.get(35).unwrap().clone()), decl.borrow()).unwrap());
    }

    #[test]
    fn P5RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        assert!(refine::check_refinement(SystemRepresentation::Component(optimized_components.get(36).unwrap().clone()), SystemRepresentation::Component(optimized_components.get(36).unwrap().clone()), decl.borrow()).unwrap());
    }

    #[test]
    fn P6RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        assert!(refine::check_refinement(SystemRepresentation::Component(optimized_components.get(37).unwrap().clone()), SystemRepresentation::Component(optimized_components.get(37).unwrap().clone()), decl.borrow()).unwrap());
    }

    #[test]
    fn P7RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        assert!(refine::check_refinement(SystemRepresentation::Component(optimized_components.get(38).unwrap().clone()), SystemRepresentation::Component(optimized_components.get(38).unwrap().clone()), decl.borrow()).unwrap());
    }

    #[test]
    fn L1RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        assert!(refine::check_refinement(SystemRepresentation::Component(optimized_components.get(39).unwrap().clone()), SystemRepresentation::Component(optimized_components.get(39).unwrap().clone()), decl.borrow()).unwrap());
    }

    #[test]
    fn L2RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        assert!(refine::check_refinement(SystemRepresentation::Component(optimized_components.get(40).unwrap().clone()), SystemRepresentation::Component(optimized_components.get(40).unwrap().clone()), decl.borrow()).unwrap());
    }

    #[test]
    fn L3RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        assert!(refine::check_refinement(SystemRepresentation::Component(optimized_components.get(41).unwrap().clone()), SystemRepresentation::Component(optimized_components.get(41).unwrap().clone()), decl.borrow()).unwrap());
    }

    #[test]
    fn L4RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        assert!(refine::check_refinement(SystemRepresentation::Component(optimized_components.get(42).unwrap().clone()), SystemRepresentation::Component(optimized_components.get(42).unwrap().clone()), decl.borrow()).unwrap());
    }

    #[test]
    fn L5RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        assert!(refine::check_refinement(SystemRepresentation::Component(optimized_components.get(43).unwrap().clone()), SystemRepresentation::Component(optimized_components.get(43).unwrap().clone()), decl.borrow()).unwrap());
    }

    #[test]
    fn L6RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        assert!(refine::check_refinement(SystemRepresentation::Component(optimized_components.get(44).unwrap().clone()), SystemRepresentation::Component(optimized_components.get(44).unwrap().clone()), decl.borrow()).unwrap());
    }

    #[test]
    fn L7RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        assert!(refine::check_refinement(SystemRepresentation::Component(optimized_components.get(45).unwrap().clone()), SystemRepresentation::Component(optimized_components.get(45).unwrap().clone()), decl.borrow()).unwrap());
    }

    #[test]
    fn Z1RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        assert!(refine::check_refinement(SystemRepresentation::Component(optimized_components.get(46).unwrap().clone()), SystemRepresentation::Component(optimized_components.get(46).unwrap().clone()), decl.borrow()).unwrap());
    }

    #[test]
    fn Z2RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        assert!(refine::check_refinement(SystemRepresentation::Component(optimized_components.get(47).unwrap().clone()), SystemRepresentation::Component(optimized_components.get(47).unwrap().clone()), decl.borrow()).unwrap());
    }

    #[test]
    fn Z3RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        assert!(refine::check_refinement(SystemRepresentation::Component(optimized_components.get(48).unwrap().clone()), SystemRepresentation::Component(optimized_components.get(48).unwrap().clone()), decl.borrow()).unwrap());
    }

    #[test]
    fn Z4RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        assert!(refine::check_refinement(SystemRepresentation::Component(optimized_components.get(49).unwrap().clone()), SystemRepresentation::Component(optimized_components.get(49).unwrap().clone()), decl.borrow()).unwrap());
    }

    #[test]
    fn Z5RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        assert!(refine::check_refinement(SystemRepresentation::Component(optimized_components.get(50).unwrap().clone()), SystemRepresentation::Component(optimized_components.get(50).unwrap().clone()), decl.borrow()).unwrap());
    }

    #[test]
    fn Z6RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        assert!(refine::check_refinement(SystemRepresentation::Component(optimized_components.get(51).unwrap().clone()), SystemRepresentation::Component(optimized_components.get(51).unwrap().clone()), decl.borrow()).unwrap());
    }

    #[test]
    fn Z7RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        assert!(refine::check_refinement(SystemRepresentation::Component(optimized_components.get(52).unwrap().clone()),
                                         SystemRepresentation::Component(optimized_components.get(52).unwrap().clone()),
                                         decl.borrow()).unwrap());
    }

//     // Rest of the tests

    #[test]
    fn T1T2RefinesT3() {
        //right side could not match a output from left side o1: ["ro", "o", "i", "rand"], o2 ["ro"] - Jecdar pass
        // let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        // let optimized_components = optimize_components(automataList, &decl);
        // assert!(refine::check_refinement(
        //     SystemRepresentation::Composition(Box::from(SystemRepresentation::Component(optimized_components.get(0).unwrap().clone())),
        //                                       Box::from(SystemRepresentation::Component(optimized_components.get(1).unwrap().clone()))),
        //     SystemRepresentation::Component(optimized_components.get(2).unwrap().clone()),
        //     decl.borrow()).unwrap());
    }

    #[test]
    fn C1RefinesC2() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        assert!(refine::check_refinement(SystemRepresentation::Component(optimized_components.get(3).unwrap().clone()),
                                         SystemRepresentation::Component(optimized_components.get(4).unwrap().clone()),
                                         decl.borrow()).unwrap());
    }

    #[test]
    fn C2RefinesC1() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        assert!(refine::check_refinement(SystemRepresentation::Component(optimized_components.get(4).unwrap().clone()),
                                         SystemRepresentation::Component(optimized_components.get(3).unwrap().clone()),
                                         decl.borrow()).unwrap());
    }

    #[test]
    fn T0T1T2RefinesT3() {
        //right side could not match a output from left side o1: ["dio", "ro", "o", "i", "rand"], o2 ["ro"] - Jecdar pass
        // let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        // let optimized_components = optimize_components(automataList, &decl);
        // assert!(refine::check_refinement(
        //     SystemRepresentation::Composition(
        //         Box::from(SystemRepresentation::Composition(Box::from(SystemRepresentation::Component(optimized_components.get(11).unwrap().clone())),
        //                                                     Box::from(SystemRepresentation::Component(optimized_components.get(0).unwrap().clone())),)),
        //         Box::from(SystemRepresentation::Component(optimized_components.get(1).unwrap().clone()))),
        //     SystemRepresentation::Component(optimized_components.get(2).unwrap().clone()),
        //     decl.borrow()).unwrap());
    }

    #[test]
    fn F1F2RefinesF3() {
        //right side could not match a output from left side o1: ["ro", "i", "o", "rand"], o2 ["ro"] - Jecdar pass
        // let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        // let optimized_components = optimize_components(automataList, &decl);
        // assert!(refine::check_refinement(
        //     SystemRepresentation::Composition(Box::from(SystemRepresentation::Component(optimized_components.get(7).unwrap().clone())),
        //                                       Box::from(SystemRepresentation::Component(optimized_components.get(8).unwrap().clone()))),
        //     SystemRepresentation::Component(optimized_components.get(9).unwrap().clone()),
        //     decl.borrow()).unwrap());
    }

    #[test]
    fn T4RefinesT3() {
        //right side could not match a output from left side o1: ["ro", "go"], o2 ["ro"] - jecdar pass
        // let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        // let optimized_components = optimize_components(automataList, &decl);
        // assert!(refine::check_refinement(SystemRepresentation::Component(optimized_components.get(10).unwrap().clone()),
        //                                  SystemRepresentation::Component(optimized_components.get(2).unwrap().clone()),
        //                                  decl.borrow()).unwrap());
    }

    #[test]
    fn T6RefinesT5() {
        //right side could not match a output from left side o1: ["ro", "go"], o2 ["ro"] - jecdar pass
        // let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        // let optimized_components = optimize_components(automataList, &decl);
        // assert!(refine::check_refinement(SystemRepresentation::Component(optimized_components.get(13).unwrap().clone()),
        //                                  SystemRepresentation::Component(optimized_components.get(12).unwrap().clone()),
        //                                  decl.borrow()).unwrap());
    }

    #[test]
    fn T7NotRefinesT8() {
        //Refinement passes, tho should fail ! same symbols
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        // should fail because left side has more inputs
        assert!(!refine::check_refinement(SystemRepresentation::Component(optimized_components.get(14).unwrap().clone()),
                                          SystemRepresentation::Component(optimized_components.get(15).unwrap().clone()),
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn T9NotRefinesT8() {
        //Refinement passes, tho should fail ! same symbols
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        assert!(!refine::check_refinement(SystemRepresentation::Component(optimized_components.get(16).unwrap().clone()),
                                          SystemRepresentation::Component(optimized_components.get(15).unwrap().clone()),
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn T10NotRefinesT11() {
        //Refinement passes, tho should fail !
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
// should fail because left side has more inputs
        assert!(!refine::check_refinement(SystemRepresentation::Component(optimized_components.get(17).unwrap().clone()),
                                          SystemRepresentation::Component(optimized_components.get(18).unwrap().clone()),
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn N1RefinesN2() {
        //right side could not match a output from left side o1: ["o1", "o2"], o2 ["o1"]
        // let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        // let optimized_components = optimize_components(automataList, &decl);
        // assert!(refine::check_refinement(SystemRepresentation::Component(optimized_components.get(19).unwrap().clone()),
        //                                  SystemRepresentation::Component(optimized_components.get(20).unwrap().clone()),
        //                                  decl.borrow()).unwrap());

    }

    #[test]
    fn N3NotRefinesN4() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        // should fail because right side has more inputs
        assert!(!refine::check_refinement(SystemRepresentation::Component(optimized_components.get(21).unwrap().clone()),
                                          SystemRepresentation::Component(optimized_components.get(22).unwrap().clone()),
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn D2RefinesD1() {
        // right side could not match a output from left side o1: ["o1", "o2", "o"], o2 ["o1", "o2"]
        // let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        // let optimized_components = optimize_components(automataList, &decl);
        // assert!(refine::check_refinement(SystemRepresentation::Component(optimized_components.get(24).unwrap().clone()),
        //                                  SystemRepresentation::Component(optimized_components.get(23).unwrap().clone()),
        //                                  decl.borrow()).unwrap());
    }

    #[test]
    fn D1NotRefinesD2() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
// should fail because right side has more outputs
        assert!(!refine::check_refinement(SystemRepresentation::Component(optimized_components.get(23).unwrap().clone()),
                                          SystemRepresentation::Component(optimized_components.get(24).unwrap().clone()),
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn K1NotRefinesK2() {
        //Should fail, but passes ?
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        assert!(!refine::check_refinement(SystemRepresentation::Component(optimized_components.get(25).unwrap().clone()),
                                          SystemRepresentation::Component(optimized_components.get(26).unwrap().clone()),
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn K3NotRefinesK4() {
        //should fail, tho passes ?!
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        assert!(!refine::check_refinement(SystemRepresentation::Component(optimized_components.get(27).unwrap().clone()),
                                          SystemRepresentation::Component(optimized_components.get(28).unwrap().clone()),
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn K5NotRefinesK6() {
        //Should fail, tho passes ?!?
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        assert!(!refine::check_refinement(SystemRepresentation::Component(optimized_components.get(29).unwrap().clone()),
                                          SystemRepresentation::Component(optimized_components.get(30).unwrap().clone()),
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn P0RefinesP1() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        assert!(refine::check_refinement(SystemRepresentation::Component(optimized_components.get(31).unwrap().clone()),
                                         SystemRepresentation::Component(optimized_components.get(32).unwrap().clone())
                                         , decl.borrow()).unwrap());
    }

    #[test]
    fn P2NotRefinesP3() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        assert!(!refine::check_refinement(SystemRepresentation::Component(optimized_components.get(33).unwrap().clone()),
                                          SystemRepresentation::Component(optimized_components.get(34).unwrap().clone()),
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn P4RefinesP5() {
        //right side could not match a output from left side o1: ["o"], o2 [] -- jecdar pass
        //making o inner output would solve the problem
        // let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        // let optimized_components = optimize_components(automataList, &decl);
        // assert!(refine::check_refinement(SystemRepresentation::Component(optimized_components.get(35).unwrap().clone()),
        //                                  SystemRepresentation::Component(optimized_components.get(36).unwrap().clone()),
        //                                  decl.borrow()).unwrap());
    }

    #[test]
    fn P6RefinesP7() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        assert!(refine::check_refinement(SystemRepresentation::Component(optimized_components.get(37).unwrap().clone()),
                                         SystemRepresentation::Component(optimized_components.get(38).unwrap().clone())
                                         , decl.borrow()).unwrap());
    }

    #[test]
    fn L1L2NotRefinesL3(){
        //test passes but for wrong reasons ?
        //right side could not match a output from left side o1: ["o", "ro"], o2 ["ro"]
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        assert!(!refine::check_refinement(
            SystemRepresentation::Composition(Box::from(SystemRepresentation::Component(optimized_components.get(39).unwrap().clone())),
                                              Box::from(SystemRepresentation::Component(optimized_components.get(40).unwrap().clone()))),
            SystemRepresentation::Component(optimized_components.get(41).unwrap().clone()),
            decl.borrow()).unwrap());
}

    #[test]
    fn L4RefinesL5() {
        //should pass tho fails
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        assert!(refine::check_refinement(SystemRepresentation::Component(optimized_components.get(42).unwrap().clone()),
                                         SystemRepresentation::Component(optimized_components.get(43).unwrap().clone()),
                                         decl.borrow()).unwrap());
    }

    #[test]
    fn L6NotRefinesL7() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        assert!(!refine::check_refinement(SystemRepresentation::Component(optimized_components.get(44).unwrap().clone()),
                                          SystemRepresentation::Component(optimized_components.get(45).unwrap().clone()),
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn Z1RefinesZ2() {
        //right side could not match a output from left side o1: ["o", "go"], o2 ["o"]
        // let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        // let optimized_components = optimize_components(automataList, &decl);
        // assert!(refine::check_refinement(SystemRepresentation::Component(optimized_components.get(46).unwrap().clone()),
        //                                  SystemRepresentation::Component(optimized_components.get(47).unwrap().clone()),
        //                                  decl.borrow()).unwrap());
    }

    #[test]
    fn Z3RefinesZ4() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        assert!(refine::check_refinement(SystemRepresentation::Component(optimized_components.get(48).unwrap().clone()),
                                         SystemRepresentation::Component(optimized_components.get(49).unwrap().clone()),
                                         decl.borrow()).unwrap());
    }

    #[test]
    fn Z5Z6NotRefinesZ7() {
        //right side could not match a output from left side o1: ["i"], o2 []
        //test passes but for wrong reasons ?
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        assert!(!refine::check_refinement(
            SystemRepresentation::Composition(Box::from(SystemRepresentation::Component(optimized_components.get(50).unwrap().clone())),
                                              Box::from(SystemRepresentation::Component(optimized_components.get(51).unwrap().clone()))),
            SystemRepresentation::Component(optimized_components.get(52).unwrap().clone()),
            decl.borrow()).unwrap());
    }

    #[test]
    fn Q1NotRefinesQ2() {
        //refinement should not hold tho it holds ?
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        assert!(!refine::check_refinement(SystemRepresentation::Component(optimized_components.get(53).unwrap().clone()),
                                          SystemRepresentation::Component(optimized_components.get(54).unwrap().clone()),
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn Q2NotRefinesQ1() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        assert!(!refine::check_refinement(SystemRepresentation::Component(optimized_components.get(54).unwrap().clone()),
                                          SystemRepresentation::Component(optimized_components.get(53).unwrap().clone()),
                                          decl.borrow()).unwrap());
    }
}