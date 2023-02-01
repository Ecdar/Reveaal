use std::{collections::HashSet, error::Error, fmt};

use crate::{
    component::{Component, State},
    ModelObjects::statepair::StatePair,
    TransitionSystems::{CompositionType, TransitionSystem, TransitionSystemPtr},
};

use super::{reachability::Path, specifics::SpecificState};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Composition {
    Quotient,
    Refinement,
    Composition,
    Conjunction,
    Simple,
}
impl Composition {
    pub fn op(&self) -> String {
        match self {
            Self::Quotient => r"\\",
            Self::Refinement => "<=",
            Self::Composition => "||",
            Self::Conjunction => "&&",
            Self::Simple => panic!("Simple composition type should not be displayed"),
        }
        .to_string()
    }
}

impl fmt::Display for Composition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Quotient => write!(f, "Quotient"),
            Self::Refinement => write!(f, "Refinement"),
            Self::Composition => write!(f, "Composition"),
            Self::Conjunction => write!(f, "Conjunction"),
            Self::Simple => write!(f, "Component"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct System {
    pub name: String,
    pub comp: Composition,
}
impl System {
    pub fn refinement(sys1: &dyn TransitionSystem, sys2: &dyn TransitionSystem) -> Self {
        Self {
            name: format!("{} <= {}", sys1.to_string(), sys2.to_string()),
            comp: Composition::Refinement,
        }
    }
    pub fn from(sys: &dyn TransitionSystem) -> Self {
        Self {
            name: sys.to_string(),
            comp: sys.get_composition_type().into(),
        }
    }
    pub fn from2(
        sys1: &dyn TransitionSystem,
        sys2: &dyn TransitionSystem,
        comp: Composition,
    ) -> Self {
        Self {
            name: format!("{} {} {}", sys1.to_string(), comp.op(), sys2.to_string()),
            comp: Composition::Composition,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActionSet {
    pub system: String,
    pub actions: HashSet<String>,
    pub is_input: bool,
}

impl fmt::Display for ActionSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let singular = false; //self.actions.len() == 1;
        let action_type = if self.is_input { "Input" } else { "Output" };

        if singular {
            write!(
                f,
                "{} {:?} in system '{}'",
                action_type, self.actions, self.system
            )
        } else {
            write!(
                f,
                "{}s {:?} in system '{}'",
                action_type, self.actions, self.system
            )
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Action {
    pub name: String,
    pub is_input: bool,
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_input {
            write!(f, "Input \"{}\"", self.name)
        } else {
            write!(f, "Output \"{}\"", self.name)
        }
    }
}

impl Action {
    pub fn new(name: String, is_input: bool) -> Self {
        Self { name, is_input }
    }
}

#[derive(Clone, Debug)]
pub enum QueryResult {
    RecipeFailure(SystemRecipeFailure),
    Reachability(PathResult),
    Refinement(RefinementResult),
    Consistency(ConsistencyResult),
    Determinism(DeterminismResult),
    GetComponent(Component),
    CustomError(String),
}

pub type PathResult = Result<Path, PathFailure>;

//TODO: add refinement Ok result
#[allow(clippy::result_large_err)]
pub type RefinementResult = Result<(), RefinementFailure>;

pub type ConsistencyResult = Result<(), ConsistencyFailure>;

pub type DeterminismResult = Result<(), DeterminismFailure>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PathFailure {
    Unknown(String),
}

#[allow(clippy::result_large_err)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RefinementFailure {
    CutsDelaySolutions {
        system: System,
        action: Action,
        state: SpecificState,
    },
    CannotMatch {
        system: System,
        action: Action,
        state: SpecificState,
    },
    Precondition(RefinementPrecondition),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RefinementPrecondition {
    EmptyChild { child: String, system: System },
    EmptyInitialState { system: System },
    InconsistentChild(ConsistencyFailure, System),
    ActionMismatch(ActionFailure, System),
}

impl RefinementFailure {
    pub fn empty_child(
        sys1: &dyn TransitionSystem,
        sys2: &dyn TransitionSystem,
        left: bool,
    ) -> RefinementResult {
        Err(RefinementFailure::Precondition(
            RefinementPrecondition::EmptyChild {
                child: if left {
                    sys1.to_string()
                } else {
                    sys2.to_string()
                },
                system: System::refinement(sys1, sys2),
            },
        ))
    }

    pub fn empty_initial(
        sys1: &dyn TransitionSystem,
        sys2: &dyn TransitionSystem,
    ) -> RefinementResult {
        Err(RefinementFailure::Precondition(
            RefinementPrecondition::EmptyInitialState {
                system: System::refinement(sys1, sys2),
            },
        ))
    }

    pub fn cannot_match(
        sys1: &dyn TransitionSystem,
        sys2: &dyn TransitionSystem,
        action: impl Into<String>,
        state: &StatePair,
    ) -> RefinementResult {
        let action: String = action.into();
        let is_input = sys1.inputs_contain(&action) || sys2.inputs_contain(&action);
        Err(RefinementFailure::CannotMatch {
            system: System::refinement(sys1, sys2),
            action: Action::new(action, is_input),
            state: SpecificState::from_state_pair(state, sys1, sys2),
        })
    }

    pub fn cuts_delays(
        sys1: &dyn TransitionSystem,
        sys2: &dyn TransitionSystem,
        action: impl Into<String>,
        state: &StatePair,
    ) -> RefinementResult {
        let action: String = action.into();
        let is_input = sys1.inputs_contain(&action) || sys2.inputs_contain(&action);
        Err(RefinementFailure::CutsDelaySolutions {
            system: System::refinement(sys1, sys2),
            action: Action::new(action, is_input),
            state: SpecificState::from_state_pair(state, sys1, sys2),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ActionFailure {
    NotSubset(ActionSet, ActionSet),
    NotDisjoint(ActionSet, ActionSet),
}
#[allow(clippy::result_large_err)]
impl ActionFailure {
    pub fn not_disjoint<T>(
        (sys1, actions1): (&dyn TransitionSystem, HashSet<String>),
        (sys2, actions2): (&dyn TransitionSystem, HashSet<String>),
    ) -> Result<T, ActionFailure> {
        let is_input1 = sys1.get_input_actions() == actions1;
        let is_input2 = sys2.get_input_actions() == actions2;

        debug_assert!(is_input1 || sys1.get_output_actions() == actions1);
        debug_assert!(is_input2 || sys2.get_output_actions() == actions2);

        Err(ActionFailure::NotDisjoint(
            ActionSet {
                system: sys1.to_string(),
                actions: actions1,
                is_input: is_input1,
            },
            ActionSet {
                system: sys2.to_string(),
                actions: actions2,
                is_input: is_input2,
            },
        ))
    }

    pub fn not_disjoint_IO(
        name: impl Into<String>,
        inputs: HashSet<String>,
        outputs: HashSet<String>,
    ) -> Result<(), ActionFailure> {
        let system = name.into();
        Err(ActionFailure::NotDisjoint(
            ActionSet {
                system: system.clone(),
                actions: inputs,
                is_input: true,
            },
            ActionSet {
                system,
                actions: outputs,
                is_input: false,
            },
        ))
    }

    pub fn not_subset(
        (sys1, actions1): (&dyn TransitionSystem, HashSet<String>),
        (sys2, actions2): (&dyn TransitionSystem, HashSet<String>),
    ) -> Result<(), ActionFailure> {
        let is_input1 = sys1.get_input_actions() == actions1;
        let is_input2 = sys2.get_input_actions() == actions2;

        debug_assert!(is_input1 || sys1.get_output_actions() == actions1);
        debug_assert!(is_input2 || sys2.get_output_actions() == actions2);

        Err(ActionFailure::NotSubset(
            ActionSet {
                system: sys1.to_string(),
                actions: actions1,
                is_input: is_input1,
            },
            ActionSet {
                system: sys2.to_string(),
                actions: actions2,
                is_input: is_input2,
            },
        ))
    }

    pub fn to_precondition(
        self,
        sys1: &dyn TransitionSystem,
        sys2: &dyn TransitionSystem,
    ) -> RefinementPrecondition {
        RefinementPrecondition::ActionMismatch(self, System::refinement(sys1, sys2))
    }

    pub fn to_recipe_failure(self, sys: &dyn TransitionSystem) -> SystemRecipeFailure {
        SystemRecipeFailure::Action(self, System::from(sys))
    }

    pub fn to_simple_failure(self, sys: impl Into<String>) -> SystemRecipeFailure {
        SystemRecipeFailure::Action(
            self,
            System {
                name: sys.into(),
                comp: Composition::Simple,
            },
        )
    }

    pub fn to_rfq(self, T: &TransitionSystemPtr, S: &TransitionSystemPtr) -> SystemRecipeFailure {
        SystemRecipeFailure::Action(
            self,
            System::from2(T.as_ref(), S.as_ref(), Composition::Quotient),
        )
    }

    pub fn to_rfcomp(
        self,
        left: TransitionSystemPtr,
        right: TransitionSystemPtr,
    ) -> SystemRecipeFailure {
        SystemRecipeFailure::Action(
            self,
            System::from2(left.as_ref(), right.as_ref(), Composition::Composition),
        )
    }

    pub fn to_rfconj(
        self,
        left: TransitionSystemPtr,
        right: TransitionSystemPtr,
    ) -> SystemRecipeFailure {
        SystemRecipeFailure::Action(
            self,
            System::from2(left.as_ref(), right.as_ref(), Composition::Conjunction),
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SystemRecipeFailure {
    Action(ActionFailure, System),
    Inconsistent(ConsistencyFailure, System),
    ClockReduction(ClockReductionFailure),
    CustomError(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClockReductionFailure {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConsistencyFailure {
    NoInitialState {
        system: String,
    },
    NotDeterministic(DeterminismFailure),
    InconsistentLoc {
        system: String,
        state: SpecificState,
    },
    InconsistentFrom {
        system: String,
        state: SpecificState,
    },
}

impl ConsistencyFailure {
    pub fn no_initial_state(system: &dyn TransitionSystem) -> ConsistencyResult {
        Err(ConsistencyFailure::NoInitialState {
            system: system.to_string(),
        })
    }
    pub fn inconsistent(system: &dyn TransitionSystem, state: &State) -> ConsistencyResult {
        Err(ConsistencyFailure::InconsistentLoc {
            system: system.to_string(),
            state: SpecificState::from_state(state, system),
        })
    }
    pub fn inconsistent_from(
        system: &dyn TransitionSystem,
        //action: impl Into<String>,
        state: &State,
    ) -> ConsistencyResult {
        //let action: String = action.into();
        //let is_input = system.inputs_contain(&action);
        Err(ConsistencyFailure::InconsistentFrom {
            system: system.to_string(),
            //action: Action::new(action, is_input),
            state: SpecificState::from_state(state, system),
        })
    }

    pub fn to_precondition(
        self,
        sys1: &dyn TransitionSystem,
        sys2: &dyn TransitionSystem,
    ) -> RefinementPrecondition {
        RefinementPrecondition::InconsistentChild(self, System::refinement(sys1, sys2))
    }

    pub fn to_recipe_failure(self, sys: &dyn TransitionSystem) -> SystemRecipeFailure {
        SystemRecipeFailure::Inconsistent(self, System::from(sys))
    }

    pub fn to_recipe_failure2(self, sys: System) -> SystemRecipeFailure {
        SystemRecipeFailure::Inconsistent(self, sys)
    }
    pub fn to_rfq(self, T: &TransitionSystemPtr, S: &TransitionSystemPtr) -> SystemRecipeFailure {
        SystemRecipeFailure::Inconsistent(
            self,
            System::from2(T.as_ref(), S.as_ref(), Composition::Quotient),
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeterminismFailure {
    pub system: String,
    pub action: Action,
    pub state: SpecificState,
}

impl DeterminismFailure {
    pub fn from(
        system: &dyn TransitionSystem,
        action: impl Into<String>,
        state: &State,
    ) -> DeterminismResult {
        let action: String = action.into();
        let is_input = system.inputs_contain(&action);
        Err(DeterminismFailure {
            system: system.to_string(),
            action: Action::new(action, is_input),
            state: SpecificState::from_state(state, system),
        })
    }
}

// ---------------------------- //
// --- Format Display Impl  --- //
// ---------------------------- //

impl std::fmt::Display for DeterminismFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "The system '{}' is not deterministic in state {} for {}",
            self.system, self.state, self.action
        )
    }
}

impl std::fmt::Display for RefinementFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RefinementFailure::CutsDelaySolutions {
                system,
                action,
                state,
            } => write!(
                f,
                "The refinement '{}' fails because delay solutions are cut in state {} for {}",
                system.name, state, action
            ),
            RefinementFailure::CannotMatch {
                system,
                action,
                state,
            } => write!(
                f,
                "The refinement '{}' fails in state {} because {} cannot be matched",
                system.name, state, action
            ),
            RefinementFailure::Precondition(precond) => precond.fmt(f),
        }
    }
}

impl std::fmt::Display for ConsistencyFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConsistencyFailure::NoInitialState { system } => write!(
                f,
                "The system '{}' is inconsistent because it has no initial state",
                system
            ),
            ConsistencyFailure::NotDeterministic(determ) => determ.fmt(f),
            ConsistencyFailure::InconsistentLoc { system, state }
            | ConsistencyFailure::InconsistentFrom { system, state } => write!(
                f,
                "The system '{}' is inconsistent because there are no saving outputs from state {}",
                system, state
            ),
        }
    }
}

impl std::fmt::Display for SystemRecipeFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SystemRecipeFailure::Action(action, system) => write!(
                f,
                "{} in {} is invalid: {}",
                system.comp, system.name, action
            ),
            SystemRecipeFailure::Inconsistent(cf, system) => {
                write!(f, "{} in {} is invalid: {}", system.comp, system.name, cf)
            }
            SystemRecipeFailure::ClockReduction(cr) => write!(f, "Clock reduction failed: {}", cr),
            SystemRecipeFailure::CustomError(e) => write!(f, "Unknown error: {}", e),
        }
    }
}

impl std::fmt::Display for ActionFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActionFailure::NotSubset(a, b) => {
                write!(f, "{} are not a subset of {}", a, b)
            }
            ActionFailure::NotDisjoint(a, b) => {
                write!(f, "{} are not disjoint from {}", a, b)
            }
        }
    }
}

impl std::fmt::Display for ClockReductionFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unknown error occured during clock reduction")
    }
}

