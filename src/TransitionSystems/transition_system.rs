use crate::DBMLib::dbm::Federation;
use crate::EdgeEval::constraint_applyer::apply_constraints_to_state;
use crate::ModelObjects::component::{
    Channel, Component, DeclarationProvider, Declarations, DecoratedLocation, Location,
    LocationType, State, SyncType, Transition,
};
use crate::ModelObjects::max_bounds::MaxBounds;
use crate::System::local_consistency;
use crate::System::pruning;
use dyn_clone::{clone_trait_object, DynClone};
use std::collections::hash_set::HashSet;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, std::cmp::PartialEq, Eq, Hash)]
pub enum LocationID {
    Conjunction(Box<LocationID>, Box<LocationID>),
    Composition(Box<LocationID>, Box<LocationID>),
    Quotient(Box<LocationID>, Box<LocationID>),
    Simple(String),
}

impl Display for LocationID {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            LocationID::Conjunction(left, right) => {
                match **left {
                    LocationID::Conjunction(_, _) => write!(f, "{}", *left)?,
                    LocationID::Simple(_) => write!(f, "{}", *left)?,
                    _ => write!(f, "({})", *left)?,
                };
                write!(f, "&&")?;
                match **right {
                    LocationID::Conjunction(_, _) => write!(f, "{}", *right)?,
                    LocationID::Simple(_) => write!(f, "{}", *right)?,
                    _ => write!(f, "({})", *right)?,
                };
            }
            LocationID::Composition(left, right) => {
                match **left {
                    LocationID::Composition(_, _) => write!(f, "{}", *left)?,
                    LocationID::Simple(_) => write!(f, "{}", *left)?,
                    _ => write!(f, "({})", *left)?,
                };
                write!(f, "||")?;
                match **right {
                    LocationID::Composition(_, _) => write!(f, "{}", *right)?,
                    LocationID::Simple(_) => write!(f, "{}", *right)?,
                    _ => write!(f, "({})", *right)?,
                };
            }
            LocationID::Quotient(left, right) => {
                match **left {
                    LocationID::Simple(_) => write!(f, "{}", *left)?,
                    _ => write!(f, "({})", *left)?,
                };
                write!(f, "\\\\")?;
                match **right {
                    LocationID::Simple(_) => write!(f, "{}", *right)?,
                    _ => write!(f, "({})", *right)?,
                };
            }
            LocationID::Simple(name) => write!(f, "{}", name)?,
        }
        Ok(())
    }
}

#[derive(Debug, Clone, std::cmp::PartialEq, Eq, Hash, Copy)]
pub enum CompositionType {
    Conjunction,
    Composition,
    Quotient,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocationTuple {
    pub id: LocationID,
    invariant: Option<Federation>,
    pub loc_type: LocationType,
    left: Option<Box<LocationTuple>>,
    right: Option<Box<LocationTuple>>,
}

impl LocationTuple {
    pub fn simple(location: &Location, decls: &Declarations, dim: u32) -> Self {
        let invariant = if let Some(inv) = location.get_invariant() {
            let mut fed = Federation::full(dim);
            apply_constraints_to_state(&inv, decls, &mut fed);
            Some(fed)
        } else {
            None
        };
        LocationTuple {
            id: LocationID::Simple(location.get_id().clone()),
            invariant,
            loc_type: location.get_location_type().clone(),
            left: None,
            right: None,
        }
    }

    //Merge two locations keeping the invariants seperate
    pub fn merge(left: &Self, right: &Self, comp: CompositionType) -> Self {
        let id = match comp {
            CompositionType::Quotient => {
                LocationID::Quotient(Box::new(left.id.clone()), Box::new(right.id.clone()))
            }
            _ => panic!("Invalid merge type {:?}", comp),
        };

        if left.loc_type == right.loc_type
            && (left.loc_type == LocationType::Universal
                || left.loc_type == LocationType::Inconsistent)
        {
            return left.clone();
        }

        let loc_type =
            if left.loc_type == LocationType::Initial && right.loc_type == LocationType::Initial {
                LocationType::Initial
            } else {
                LocationType::Normal
            };

        LocationTuple {
            id,
            invariant: None,
            loc_type,
            left: Some(Box::new(left.clone())),
            right: Some(Box::new(right.clone())),
        }
    }

