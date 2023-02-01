use std::{collections::HashMap, fmt};

use edbm::util::constraints::{ClockIndex, Conjunction, Constraint, Disjunction};

use crate::{
    component::State,
    ModelObjects::statepair::StatePair,
    TransitionSystems::{ComponentInfo, LocationID, TransitionSystem},
};

trait CompInfoIterator {
    fn iter_comp_infos(&self) -> Box<dyn Iterator<Item = &ComponentInfo> + '_>;
}

impl<T: TransitionSystem> CompInfoIterator for T {
    fn iter_comp_infos(&self) -> Box<dyn Iterator<Item = &ComponentInfo> + '_> {
        self.iter_comp_infos()
    }
}

// impl<T: CompInfoIterator> CompInfoIterator for &T {
//     fn iter_comp_infos(&self) -> Box<dyn Iterator<Item = &ComponentInfo>> {
//         self.iter_comp_infos()
//     }
// }

// impl<A: CompInfoIterator, B: CompInfoIterator> CompInfoIterator for (A, B) {
//     fn iter_comp_infos(&self) -> Box<dyn Iterator<Item = &ComponentInfo>> {
//         Box::new(self.0.iter_comp_infos().chain(self.1.iter_comp_infos()))
//     }
// }

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum CompID {
    Comp(u32),
    Quotient,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SpecificComp {
    pub name: String,
    pub id: CompID,
}

impl SpecificComp {
    pub fn new(name: String, id: u32) -> Self {
        Self {
            name,
            id: CompID::Comp(id),
        }
    }

    fn quotient() -> Self {
        Self {
            name: "QUOTIENT".to_owned(),
            id: CompID::Quotient,
        }
    }

    pub fn id(&self) -> u32 {
        match self.id {
            CompID::Comp(id) => id,
            CompID::Quotient => panic!("Cannot get component id of QUOTIENT"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SpecificLocation {
    pub comp: SpecificComp,
    pub location_id: String,
}

impl SpecificLocation {
    pub fn new(
        component_name: impl Into<String>,
        location_id: impl Into<String>,
        component_id: u32,
    ) -> Self {
        Self {
            comp: SpecificComp::new(component_name.into(), component_id),
            location_id: location_id.into(),
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
    Clock(SpecificClock),
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
                _ => SpecificClockVar::Clock(
                    sys.get(&clock)
                        .unwrap_or_else(|| panic!("Clock {} not found in map {:?}", clock, sys))
                        .clone(),
                ),
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
    pub locations: Vec<SpecificLocation>,
    pub constraints: SpecificDisjunction,
}

impl SpecificState {
    pub fn from_state(state: &State, sys: &dyn TransitionSystem) -> Self {
        let locations = specific_locations(&state.decorated_locations.id, sys);
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
        // let locs1 = specific_locations(&state.locations1.id, sys1);
        // let locs2 = specific_locations(&state.locations2.id, sys2);
        let locations = state_pair_specific_locations(state, sys1, sys2);

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
        let locs = self
            .locations
            .iter()
            .map(|l| format!("{}:{}", l.comp.name, l.location_id))
            .collect::<Vec<_>>()
            .join(", ");

        write!(f, "({})", locs)
        // TODO: maybe show constraints
        // write!(f, "({} | {})", locs, self.constraints)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SpecificClock {
    pub name: String,
    pub id: ClockIndex,
    pub comp: SpecificComp,
}

impl SpecificClock {
    pub fn new(name: String, id: ClockIndex, comp: SpecificComp) -> Self {
        Self { name, id, comp }
    }

    fn quotient(id: ClockIndex) -> Self {
        Self {
            name: "QUOTIENT".to_string(),
            id,
            comp: SpecificComp::quotient(),
        }
    }
}

pub fn specific_clock_comp_map(sys: &dyn TransitionSystem) -> HashMap<ClockIndex, SpecificClock> {
    sys.iter_comp_infos()
        .flat_map(|comp| {
            //let name = comp.name.clone();
            //let id = comp.id;
            comp.declarations
                .clocks
                .iter()
                .map(move |(clock, &clock_id)| {
                    (
                        clock_id,
                        SpecificClock::new(
                            clock.clone(),
                            clock_id,
                            SpecificComp::new(comp.name.clone(), comp.id),
                        ),
                    )
                })
        })
        .chain(
            sys.get_quotient_clock()
                .map(|c| (c, SpecificClock::quotient(c))),
        )
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

pub type SpecificLocations = Vec<SpecificLocation>;

pub fn state_pair_specific_locations(
    state: &StatePair,
    sys1: &dyn TransitionSystem,
    sys2: &dyn TransitionSystem,
) -> SpecificLocations {
    let locs1 = specific_locations(&state.locations1.id, sys1);
    let locs2 = specific_locations(&state.locations2.id, sys2);
    locs1.into_iter().chain(locs2.into_iter()).collect()
}

pub fn state_specific_locations(state: &State, sys: &dyn TransitionSystem) -> SpecificLocations {
    specific_locations(&state.decorated_locations.id, sys)
}

pub fn specific_locations(
    location_id: &LocationID,
    sys: &dyn TransitionSystem,
) -> SpecificLocations {
    location_id
        .iter_loc_names()
        .zip(sys.iter_comp_infos())
        .map(|(name, comp)| SpecificLocation::new(comp.name.clone(), name.clone(), comp.id))
        .collect()
}

mod proto_conversions {
    use super::*;
    use crate::ProtobufServer::services::{
        ComponentClock as ProtoComponentClock, Conjunction as ProtoConjunction,
        Constraint as ProtoConstraint, Disjunction as ProtoDisjunction,
        Federation as ProtoFederation, Location as ProtoLocation,
        LocationTuple as ProtoLocationTuple, SpecificComponent as ProtoSpecificComponent,
        State as ProtoState,
    };
    impl From<SpecificState> for ProtoState {
        fn from(state: SpecificState) -> Self {
            ProtoState {
                location_tuple: Some(ProtoLocationTuple {
                    locations: state.locations.into_iter().map(|l| l.into()).collect(),
                }),
                federation: Some(state.constraints.into()),
            }
        }
    }

    impl From<SpecificLocation> for ProtoLocation {
        fn from(loc: SpecificLocation) -> Self {
            ProtoLocation {
                id: loc.location_id,
                specific_component: loc.comp.into(),
            }
        }
    }

    impl From<SpecificComp> for Option<ProtoSpecificComponent> {
        fn from(comp: SpecificComp) -> Self {
            match comp.id {
                CompID::Comp(component_index) => Some(ProtoSpecificComponent {
                    component_name: comp.name,
                    component_index,
                }),
                CompID::Quotient => None,
            }
        }
    }

    impl From<SpecificDisjunction> for ProtoFederation {
        fn from(disj: SpecificDisjunction) -> Self {
            ProtoFederation {
                disjunction: Some(ProtoDisjunction {
                    conjunctions: disj
                        .conjunctions
                        .into_iter()
                        .map(|conj| conj.into())
                        .collect(),
                }),
            }
        }
    }

    impl From<SpecificConjunction> for ProtoConjunction {
        fn from(conj: SpecificConjunction) -> Self {
            ProtoConjunction {
                constraints: conj.constraints.into_iter().map(|c| c.into()).collect(),
            }
        }
    }

    impl From<SpecificConstraint> for ProtoConstraint {
        fn from(constraint: SpecificConstraint) -> Self {
            Self {
                x: constraint.i.into(),
                y: constraint.j.into(),
                strict: constraint.strict,
                c: constraint.c,
            }
        }
    }

    impl From<SpecificClockVar> for Option<ProtoComponentClock> {
        fn from(clock: SpecificClockVar) -> Self {
            match clock {
                SpecificClockVar::Zero => None,
                SpecificClockVar::Clock(c) => Some(c.into()),
            }
        }
    }

    impl From<SpecificClock> for ProtoComponentClock {
        fn from(clock: SpecificClock) -> Self {
            Self {
                specific_component: clock.comp.into(),
                clock_name: clock.name,
            }
        }
    }
}
