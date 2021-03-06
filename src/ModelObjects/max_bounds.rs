#[derive(Clone)]
pub struct MaxBounds {
    pub clock_bounds: Vec<i32>,
}

impl MaxBounds {
    pub fn create(dimensions: u32) -> Self {
        MaxBounds {
            clock_bounds: vec![0; dimensions as usize],
        }
    }

    pub fn add_bound(&mut self, clock: u32, bound: i32) {
        if self.clock_bounds[clock as usize] < bound {
            self.clock_bounds[clock as usize] = bound;
        }
    }

    pub fn add_bounds(&mut self, bounds: &MaxBounds) {
        for clock in 0..bounds.clock_bounds.len() {
            self.add_bound(clock as u32, bounds.get(clock));
        }
    }

    fn get(&self, clock: usize) -> i32 {
        self.clock_bounds[clock]
    }
}
