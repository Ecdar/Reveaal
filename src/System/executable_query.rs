use crate::DataReader::json_writer::component_to_json;
use crate::ModelObjects::component::Component;
use crate::ModelObjects::system::UncachedSystem;
use crate::ModelObjects::system_declarations::SystemDeclarations;
use crate::System::save_component::combine_components;
use crate::System::{extra_actions, refine};

pub enum QueryResult {
    Refinement(bool),
    GetComponent(Component),
    Consistency(bool),
    Determinism(bool),
    Error(String),
}

pub trait ExecutableQuery {
    fn execute(self: Box<Self>) -> QueryResult;
}

pub struct RefinementExecutor<'a> {
    pub sys1: UncachedSystem<'a>,
    pub sys2: UncachedSystem<'a>,
    pub decls: SystemDeclarations,
}

impl<'a> ExecutableQuery for RefinementExecutor<'a> {
    fn execute(self: Box<Self>) -> QueryResult {
        let mut extra_components = vec![];
        let (sys1, sys2, decl) = extra_actions::add_extra_inputs_outputs(
            self.sys1,
            self.sys2,
            &self.decls,
            &mut extra_components,
        );

        match refine::check_refinement(sys1, sys2, &decl) {
            Ok(res) => {
                println!("Refinement result: {:?}", res);
                QueryResult::Refinement(res)
            }
            Err(err_msg) => QueryResult::Error(err_msg),
        }
    }
}

pub struct GetComponentExecutor<'a> {
    pub system: UncachedSystem<'a>,
    pub comp_name: String,
    pub decls: SystemDeclarations,
}

impl<'a> ExecutableQuery for GetComponentExecutor<'a> {
    fn execute(self: Box<Self>) -> QueryResult {
        let mut comp = combine_components(&self.system, &self.decls);
        comp.name = self.comp_name;

        component_to_json(&comp);

        QueryResult::GetComponent(comp)
    }
}

pub struct ConsistencyExecutor<'a> {
    pub system: UncachedSystem<'a>,
}

impl<'a> ExecutableQuery for ConsistencyExecutor<'a> {
    fn execute(self: Box<Self>) -> QueryResult {
        QueryResult::Consistency(self.system.precheck_sys_rep())
    }
}

pub struct DeterminismExecutor<'a> {
    pub system: UncachedSystem<'a>,
}

impl<'a> ExecutableQuery for DeterminismExecutor<'a> {
    fn execute(self: Box<Self>) -> QueryResult {
        let is_deterministic = self.system.all_components_are_deterministic();

        QueryResult::Determinism(is_deterministic)
    }
}
