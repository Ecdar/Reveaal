use crate::ModelObjects::component::{Channel, Component, Location, SyncType, Transition};
use crate::ModelObjects::max_bounds::MaxBounds;
use std::collections::hash_set::HashSet;

#[derive(Debug, Clone)]
pub struct LocationTuple<'a> {
    locations: Vec<&'a Location>,
}

impl<'a> LocationTuple<'a> {
    pub fn get(&self, index: usize) -> &Location {
        self.locations.get(index).unwrap()
    }

    pub fn simple(location: &'a Location) -> Self {
        LocationTuple {
            locations: vec![location],
        }
    }

    pub fn compose(left: Self, right: Self) -> Self {
        let mut locations = left.locations;
        locations.extend(right.locations);

        LocationTuple { locations }
    }
}

pub trait TransitionSystem<'a> {
    fn get_max_bounds(&self) -> MaxBounds;

    fn next_transitions<'b>(
        &'b self,
        location: &LocationTuple<'b>,
        action: &str,
        sync_type: &SyncType,
        index: &mut usize,
    ) -> Vec<Transition<'b>>;

    fn get_input_actions(&self) -> HashSet<String>;

    fn get_output_actions(&self) -> HashSet<String>;

    fn get_initial_location<'b>(&'b self) -> LocationTuple<'b>;

    fn get_all_locations<'b>(&'b self) -> Vec<LocationTuple<'b>>;

    fn get_num_clocks(&self) -> u32;

    //I think this should be implemented elsewhere
    //fn check_consistency(&self) -> bool;
}

impl TransitionSystem<'_> for Component {
    fn get_max_bounds(&self) -> MaxBounds {
        self.get_max_bounds(self.get_num_clocks())
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
        LocationTuple::simple(self.get_initial_location())
    }

    fn get_all_locations<'b>(&'b self) -> Vec<LocationTuple<'b>> {
        self.get_locations()
            .iter()
            .map(|loc| LocationTuple::simple(loc))
            .collect()
    }

    fn next_transitions<'b>(
        &'b self,
        location: &LocationTuple<'b>,
        action: &str,
        sync_type: &SyncType,
        index: &mut usize,
    ) -> Vec<Transition<'b>> {
        let location = location.get(*index);
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
}
