use crate::DBMLib::lib;
use crate::ModelObjects::max_bounds::MaxBounds;
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, std::cmp::PartialEq)]
pub struct Zone {
    pub(crate) dimension: u32,
    pub(in crate::DBMLib) matrix: Vec<i32>,
}

impl Zone {
    pub fn from(vec: Vec<i32>, dim: u32) -> Self {
        assert_eq!((dim * dim) as usize, vec.len());

        Self {
            dimension: dim,
            matrix: vec,
        }
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

    pub fn zero(&mut self) {
        lib::rs_dbm_zero(self.matrix.as_mut_slice(), self.dimension)
    }

    pub fn dbm_minus_dbm(&self, other: &Self) -> Federation {
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
}

impl Display for Zone {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for i in 0..self.dimension {
            f.write_str("( ")?;
            for j in 0..self.dimension {
                let (rel, val) = self.get_constraint(i, j);
                f.write_fmt(format_args!("({}, {})", rel, val))?;
            }
            f.write_str(")\n")?;
        }

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Federation {
    pub zones: Vec<Zone>,
    pub dimension: u32,
}

impl Federation {
    pub fn new(zones: Vec<Zone>, dimension: u32) -> Self {
        Self { zones, dimension }
    }

    pub fn empty(dimension: u32) -> Self {
        Self {
            zones: vec![],
            dimension,
        }
    }

    pub fn zero(dimension: u32) -> Self {
        let mut zone = Zone::new(dimension);
        zone.zero();

        Self {
            zones: vec![zone],
            dimension,
        }
    }

    pub fn full(dimension: u32) -> Self {
        Self {
            zones: vec![Zone::init(dimension)],
            dimension,
        }
    }

    pub fn intersection(&mut self, other: &Self) -> bool {
        assert_eq!(self.dimension, other.dimension);

        let result = self.minus_fed(&other.inverse());
        self.zones = result.zones;

        !self.zones.is_empty()
    }

    pub fn inverse(&self) -> Federation {
        let result = Federation::full(self.dimension);
        result.minus_fed(self)
    }

    // self ⊆ other
    // other not ⊂ self <=> self ⊆ other
    pub fn is_subset_eq(&mut self, other: &mut Self) -> bool {
        assert_eq!(self.dimension, other.dimension);

        let mut zones_self: Vec<*mut i32> = self
            .zones
            .iter_mut()
            .map(|zone| zone.matrix.as_mut_ptr())
            .collect();
        let mut zones_other: Vec<*mut i32> = other
            .zones
            .iter_mut()
            .map(|zone| zone.matrix.as_mut_ptr())
            .collect();

        lib::rs_fed_is_subset_eq(&mut zones_self, &mut zones_other, self.dimension)
    }

    pub fn can_delay_indefinitely(&mut self) -> bool {
        if self.is_empty() {
            return false;
        }

        let mut delayed_fed = self.clone();
        delayed_fed.up();

        delayed_fed.is_subset_eq(self)
    }

    pub fn is_valid(&self) -> bool {
        if self.is_empty() {
            return false;
        }

        for zone in &self.zones {
            if !zone.is_valid() {
                return false;
            }
        }

        true
    }

    pub fn extrapolate_max_bounds(&mut self, max_bounds: &MaxBounds) {
        let mut zones: Vec<*mut i32> = self
            .zones
            .iter_mut()
            .map(|zone| zone.matrix.as_mut_ptr())
            .collect();

        self.zones = lib::rs_fed_extrapolate_max_bounds(
            &mut zones,
            self.dimension,
            &max_bounds.clock_bounds,
        )
        .zones;
    }

    pub fn update(&mut self, clock_index: u32, value: i32) {
        let mut zones: Vec<*mut i32> = self
            .zones
            .iter_mut()
            .map(|zone| zone.matrix.as_mut_ptr())
            .collect();

        let result =
            lib::rs_fed_update_clock_int_value(&mut zones, self.dimension, clock_index, value);
        self.zones = result.zones;
    }

    pub fn up(&mut self) {
        for zone in &mut self.zones {
            zone.up();
        }
    }

    pub fn minus_fed(&self, other: &Self) -> Federation {
        assert_eq!(self.dimension, other.dimension);

        let self_zones: Vec<*const i32> =
            self.zones.iter().map(|zone| zone.matrix.as_ptr()).collect();
        let other_zones: Vec<*const i32> = other
            .zones
            .iter()
            .map(|zone| zone.matrix.as_ptr())
            .collect();

        lib::rs_dbm_fed_minus_fed(&self_zones, &other_zones, self.dimension)
    }

    pub fn add_fed(&mut self, other: Self) {
        for zone in other.zones {
            self.add(zone);
        }
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

    pub fn iter_mut_zones(&mut self) -> impl Iterator<Item = &mut Zone> + '_ {
        self.zones.iter_mut()
    }

    pub fn add_lte_constraint(&mut self, var_index_i: u32, var_index_j: u32, bound: i32) -> bool {
        lib::rs_fed_add_LTE_constraint(self, var_index_i, var_index_j, bound)
    }

    pub fn add_lt_constraint(&mut self, var_index_i: u32, var_index_j: u32, bound: i32) -> bool {
        lib::rs_fed_add_LT_constraint(self, var_index_i, var_index_j, bound)
    }

    pub fn add_eq_constraint(&mut self, var_index_i: u32, var_index_j: u32) -> bool {
        lib::rs_fed_add_EQ_constraint(self, var_index_i, var_index_j)
    }

    pub fn add_eq_const_constraint(&mut self, var_index: u32, bound: i32) -> bool {
        lib::rs_fed_add_EQ_const_constraint(self, var_index, bound)
    }

    pub fn add_and_constraint(
        &mut self,
        var_index_i: u32,
        var_index_j: u32,
        constraint1: i32,
        constraint2: i32,
    ) -> bool {
        lib::rs_fed_add_and_constraint(self, var_index_i, var_index_j, constraint1, constraint2)
    }

    pub fn free_clock(&mut self, clock_index: u32) {
        lib::rs_fed_free_clock(self, clock_index);
    }
}
impl Display for Federation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Federation[{}]{{", self.zones.len()))?;
        for zone in &self.zones {
            f.write_fmt(format_args!("\n{}", zone))?;
        }
        f.write_str("}")?;
        Ok(())
    }
}
