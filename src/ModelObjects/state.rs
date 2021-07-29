

/// State is a struct used for initial verification of consistency, and determinism as a state that also hols a dbm
/// This is done as the type used in refinement state pair assumes to sides of an operation
/// this should probably be refactored as it causes unnecessary confusion
#[derive(Clone, std::cmp::PartialEq)]
pub struct State<'a> {
    pub decorated_locations: LocationTuple<'a>,
    pub zone: Zone,
}

impl<'a> State<'a> {
    pub fn create(decorated_locations: LocationTuple<'a>, zone: Zone) -> Self {
        State {
            decorated_locations,
            zone,
        }
    }

    pub fn from_location(decorated_locations: LocationTuple<'a>, dimensions: u32) -> Option<Self> {
        let mut zone = Zone::init(dimensions);

        if !decorated_locations.apply_invariants(&mut zone) {
            return None;
        }

        Some(State {
            decorated_locations,
            zone,
        })
    }

    pub fn is_subset_of(&self, other: &Self) -> bool {
        if self.decorated_locations != other.decorated_locations {
            return false;
        }

        self.zone.is_subset_eq(&other.zone)
    }

    pub fn get_location(&self, index: usize) -> &Location {
        self.decorated_locations.get_location(index)
    }

    pub fn get_declarations(&self, index: usize) -> &Declarations {
        self.decorated_locations.get_decl(index)
    }
}