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
        let query = parse_queries::parse("consistency: G1").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string(),
        };
        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        match leftSys {
            SystemRepresentation::Component(Component) => {
                assert!(Component.check_consistency(true));
            }
            _ => {}
        }
    }
    #[test]
    fn testG2() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("consistency: G2").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string(),
        };
        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        match leftSys {
            SystemRepresentation::Component(Component) => {
                assert!(Component.check_consistency(true));
            }
            _ => {}
        }
    }

    #[test]
    fn testG3() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("consistency: G3").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string(),
        };
        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        match leftSys {
            SystemRepresentation::Component(Component) => {
                assert!(!Component.check_consistency(true));
            }
            _ => {}
        }
    }

    #[test]
    fn testG4() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("consistency: G4").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string(),
        };
        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        match leftSys {
            SystemRepresentation::Component(Component) => {
                assert!(!Component.check_consistency(true));
            }
            _ => {}
        }
    }

    #[test]
    fn testG5() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("consistency: G5").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string(),
        };
        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        match leftSys {
            SystemRepresentation::Component(Component) => {
                assert!(!Component.check_consistency(true));
            }
            _ => {}
        }
    }

    #[test]
    fn testG6() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("consistency: G6").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string(),
        };
        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        match leftSys {
            SystemRepresentation::Component(Component) => {
                assert!(Component.check_consistency(true));
            }
            _ => {}
        }
    }

    #[test]
    fn testG7() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("consistency: G7").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string(),
        };
        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        match leftSys {
            SystemRepresentation::Component(Component) => {
                assert!(!Component.check_consistency(true));
            }
            _ => {}
        }
    }

    #[test]
    fn testG8() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("consistency: G8").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string(),
        };
        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        match leftSys {
            SystemRepresentation::Component(Component) => {
                assert!(Component.check_consistency(true));
            }
            _ => {}
        }
    }

    #[test]
    fn testG9() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("consistency: G9").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string(),
        };
        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        match leftSys {
            SystemRepresentation::Component(Component) => {
                assert!(!Component.check_consistency(true));
            }
            _ => {}
        }
    }

    #[test]
    fn testG10() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("consistency: G10").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string(),
        };
        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        match leftSys {
            SystemRepresentation::Component(Component) => {
                assert!(!Component.check_consistency(true));
            }
            _ => {}
        }
    }

    #[test]
    fn testG11() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("consistency: G11").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string(),
        };
        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        match leftSys {
            SystemRepresentation::Component(Component) => {
                assert!(!Component.check_consistency(true));
            }
            _ => {}
        }
    }

    #[test]
    fn testG12() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("consistency: G12").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string(),
        };
        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        match leftSys {
            SystemRepresentation::Component(Component) => {
                assert!(!Component.check_consistency(true));
            }
            _ => {}
        }
    }

    #[test]
    fn testG13() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("consistency: G13").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string(),
        };
        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        match leftSys {
            SystemRepresentation::Component(Component) => {
                assert!(Component.check_consistency(true));
            }
            _ => {}
        }
    }

    #[test]
    fn testG14() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("consistency: G14").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string(),
        };
        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        match leftSys {
            SystemRepresentation::Component(Component) => {
                assert!(!Component.check_consistency(true));
            }
            _ => {}
        }
    }

    #[test]
    fn testG15() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("consistency: G15").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string(),
        };
        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        match leftSys {
            SystemRepresentation::Component(Component) => {
                assert!(Component.check_consistency(true));
            }
            _ => {}
        }
    }

    #[test]
    fn testG16() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("consistency: G16").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string(),
        };
        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        match leftSys {
            SystemRepresentation::Component(Component) => {
                assert!(!Component.check_consistency(true));
            }
            _ => {}
        }
    }

    #[test]
    fn testG17() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("consistency: G17").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string(),
        };
        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        match leftSys {
            SystemRepresentation::Component(Component) => {
                assert!(Component.check_consistency(true));
            }
            _ => {}
        }
    }

    #[test]
    fn testG18() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("consistency: G18").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string(),
        };
        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        match leftSys {
            SystemRepresentation::Component(Component) => {
                assert!(Component.check_consistency(true));
            }
            _ => {}
        }
    }

    #[test]
    fn testG19() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("consistency: G19").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string(),
        };
        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        match leftSys {
            SystemRepresentation::Component(Component) => {
                assert!(!Component.check_consistency(true));
            }
            _ => {}
        }
    }

    #[test]
    fn testG20() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("consistency: G20").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string(),
        };
        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        match leftSys {
            SystemRepresentation::Component(Component) => {
                assert!(Component.check_consistency(true));
            }
            _ => {}
        }
    }

    #[test]
    fn testG21() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("consistency: G21").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string(),
        };
        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        match leftSys {
            SystemRepresentation::Component(Component) => {
                assert!(Component.check_consistency(true));
            }
            _ => {}
        }
    }
}
