use crate::DataReader::component_loader::{JsonProjectLoader, XmlProjectLoader};
use crate::DataReader::parse_queries;
use crate::ModelObjects::queries::Query;
use crate::System::executable_query::QueryResult;
use crate::System::extract_system_rep::create_executable_query;

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
    let mut project_loader = XmlProjectLoader::new(project_path);
    let query = parse_queries::parse(QUERY).remove(0);
    let q = Query {
        query: Option::from(query),
        comment: "".to_string(),
    };

    let query = create_executable_query(&q, &mut project_loader);

    query.execute().unwrap()
}

pub fn json_run_query(PATH: &str, QUERY: &str) -> QueryResult {
    let mut project_loader = JsonProjectLoader::new(String::from(PATH));
    let query = parse_queries::parse(QUERY).remove(0);
    let q = Query {
        query: Option::from(query),
        comment: "".to_string(),
    };

    let query = create_executable_query(&q, &mut project_loader);

    query.execute().unwrap()
}
