#[cfg(test)]
mod determinism_tests {
    use crate::ModelObjects::{xml_parser, parse_queries};
    use crate::ModelObjects::representations::SystemRepresentation;
    use crate::tests::refinement::Helper::optimize_components;
    use crate::ModelObjects::queries::Query;
    use crate::System::extract_system_rep::create_system_rep_from_query;

    static PATH: &str = "samples/xml/ConsTests.xml";
    
    #[test]
    fn testG1() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("determinism: G1").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        match leftSys {
            SystemRepresentation::Component(Component) =>{
                assert!(Component.is_deterministic());
            }
            _ => {}
        }
    }
    #[test]
    fn testG2(){
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("determinism: G2").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        match leftSys {
            SystemRepresentation::Component(Component) =>{
                assert!(Component.is_deterministic());
            }
            _ => {}
        }
}

    #[test]
    fn testG3(){
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("determinism: G3").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        match leftSys {
            SystemRepresentation::Component(Component) =>{
                assert!(Component.is_deterministic());
            }
            _ => {}
        }
}
    #[test]
    fn testG4(){
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("determinism: G4").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        match leftSys {
            SystemRepresentation::Component(Component) =>{
                assert!(Component.is_deterministic());
            }
            _ => {}
        }
}

    #[test]
    fn testG5() {
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("determinism: G5").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        match leftSys {
            SystemRepresentation::Component(Component) => {
                assert!(Component.is_deterministic());
            }
            _ => {}
        }
    }
    #[test]
    fn testG6(){
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("determinism: G6").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        match leftSys {
            SystemRepresentation::Component(Component) =>{
                assert!(Component.is_deterministic());
            }
            _ => {}
        }
}

    #[test]
    fn testG7(){
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("determinism: G7").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        match leftSys {
            SystemRepresentation::Component(Component) =>{
                assert!(Component.is_deterministic());
            }
            _ => {}
        }
}

    #[test]
    fn testG8(){
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("determinism: G8").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        match leftSys {
            SystemRepresentation::Component(Component) =>{
                assert!(Component.is_deterministic());
            }
            _ => {}
        }
}

    #[test]
    fn testG9(){ // shouldn't be deterministic
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("determinism: G9").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        match leftSys {
            SystemRepresentation::Component(Component) =>{
                assert!(!Component.is_deterministic());
            }
            _ => {}
        }
}

    #[test]
    fn testG10(){
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("determinism: G10").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        match leftSys {
            SystemRepresentation::Component(Component) =>{
                assert!(Component.is_deterministic());
            }
            _ => {}
        }
}

    #[test]
    fn testG11(){
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("determinism: G11").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        match leftSys {
            SystemRepresentation::Component(Component) =>{
                assert!(Component.is_deterministic());
            }
            _ => {}
        }
}

    #[test]
    fn testG12(){
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("determinism: G12").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        match leftSys {
            SystemRepresentation::Component(Component) =>{
                assert!(Component.is_deterministic());
            }
            _ => {}
        }
}

    #[test]
    fn testG13(){
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("determinism: G13").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        match leftSys {
            SystemRepresentation::Component(Component) =>{
                assert!(Component.is_deterministic());
            }
            _ => {}
        }
}

    #[test]
    fn testG14(){
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("determinism: G14").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        match leftSys {
            SystemRepresentation::Component(Component) =>{
                assert!(!Component.is_deterministic());
            }
            _ => {}
        }
}

    #[test]
    fn testG15(){
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("determinism: G15").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        match leftSys {
            SystemRepresentation::Component(Component) =>{
                assert!(Component.is_deterministic());
            }
            _ => {}
        }
}

    #[test]
    fn testG16(){
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("determinism: G16").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        match leftSys {
            SystemRepresentation::Component(Component) =>{
                assert!(!Component.is_deterministic());
            }
            _ => {}
        }
}

    #[test]
    fn testG17(){
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("determinism: G17").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        match leftSys {
            SystemRepresentation::Component(Component) =>{
                assert!(Component.is_deterministic());
            }
            _ => {}
        }
}

    #[test]
    fn testG22(){
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("determinism: G22").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        match leftSys {
            SystemRepresentation::Component(Component) =>{
                assert!(Component.is_deterministic());
            }
            _ => {}
        }
}

    #[test]
    fn testG23(){ //shouldn't be deterministic
        let (automataList, decl, _) = xml_parser::parse_xml(PATH);
        let optimized_components = optimize_components(automataList, &decl);
        let query = parse_queries::parse("determinism: G23").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };

        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        match leftSys {
            SystemRepresentation::Component(Component) =>{
                assert!(!Component.is_deterministic());
            }
            _ => {}
        }
}
}