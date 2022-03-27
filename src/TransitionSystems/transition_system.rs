use crate::DBMLib::dbm::Zone;
use crate::ModelObjects::component::{
    Channel, Component, DeclarationProvider, Declarations, DecoratedLocation, Location,
    LocationType, State, SyncType, Transition,
};
use crate::ModelObjects::max_bounds::MaxBounds;
use crate::System::local_consistency;
use dyn_clone::{clone_trait_object, DynClone};
use std::collections::hash_set::HashSet;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct LocationTuple<'a> {
    pub locations: HashMap<usize, (Option<&'a Location>, Declarations)>,
    pub ignore_invariants: HashSet<usize>,
}

impl<'a> LocationTuple<'a> {
    pub fn create(locations: HashMap<usize, (Option<&'a Location>, Declarations)>) -> Self {
        LocationTuple {
            locations,
            ignore_invariants: HashSet::new(),
        }
    }

    pub fn create_empty() -> Self {
        LocationTuple {
            locations: HashMap::new(),
            ignore_invariants: HashSet::new(),
        }
    }

    pub fn copy_range(&self, start: usize, end: usize) -> Self {
        let mut new_location = LocationTuple::create_empty();
        for i in start..end {
            if let Some((loc, decl)) = self.locations.get(&i) {
                new_location.locations.insert(i, (*loc, decl.clone()));

                if self.ignore_invariants.contains(&i) {
                    new_location.ignore_invariants.insert(i);
                }
            }
        }

        new_location
    }

    pub fn add_location_tuple(&mut self, other: &Self) {
        for (index, (loc, decl)) in other.iter() {
            self.locations.insert(*index, (*loc, decl.clone()));
        }

        self.ignore_invariants = &self.ignore_invariants | &other.ignore_invariants;
    }

    pub fn set_default_decl(&mut self, index: usize, decl: Declarations) {
        if !self.locations.contains_key(&index) {
            self.set_location(index, None, decl);
        }
    }

    pub fn set_location(
        &mut self,
        index: usize,
        location: Option<&'a Location>,
        decl: Declarations,
    ) {
        self.locations.insert(index, (location, decl));
    }

    pub fn get_location(&self, index: usize) -> &Location {
        let (loc, _) = self.locations.get(&index).unwrap();
        loc.unwrap()
    }

    pub fn try_get_location(&self, index: usize) -> Option<&Location> {
        let (loc, _) = self.locations.get(&index)?;
        *loc
    }

    pub fn ignore_all_invariants(&mut self) {
        for index in self.locations.keys() {
            self.ignore_invariants.insert(*index);
        }
    }

    pub fn get_decl(&self, index: usize) -> &Declarations {
        let (_, decl) = self.locations.get(&index).unwrap();
        decl
    }

    pub fn simple_indexed(
        index: usize,
        location: &'a Location,
        declaration: &Declarations,
    ) -> Self {
        let mut locations = HashMap::new();
        locations.insert(index, (Some(location), declaration.clone()));

        LocationTuple::create(locations)
    }

    pub fn simple(location: &'a Location, declaration: &Declarations) -> Self {
        Self::simple_indexed(0, location, declaration)
    }

    //Merge two locations if overlapping locations right will override left
    pub fn merge(mut left: Self, right: &Self) -> Self {
        for (index, (loc, decl)) in &right.locations {
            left.locations.insert(*index, (*loc, decl.clone()));
        }
        left.ignore_invariants = left
            .ignore_invariants
            .union(&right.ignore_invariants)
            .cloned()
            .collect();
        left
    }

    pub fn compose(mut left: Self, right: Self) -> Self {
        let offset = left.locations.len();
        for (index, (loc, decl)) in right.locations {
            left.locations.insert(index + offset, (loc, decl));
        }
        left.ignore_invariants = left
            .ignore_invariants
            .union(&right.ignore_invariants)
            .cloned()
            .collect();
        left
    }

    pub fn compose_iter<T>(locations_iterator: T) -> Self
    where
        T: IntoIterator<Item = Self>,
    {
        let mut locations = LocationTuple::create_empty();

        for new_location in locations_iterator {
            locations = LocationTuple::compose(locations, new_location);
        }

        locations
    }

    pub fn to_string(&self) -> String {
        let mut result = "(".to_string();
        let mut key_vec: Vec<usize> = self.locations.keys().cloned().collect();
        key_vec.sort();
        let len = key_vec.len();
        for i in 0..len - 1 {
            if let Some(location) = self.try_get_location(key_vec[i]) {
                let name = location.get_id();
                result.push_str(&format!("{},", name));
            }
        }
        if let Some(location) = self.try_get_location(key_vec[len - 1]) {
            let name = location.get_id();
            result.push_str(&format!("{}", name));
        }
        result.push_str(")");
        result
    }
    pub fn len(&self) -> usize {
        self.locations.len()
    }
    pub fn iter(
        &self,
    ) -> std::collections::hash_map::Iter<'_, usize, (Option<&'a Location>, Declarations)> {
        self.locations.iter()
    }