    //Compose two locations intersecting the invariants
    pub fn compose(left: &Self, right: &Self, comp: CompositionType) -> Self {
        let id = match comp {
            CompositionType::Conjunction => {
                LocationID::Conjunction(Box::new(left.id.clone()), Box::new(right.id.clone()))
            }
            CompositionType::Composition => {
                LocationID::Composition(Box::new(left.id.clone()), Box::new(right.id.clone()))
            }
            _ => panic!("Invalid composition type {:?}", comp),
        };

        if left.loc_type == right.loc_type && (left.is_universal() || left.is_inconsistent()) {
            return left.clone();
        }

        let invariant = if let Some(inv1) = &left.invariant {
            if let Some(inv2) = &right.invariant {
                Some(inv1.intersection(inv2))
            } else {
                Some(inv1.clone())
            }
        } else {
            right.invariant.clone()
        };

        let loc_type =
            if left.loc_type == LocationType::Initial && right.loc_type == LocationType::Initial {
                LocationType::Initial
            } else {
                LocationType::Normal
            };

        LocationTuple {
            id,
            invariant,
            loc_type,
            left: Some(Box::new(left.clone())),
            right: Some(Box::new(right.clone())),
        }
    }

    pub fn get_invariants(&self) -> Option<&Federation> {
        self.invariant.as_ref()
    }

    pub fn apply_invariants(&self, zone: &mut Federation) -> bool {
        if let Some(inv) = &self.invariant {
            zone.intersect(&inv);
        }
        zone.is_valid()
    }

    pub fn get_left(&self) -> &LocationTuple {
        if self.is_universal() || self.is_inconsistent() {
            return &self;
        }
        self.left.as_ref().unwrap()
    }

    pub fn get_right(&self) -> &LocationTuple {
        if self.is_universal() || self.is_inconsistent() {
            return &self;
        }
        self.right.as_ref().unwrap()
    }

    pub fn is_initial(&self) -> bool {
        self.loc_type == LocationType::Initial
    }

    pub fn is_universal(&self) -> bool {
        self.loc_type == LocationType::Universal
    }

    pub fn is_inconsistent(&self) -> bool {
        self.loc_type == LocationType::Inconsistent
    }
}

pub type TransitionSystemPtr = Box<dyn TransitionSystem>;

pub trait TransitionSystem: DynClone {
    fn get_max_bounds(&self, dim: u32) -> MaxBounds;

    fn next_transitions(
        &self,
        location: &LocationTuple,
        action: &str,
        sync_type: &SyncType,
        index: &mut usize,
        dim: u32,
    ) -> Vec<Transition>;

    fn next_outputs(&self, location: &LocationTuple, action: &str, dim: u32) -> Vec<Transition> {
        let mut index = 0;
        self.next_transitions(location, action, &SyncType::Output, &mut index, dim)
    }

    fn next_inputs(&self, location: &LocationTuple, action: &str, dim: u32) -> Vec<Transition> {
        let mut index = 0;
        self.next_transitions(location, action, &SyncType::Input, &mut index, dim)
    }

    fn get_input_actions(&self) -> HashSet<String>;

    fn inputs_contain(&self, action: &str) -> bool {
        self.get_input_actions().contains(action)
    }

    fn get_output_actions(&self) -> HashSet<String>;

    fn outputs_contain(&self, action: &str) -> bool {
        self.get_output_actions().contains(action)
    }

    fn get_actions(&self) -> HashSet<String>;

    fn actions_contain(&self, action: &str) -> bool {
        self.get_actions().contains(action)
    }

    fn get_initial_location(&self, dim: u32) -> Option<LocationTuple>;

    fn get_all_locations(&self, dim: u32) -> Vec<LocationTuple>;

    fn get_num_clocks(&self) -> u32;

    fn get_decls(&self) -> Vec<&Declarations>;

    fn precheck_sys_rep(&self, dim: u32) -> bool;

    fn initialize(&mut self, dimensions: u32) {}

    fn is_deterministic(&self, dim: u32) -> bool;

    fn is_locally_consistent(&self, dimensions: u32) -> bool;

    fn set_clock_indices(&mut self, index: &mut u32);

    fn get_initial_state(&self, dimensions: u32) -> Option<State>;

    fn get_max_clock_index(&self) -> u32;

    fn get_mut_children(&mut self) -> (&mut TransitionSystemPtr, &mut TransitionSystemPtr);

    fn get_children(&self) -> (&TransitionSystemPtr, &TransitionSystemPtr);

