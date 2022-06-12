use crate::DBMLib::dbm::Federation;
use crate::EdgeEval::constraint_applyer::apply_constraints_to_state;
use crate::ModelObjects::component::{
    Channel, Component, DeclarationProvider, Declarations, DecoratedLocation, Location,
    LocationType, State, SyncType, Transition,
};
use crate::ModelObjects::max_bounds::MaxBounds;
use crate::ModelObjects::representations::QueryExpression;
use crate::System::local_consistency;
use crate::System::pruning;
use dyn_clone::{clone_trait_object, DynClone};
use std::collections::hash_set::HashSet;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub enum LocationID {
    Conjunction(Box<LocationID>, Box<LocationID>),
    Composition(Box<LocationID>, Box<LocationID>),
    Quotient(Box<LocationID>, Box<LocationID>),
    Simple(String),
}

impl LocationID {
    pub fn from_string(string: &str) -> Self {
        let query = crate::DataReader::parse_queries::parse_to_expression_tree(&format!(
            "consistency: {}",
            string
        ))
        .remove(0);

        match query {
            QueryExpression::Consistency(x) => (*x).into(),
            _ => panic!(""),
        }
    }
}

impl From<QueryExpression> for LocationID {
    fn from(item: QueryExpression) -> Self {
        match item {
            QueryExpression::Conjunction(left, right) => {
                LocationID::Conjunction(Box::new((*left).into()), Box::new((*right).into()))
            }
            QueryExpression::Composition(left, right) => {
                LocationID::Composition(Box::new((*left).into()), Box::new((*right).into()))
            }
            QueryExpression::Quotient(left, right) => {
                LocationID::Quotient(Box::new((*left).into()), Box::new((*right).into()))
            }
            QueryExpression::Parentheses(inner) => (*inner).into(),
            QueryExpression::VarName(name) => LocationID::Simple(name),
            _ => panic!(
                "Cannot convert queryexpression with {:?} to LocationID",
                item
            ),
        }
    }
}

impl Display for LocationID {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            LocationID::Conjunction(left, right) => {
                match *(*left) {
                    LocationID::Conjunction(_, _) => write!(f, "{}", (*left))?,
                    LocationID::Simple(_) => write!(f, "{}", (*left))?,
                    _ => write!(f, "({})", (*left))?,
                };
                write!(f, "&&")?;
                match *(*right) {
                    LocationID::Conjunction(_, _) => write!(f, "{}", (*right))?,
                    LocationID::Simple(_) => write!(f, "{}", (*right))?,
                    _ => write!(f, "({})", (*right))?,
                };
            }
            LocationID::Composition(left, right) => {
                match *(*left) {
                    LocationID::Composition(_, _) => write!(f, "{}", (*left))?,
                    LocationID::Simple(_) => write!(f, "{}", (*left))?,
                    _ => write!(f, "({})", (*left))?,
                };
                write!(f, "||")?;
                match *(*right) {
                    LocationID::Composition(_, _) => write!(f, "{}", (*right))?,
                    LocationID::Simple(_) => write!(f, "{}", (*right))?,
                    _ => write!(f, "({})", (*right))?,
                };
            }
            LocationID::Quotient(left, right) => {
                match *(*left) {
                    LocationID::Simple(_) => write!(f, "{}", (*left))?,
                    _ => write!(f, "({})", (*left))?,
                };
                write!(f, "\\\\")?;
                match *(*right) {
                    LocationID::Simple(_) => write!(f, "{}", (*right))?,
                    _ => write!(f, "({})", (*right))?,
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

#[derive(Debug, Clone)]
pub struct LocationTuple {
    pub id: LocationID,
    invariant: Option<Federation>,
    pub loc_type: LocationType,
    left: Option<Box<LocationTuple>>,
    right: Option<Box<LocationTuple>>,
}

impl PartialEq for LocationTuple {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.loc_type == other.loc_type
    }
}

impl LocationTuple {
    pub fn simple(location: &Location, decls: &Declarations, dim: u32) -> Self {
        let invariant = if let Some(inv) = location.get_invariant() {
            //println!("invariant of loc {}: {}", location.get_id(), inv);
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
            invariant: None, //left.invariant.clone(),
            loc_type,
            left: Some(Box::new(left.clone())),
            right: Some(Box::new(right.clone())),
        }
    }

    //Compose two locations intersecting the invariants
    pub fn compose(left: &Self, right: &Self, comp: CompositionType) -> Self {
        /*println!(
            "Composing {} inv {} and {} inv {}",
            left.id,
            left.get_invariants().unwrap_or(&Federation::full(1)),
            right.id,
            right.get_invariants().unwrap_or(&Federation::full(1))
        );*/
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
    fn get_max_bounds(&self) -> MaxBounds;

    fn get_dim(&self) -> u32;

    fn next_transitions_if_available(
        &self,
        location: &LocationTuple,
        action: &str,
    ) -> Vec<Transition> {
        if self.actions_contain(action) {
            self.next_transitions(location, action)
        } else {
            vec![]
        }
    }

    fn next_transitions(&self, location: &LocationTuple, action: &str) -> Vec<Transition>;

    fn next_outputs(&self, location: &LocationTuple, action: &str) -> Vec<Transition> {
        debug_assert!(self.get_output_actions().contains(action));
        self.next_transitions(location, action)
    }

    fn next_inputs(&self, location: &LocationTuple, action: &str) -> Vec<Transition> {
        debug_assert!(self.get_input_actions().contains(action));
        self.next_transitions(location, action)
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

    fn get_initial_location(&self) -> Option<LocationTuple>;

    fn get_all_locations(&self) -> Vec<LocationTuple>;

    fn get_decls(&self) -> Vec<&Declarations>;

    fn precheck_sys_rep(&self) -> bool;

    fn is_deterministic(&self) -> bool;

    fn is_locally_consistent(&self) -> bool;

    fn get_initial_state(&self) -> Option<State>;

    fn get_mut_children(&mut self) -> (&mut TransitionSystemPtr, &mut TransitionSystemPtr);

    fn get_children(&self) -> (&TransitionSystemPtr, &TransitionSystemPtr);

    fn get_composition_type(&self) -> CompositionType;
}

clone_trait_object!(TransitionSystem);
