use crate::DBMLib::dbm::Federation;
use crate::ModelObjects::max_bounds::MaxBounds;
use crate::TransitionSystems::{LocationTuple, TransitionSystemPtr};
use std::fmt::{Display, Formatter};

#[derive(Clone)]
pub struct StatePair<'a> {
    pub locations1: LocationTuple<'a>,
    pub locations2: LocationTuple<'a>,
    pub zone: Federation,
}

impl<'b> StatePair<'b> {
    pub fn create<'a>(
        dimensions: u32,
        locations1: LocationTuple<'a>,
        locations2: LocationTuple<'a>,
    ) -> StatePair<'a> {
        let mut zone = Federation::zero(dimensions);
        zone.up();

        StatePair {
            locations1,
            locations2,
            zone,
        }
    }

    pub fn get_dimensions(&self) -> u32 {
        self.zone.get_dimensions()
    }

    pub fn get_locations1(&self) -> &LocationTuple<'b> {
        &self.locations1
    }

    pub fn get_locations2(&self) -> &LocationTuple<'b> {
        &self.locations2
    }

    //Used to allow borrowing both states as mutable
    pub fn get_mut_states(
        &mut self,
        is_states1: bool,
    ) -> (&mut LocationTuple<'b>, &mut LocationTuple<'b>) {
        if is_states1 {
            (&mut self.locations1, &mut self.locations2)
        } else {
            (&mut self.locations2, &mut self.locations1)
        }
    }

    pub fn get_locations(&self, is_states1: bool) -> (&LocationTuple<'b>, &LocationTuple<'b>) {
        if is_states1 {
            (&self.locations1, &self.locations2)
        } else {
            (&self.locations2, &self.locations1)
        }
    }

    pub fn calculate_max_bound(
        &mut self,
        sys1: &TransitionSystemPtr,
        sys2: &TransitionSystemPtr,
    ) -> MaxBounds {
        let dim = self.zone.get_dimensions();

        let mut bounds = sys1.get_max_bounds(dim);
        bounds.add_bounds(&sys2.get_max_bounds(dim));

        bounds
    }
}

impl<'b> Display for StatePair<'b> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "Pair{{[({}), ({})] {}}}",
            self.locations1
                .iter()
                .map(|l| l.get_id().clone())
                .collect::<Vec<_>>()
                .join(", "),
            self.locations2
                .iter()
                .map(|l| l.get_id().clone())
                .collect::<Vec<_>>()
                .join(", "),
            self.zone
        ))?;

        Ok(())
    }
}
