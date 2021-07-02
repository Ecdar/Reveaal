use crate::DBMLib::dbm::Zone;
use std::collections::HashMap;

#[derive(Clone)]
pub struct MaxBounds {
    clock_bounds: HashMap<u32, i32>,
}

impl MaxBounds {
    pub fn create() -> Self {
        MaxBounds {
            clock_bounds: HashMap::new(),
        }
    }

    pub fn add_bound(&mut self, clock: u32, bound: i32) {
        if !self.clock_bounds.contains_key(&clock) {
            self.clock_bounds.insert(clock, bound);
        } else if bound > self.clock_bounds[&clock] {
            *self.clock_bounds.get_mut(&clock).unwrap() = bound;
        }
    }

    pub fn clock_count(&self) -> usize {
        self.clock_bounds.len()
    }

    pub fn get(&self, clock: u32) -> i32 {
        *self.clock_bounds.get(&clock).unwrap()
    }

    pub fn add_bounds(&mut self, bounds: &MaxBounds) {
        for (clock, constraint) in &bounds.clock_bounds {
            self.add_bound(*clock, *constraint);
        }
    }

    pub fn is_zone_within_bounds(&self, zone: &mut Zone) -> bool {
        for (clock, max_bound) in &self.clock_bounds {
            let (_, zone_lower_bound) = zone.get_constraint(0, *clock);
            if *max_bound > zone_lower_bound {
                return true;
            }
        }
        false
    }
}
