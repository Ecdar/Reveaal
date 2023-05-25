use edbm::{util::constraints::ClockIndex, zones::OwnedFederation};

use crate::ModelObjects::location::{Location, LocationType};
use crate::{
    EdgeEval::constraint_applyer::apply_constraints_to_state, ModelObjects::component::Declarations,
};

use super::LocationID;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum CompositionType {
    Conjunction,
    Composition,
    Quotient,
    Simple,
}

#[derive(Clone, Debug)]
pub struct LocationTree {
    pub id: LocationID,
    /// The invariant for the `Location`
    pub invariant: Option<OwnedFederation>,
    loc_type: LocationType,
    left: Option<Box<LocationTree>>,
    right: Option<Box<LocationTree>>,
}

impl PartialEq for LocationTree {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.loc_type == other.loc_type
    }
}

impl LocationTree {
    pub fn universal() -> Self {
        LocationTree {
            id: LocationID::Special(crate::System::specifics::SpecialLocation::Universal),
            invariant: None,
            loc_type: LocationType::Universal,
            left: None,
            right: None,
        }
    }

    pub fn error(dim: ClockIndex, quotient_clock_index: ClockIndex) -> Self {
        let inv = OwnedFederation::universe(dim).constrain_eq(quotient_clock_index, 0);

        LocationTree {
            id: LocationID::Special(crate::System::specifics::SpecialLocation::Error),
            invariant: Some(inv),
            loc_type: LocationType::Inconsistent,
            left: None,
            right: None,
        }
    }

    pub fn simple(location: &Location, decls: &Declarations, dim: ClockIndex) -> Self {
        let invariant = if let Some(inv) = location.get_invariant() {
            let mut fed = OwnedFederation::universe(dim);
            fed = apply_constraints_to_state(inv, decls, fed).unwrap();
            Some(fed)
        } else {
            None
        };
        LocationTree {
            id: LocationID::Simple(location.get_id().clone()),
            invariant,
            loc_type: location.get_location_type(),
            left: None,
            right: None,
        }
    }
    /// This method is used to a build partial [`LocationTree`].
    /// A partial [`LocationTree`] means it has a [`LocationID`] that is [`LocationID::AnyLocation`].
    /// A partial [`LocationTree`] has `None` in the field `invariant` since a partial [`LocationTree`]
    /// covers more than one location, and therefore there is no specific `invariant`
    pub fn build_any_location_tree() -> Self {
        LocationTree {
            id: LocationID::AnyLocation,
            invariant: None,
            loc_type: LocationType::Any,
            left: None,
            right: None,
        }
    }

    //Merge two locations keeping the invariants seperate
    pub fn merge_as_quotient(left: &Self, right: &Self) -> Self {
        let id = LocationID::Quotient(Box::new(left.id.clone()), Box::new(right.id.clone()));

        let loc_type = left.loc_type.combine(right.loc_type);

        LocationTree {
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

        let invariant = if let Some(inv1) = &left.invariant {
            if let Some(inv2) = &right.invariant {
                Some(inv1.clone().intersection(inv2))
            } else {
                Some(inv1.clone())
            }
        } else {
            right.invariant.clone()
        };

        let loc_type = left.loc_type.combine(right.loc_type);

        LocationTree {
            id,
            invariant,
            loc_type,
            left: Some(Box::new(left.clone())),
            right: Some(Box::new(right.clone())),
        }
    }

    pub fn get_invariants(&self) -> Option<&OwnedFederation> {
        self.invariant.as_ref()
    }

    pub fn apply_invariants(&self, mut fed: OwnedFederation) -> OwnedFederation {
        if let Some(inv) = &self.invariant {
            fed = fed.intersection(inv);
        }
        fed
    }

    pub fn get_left(&self) -> &LocationTree {
        self.left.as_ref().unwrap()
    }

    pub fn get_right(&self) -> &LocationTree {
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

    /// This function is used when you want to compare [`LocationTree`]s that can contain partial locations.
    pub fn compare_partial_locations(&self, other: &LocationTree) -> bool {
        match (&self.id, &other.id) {
            (LocationID::Composition(..), LocationID::Composition(..))
            | (LocationID::Conjunction(..), LocationID::Conjunction(..))
            | (LocationID::Quotient(..), LocationID::Quotient(..)) => {
                self.get_left().compare_partial_locations(other.get_left())
                    && self
                        .get_right()
                        .compare_partial_locations(other.get_right())
            }
            (LocationID::AnyLocation, LocationID::Simple { .. })
            | (LocationID::Simple { .. }, LocationID::AnyLocation)
            | (LocationID::AnyLocation, LocationID::AnyLocation) => true,
            (LocationID::Simple(loc_id_1), LocationID::Simple(loc_id_2)) => loc_id_1 == loc_id_2,
            (_, _) => false,
        }
    }
}
