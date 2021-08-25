use crate::DataReader::component_loader::{
    ComponentLoader, JsonComponentLoader, XmlComponentLoader,
};
use crate::DataReader::json_reader;
use crate::DataReader::{parse_queries, xml_parser};
use crate::ModelObjects::component::Component;
use crate::ModelObjects::queries::Query;
use crate::ModelObjects::system_declarations::SystemDeclarations;
use crate::System::executable_query::QueryResult;
use crate::System::extract_system_rep::create_executable_query;
use crate::System::input_enabler;

pub fn xml_refinement_check(PATH: &str, QUERY: &str) -> bool {
    match xml_run_query(PATH, QUERY) {
        QueryResult::Refinement(result) => result,
        QueryResult::Error(err) => panic!(err),
        _ => panic!("Not a refinement check"),
    }
}

pub fn json_refinement_check(PATH: &str, QUERY: &str) -> bool {
    match json_run_query(PATH, QUERY) {
        QueryResult::Refinement(result) => result,
        QueryResult::Error(err) => panic!(err),
        _ => panic!("Not a refinement check"),
    }
}

pub fn xml_run_query(PATH: &str, QUERY: &str) -> QueryResult {
    let project_path = String::from(PATH);
    let mut project_loader = XmlComponentLoader::new(project_path);
    let query = parse_queries::parse(QUERY).remove(0);
    let q = Query {
        query: Option::from(query),
        comment: "".to_string(),
    };

    let query = create_executable_query(&q, &mut project_loader);

    query.execute()
}

pub fn json_run_query(PATH: &str, QUERY: &str) -> QueryResult {
    let project_path = String::from(PATH);
    let mut project_loader = JsonComponentLoader::new(project_path);
    let query = parse_queries::parse(QUERY).remove(0);
    let q = Query {
        query: Option::from(query),
        comment: "".to_string(),
    };

    let query = create_executable_query(&q, &mut project_loader);

    query.execute()
}
