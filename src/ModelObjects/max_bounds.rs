use std::collections::HashMap;

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
}
