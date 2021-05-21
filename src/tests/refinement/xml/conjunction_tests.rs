#[cfg(test)]
mod conjunction_tests {
    use crate::tests::refinement::Helper::optimize_components;
    use crate::ModelObjects::representations::SystemRepresentation;
    use crate::ModelObjects::xml_parser;
    use crate::System::refine;
    use std::borrow::Borrow;

    static PATH: &str = "samples/xml/conjun.xml";

    #[test]
    fn P0ConjP1RefP2() {
        //passes the test but for wrong reasons ?
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);

        assert_eq!(optimized_components.get(0).unwrap().get_name(), "P0");
        assert_eq!(optimized_components.get(1).unwrap().get_name(), "P1");
        assert_eq!(optimized_components.get(2).unwrap().get_name(), "P2");

        assert!(!refine::check_refinement(
            SystemRepresentation::Conjunction(
                Box::from(SystemRepresentation::Component(
                    optimized_components.get(1).unwrap().clone()
                )),
                Box::from(SystemRepresentation::Component(
                    optimized_components.get(0).unwrap().clone()
                ))
            ),
            SystemRepresentation::Component(optimized_components.get(2).unwrap().clone()),
            decl.borrow()
        )
        .unwrap());
    }

    #[test]
    fn P3ConjP4CompP5RefP6() {
        //passes the test but for wrong reasons ?
        //right side could not match a output from left side o1: ["o", "o1", "o2", "k", "o", "o1", "o2", "k", "go"], o2 ["o", "o1", "o2", "go"]
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);

        assert_eq!(optimized_components.get(3).unwrap().get_name(), "P3");
        assert_eq!(optimized_components.get(4).unwrap().get_name(), "P4");
        assert_eq!(optimized_components.get(5).unwrap().get_name(), "P5");
        assert_eq!(optimized_components.get(6).unwrap().get_name(), "P6");

        assert!(!refine::check_refinement(
            SystemRepresentation::Conjunction(
                Box::from(SystemRepresentation::Composition(
                    Box::from(SystemRepresentation::Component(
                        optimized_components.get(3).unwrap().clone()
                    )),
                    Box::from(SystemRepresentation::Component(
                        optimized_components.get(4).unwrap().clone()
                    ))
                )),
                Box::from(SystemRepresentation::Component(
                    optimized_components.get(5).unwrap().clone()
                ))
            ),
            SystemRepresentation::Component(optimized_components.get(6).unwrap().clone()),
            decl.borrow()
        )
        .unwrap());
    }

    #[test]
    fn P7ConjP8ConjP9RefP10() {
        //tests fails with weird DBM's, Parsing error ?
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);

        assert_eq!(optimized_components.get(7).unwrap().get_name(), "P7");
        assert_eq!(optimized_components.get(8).unwrap().get_name(), "P8");
        assert_eq!(optimized_components.get(9).unwrap().get_name(), "P9");
        assert_eq!(optimized_components.get(10).unwrap().get_name(), "P10");

        assert!(!refine::check_refinement(
            SystemRepresentation::Conjunction(
                Box::from(SystemRepresentation::Conjunction(
                    Box::from(SystemRepresentation::Component(
                        optimized_components.get(7).unwrap().clone()
                    )),
                    Box::from(SystemRepresentation::Component(
                        optimized_components.get(8).unwrap().clone()
                    ))
                )),
                Box::from(SystemRepresentation::Component(
                    optimized_components.get(9).unwrap().clone()
                ))
            ),
            SystemRepresentation::Component(optimized_components.get(10).unwrap().clone()),
            decl.borrow()
        )
        .unwrap());
    }

    #[test]
    fn P11ConjP12RefP13() {
        //passes the test but for wrong reasons ?
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);

        assert_eq!(optimized_components.get(11).unwrap().get_name(), "P11");
        assert_eq!(optimized_components.get(12).unwrap().get_name(), "P12");
        assert_eq!(optimized_components.get(13).unwrap().get_name(), "P13");

        assert!(!refine::check_refinement(
            SystemRepresentation::Conjunction(
                Box::from(SystemRepresentation::Component(
                    optimized_components.get(11).unwrap().clone()
                )),
                Box::from(SystemRepresentation::Component(
                    optimized_components.get(12).unwrap().clone()
                ))
            ),
            SystemRepresentation::Component(optimized_components.get(13).unwrap().clone()),
            decl.borrow()
        )
        .unwrap());
    }
}
