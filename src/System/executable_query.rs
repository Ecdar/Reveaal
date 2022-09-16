use edbm::util::constraints::ClockIndex;
use log::info;

use crate::DataReader::component_loader::ComponentLoader;
use crate::ModelObjects::component::Component;
use crate::System::refine;
use crate::System::save_component::combine_components;
use crate::TransitionSystems::TransitionSystemPtr;

use super::extract_system_rep::SystemRecipe;
use super::save_component::PruningStrategy;

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
    fn execute(self: Box<Self>) -> QueryResult;
}

pub struct RefinementExecutor {
    pub sys1: TransitionSystemPtr,
    pub sys2: TransitionSystemPtr,
}

impl ExecutableQuery for RefinementExecutor {
    fn execute(self: Box<Self>) -> QueryResult {
        let (sys1, sys2) = (self.sys1, self.sys2);

        match refine::check_refinement(sys1, sys2) {
            Ok(res) => {
                info!("Refinement result: {:?}", res);
                QueryResult::Refinement(res)
            }
            Err(err_msg) => QueryResult::Error(err_msg),
        }
    }
}

pub struct GetComponentExecutor<'a> {
    pub system: TransitionSystemPtr,
    pub comp_name: String,
    pub component_loader: &'a mut dyn ComponentLoader,
}

impl<'a> ExecutableQuery for GetComponentExecutor<'a> {
    fn execute(self: Box<Self>) -> QueryResult {
        let mut comp = combine_components(&self.system, PruningStrategy::Reachable);
        comp.name = self.comp_name;

        comp.create_edge_io_split();

        self.component_loader.save_component(comp.clone());

        QueryResult::GetComponent(comp)
    }
}

pub struct ConsistencyExecutor {
    pub recipe: Box<SystemRecipe>,
    pub dim: ClockIndex,
}

impl<'a> ExecutableQuery for ConsistencyExecutor {
    fn execute(self: Box<Self>) -> QueryResult {
        let res = match self.recipe.compile(self.dim) {
            Ok(system) => system.precheck_sys_rep(),
            Err(_) => false,
        };

        QueryResult::Consistency(res)
    }
}

pub struct DeterminismExecutor {
    pub system: TransitionSystemPtr,
}

impl<'a> ExecutableQuery for DeterminismExecutor {
    fn execute(self: Box<Self>) -> QueryResult {
        let is_deterministic = self.system.is_deterministic();

        QueryResult::Determinism(is_deterministic)
    }
}
