use crate::DBMLib::dbm::Zone;
use crate::ModelObjects::component::DecoratedLocationTuple;
use crate::ModelObjects::max_bounds::MaxBounds;
use crate::ModelObjects::representations::SystemRepresentation;
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug)]
pub struct StatePair<'a> {
    pub locations1: DecoratedLocationTuple<'a>,
    pub locations2: DecoratedLocationTuple<'a>,
    pub zone: Zone,
}

impl<'b> StatePair<'b> {
    pub fn create<'a>(
        locations1: DecoratedLocationTuple<'a>,
        locations2: DecoratedLocationTuple<'a>,
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
        }
    }

    pub fn get_locations1(&self) -> &DecoratedLocationTuple<'b> {
        &self.locations1
    }

    pub fn get_locations2(&self) -> &DecoratedLocationTuple<'b> {
        &self.locations2
    }

    //Used to allow borrowing both states as mutable
    pub fn get_mut_states(
        &mut self,
        is_states1: bool,
    ) -> (
        &mut DecoratedLocationTuple<'b>,
        &mut DecoratedLocationTuple<'b>,
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
    ) -> (&DecoratedLocationTuple<'b>, &DecoratedLocationTuple<'b>) {
        if is_states1 {
            (&self.locations1, &self.locations2)
        } else {
            (&self.locations2, &self.locations1)
        }
    }

    pub fn calculate_max_bound(
        &mut self,
        sys1: &SystemRepresentation,
        sys2: &SystemRepresentation,
    ) -> MaxBounds {
        let mut bounds = MaxBounds::create(self.zone.dimension);

        bounds.add_bounds(&sys1.get_max_bounds(self.zone.dimension));
        bounds.add_bounds(&sys2.get_max_bounds(self.zone.dimension));

        bounds
    }
}

impl<'b> Display for StatePair<'b> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("Pair: ({{")?;
        for l in &self.locations1 {
            f.write_fmt(format_args!("{}, ", l.get_location().get_id()))?;
        }
        f.write_str("}}, {{")?;
        for l in &self.locations2 {
            f.write_fmt(format_args!("{}, ", l.get_location().get_id()))?;
        }
        f.write_str("}}")?;
        f.write_fmt(format_args!("Zone: {}", self.zone))?;

        Ok(())
    }
}
