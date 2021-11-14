use crate::DataReader::component_loader::ProjectLoader;
use crate::DataReader::json_writer::component_to_json;
use crate::ModelObjects::component::Component;
use crate::ModelObjects::system_declarations::SystemDeclarations;
use crate::System::save_component::combine_components;
use crate::System::{extra_actions, refine};
use crate::TransitionSystems::TransitionSystemPtr;
use std::error::Error;

pub enum QueryResult {
    Refinement(bool),
    GetComponent(Component),
    Consistency(bool),
    Determinism(bool),
    Error(String),
}

impl QueryResult {
    pub fn print_result(&self, query_str: &str) {
        match self {
            QueryResult::Refinement(true) => satisfied(query_str),
            QueryResult::Refinement(false) => not_satisfied(query_str),

            QueryResult::Consistency(true) => satisfied(query_str),
            QueryResult::Consistency(false) => not_satisfied(query_str),

            QueryResult::Determinism(true) => satisfied(query_str),
            QueryResult::Determinism(false) => not_satisfied(query_str),

            QueryResult::GetComponent(_) => {
                println!("{} -- Component succesfully created", query_str)
            }

            QueryResult::Error(_) => println!("{} -- Failed", query_str),
        };
    }
}

fn satisfied(query_str: &str) {
    println!("{} -- Property is satisfied", query_str);
}

fn not_satisfied(query_str: &str) {
    println!("{} -- Property is NOT satisfied", query_str);
}

pub trait ExecutableQuery {
    fn execute(self: Box<Self>) -> Result<QueryResult, Box<dyn Error>>;
}

pub struct RefinementExecutor {
    pub sys1: TransitionSystemPtr,
    pub sys2: TransitionSystemPtr,
    pub decls: SystemDeclarations,
}

impl ExecutableQuery for RefinementExecutor {
    fn execute(self: Box<Self>) -> Result<QueryResult, Box<dyn Error>> {
        let (sys1, sys2) = extra_actions::add_extra_inputs_outputs(self.sys1, self.sys2);

        match refine::check_refinement(sys1, sys2)? {
            Ok(res) => {
                println!("Refinement result: {:?}", res);
                Ok(QueryResult::Refinement(res))
            }
            Err(err_msg) => Ok(QueryResult::Error(err_msg)),
        }
    }
}

pub struct GetComponentExecutor<'a> {
    pub system: TransitionSystemPtr,
    pub comp_name: String,
    pub project_loader: &'a mut Box<dyn ProjectLoader>,
}

impl<'a> ExecutableQuery for GetComponentExecutor<'a> {
    fn execute(self: Box<Self>) -> Result<QueryResult, Box<dyn Error>> {
        let mut comp = combine_components(&self.system);
        comp.name = self.comp_name;

        let project_path = self.project_loader.get_project_path();
        component_to_json(project_path, &comp);
        self.project_loader.unload_component(&comp.name);

        Ok(QueryResult::GetComponent(comp))
    }
}

pub struct ConsistencyExecutor {
    pub system: TransitionSystemPtr,
}

impl<'a> ExecutableQuery for ConsistencyExecutor {
    fn execute(self: Box<Self>) -> Result<QueryResult, Box<dyn Error>> {
        let dim = self.system.get_num_clocks() + 1;
        Ok(QueryResult::Consistency(self.system.precheck_sys_rep(dim)))
    }
}

pub struct DeterminismExecutor {
    pub system: TransitionSystemPtr,
}

impl<'a> ExecutableQuery for DeterminismExecutor {
    fn execute(self: Box<Self>) -> Result<QueryResult, Box<dyn Error>> {
        let dim = self.system.get_num_clocks() + 1;
        let is_deterministic = self.system.is_deterministic(dim);

        Ok(QueryResult::Determinism(is_deterministic))
    }
}