    pub fn iter_values(
        &self,
    ) -> std::collections::hash_map::Values<'_, usize, (Option<&Location>, Declarations)> {
        self.locations.values()
    }

    pub fn apply_invariants(&self, zone: &mut Zone) -> bool {
        let mut success = true;

        for (index, (opt_location, decl)) in self.iter() {
            if let Some(location) = opt_location {
                if !self.ignore_invariants.contains(index) {
                    success =
                        success && DecoratedLocation::create(location, decl).apply_invariant(zone);
                }
            }
        }
        success
    }

    pub fn apply_invariant_for_location(&self, index: usize, zone: &mut Zone) -> bool {
        if self.ignore_invariants.contains(&index) {
            true
        } else {
            if let Some((opt_location, decl)) = self.locations.get(&index) {
                if let Some(location) = opt_location {
                    DecoratedLocation::create(location, decl).apply_invariant(zone)
                } else {
                    true
                }
            } else {
                panic!("Couldnt find location in index {}", index)
            }
        }
    }

    pub fn is_initial(&self) -> bool {
        for (opt_location, _) in self.iter_values() {
            if let Some(location) = opt_location {
                if location.location_type != LocationType::Initial {
                    return false;
                }
            }
        }
        true
    }
}

pub type TransitionSystemPtr = Box<dyn TransitionSystem<'static>>;

pub trait TransitionSystem<'a>: DynClone {
    fn get_max_bounds(&self, dim: u32) -> MaxBounds;

    fn next_transitions<'b>(
        &'b self,
        location: &LocationTuple<'b>,
        action: &str,
        sync_type: &SyncType,
        index: &mut usize,
        dim: u32,
    ) -> Vec<Transition<'b>>;

    fn next_outputs<'b>(
        &'b self,
        location: &LocationTuple<'b>,
        action: &str,
        dim: u32,
    ) -> Vec<Transition<'b>> {
        let mut index = 0;
        self.next_transitions(location, action, &SyncType::Output, &mut index, dim)
    }

    fn next_inputs<'b>(
        &'b self,
        location: &LocationTuple<'b>,
        action: &str,
        dim: u32,
    ) -> Vec<Transition<'b>> {
        let mut index = 0;
        self.next_transitions(location, action, &SyncType::Input, &mut index, dim)
    }

    fn get_input_actions(&self) -> HashSet<String>;

    fn get_output_actions(&self) -> HashSet<String>;

    fn get_actions(&self) -> HashSet<String>;

    fn get_initial_location<'b>(&'b self) -> Option<LocationTuple<'b>>;

    fn get_all_locations<'b>(&'b self, index: &mut usize) -> Vec<LocationTuple<'b>>;

    fn get_num_clocks(&self) -> u32;

    fn get_components<'b>(&'b self) -> Vec<&'b Component>;

    fn precheck_sys_rep(&self, dim: u32) -> bool;

    fn initialize(&mut self, dimensions: u32) {}

    fn is_deterministic(&self, dim: u32) -> bool;

    fn is_locally_consistent(&self, dimensions: u32) -> bool;

    fn set_clock_indices(&mut self, index: &mut u32);

    fn get_initial_state(&self, dimensions: u32) -> State;

    fn get_max_clock_index(&self) -> u32;

    fn get_mut_children(&mut self) -> Vec<&mut TransitionSystemPtr>;

    fn get_children(&self) -> Vec<&TransitionSystemPtr>;
}

clone_trait_object!(TransitionSystem<'static>);

impl TransitionSystem<'_> for Component {
    fn set_clock_indices(&mut self, index: &mut u32) {
        self.declarations.set_clock_indices(*index);

        *index += self.get_num_clocks();
    }

    fn get_max_clock_index(&self) -> u32 {
        *(self.declarations.clocks.values().max().unwrap_or(&0))
    }

    fn get_components<'b>(&'b self) -> Vec<&'b Component> {
        vec![self]
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

    fn get_initial_location<'b>(&'b self) -> Option<LocationTuple<'b>> {
        let loc = self.get_initial_location()?;
        Some(LocationTuple::simple(loc, &self.declarations))
    }

    fn get_all_locations<'b>(&'b self, index: &mut usize) -> Vec<LocationTuple<'b>> {
        let locations = self
            .get_locations()
            .iter()
            .map(|loc| LocationTuple::simple_indexed(*index, loc, &self.declarations))
            .collect();
        *index += 1;

        locations
    }

    fn next_transitions<'b>(
        &'b self,
        locations: &LocationTuple<'b>,
        action: &str,
        sync_type: &SyncType,
        index: &mut usize,
        dim: u32,
    ) -> Vec<Transition<'b>> {
        if let Some(location) = locations.try_get_location(*index) {
            let next_edges = self.get_next_edges(location, action, *sync_type);
            let mut open_transitions = vec![];
            for edge in next_edges {
                let transition = Transition::from(&vec![(self, edge, *index)], locations, dim);
                open_transitions.push(transition);
            }
            *index += 1;
            open_transitions
        } else {
            *index += 1;
            vec![]
        }
    }

    fn precheck_sys_rep(&self, dim: u32) -> bool {
        self.check_consistency(dim, true)
    }

    fn is_deterministic(&self, dim: u32) -> bool {
        Component::is_deterministic(self, dim)
    }

    fn is_locally_consistent(&self, dimensions: u32) -> bool {
        local_consistency::is_least_consistent(self, dimensions)
    }

    fn get_initial_state(&self, dimensions: u32) -> State {
        let init_loc = LocationTuple::simple(
            self.get_initial_location().unwrap(),
            self.get_declarations(),
        );
        let mut zone = Zone::init(dimensions);
        if !init_loc.apply_invariants(&mut zone) {
            panic!("Invalid starting state");
        }

        State {
            decorated_locations: init_loc,
            zone,
        }
    }

    fn get_mut_children(&mut self) -> Vec<&mut TransitionSystemPtr> {
        vec![]
    }

    fn get_children(&self) -> Vec<&TransitionSystemPtr> {
        vec![]
    }
}
