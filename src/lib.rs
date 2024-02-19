pub mod cli;
pub mod data_reader;
pub mod edge_eval;
pub mod logging;
pub mod model_objects;
pub mod protobuf_server;
pub mod simulation;
pub mod system;
pub mod transition_systems;

pub use crate::data_reader::component_loader::{
    ComponentLoader, JsonProjectLoader, ProjectLoader, XmlProjectLoader,
};
pub use crate::data_reader::{parse_queries, xml_parser};
use crate::protobuf_server::services::query_request::Settings;
pub use crate::system::extract_system_rep;
pub use protobuf_server::start_grpc_server_with_tokio;

#[cfg(not(test))]
/// The default settings
pub const DEFAULT_SETTINGS: Settings = Settings {
    disable_clock_reduction: true,
};
#[cfg(test)]
/// The default settings
pub const DEFAULT_SETTINGS: Settings = Settings {
    disable_clock_reduction: false,
};

#[macro_use]
extern crate pest_derive;
extern crate colored;
extern crate core;
extern crate serde;
extern crate serde_xml_rs;
extern crate xml;
#[macro_use]
extern crate lazy_static;

#[cfg(test)]
mod test_helpers {
    use crate::extract_system_rep::{create_executable_query, ExecutableQueryError};
    use crate::model_objects::expressions::QueryExpression;
    use crate::model_objects::Query;
    use crate::system::query_failures::QueryResult;
    use crate::{parse_queries, JsonProjectLoader, XmlProjectLoader};

    pub fn json_run_query(path: &str, query: &str) -> Result<QueryResult, ExecutableQueryError> {
        let mut project_loader =
            JsonProjectLoader::new_loader(String::from(path), crate::DEFAULT_SETTINGS);
        let query = parse_queries::parse_to_expression_tree(query)
            .unwrap()
            .remove(0);
        let q = Query {
            query: Option::from(query),
            comment: "".to_string(),
        };
        // FIXME: After implementing clock reduction on component level, a few tests are failing due to inconsistencies with initial state and global clock. Turn disabled_clock_reduction boolean to true to ignore inconsistencies
        if let Some(query_type) = q.get_query() {
            match query_type {
                QueryExpression::Reachability { .. } => {
                    project_loader.get_settings_mut().disable_clock_reduction = true;
                }
                QueryExpression::Refinement(_, _)
                | QueryExpression::Consistency(_)
                | QueryExpression::Implementation(_)
                | QueryExpression::Determinism(_)
                | QueryExpression::Specification(_)
                | QueryExpression::Syntax(_)
                | QueryExpression::BisimMinim(_)
                | QueryExpression::GetComponent(_)
                | QueryExpression::Prune(_) => {
                    project_loader.get_settings_mut().disable_clock_reduction = false;
                }
            }
        }

        let mut comp_loader = project_loader.to_comp_loader();
        let query = create_executable_query(&q, &mut *comp_loader)?;

        Ok(query.execute())
    }
    pub fn xml_run_query(path: &str, query: &str) -> QueryResult {
        let project_path = String::from(path);
        let project_loader = XmlProjectLoader::new_loader(project_path, crate::DEFAULT_SETTINGS);
        let query = parse_queries::parse_to_expression_tree(query)
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
}
