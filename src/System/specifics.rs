use std::{collections::HashMap, fmt};

use edbm::util::constraints::{ClockIndex, Conjunction, Constraint, Disjunction};

use crate::{
    component::State,
    ModelObjects::statepair::StatePair,
    Simulation::decision::Decision,
    TransitionSystems::{
        transition_system::ComponentInfoTree, CompositionType, LocationID, TransitionID,
        TransitionSystem,
    },
};

use super::reachability::Path;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SpecificDecision {
    pub source_state: SpecificState,
    pub action: String,
    pub edges: Vec<SpecificEdge>,
    pub destination_state: SpecificState,
}

fn transition_id_to_specific_edges(
    id: TransitionID,
    system: &dyn TransitionSystem,
    edges: &mut Vec<SpecificEdge>,
) {
    match id {
        TransitionID::Conjunction(left, right) => {
            assert_eq!(system.get_composition_type(), CompositionType::Conjunction);
            let (l, r) = system.get_children();
            transition_id_to_specific_edges(*left, &**l, edges);
            transition_id_to_specific_edges(*right, &**r, edges);
        }
        TransitionID::Composition(left, right) => {
            assert_eq!(system.get_composition_type(), CompositionType::Composition);
            let (l, r) = system.get_children();
            transition_id_to_specific_edges(*left, &**l, edges);
            transition_id_to_specific_edges(*right, &**r, edges);
        }
        TransitionID::Quotient(lefts, rights) => {
            assert_eq!(system.get_composition_type(), CompositionType::Quotient);
            let (l, r) = system.get_children();
            for left in lefts {
                transition_id_to_specific_edges(left, &**l, edges);
            }
            for right in rights {
                transition_id_to_specific_edges(right, &**r, edges);
            }
        }
        TransitionID::Simple(edge_id) => {
            assert_eq!(system.get_composition_type(), CompositionType::Simple);
            if let ComponentInfoTree::Info(info) = system.comp_infos() {
                let edge = SpecificEdge::new(info.name.clone(), edge_id, info.id);
                edges.push(edge);
            } else {
                unreachable!("Simple transition system should have ComponentInfoTree::Info")
            }
        }
        TransitionID::None => {}
    }
}

