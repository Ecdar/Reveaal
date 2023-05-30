use crate::extract_system_rep::ExecutableQueryError;
use crate::logging::setup_logger;
use crate::DataReader::component_loader::{JsonProjectLoader, XmlProjectLoader};
use crate::DataReader::parse_queries;
use crate::ModelObjects::Query;
use crate::System::extract_system_rep::create_executable_query;
use crate::System::query_failures::QueryResult;
use crate::TransitionSystems::transition_system::component_loader_to_transition_system;
use crate::TransitionSystems::TransitionSystemPtr;

fn try_setup_logging() {
    #[cfg(feature = "logging")]
    let _ = setup_logger();
}

pub fn xml_refinement_check(PATH: &str, QUERY: &str) -> bool {
    try_setup_logging();
    match xml_run_query(PATH, QUERY) {
        QueryResult::Refinement(Ok(())) => true,
        QueryResult::Refinement(Err(_)) => false,
        QueryResult::CustomError(err) => panic!("{}", err),
        _ => panic!("Not a refinement check"),
    }
}

pub fn json_refinement_check(PATH: &str, QUERY: &str) -> bool {
    try_setup_logging();

    match json_run_query(PATH, QUERY).unwrap() {
        QueryResult::Refinement(Ok(())) => true,
        QueryResult::Refinement(Err(_)) => false,
        QueryResult::CustomError(err) => panic!("{}", err),
        _ => panic!("Not a refinement check"),
    }
}

pub fn xml_run_query(PATH: &str, QUERY: &str) -> QueryResult {
    let project_path = String::from(PATH);
    let project_loader = XmlProjectLoader::new_loader(project_path, crate::tests::TEST_SETTINGS);
    let query = parse_queries::parse_to_expression_tree(QUERY)
        .unwrap()
        .remove(0);
    let q = Query {
        query: Option::from(query),
        comment: "".to_string(),
    };

    let mut comp_loader = project_loader.to_comp_loader();
    let query = create_executable_query(&q, &mut *comp_loader).unwrap();

    query.execute()
}

pub fn json_run_query(PATH: &str, QUERY: &str) -> Result<QueryResult, ExecutableQueryError> {
    let project_loader =
        JsonProjectLoader::new_loader(String::from(PATH), crate::tests::TEST_SETTINGS);
    let query = parse_queries::parse_to_expression_tree(QUERY)
        .unwrap()
        .remove(0);
    let q = Query {
        query: Option::from(query),
        comment: "".to_string(),
    };

    let mut comp_loader = project_loader.to_comp_loader();
    let query = create_executable_query(&q, &mut *comp_loader)?;

    Ok(query.execute())
}

pub fn json_get_system(PATH: &str, COMP: &str) -> TransitionSystemPtr {
    let project_loader =
        JsonProjectLoader::new_loader(String::from(PATH), crate::tests::TEST_SETTINGS);
    let mut loader = project_loader.to_comp_loader();
    component_loader_to_transition_system(&mut *loader, COMP)
}
