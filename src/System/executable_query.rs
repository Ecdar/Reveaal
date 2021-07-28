use crate::DataReader::json_writer::component_to_json;
use crate::ModelObjects::component::Component;
use crate::ModelObjects::system_declarations::SystemDeclarations;
use crate::System::save_component::combine_components;
use crate::System::{extra_actions, refine};
use crate::TransitionSystems::TransitionSystemPtr;

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

pub struct RefinementExecutor {
    pub sys1: TransitionSystemPtr,
    pub sys2: TransitionSystemPtr,
    pub decls: SystemDeclarations,
}

impl ExecutableQuery for RefinementExecutor {
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

pub struct GetComponentExecutor {
    pub system: TransitionSystemPtr,
    pub comp_name: String,
    pub decls: SystemDeclarations,
}

impl<'a> ExecutableQuery for GetComponentExecutor {
    fn execute(self: Box<Self>) -> QueryResult {
        let mut comp = combine_components(&self.system, &self.decls);
        comp.name = self.comp_name;

        component_to_json(&comp);

        QueryResult::GetComponent(comp)
    }
}

pub struct ConsistencyExecutor {
    pub system: TransitionSystemPtr,
}

impl<'a> ExecutableQuery for ConsistencyExecutor {
    fn execute(self: Box<Self>) -> QueryResult {
        let dim = self.system.get_num_clocks() + 1;
        QueryResult::Consistency(self.system.precheck_sys_rep(dim))
    }
}

pub struct DeterminismExecutor {
    pub system: TransitionSystemPtr,
}

impl<'a> ExecutableQuery for DeterminismExecutor {
    fn execute(self: Box<Self>) -> QueryResult {
        let dim = self.system.get_num_clocks() + 1;
        let is_deterministic = self.system.is_deterministic(dim);

        QueryResult::Determinism(is_deterministic)
    }
}
