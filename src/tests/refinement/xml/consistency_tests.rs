#[cfg(test)]
mod consistency_tests {
    use crate::tests::refinement::Helper::optimize_components;
    use crate::DataReader::{parse_queries, xml_parser};
    use crate::ModelObjects::queries::Query;
    use crate::ModelObjects::representations::SystemRepresentation;
    use crate::System::extract_system_rep::create_system_rep_from_query;

    static PATH: &str = "samples/xml/ConsTests.xml";

    #[test]
    fn testG1() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl); // input enabler samt opsætter clock indcies
        let query = parse_queries::parse("consistency: G1").remove(0);
        let q = Query {
            query: Option::from(query),
            comment: "".to_string(),
        };
        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;

        assert!(leftSys.precheck_sys_rep());
    }
    #[test]
    fn testG2() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("consistency: G2").remove(0);
        let q = Query {
            query: Option::from(query),
            comment: "".to_string(),
        };
        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        assert!(leftSys.precheck_sys_rep());
    }

    #[test]
    fn testG3() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("consistency: G3").remove(0);
        let q = Query {
            query: Option::from(query),
            comment: "".to_string(),
        };
        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        assert!(!leftSys.precheck_sys_rep());
    }

    #[test]
    fn testG4() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("consistency: G4").remove(0);
        let q = Query {
            query: Option::from(query),
            comment: "".to_string(),
        };
        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        assert!(!leftSys.precheck_sys_rep());
    }

    #[test]
    fn testG5() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("consistency: G5").remove(0);
        let q = Query {
            query: Option::from(query),
            comment: "".to_string(),
        };
        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        assert!(!leftSys.precheck_sys_rep());
    }

    #[test]
    fn testG6() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("consistency: G6").remove(0);
        let q = Query {
            query: Option::from(query),
            comment: "".to_string(),
        };
        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        assert!(leftSys.precheck_sys_rep());
    }

    #[test]
    fn testG7() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("consistency: G7").remove(0);
        let q = Query {
            query: Option::from(query),
            comment: "".to_string(),
        };
        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        assert!(!leftSys.precheck_sys_rep());
    }

    #[test]
    fn testG8() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("consistency: G8").remove(0);
        let q = Query {
            query: Option::from(query),
            comment: "".to_string(),
        };
        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        assert!(leftSys.precheck_sys_rep());
    }

    #[test]
    fn testG9() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("consistency: G9").remove(0);
        let q = Query {
            query: Option::from(query),
            comment: "".to_string(),
        };
        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        assert!(!leftSys.precheck_sys_rep());
    }

    #[test]
    fn testG10() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("consistency: G10").remove(0);
        let q = Query {
            query: Option::from(query),
            comment: "".to_string(),
        };
        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        assert!(!leftSys.precheck_sys_rep());
    }

    #[test]
    fn testG11() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("consistency: G11").remove(0);
        let q = Query {
            query: Option::from(query),
            comment: "".to_string(),
        };
        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        assert!(!leftSys.precheck_sys_rep());
    }

    #[test]
    fn testG12() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("consistency: G12").remove(0);
        let q = Query {
            query: Option::from(query),
            comment: "".to_string(),
        };
        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        assert!(!leftSys.precheck_sys_rep());
    }

    #[test]
    fn testG13() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("consistency: G13").remove(0);
        let q = Query {
            query: Option::from(query),
            comment: "".to_string(),
        };
        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        assert!(leftSys.precheck_sys_rep());
    }

    #[test]
    fn testG14() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("consistency: G14").remove(0);
        let q = Query {
            query: Option::from(query),
            comment: "".to_string(),
        };
        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        assert!(!leftSys.precheck_sys_rep());
    }

    #[test]
    fn testG15() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("consistency: G15").remove(0);
        let q = Query {
            query: Option::from(query),
            comment: "".to_string(),
        };
        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        assert!(leftSys.precheck_sys_rep());
    }

    #[test]
    fn testG16() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("consistency: G16").remove(0);
        let q = Query {
            query: Option::from(query),
            comment: "".to_string(),
        };
        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        assert!(!leftSys.precheck_sys_rep());
    }

    #[test]
    fn testG17() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("consistency: G17").remove(0);
        let q = Query {
            query: Option::from(query),
            comment: "".to_string(),
        };
        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        assert!(leftSys.precheck_sys_rep());
    }

    #[test]
    fn testG18() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("consistency: G18").remove(0);
        let q = Query {
            query: Option::from(query),
            comment: "".to_string(),
        };
        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        assert!(leftSys.precheck_sys_rep());
    }

    #[test]
    fn testG19() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("consistency: G19").remove(0);
        let q = Query {
            query: Option::from(query),
            comment: "".to_string(),
        };
        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        assert!(!leftSys.precheck_sys_rep());
    }

    #[test]
    fn testG20() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("consistency: G20").remove(0);
        let q = Query {
            query: Option::from(query),
            comment: "".to_string(),
        };
        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        assert!(leftSys.precheck_sys_rep());
    }

    #[test]
    fn testG21() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("consistency: G21").remove(0);
        let q = Query {
            query: Option::from(query),
            comment: "".to_string(),
        };
        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        assert!(leftSys.precheck_sys_rep());
    }
}