impl std::fmt::Display for RefinementPrecondition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RefinementPrecondition::EmptyChild { child, system } => write!(
                f,
                "The refinement '{}' fails because '{}' is empty",
                system.name, child
            ),
            RefinementPrecondition::EmptyInitialState { system } => write!(
                f,
                "The refinement '{}' fails because it has no initial state",
                system.name
            ),
            RefinementPrecondition::InconsistentChild(cf, system) => write!(
                f,
                "The refinement '{}' fails because of inconsistent system: {}",
                system.name, cf
            ),
            RefinementPrecondition::ActionMismatch(action, system) => {
                write!(
                    f,
                    "The refinement '{}' fails based on actions: {}",
                    system.name, action
                )
            }
        }
    }
}

// ---------------------------- //
// - Ugly generics begin here - //
// --- You have been warned --- //
// ---------------------------- //

impl Error for dyn Query {}
impl Error for SystemRecipeFailure {}
impl Error for ClockReductionFailure {}

trait Query: fmt::Display + fmt::Debug {}

impl Query for RefinementFailure {}
impl Query for ConsistencyFailure {}
impl Query for DeterminismFailure {}

impl From<RefinementPrecondition> for RefinementFailure {
    fn from(failure: RefinementPrecondition) -> Self {
        RefinementFailure::Precondition(failure)
    }
}

