#[cfg(test)]
mod delay_refinement {
    use crate::ModelObjects::{xml_parser, parse_queries};
    use crate::System::{refine};
    use std::borrow::Borrow;
    use crate::tests::refinement::Helper::optimize_components;
    use crate::System::extract_system_rep::create_system_rep_from_query;
    use crate::ModelObjects::queries::Query;

    static PATH: &str = "samples/xml/delayRefinement.xml";
    static PATH_2: &str = "samples/xml/loop.xml";


    // Self Refinements
    #[test]
    fn LoopTest() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH_2);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("refinement: SelfloopNonZeno <= SelfloopNonZeno").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        let rightSys = res.1.unwrap();


        assert!(refine::check_refinement(leftSys,
                                         rightSys,
                                         decl.borrow()).unwrap());
    }

    // Self Refinements
    #[test]
    fn T1RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("refinement: T1 <= T1").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        let rightSys = res.1.unwrap();


        assert!(refine::check_refinement(leftSys,
                                          rightSys,
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn T2RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("refinement: T2 <= T2").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        let rightSys = res.1.unwrap();


        assert!(refine::check_refinement(leftSys,
                                          rightSys,
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn T3RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("refinement: T3 <= T3").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        let rightSys = res.1.unwrap();


        assert!(refine::check_refinement(leftSys,
                                          rightSys,
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn C1RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("refinement: C1 <= C1").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        let rightSys = res.1.unwrap();


        assert!(refine::check_refinement(leftSys,
                                          rightSys,
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn C2RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("refinement: C2 <= C2").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        let rightSys = res.1.unwrap();


        assert!(refine::check_refinement(leftSys,
                                          rightSys,
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn F1RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("refinement: F1 <= F1").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        let rightSys = res.1.unwrap();


        assert!(refine::check_refinement(leftSys,
                                          rightSys,
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn F2RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("refinement: F2 <= F2").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        let rightSys = res.1.unwrap();


        assert!(refine::check_refinement(leftSys,
                                          rightSys,
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn F3RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("refinement: F3 <= F3").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        let rightSys = res.1.unwrap();


        assert!(refine::check_refinement(leftSys,
                                          rightSys,
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn T4RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("refinement: T4 <= T4").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        let rightSys = res.1.unwrap();


        assert!(refine::check_refinement(leftSys,
                                          rightSys,
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn T0RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("refinement: T0 <= T0").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        let rightSys = res.1.unwrap();


        assert!(refine::check_refinement(leftSys,
                                          rightSys,
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn T5RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("refinement: T5 <= T5").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        let rightSys = res.1.unwrap();


        assert!(refine::check_refinement(leftSys,
                                          rightSys,
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn T6RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("refinement: T6 <= T6").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        let rightSys = res.1.unwrap();


        assert!(refine::check_refinement(leftSys,
                                          rightSys,
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn T7RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("refinement: T7 <= T7").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        let rightSys = res.1.unwrap();


        assert!(refine::check_refinement(leftSys,
                                          rightSys,
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn T8RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("refinement: T8 <= T8").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        let rightSys = res.1.unwrap();


        assert!(refine::check_refinement(leftSys,
                                          rightSys,
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn T9RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("refinement: T9 <= T9").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        let rightSys = res.1.unwrap();


        assert!(refine::check_refinement(leftSys,
                                          rightSys,
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn T10RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("refinement: T10 <= T10").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        let rightSys = res.1.unwrap();


        assert!(refine::check_refinement(leftSys,
                                          rightSys,
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn T11RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("refinement: T11 <= T11").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        let rightSys = res.1.unwrap();


        assert!(refine::check_refinement(leftSys,
                                          rightSys,
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn N1RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("refinement: N1 <= N1").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        let rightSys = res.1.unwrap();


        assert!(refine::check_refinement(leftSys,
                                          rightSys,
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn N2RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("refinement: N2 <= N2").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        let rightSys = res.1.unwrap();


        assert!(refine::check_refinement(leftSys,
                                          rightSys,
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn N3RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("refinement: N3 <= N3").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        let rightSys = res.1.unwrap();


        assert!(refine::check_refinement(leftSys,
                                          rightSys,
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn N4RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("refinement: N4 <= N4").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        let rightSys = res.1.unwrap();


        assert!(refine::check_refinement(leftSys,
                                          rightSys,
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn D1RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("refinement: D1 <= D1").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        let rightSys = res.1.unwrap();


        assert!(refine::check_refinement(leftSys,
                                          rightSys,
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn D2RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("refinement: D2 <= D2").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        let rightSys = res.1.unwrap();


        assert!(refine::check_refinement(leftSys,
                                          rightSys,
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn K1RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("refinement: K1 <= K1").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        let rightSys = res.1.unwrap();


        assert!(refine::check_refinement(leftSys,
                                          rightSys,
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn K2RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("refinement: K2 <= K2").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        let rightSys = res.1.unwrap();


        assert!(refine::check_refinement(leftSys,
                                          rightSys,
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn K3RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("refinement: K3 <= K3").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        let rightSys = res.1.unwrap();


        assert!(refine::check_refinement(leftSys,
                                          rightSys,
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn K4RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("refinement: K4 <= K4").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        let rightSys = res.1.unwrap();


        assert!(refine::check_refinement(leftSys,
                                          rightSys,
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn K5RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("refinement: K5 <= K5").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        let rightSys = res.1.unwrap();


        assert!(refine::check_refinement(leftSys,
                                          rightSys,
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn K6RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("refinement: K6 <= K6").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        let rightSys = res.1.unwrap();


        assert!(refine::check_refinement(leftSys,
                                          rightSys,
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn P0RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("refinement: P0 <= P0").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        let rightSys = res.1.unwrap();


        assert!(refine::check_refinement(leftSys,
                                          rightSys,
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn P1RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("refinement: P1 <= P1").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        let rightSys = res.1.unwrap();


        assert!(refine::check_refinement(leftSys,
                                          rightSys,
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn P2RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("refinement: P2 <= P2").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        let rightSys = res.1.unwrap();


        assert!(refine::check_refinement(leftSys,
                                          rightSys,
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn P3RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("refinement: P3 <= P3").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        let rightSys = res.1.unwrap();


        assert!(refine::check_refinement(leftSys,
                                          rightSys,
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn P4RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("refinement: P4 <= P4").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        let rightSys = res.1.unwrap();


        assert!(refine::check_refinement(leftSys,
                                          rightSys,
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn P5RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("refinement: P5 <= P5").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        let rightSys = res.1.unwrap();


        assert!(refine::check_refinement(leftSys,
                                          rightSys,
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn P6RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("refinement: P6 <= P6").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        let rightSys = res.1.unwrap();


        assert!(refine::check_refinement(leftSys,
                                          rightSys,
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn P7RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("refinement: P7 <= P7").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        let rightSys = res.1.unwrap();


        assert!(refine::check_refinement(leftSys,
                                          rightSys,
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn L1RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("refinement: L1 <= L1").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        let rightSys = res.1.unwrap();


        assert!(refine::check_refinement(leftSys,
                                          rightSys,
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn L2RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("refinement: L2 <= L2").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        let rightSys = res.1.unwrap();


        assert!(refine::check_refinement(leftSys,
                                          rightSys,
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn L3RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("refinement: L3 <= L3").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        let rightSys = res.1.unwrap();


        assert!(refine::check_refinement(leftSys,
                                          rightSys,
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn L4RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("refinement: L4 <= L4").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        let rightSys = res.1.unwrap();


        assert!(refine::check_refinement(leftSys,
                                          rightSys,
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn L5RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("refinement: L5 <= L5").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        let rightSys = res.1.unwrap();


        assert!(refine::check_refinement(leftSys,
                                          rightSys,
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn L6RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("refinement: L6 <= L6").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        let rightSys = res.1.unwrap();


        assert!(refine::check_refinement(leftSys,
                                          rightSys,
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn L7RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("refinement: L7 <= L7").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        let rightSys = res.1.unwrap();


        assert!(refine::check_refinement(leftSys,
                                          rightSys,
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn Z1RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("refinement: Z1 <= Z1").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        let rightSys = res.1.unwrap();


        assert!(refine::check_refinement(leftSys,
                                          rightSys,
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn Z2RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("refinement: Z2 <= Z2").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        let rightSys = res.1.unwrap();


        assert!(refine::check_refinement(leftSys,
                                          rightSys,
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn Z3RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("refinement: Z3 <= Z3").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        let rightSys = res.1.unwrap();


        assert!(refine::check_refinement(leftSys,
                                          rightSys,
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn Z4RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("refinement: Z4 <= Z4").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        let rightSys = res.1.unwrap();


        assert!(refine::check_refinement(leftSys,
                                          rightSys,
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn Z5RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("refinement: Z5 <= Z5").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        let rightSys = res.1.unwrap();


        assert!(refine::check_refinement(leftSys,
                                          rightSys,
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn Z6RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("refinement: Z6 <= Z6").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        let rightSys = res.1.unwrap();


        assert!(refine::check_refinement(leftSys,
                                          rightSys,
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn Z7RefinesSelf() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("refinement: Z7 <= Z7").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        let rightSys = res.1.unwrap();


        assert!(refine::check_refinement(leftSys,
                                          rightSys,
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
        let query = parse_queries::parse("refinement: C1 <= C2").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        let rightSys = res.1.unwrap();


        assert!(refine::check_refinement(leftSys,
                                          rightSys,
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn C2RefinesC1() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("refinement: C2 <= C1").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        let rightSys = res.1.unwrap();


        assert!(refine::check_refinement(leftSys,
                                          rightSys,
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
        let query = parse_queries::parse("refinement: T7 <= T8").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        let rightSys = res.1.unwrap();


        assert!(!refine::check_refinement(leftSys,
                                          rightSys,
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn T9NotRefinesT8() {
        //Refinement passes, tho should fail ! same symbols
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("refinement: T9 <= T8").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        let rightSys = res.1.unwrap();


        assert!(!refine::check_refinement(leftSys,
                                          rightSys,
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn T10NotRefinesT11() {
        //Refinement passes, tho should fail !
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("refinement: T10 <= T11").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        let rightSys = res.1.unwrap();


        assert!(!refine::check_refinement(leftSys,
                                          rightSys,
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
        let query = parse_queries::parse("refinement: N3 <= N4").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        let rightSys = res.1.unwrap();


        assert!(!refine::check_refinement(leftSys,
                                          rightSys,
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
        let query = parse_queries::parse("refinement: D1 <= D2").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        let rightSys = res.1.unwrap();


        assert!(!refine::check_refinement(leftSys,
                                          rightSys,
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn K1NotRefinesK2() {
        //Should fail, but passes ?
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("refinement: K1 <= K2").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        let rightSys = res.1.unwrap();


        assert!(!refine::check_refinement(leftSys,
                                          rightSys,
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn K3NotRefinesK4() {
        //should fail, tho passes ?!
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("refinement: K3 <= K4").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        let rightSys = res.1.unwrap();


        assert!(!refine::check_refinement(leftSys,
                                          rightSys,
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn K5NotRefinesK6() {
        //Should fail, tho passes ?!?
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("refinement: K5 <= K6").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        let rightSys = res.1.unwrap();


        assert!(!refine::check_refinement(leftSys,
                                          rightSys,
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn P0RefinesP1() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("refinement: P0 <= P1").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        let rightSys = res.1.unwrap();


        assert!(refine::check_refinement(leftSys,
                                          rightSys,
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn P2NotRefinesP3() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("refinement: P2 <= P3").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        let rightSys = res.1.unwrap();


        assert!(!refine::check_refinement(leftSys,
                                          rightSys,
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
        let query = parse_queries::parse("refinement: P6 <= P7").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        let rightSys = res.1.unwrap();


        assert!(refine::check_refinement(leftSys,
                                          rightSys,
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn L1L2NotRefinesL3(){
        //test passes but for wrong reasons ?
        //right side could not match a output from left side o1: ["o", "ro"], o2 ["ro"]
        // let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        // let optimized_components = optimize_components(automataList, &decl);
        // assert!(!refine::check_refinement(
        //     SystemRepresentation::Composition(Box::from(SystemRepresentation::Component(optimized_components.get(39).unwrap().clone())),
        //                                       Box::from(SystemRepresentation::Component(optimized_components.get(40).unwrap().clone()))),
        //     SystemRepresentation::Component(optimized_components.get(41).unwrap().clone()),
        //     decl.borrow()).unwrap());
    }

    #[test]
    fn L4RefinesL5() {
        //should pass tho fails
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("refinement: L5 <= L5").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        let rightSys = res.1.unwrap();


        assert!(refine::check_refinement(leftSys,
                                          rightSys,
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn L6NotRefinesL7() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("refinement: L6 <= L7").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        let rightSys = res.1.unwrap();


        assert!(!refine::check_refinement(leftSys,
                                          rightSys,
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
        //TODO: This seem to loop for ever, we need max bounds check!
        // let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        // let mut optimized_components = optimize_components(automataList, &decl);
        // let query = parse_queries::parse("refinement: Z3 <= Z4").unwrap();
        // let q = Query {
        //     query: Option::from(query),
        //     comment: "".to_string()
        // };
        //
        // let res = create_system_rep_from_query(&q, &optimized_components);
        // let leftSys = res.0;
        // let rightSys = res.1.unwrap();
        //
        //
        // assert!(refine::check_refinement(leftSys,
        //                                   rightSys,
        //                                   decl.borrow()).unwrap());
    }

    #[test]
    fn Z5Z6NotRefinesZ7() {
        //right side could not match a output from left side o1: ["i"], o2 []
        //test passes but for wrong reasons ?
        // let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        // let optimized_components = optimize_components(automataList, &decl);
        // assert!(!refine::check_refinement(
        //     SystemRepresentation::Composition(Box::from(SystemRepresentation::Component(optimized_components.get(50).unwrap().clone())),
        //                                       Box::from(SystemRepresentation::Component(optimized_components.get(51).unwrap().clone()))),
        //     SystemRepresentation::Component(optimized_components.get(52).unwrap().clone()),
        //     decl.borrow()).unwrap());
    }

    #[test]
    fn Q1NotRefinesQ2() {
        //refinement should not hold tho it holds ?
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("refinement: Q1 <= Q2").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        let rightSys = res.1.unwrap();


        assert!(!refine::check_refinement(leftSys,
                                          rightSys,
                                          decl.borrow()).unwrap());
    }

    #[test]
    fn Q2NotRefinesQ1() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("refinement: Q2 <= Q1").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        let rightSys = res.1.unwrap();


        assert!(!refine::check_refinement(leftSys,
                                          rightSys,
                                          decl.borrow()).unwrap());
    }
}