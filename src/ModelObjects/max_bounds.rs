use crate::DBMLib::dbm::Zone;
use std::collections::HashMap;
use std::i32;

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

    pub fn clock_count(&self) -> usize {
        self.clock_bounds.len()
    }

    pub fn add_bounds(&mut self, bounds: &MaxBounds) {
        for clock in 0..bounds.clock_bounds.len() {
            self.add_bound(clock as u32, bounds.get(clock));
        }
    }

    pub fn set_zeroes_as_strict(&mut self) {
        self.clock_bounds = self
            .clock_bounds
            .iter()
            .map(|x| return if *x == 0 { 1 } else { *x })
            .collect();
        self.clock_bounds[0] = 0;
    }

    fn get(&self, clock: usize) -> i32 {
        self.clock_bounds[clock]
    }
}
