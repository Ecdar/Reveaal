use crate::DBMLib::dbm::Zone;
use crate::ModelObjects::component::DecoratedLocation;
use crate::ModelObjects::max_bounds::MaxBounds;
use crate::ModelObjects::representations::SystemRepresentation;

#[derive(Clone)]
pub struct StatePair<'a> {
    pub locations1: Vec<DecoratedLocation<'a>>,
    pub locations2: Vec<DecoratedLocation<'a>>,
    pub zone: Zone,
    pub max_bounds: MaxBounds,
}

impl<'b> StatePair<'b> {
    pub fn create<'a>(
        locations1: Vec<DecoratedLocation<'a>>,
        locations2: Vec<DecoratedLocation<'a>>,
    ) -> StatePair<'a> {
        let mut dimensions = 1;
        for state in &locations1 {
            dimensions += state.get_dimensions();
        }

        for state in &locations2 {
            dimensions += state.get_dimensions();
        }

        let mut zone = Zone::new(dimensions);
        zone.zero();
        zone.up();

        StatePair {
            locations1,
            locations2,
            zone,
            max_bounds: MaxBounds::create(),
        }
    }

    pub fn get_locations1(&self) -> &Vec<DecoratedLocation<'b>> {
        &self.locations1
    }

    pub fn get_locations2(&self) -> &Vec<DecoratedLocation<'b>> {
        &self.locations2
    }

    //Used to allow borrowing both states as mutable
    pub fn get_mut_states(
        &mut self,
        is_states1: bool,
    ) -> (
        &mut Vec<DecoratedLocation<'b>>,
        &mut Vec<DecoratedLocation<'b>>,
    ) {
        if is_states1 {
            (&mut self.locations1, &mut self.locations2)
        } else {
            (&mut self.locations2, &mut self.locations1)
        }
    }

    pub fn get_states(
        &self,
        is_states1: bool,
    ) -> (&Vec<DecoratedLocation<'b>>, &Vec<DecoratedLocation<'b>>) {
        if is_states1 {
            (&self.locations1, &self.locations2)
        } else {
            (&self.locations2, &self.locations1)
        }
    }

    pub fn get_bounds_mut(&mut self) -> &mut MaxBounds {
        &mut self.max_bounds
    }

    pub fn has_exceeded_max_bounds(&mut self) -> bool {
        !self.max_bounds.is_zone_within_bounds(&mut self.zone)
    }

    pub fn calculate_max_bound(
        &mut self,
        sys1: &SystemRepresentation,
        sys2: &SystemRepresentation,
        is_state1: bool,
    ) {
        let (locations1, locations2) = self.get_mut_states(is_state1);
        let mut bounds = MaxBounds::create();

        let mut comp_index = 0;
        sys1.all_components(&mut |comp| {
            let loc = locations1[comp_index].get_location();
            bounds.add_bounds(&mut comp.get_max_bounds(loc));

            comp_index += 1;
            comp_index < locations1.len() // stop iteration when there are no more locations
        });

        comp_index = 0;
        sys2.all_components(&mut |comp| {
            let loc = locations2[comp_index].get_location();
            bounds.add_bounds(&mut comp.get_max_bounds(loc));

            comp_index += 1;
            comp_index < locations2.len() // stop iteration when there are no more locations
        });

        self.max_bounds = bounds;
    }
}