    fn get_composition_type(&self) -> CompositionType;
}

clone_trait_object!(TransitionSystem);

impl TransitionSystem for Component {
    fn get_composition_type(&self) -> CompositionType {
        panic!("Components do not have a composition type")
    }

    fn set_clock_indices(&mut self, index: &mut u32) {
        self.declarations.set_clock_indices(*index);

        *index += self.get_num_clocks();
    }

    fn get_max_clock_index(&self) -> u32 {
        *(self.declarations.clocks.values().max().unwrap_or(&0))
    }

    fn get_decls(&self) -> Vec<&Declarations> {
        vec![self.get_declarations()]
    }

    fn get_max_bounds(&self, dim: u32) -> MaxBounds {
        self.get_max_bounds(dim)
    }

    fn get_input_actions(&self) -> HashSet<String> {
        let channels: Vec<Channel> = self.get_input_actions();

        channels.into_iter().map(|c| c.name).collect()
    }

    fn get_output_actions(&self) -> HashSet<String> {
        let channels: Vec<Channel> = self.get_output_actions();

        channels.into_iter().map(|c| c.name).collect()
    }

    fn get_actions(&self) -> HashSet<String> {
        let channels: Vec<Channel> = self.get_actions();

        channels.into_iter().map(|c| c.name).collect()
    }

    fn get_num_clocks(&self) -> u32 {
        self.declarations.get_clock_count()
    }

    fn get_initial_location(&self, dim: u32) -> Option<LocationTuple> {
        let loc = self.get_initial_location()?;
        Some(LocationTuple::simple(loc, &self.declarations, dim))
    }

    fn get_all_locations(&self, dim: u32) -> Vec<LocationTuple> {
        let locations = self
            .get_locations()
            .iter()
            .map(|loc| LocationTuple::simple(loc, &self.declarations, dim))
            .collect();

        locations
    }

    fn next_transitions(
        &self,
        locations: &LocationTuple,
        action: &str,
        sync_type: &SyncType,
        index: &mut usize,
        dim: u32,
    ) -> Vec<Transition> {
        if !crate::ModelObjects::component::contain(
            &match sync_type {
                SyncType::Input => self.get_input_actions(),
                SyncType::Output => self.get_output_actions(),
            },
            action,
        ) {
            return vec![];
        }

        if locations.is_universal() {
            return vec![Transition::new(locations.clone(), dim)];
        }

        if locations.is_inconsistent() && *sync_type == SyncType::Input {
            return vec![Transition::new(locations.clone(), dim)];
        }

        match &locations.id {
            LocationID::Simple(loc_id) => {
                let location = self
                    .get_locations()
                    .iter()
                    .find(|l| (*l.get_id() == *loc_id))
                    .unwrap();
                let mut open_transitions = vec![];

                let next_edges = self.get_next_edges(location, action, *sync_type);
                for edge in next_edges {
                    let transition = Transition::from((self, edge, *index), locations, dim);
                    open_transitions.push(transition);
                }

                *index += 1;
                open_transitions
            }
            _ => panic!("Unsupported location type {:?} for Component", locations.id),
        }
    }

    fn precheck_sys_rep(&self, dim: u32) -> bool {
        if !self.is_deterministic(dim) {
            println!("{} NOT DETERMINISTIC", self.get_name());
            return false;
        }

        if !self.is_locally_consistent(dim) {
            println!("NOT CONSISTENT");
            return false;
        }

        true
        //self.check_consistency(dim, true)
    }

    fn is_deterministic(&self, dim: u32) -> bool {
        Component::is_deterministic(self, dim)
    }

    fn is_locally_consistent(&self, dimensions: u32) -> bool {
        local_consistency::is_least_consistent(self, dimensions)
    }

    fn get_initial_state(&self, dimensions: u32) -> Option<State> {
        let init_loc = LocationTuple::simple(
            self.get_initial_location().unwrap(),
            self.get_declarations(),
            dimensions,
        );

        State::from_location(init_loc, dimensions)
        /*let mut zone = Federation::init(dimensions);
        if !init_loc.apply_invariants(&mut zone) {
            println!("Empty initial state");
            return None;
        }

        Some(State {
            decorated_locations: init_loc,
            zone,
        })*/
    }

    fn get_mut_children(&mut self) -> (&mut TransitionSystemPtr, &mut TransitionSystemPtr) {
        unimplemented!()
    }

    fn get_children(&self) -> (&TransitionSystemPtr, &TransitionSystemPtr) {
        unimplemented!()
    }
}
