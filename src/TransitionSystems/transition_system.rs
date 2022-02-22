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
    locations: HashMap<usize, (&'a Location, Declarations)>,
}

impl<'a> LocationTuple<'a> {
    pub fn get_location(&self, index: usize) -> &Location {
        let (loc, _) = self.locations.get(&index).unwrap();
        loc
    }

    pub fn try_get_location(&self, index: usize) -> Option<&Location> {
        let (loc, _) = self.locations.get(&index)?;
        Some(loc)
    }

    pub fn get_decl(&self, index: usize) -> &Declarations {
        let (_, decl) = self.locations.get(&index).unwrap();
        decl
    }

    pub fn set_location(&mut self, index: usize, new_loc: &'a Location) {
        let (_, decl) = self.locations.remove(&index).unwrap();
        self.locations.insert(index, (new_loc, decl));
    }

    pub fn simple(location: &'a Location, declaration: &Declarations) -> Self {
        let mut locations = HashMap::new();
        locations.insert(0, (location, declaration.clone()));

        LocationTuple { locations }
    }

    pub fn compose(mut left: Self, right: Self) -> Self {
        let offset = left.locations.len();
        for (index, (loc, decl)) in right.locations {
            left.locations.insert(index + offset, (loc, decl));
        }
        left
    }

    pub fn compose_iter<T>(locations_iterator: T) -> Self
    where
        T: IntoIterator<Item = Self>,
    {
        let mut locations = LocationTuple {
            locations: HashMap::new(),
        };

        for new_location in locations_iterator {
            locations = LocationTuple::compose(locations, new_location);
        }

        locations
    }

    pub fn to_string(&self) -> String {
        let len = self.locations.len();

        let mut result = "(".to_string();
        for i in 0..len - 1 {
            let name = self.locations.get(&i).unwrap().0.get_id();
            result.push_str(&format!("{},", name));
        }
        let name = self.locations.get(&(len - 1)).unwrap().0.get_id();
        result.push_str(&format!("{})", name));
        result
    }
    pub fn len(&self) -> usize {
        self.locations.len()
    }
    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, usize, (&Location, Declarations)> {
        self.locations.iter()
    }

    pub fn iter_values(
        &self,
    ) -> std::collections::hash_map::Values<'_, usize, (&Location, Declarations)> {
        self.locations.values()
    }

    pub fn apply_invariants(&self, zone: &mut Zone) -> bool {
        let mut success = true;

        for (location, decl) in self.iter_values() {
            success = success && DecoratedLocation::create(location, decl).apply_invariant(zone);
        }
        success
    }

    pub fn is_initial(&self) -> bool {
        for (location, _) in self.iter_values() {
            if location.location_type != LocationType::Initial {
                return false;
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

    fn get_all_locations<'b>(&'b self) -> Vec<LocationTuple<'b>>;

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

    fn get_all_locations<'b>(&'b self) -> Vec<LocationTuple<'b>> {
        self.get_locations()
            .iter()
            .map(|loc| LocationTuple::simple(loc, &self.declarations))
            .collect()
    }

    fn next_transitions<'b>(
        &'b self,
        locations: &LocationTuple<'b>,
        action: &str,
        sync_type: &SyncType,
        index: &mut usize,
        dim: u32,
    ) -> Vec<Transition<'b>> {
        let location = locations.get_location(*index);
        let next_edges = self.get_next_edges(location, action, *sync_type);

        let mut open_transitions = vec![];
        for edge in next_edges {
            let transition = Transition::from(&vec![(self, edge, *index)], locations, dim);

            open_transitions.push(transition);
        }

        *index += 1;

        open_transitions
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
