use crate::DBMLib::dbm::Zone;
use crate::ModelObjects::component::{
    Channel, Component, DeclarationProvider, Declarations, DecoratedLocation, Location, State,
    SyncType, Transition,
};
use crate::ModelObjects::max_bounds::MaxBounds;
use crate::System::local_consistency;
use dyn_clone::{clone_trait_object, DynClone};
use std::collections::hash_set::HashSet;

#[derive(Debug, Clone, PartialEq)]
pub struct LocationTuple<'a> {
    locations: Vec<&'a Location>,
    declarations: Vec<Declarations>,
}

impl<'a> LocationTuple<'a> {
    pub fn get_location(&self, index: usize) -> &Location {
        self.locations.get(index).unwrap()
    }

    pub fn get_decl(&self, index: usize) -> &Declarations {
        self.declarations.get(index).unwrap()
    }

    pub fn set_location(&mut self, index: usize, new_loc: &'a Location) {
        self.locations[index] = new_loc;
    }

    pub fn simple(location: &'a Location, declaration: &Declarations) -> Self {
        LocationTuple {
            locations: vec![location],
            declarations: vec![declaration.clone()],
        }
    }

    pub fn compose(left: Self, right: Self) -> Self {
        let mut locations = left.locations;
        locations.extend(right.locations);
        let mut declarations = left.declarations;
        declarations.extend(right.declarations);
        LocationTuple {
            locations,
            declarations,
        }
    }

    pub fn to_string(&self) -> String {
        let len = self.locations.len();

        let mut result = "(".to_string();
        for i in 0..len - 1 {
            let name = self.locations.get(i).unwrap().get_id();
            result.push_str(&format!("{},", name));
        }
        let name = self.locations.get(len - 1).unwrap().get_id();
        result.push_str(&format!("{})", name));
        result
    }
    pub fn len(&self) -> usize {
        self.locations.len()
    }
    pub fn iter(&self) -> std::slice::Iter<&Location> {
        self.locations.iter()
    }

    pub fn iter_zipped(
        &self,
    ) -> std::iter::Zip<std::slice::Iter<&Location>, std::slice::Iter<Declarations>> {
        self.locations.iter().zip(self.declarations.iter())
    }

    pub fn apply_invariants(&self, zone: &mut Zone) -> bool {
        let mut success = true;

        for (location, decl) in self.locations.iter().zip(self.declarations.iter()) {
            success = success && DecoratedLocation::create(location, decl).apply_invariant(zone);
        }
        success
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
    ) -> Vec<Transition<'b>>;

    fn next_outputs<'b>(
        &'b self,
        location: &LocationTuple<'b>,
        action: &str,
    ) -> Vec<Transition<'b>> {
        let mut index = 0;
        self.next_transitions(location, action, &SyncType::Output, &mut index)
    }

    fn next_inputs<'b>(
        &'b self,
        location: &LocationTuple<'b>,
        action: &str,
    ) -> Vec<Transition<'b>> {
        let mut index = 0;
        self.next_transitions(location, action, &SyncType::Input, &mut index)
    }

    fn get_input_actions(&self) -> HashSet<String>;

    fn get_output_actions(&self) -> HashSet<String>;

    fn get_initial_location<'b>(&'b self) -> LocationTuple<'b>;

    fn get_all_locations<'b>(&'b self) -> Vec<LocationTuple<'b>>;

    fn get_num_clocks(&self) -> u32;

    fn get_components<'b>(&'b self) -> Vec<&'b Component>;

    fn precheck_sys_rep(&self, dim: u32) -> bool;

    fn initialize(&mut self, dimensions: u32) {}

    fn is_deterministic(&self, dim: u32) -> bool;

    fn is_locally_consistent(&self, dimensions: u32) -> bool;

    fn set_clock_indices(&mut self, index: &mut u32);

    fn get_initial_state(&self, dimensions: u32) -> State;
}

clone_trait_object!(TransitionSystem<'static>);

impl TransitionSystem<'_> for Component {
    fn set_clock_indices(&mut self, index: &mut u32) {
        self.declarations.set_clock_indices(*index);

        *index += self.get_num_clocks();
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

    fn get_num_clocks(&self) -> u32 {
        self.declarations.get_clock_count()
    }

    fn get_initial_location<'b>(&'b self) -> LocationTuple<'b> {
        LocationTuple::simple(self.get_initial_location(), &self.declarations)
    }

    fn get_all_locations<'b>(&'b self) -> Vec<LocationTuple<'b>> {
        self.get_locations()
            .iter()
            .map(|loc| LocationTuple::simple(loc, &self.declarations))
            .collect()
    }

    fn next_transitions<'b>(
        &'b self,
        location: &LocationTuple<'b>,
        action: &str,
        sync_type: &SyncType,
        index: &mut usize,
    ) -> Vec<Transition<'b>> {
        let location = location.get_location(*index);
        let next_edges = self.get_next_edges(location, action, *sync_type);

        let mut open_transitions = vec![];
        for e in next_edges {
            open_transitions.push(Transition {
                edges: vec![(self, e, *index)],
            });
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
        let init_loc = LocationTuple::simple(self.get_initial_location(), self.get_declarations());
        let mut zone = Zone::init(dimensions);
        if !init_loc.apply_invariants(&mut zone) {
            panic!("Invalid starting state");
        }

        State {
            decorated_locations: init_loc,
            zone,
        }
    }
}
