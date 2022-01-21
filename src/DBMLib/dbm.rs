use crate::DBMLib::lib;
use crate::ModelObjects::max_bounds::MaxBounds;
use crate::ModelObjects::representations::BoolExpression;
use crate::System::input_enabler::build_guard_from_zone;
use colored::Colorize;
use std::collections::HashMap;
use std::f64;
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, std::cmp::PartialEq)]
pub struct Zone {
    pub(crate) dimension: u32,
    pub(in crate::DBMLib) matrix: Vec<i32>,
}

impl Zone {
    pub fn from(vec: Vec<i32>) -> Self {
        let size = vec.len() as f64;
        let dim = size.sqrt().floor() as u32;
        assert_eq!((dim * dim) as usize, vec.len());

        let mut zone = Self {
            dimension: dim,
            matrix: vec,
        };
        zone.close();

        zone
    }

    pub fn new(dimension: u32) -> Self {
        Self {
            dimension,
            matrix: vec![1; (dimension * dimension) as usize],
        }
    }

    pub fn is_valid(&self) -> bool {
        lib::rs_dbm_is_valid(&self.matrix[..], self.dimension)
    }

    pub fn init(dimension: u32) -> Self {
        let mut zone = Self {
            dimension,
            matrix: vec![0; (dimension * dimension) as usize],
        };

        lib::rs_dbm_init(zone.matrix.as_mut_slice(), zone.dimension);

        zone
    }

    pub fn satisfies_i_lt_j(&self, var_index_i: u32, var_index_j: u32, bound: i32) -> bool {
        lib::rs_dbm_satisfies_i_LT_j(
            &self.matrix[..],
            self.dimension,
            var_index_i,
            var_index_j,
            bound,
        )
    }

    pub fn satisfies_i_lte_j(&self, var_index_i: u32, var_index_j: u32, bound: i32) -> bool {
        lib::rs_dbm_satisfies_i_LTE_j(
            &self.matrix[..],
            self.dimension,
            var_index_i,
            var_index_j,
            bound,
        )
    }

    pub fn satisfies_i_eq_j(&self, var_index_i: u32, var_index_j: u32) -> bool {
        lib::rs_dbm_satisfies_i_EQUAL_j(&self.matrix[..], self.dimension, var_index_i, var_index_j)
    }

    pub fn satisfies_i_eq_j_bounds(
        &self,
        var_index_i: u32,
        var_index_j: u32,
        bound_i: i32,
        bound_j: i32,
    ) -> bool {
        lib::rs_dbm_satisfies_i_EQUAL_j_bounds(
            &self.matrix[..],
            self.dimension,
            var_index_i,
            var_index_j,
            bound_i,
            bound_j,
        )
    }

    pub fn constrain1(&mut self, var_index_i: u32, var_index_j: u32, constraint: i32) -> bool {
        lib::rs_dbm_constrain1(
            self.matrix.as_mut_slice(),
            self.dimension,
            var_index_i,
            var_index_j,
            constraint,
        )
    }

    pub fn add_lte_constraint(&mut self, var_index_i: u32, var_index_j: u32, bound: i32) -> bool {
        lib::rs_dbm_add_LTE_constraint(
            self.matrix.as_mut_slice(),
            self.dimension,
            var_index_i,
            var_index_j,
            bound,
        )
    }

    pub fn add_lt_constraint(&mut self, var_index_i: u32, var_index_j: u32, bound: i32) -> bool {
        lib::rs_dbm_add_LT_constraint(
            self.matrix.as_mut_slice(),
            self.dimension,
            var_index_i,
            var_index_j,
            bound,
        )
    }

    pub fn add_eq_constraint(&mut self, var_index_i: u32, var_index_j: u32) -> bool {
        lib::rs_dbm_add_EQ_constraint(
            self.matrix.as_mut_slice(),
            self.dimension,
            var_index_i,
            var_index_j,
        )
    }

    pub fn add_eq_const_constraint(&mut self, var_index: u32, bound: i32) -> bool {
        lib::rs_dbm_add_EQ_const_constraint(
            self.matrix.as_mut_slice(),
            self.dimension,
            var_index,
            bound,
        )
    }

