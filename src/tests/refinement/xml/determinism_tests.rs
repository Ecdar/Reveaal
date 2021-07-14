#[cfg(test)]
mod determinism_tests {
    use crate::tests::refinement::Helper::optimize_components;
    use crate::DataReader::{parse_queries, xml_parser};
    use crate::ModelObjects::queries::Query;
    use crate::System::extract_system_rep::create_system_rep_from_query;

    static PATH: &str = "samples/xml/ConsTests.xml";

    #[test]
    fn testG1() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("determinism: G1").remove(0);
        let q = Query {
            query: Option::from(query),
            comment: "".to_string(),
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        assert!(leftSys.all_components_are_deterministic());
    }
    #[test]
    fn testG2() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("determinism: G2").remove(0);
        let q = Query {
            query: Option::from(query),
            comment: "".to_string(),
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        assert!(leftSys.all_components_are_deterministic());
    }

    #[test]
    fn testG3() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("determinism: G3").remove(0);
        let q = Query {
            query: Option::from(query),
            comment: "".to_string(),
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        assert!(leftSys.all_components_are_deterministic());
    }
    #[test]
    fn testG4() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("determinism: G4").remove(0);
        let q = Query {
            query: Option::from(query),
            comment: "".to_string(),
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        assert!(leftSys.all_components_are_deterministic());
    }

    #[test]
    fn testG5() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("determinism: G5").remove(0);
        let q = Query {
            query: Option::from(query),
            comment: "".to_string(),
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        assert!(leftSys.all_components_are_deterministic());
    }
    #[test]
    fn testG6() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("determinism: G6").remove(0);
        let q = Query {
            query: Option::from(query),
            comment: "".to_string(),
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        assert!(leftSys.all_components_are_deterministic());
    }

    #[test]
    fn testG7() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("determinism: G7").remove(0);
        let q = Query {
            query: Option::from(query),
            comment: "".to_string(),
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        assert!(leftSys.all_components_are_deterministic());
    }

    #[test]
    fn testG8() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("determinism: G8").remove(0);
        let q = Query {
            query: Option::from(query),
            comment: "".to_string(),
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        assert!(leftSys.all_components_are_deterministic());
    }

    #[test]
    fn testG9() {
        // shouldn't be deterministic
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("determinism: G9").remove(0);
        let q = Query {
            query: Option::from(query),
            comment: "".to_string(),
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        assert!(!leftSys.all_components_are_deterministic());
    }

    #[test]
    fn testG10() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("determinism: G10").remove(0);
        let q = Query {
            query: Option::from(query),
            comment: "".to_string(),
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        assert!(leftSys.all_components_are_deterministic());
    }

    #[test]
    fn testG11() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("determinism: G11").remove(0);
        let q = Query {
            query: Option::from(query),
            comment: "".to_string(),
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        assert!(leftSys.all_components_are_deterministic());
    }

    #[test]
    fn testG12() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("determinism: G12").remove(0);
        let q = Query {
            query: Option::from(query),
            comment: "".to_string(),
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        assert!(leftSys.all_components_are_deterministic());
    }

    #[test]
    fn testG13() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("determinism: G13").remove(0);
        let q = Query {
            query: Option::from(query),
            comment: "".to_string(),
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        assert!(leftSys.all_components_are_deterministic());
    }

    #[test]
    fn testG14() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("determinism: G14").remove(0);
        let q = Query {
            query: Option::from(query),
            comment: "".to_string(),
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        assert!(!leftSys.all_components_are_deterministic());
    }

    #[test]
    fn testG15() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("determinism: G15").remove(0);
        let q = Query {
            query: Option::from(query),
            comment: "".to_string(),
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        assert!(leftSys.all_components_are_deterministic());
    }

    #[test]
    fn testG16() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("determinism: G16").remove(0);
        let q = Query {
            query: Option::from(query),
            comment: "".to_string(),
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        assert!(!leftSys.all_components_are_deterministic());
    }

    #[test]
    fn testG17() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("determinism: G17").remove(0);
        let q = Query {
            query: Option::from(query),
            comment: "".to_string(),
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        assert!(leftSys.all_components_are_deterministic());
    }

    #[test]
    fn testG22() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("determinism: G22").remove(0);
        let q = Query {
            query: Option::from(query),
            comment: "".to_string(),
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        assert!(leftSys.all_components_are_deterministic());
    }

    #[test]
    fn testG23() {
        // shouldn't be deterministic
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("determinism: G23").remove(0);
        let q = Query {
            query: Option::from(query),
            comment: "".to_string(),
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        assert!(!leftSys.all_components_are_deterministic());
    }
}
