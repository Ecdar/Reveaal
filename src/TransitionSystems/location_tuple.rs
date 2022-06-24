use super::LocationID;
use crate::{
    DBMLib::dbm::Federation,
    EdgeEval::constraint_applyer::apply_constraints_to_state,
    ModelObjects::component::{Declarations, Location, LocationType},
};
use anyhow::Result;

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
    pub fn simple(location: &Location, decls: &Declarations, dim: u32) -> Result<Self> {
        let invariant = if let Some(inv) = location.get_invariant() {
            let mut fed = Federation::full(dim);
            apply_constraints_to_state(&inv, decls, &mut fed)?;
            Some(fed)
        } else {
            None
        };
        Ok(LocationTuple {
            id: LocationID::Simple(location.get_id().clone()),
            invariant,
            loc_type: location.get_location_type().clone(),
            left: None,
            right: None,
        })
    }

    //Merge two locations keeping the invariants seperate
    pub fn merge_as_quotient(left: &Self, right: &Self) -> Self {
        let id = LocationID::Quotient(Box::new(left.id.clone()), Box::new(right.id.clone()));

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