    pub fn add_and_constraint(
        &mut self,
        var_index_i: u32,
        var_index_j: u32,
        constraint1: i32,
        constraint2: i32,
    ) -> bool {
        lib::rs_dbm_add_and_constraint(
            self.matrix.as_mut_slice(),
            self.dimension,
            var_index_i,
            var_index_j,
            constraint1,
            constraint2,
        )
    }

    pub fn constrain_var_to_val(&mut self, var_index: u32, value: i32) -> bool {
        lib::rs_dbm_constrain_var_to_val(
            self.matrix.as_mut_slice(),
            self.dimension,
            var_index,
            value,
        )
    }

    pub fn intersection(&mut self, other: &Self) -> bool {
        assert_eq!(
            self.dimension, other.dimension,
            "can not take intersection af two zones with differencing dimension"
        );

        lib::rs_dmb_intersection(
            self.matrix.as_mut_slice(),
            &other.matrix[..],
            self.dimension,
        )
    }

    pub fn update(&mut self, var_index: u32, value: i32) {
        lib::rs_dbm_update(self.matrix.as_mut_slice(), self.dimension, var_index, value)
    }

    pub fn free_clock(&mut self, clock_index: u32) {
        lib::rs_dbm_freeClock(self.matrix.as_mut_slice(), self.dimension, clock_index);
    }

    pub fn is_subset_eq(&self, other: &Self) -> bool {
        assert_eq!(
            self.dimension, other.dimension,
            "can not take intersection af two zones with differencing dimension"
        );

        lib::rs_dbm_isSubsetEq(&self.matrix[..], &other.matrix[..], self.dimension)
    }

    pub fn get_constraint(&self, var_index_i: u32, var_index_j: u32) -> (bool, i32) {
        let raw_constraint =
            lib::rs_dbm_get_constraint(&self.matrix[..], self.dimension, var_index_i, var_index_j);

        (
            lib::rs_raw_is_strict(raw_constraint),
            lib::rs_raw_to_bound(raw_constraint),
        )
    }

    pub fn is_constraint_infinity(&self, var_index_i: u32, var_index_j: u32) -> bool {
        lib::rs_dbm_get_constraint(&self.matrix[..], self.dimension, var_index_i, var_index_j)
            == lib::DBM_INF
    }

    pub fn up(&mut self) {
        lib::rs_dbm_up(self.matrix.as_mut_slice(), self.dimension)
    }

    pub fn down(&mut self) {
        lib::rs_dbm_down(self.matrix.as_mut_slice(), self.dimension)
    }

    pub fn zero(&mut self) {
        lib::rs_dbm_zero(self.matrix.as_mut_slice(), self.dimension)
    }

    pub fn dbm_minus_dbm(&self, other: &Self) -> Federation {
        // NOTE: this function is only used in crate::System::refine::build_state_pair
        // so the implement is just a copy past from that, and is not as generic as it should be.

        assert_eq!(self.dimension, other.dimension);

        lib::rs_dbm_minus_dbm(&self.matrix[..], &other.matrix[..], self.dimension)
    }

    pub fn extrapolate_max_bounds(&mut self, max_bounds: &MaxBounds) {
        lib::rs_dbm_extrapolateMaxBounds(
            &mut self.matrix.as_mut_slice(),
            self.dimension,
            max_bounds.clock_bounds.as_ptr(),
        );
    }

    pub fn canDelayIndefinitely(&self) -> bool {
        for i in 1..self.dimension {
            if !self.is_constraint_infinity(i, 0) {
                return false;
            }
        }

        true
    }

    pub fn close(&mut self) {
        lib::rs_dbm_close(&mut self.matrix, self.dimension);
    }