impl From<CompositionType> for Composition {
    fn from(comp: CompositionType) -> Self {
        match comp {
            CompositionType::Quotient => Composition::Quotient,
            CompositionType::Composition => Composition::Composition,
            CompositionType::Conjunction => Composition::Conjunction,
            CompositionType::Simple => Composition::Simple,
        }
    }
}

impl From<DeterminismFailure> for ConsistencyFailure {
    fn from(failure: DeterminismFailure) -> Self {
        ConsistencyFailure::NotDeterministic(failure)
    }
}

impl From<&str> for SystemRecipeFailure {
    fn from(system: &str) -> Self {
        SystemRecipeFailure::CustomError(system.to_string())
    }
}

impl From<String> for SystemRecipeFailure {
    fn from(system: String) -> Self {
        SystemRecipeFailure::CustomError(system)
    }
}

impl From<SystemRecipeFailure> for QueryResult {
    fn from(res: SystemRecipeFailure) -> Self {
        QueryResult::RecipeFailure(res)
    }
}

impl From<PathResult> for QueryResult {
    fn from(res: PathResult) -> Self {
        QueryResult::Reachability(res)
    }
}

impl From<ConsistencyResult> for QueryResult {
    fn from(res: ConsistencyResult) -> Self {
        QueryResult::Consistency(res)
    }
}

impl From<DeterminismResult> for QueryResult {
    fn from(res: DeterminismResult) -> Self {
        QueryResult::Determinism(res)
    }
}

impl From<RefinementResult> for QueryResult {
    fn from(res: RefinementResult) -> Self {
        QueryResult::Refinement(res)
    }
}
