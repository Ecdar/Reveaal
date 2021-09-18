use crate::DBMLib::dbm::Federation;
use crate::ModelObjects::max_bounds::MaxBounds;
use crate::TransitionSystems::{LocationTuple, TransitionSystemPtr};
use std::fmt::{Display, Formatter};

#[derive(Clone)]
pub struct StatePair<'a> {
    pub locations1: LocationTuple<'a>,
    pub locations2: LocationTuple<'a>,
    pub federation: Federation,
}

impl<'b> StatePair<'b> {
    pub fn create<'a>(
        dimensions: u32,
        locations1: LocationTuple<'a>,
        locations2: LocationTuple<'a>,
    ) -> StatePair<'a> {
        let mut federation = Federation::zero(dimensions);
        federation.up();

        StatePair {
            locations1,
            locations2,
            federation,
        }
    }

    pub fn extrapolate_max_bounds(&mut self, max_bounds: &MaxBounds) {
        self.federation.extrapolate_max_bounds(max_bounds);
    }

    pub fn is_subset_eq(&mut self, other: &mut Self) -> bool {
        self.federation.is_subset_eq(&mut other.federation)
    }

    pub fn delay(&mut self) {
        self.federation.up();
    }

    pub fn get_dimensions(&self) -> u32 {
        self.federation.dimension
    }

    pub fn get_locations1(&self) -> &LocationTuple<'b> {
        &self.locations1
    }

    pub fn get_locations2(&self) -> &LocationTuple<'b> {
        &self.locations2
    }

    pub fn get_location(&self, is_states1: bool) -> &LocationTuple<'b> {
        if is_states1 {
            &self.locations1
        } else {
            &self.locations2
        }
    }

    //Used to allow borrowing both states as mutable
    pub fn get_mut_states(
        &mut self,
        is_states1: bool,
    ) -> (
        &mut LocationTuple<'b>,
        &mut LocationTuple<'b>,
        &mut Federation,
    ) {
        if is_states1 {
            (
                &mut self.locations1,
                &mut self.locations2,
                &mut self.federation,
            )
        } else {
            (
                &mut self.locations2,
                &mut self.locations1,
                &mut self.federation,
            )
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
        let dim = self.federation.dimension;

        let mut bounds = sys1.get_max_bounds(dim);
        bounds.add_bounds(&sys2.get_max_bounds(dim));

        bounds
    }
}

impl<'b> Display for StatePair<'b> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("Pair: ({{")?;
        for l in self.locations1.iter() {
            f.write_fmt(format_args!("{}, ", l.get_id()))?;
        }
        f.write_str("}}, {{")?;
        for l in self.locations2.iter() {
            f.write_fmt(format_args!("{}, ", l.get_id()))?;
        }
        f.write_str("}}")?;

        f.write_str("Zones:\n")?;
        for zone in &self.federation.zones {
            f.write_fmt(format_args!("{}\n", zone))?;
        }

        Ok(())
    }
}