    pub fn build_guard_from_zone(
        &self,
        clock_names: &HashMap<u32, String>,
    ) -> Option<BoolExpression> {
        let mut guards: Vec<BoolExpression> = vec![];
        for (index, clock) in clock_names {
            let (upper_is_strict, upper_val) = self.get_constraint(*index, 0);
            let (lower_is_strict, lower_val) = self.get_constraint(0, *index);
            // lower bound must be different from 1 (==0)
            if lower_is_strict || lower_val != 0 {
                if lower_is_strict {
                    guards.push(BoolExpression::LessT(
                        Box::new(BoolExpression::Int(-lower_val)),
                        Box::new(BoolExpression::VarName(clock.clone())),
                    ));
                } else {
                    guards.push(BoolExpression::LessEQ(
                        Box::new(BoolExpression::Int(-lower_val)),
                        Box::new(BoolExpression::VarName(clock.clone())),
                    ));
                }
            }
            if !self.is_constraint_infinity(*index, 0) {
                if upper_is_strict {
                    guards.push(BoolExpression::LessT(
                        Box::new(BoolExpression::VarName(clock.clone())),
                        Box::new(BoolExpression::Int(upper_val)),
                    ));
                } else {
                    guards.push(BoolExpression::LessEQ(
                        Box::new(BoolExpression::VarName(clock.clone())),
                        Box::new(BoolExpression::Int(upper_val)),
                    ));
                }
            }
        }
        let res = BoolExpression::conjunction(&mut guards);
        match res {
            BoolExpression::Bool(false) => None,
            _ => Some(res),
        }
    }
}

impl Display for Zone {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("\n")?;
        for i in 0..self.dimension {
            f.write_str("{")?;
            for j in 0..self.dimension {
                if self.is_constraint_infinity(i, j) {
                    f.write_fmt(format_args!("{}", "(<∞)".to_string().bright_blue()))?;
                } else {
                    let (rel, val) = self.get_constraint(i, j);
                    let op = if rel { "<" } else { "≤" };

                    if !rel && val == 0 {
                        f.write_fmt(format_args!("{}", "(≤0)".to_string().bright_green()))?;
                    } else {
                        f.write_fmt(format_args!("({}{})", op, val))?;
                    }
                }
            }
            f.write_str("}\n")?;
        }

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Federation {
    zones: Vec<Zone>,
    dimension: u32,
}

impl Federation {
    pub fn universe(dimension: u32) -> Self {
        Federation::new(vec![Zone::init(dimension)], dimension)
    }

    pub fn new(zones: Vec<Zone>, dimension: u32) -> Self {
        // TODO check zone's dimension
        Self { zones, dimension }
    }

    fn as_raw(&self) -> Vec<*const i32> {
        self.zones.iter().map(|zone| zone.matrix.as_ptr()).collect()
    }

    pub fn minus_fed(&self, other: &Self) -> Federation {
        assert_eq!(self.dimension, other.dimension);

        lib::rs_dbm_fed_minus_fed(&self.as_raw(), &other.as_raw(), self.dimension)
    }

    pub fn is_subset_eq(&self, other: &Self) -> bool {
        self.minus_fed(other).is_empty()
    }

    pub fn intersection(&self, other: &Self) -> Federation {
        assert_eq!(self.dimension, other.dimension);

        self.minus_fed(&self.minus_fed(other))
    }

    pub fn intersect_zone(&self, zone: &Zone) -> Federation {
        let dim = zone.dimension;
        self.intersection(&Federation::new(vec![zone.clone()], dim))
    }

    pub fn inverse(&self, dimensions: u32) -> Federation {
        Federation::universe(dimensions).minus_fed(self)
    }

    pub fn add(&mut self, zone: Zone) {
        self.zones.push(zone);
    }

    pub fn is_empty(&self) -> bool {
        self.zones.is_empty()
    }

    pub fn iter_zones(&self) -> impl Iterator<Item = &Zone> + '_ {
        self.zones.iter()
    }

    pub fn num_zones(&self) -> usize {
        self.zones.len()
    }

    pub fn iter_mut_zones(&mut self) -> impl Iterator<Item = &mut Zone> + '_ {
        self.zones.iter_mut()
    }

    pub fn as_boolexpression(&self, clocks: &HashMap<String, u32>) -> Option<BoolExpression> {
        if self.num_zones() > 1 {
            panic!("Implementation cannot handle disjunct invariants")
        }

        let mut guard = Some(BoolExpression::Bool(false));

        if let Some(zone) = self.iter_zones().next() {
            guard = build_guard_from_zone(&zone, &clocks);
        }

        guard
    }
}
