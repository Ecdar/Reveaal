use crate::DBMLib::dbm::Zone;
use crate::ModelObjects::component::DecoratedLocationTuple;

#[derive(Clone)]
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
}