impl SpecificDecision {
    pub fn from_decision(decision: &Decision, system: &dyn TransitionSystem) -> Self {
        let mut edges = vec![];
        if let Some(t) = &decision.transition {
            transition_id_to_specific_edges(t.id.clone(), system, &mut edges);
        }

        Self {
            source_state: SpecificState::from_state(&decision.state, system),
            action: decision.action.clone(),
            edges,
            destination_state: SpecificState::from_state(&decision.next_state, system),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SpecificPath {
    pub path: Vec<SpecificDecision>,
}

impl SpecificPath {
    pub fn from_path(path: &Path, system: &dyn TransitionSystem) -> Self {
        Self {
            path: path
                .path
                .iter()
                .map(|d| SpecificDecision::from_decision(d, system))
                .collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SpecificComp {
    pub name: String,
    pub id: u32,
}

impl SpecificComp {
    pub fn new(name: String, id: u32) -> Self {
        Self { name, id }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SpecificEdge {
    pub comp: SpecificComp,
    pub edge_id: String,
}

impl SpecificEdge {
    pub fn new(
        component_name: impl Into<String>,
        edge_id: impl Into<String>,
        component_id: u32,
    ) -> Self {
        Self {
            comp: SpecificComp::new(component_name.into(), component_id),
            edge_id: edge_id.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SpecificDisjunction {
    pub conjunctions: Vec<SpecificConjunction>,
}

impl SpecificDisjunction {
    pub fn from(disj: Disjunction, sys: &HashMap<ClockIndex, SpecificClock>) -> Self {
        Self {
            conjunctions: disj
                .conjunctions
                .into_iter()
                .map(|c| SpecificConjunction::from(c, sys))
                .collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SpecificConjunction {
    pub constraints: Vec<SpecificConstraint>,
}

impl SpecificConjunction {
    pub fn from(conj: Conjunction, sys: &HashMap<ClockIndex, SpecificClock>) -> Self {
        Self {
            constraints: conj
                .constraints
                .into_iter()
                .map(|c| SpecificConstraint::from(c, sys))
                .collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum SpecificClockVar {
    Zero,
    ComponentClock(SpecificClock),
    SystemClock(ClockIndex),
}

/// i-j <?= c
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SpecificConstraint {
    pub i: SpecificClockVar,
    pub j: SpecificClockVar,
    pub strict: bool,
    pub c: i32,
}

impl SpecificConstraint {
    pub fn from(constraint: Constraint, sys: &HashMap<ClockIndex, SpecificClock>) -> Self {
        fn map_clock(
            clock: ClockIndex,
            sys: &HashMap<ClockIndex, SpecificClock>,
        ) -> SpecificClockVar {
            match clock {
                0 => SpecificClockVar::Zero,
                _ => match sys.get(&clock) {
                    Some(c) => SpecificClockVar::ComponentClock(c.clone()),
                    None => SpecificClockVar::SystemClock(clock),
                },
            }
        }

        Self {
            i: map_clock(constraint.i, sys),
            j: map_clock(constraint.j, sys),
            strict: constraint.ineq().is_strict(),
            c: constraint.ineq().bound(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SpecificState {
    pub locations: SpecificLocation,
    pub constraints: SpecificDisjunction,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum SpecificLocation {
    ComponentLocation {
        comp: SpecificComp,
        location_id: String,
    },
    BranchLocation(Box<SpecificLocation>, Box<SpecificLocation>),
    SpecialLocation(SpecialLocation),
}

impl SpecificLocation {
    pub fn new(
        component_name: impl Into<String>,
        location_id: impl Into<String>,
        component_id: u32,
    ) -> Self {
        Self::ComponentLocation {
            comp: SpecificComp::new(component_name.into(), component_id),
            location_id: location_id.into(),
        }
    }

    pub fn split(self) -> (Self, Self) {
        match self {
            SpecificLocation::BranchLocation(left, right) => (*left, *right),
            _ => unreachable!("Cannot split non-branch location"),
        }
    }
}

impl fmt::Display for SpecificLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SpecificLocation::ComponentLocation { comp, location_id } => {
                write!(f, "{}.{}", comp.name, location_id)
            }
            SpecificLocation::BranchLocation(left, right) => write!(f, "({}, {})", left, right),
            SpecificLocation::SpecialLocation(spec) => write!(f, "{}", spec),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum SpecialLocation {
    Universal,
    Error,
}

impl fmt::Display for SpecialLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SpecialLocation::Universal => write!(f, "[Universal]"),
            SpecialLocation::Error => write!(f, "[Error]"),
        }
    }
}

impl SpecificState {
    pub fn from_state(state: &State, sys: &dyn TransitionSystem) -> Self {
        let locations = state_specific_location(state, sys);
        let clock_map = specific_clock_comp_map(sys);

        let constraints = state.zone_ref().minimal_constraints();
        let constraints = SpecificDisjunction::from(constraints, &clock_map);
        Self {
            locations,
            constraints,
        }
    }
    pub fn from_state_pair(
        state: &StatePair,
        sys1: &dyn TransitionSystem,
        sys2: &dyn TransitionSystem,
    ) -> Self {
        let locations = state_pair_specific_location(state, sys1, sys2);

        let clock_map = specific_clock_comp_map2(sys1, sys2);

        let constraints = state.ref_zone().minimal_constraints();
        let constraints = SpecificDisjunction::from(constraints, &clock_map);
        Self {
            locations,
            constraints,
        }
    }
}

impl fmt::Display for SpecificState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let locs = &self.locations;

        write!(f, "({})", locs)
        // TODO: maybe show constraints
        // write!(f, "({} | {})", locs, self.constraints)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SpecificClock {
    pub name: String,
    pub comp: SpecificComp,
}

impl SpecificClock {
    pub fn new(name: String, comp: SpecificComp) -> Self {
        Self { name, comp }
    }
}

pub fn specific_clock_comp_map(sys: &dyn TransitionSystem) -> HashMap<ClockIndex, SpecificClock> {
    sys.comp_infos()
        .iter()
        .flat_map(|comp| {
            comp.declarations
                .clocks
                .iter()
                .map(move |(clock, &clock_id)| {
                    (
                        clock_id,
                        SpecificClock::new(
                            clock.clone(),
                            SpecificComp::new(comp.name.clone(), comp.id),
                        ),
                    )
                })
        })
        .collect()
}

pub fn specific_clock_comp_map2(
    sys1: &dyn TransitionSystem,
    sys2: &dyn TransitionSystem,
) -> HashMap<ClockIndex, SpecificClock> {
    let mut map = specific_clock_comp_map(sys1);
    map.extend(specific_clock_comp_map(sys2));
    map
}

pub fn state_pair_specific_location(
    state: &StatePair,
    sys1: &dyn TransitionSystem,
    sys2: &dyn TransitionSystem,
) -> SpecificLocation {
    let left = specific_location(&state.locations1.id, sys1);
    let right = specific_location(&state.locations2.id, sys2);
    SpecificLocation::BranchLocation(Box::new(left), Box::new(right))
}

pub fn state_specific_location(state: &State, sys: &dyn TransitionSystem) -> SpecificLocation {
    specific_location(&state.decorated_locations.id, sys)
}

pub fn specific_location(location_id: &LocationID, sys: &dyn TransitionSystem) -> SpecificLocation {
    fn inner(location_id: &LocationID, infos: ComponentInfoTree) -> SpecificLocation {
        match location_id {
            LocationID::Conjunction(left, right)
            | LocationID::Composition(left, right)
            | LocationID::Quotient(left, right) => {
                let (i_left, i_right) = infos.split();
                SpecificLocation::BranchLocation(
                    Box::new(inner(left, i_left)),
                    Box::new(inner(right, i_right)),
                )
            }
            LocationID::Simple(loc_id) => {
                let info = infos.info();
                SpecificLocation::ComponentLocation {
                    comp: SpecificComp::new(info.name.clone(), info.id),
                    location_id: loc_id.clone(),
                }
            }
            LocationID::Special(kind) => SpecificLocation::SpecialLocation(kind.clone()),
            LocationID::AnyLocation => unreachable!("AnyLocation should not be used in a state"),
        }
    }
    inner(location_id, sys.comp_infos())
}
