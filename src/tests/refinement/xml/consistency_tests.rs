#[cfg(test)]
mod consistency_tests {
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
        let query = parse_queries::parse("consistency: G1").unwrap();
        let q = Query {
            query: Option::from(query),
            comment: "".to_string()
        };
        let res = create_system_rep_from_query(&q, &optimized_components);
        let leftSys = res.0;
        match leftSys {
            SystemRepresentation::Component(Component) =>{
                assert!(Component.check_consistency(true));
            }
            _ => {}
        }
    }
}