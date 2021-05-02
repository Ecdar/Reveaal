use crate::DBMLib::lib;

struct Zone {
    dimension: u32,
    matrix: Vec<i32>,
}

impl Zone {
    pub fn is_valid(&mut self) -> bool {
        lib::rs_dbm_is_valid(self.matrix.as_mut_slice(), self.dimension)
    }

    pub fn init(dimension: u32) -> Self {
        let mut zone = Self {
            dimension,
            matrix: Vec::with_capacity((dimension * dimension) as usize),
        };

        lib::rs_dbm_init(zone.matrix.as_mut_slice(), zone.dimension);

        zone
    }

    pub fn satisfies_i_lt_j(&mut self, var_index_i: u32, var_index_j: u32, bound: i32) -> bool {
        lib::rs_dbm_satisfies_i_LT_j(
            self.matrix.as_mut_slice(),
            self.dimension,
            var_index_i,
            var_index_j,
            bound,
        )
    }

    pub fn satisfies_i_lte_j(&mut self, var_index_i: u32, var_index_j: u32, bound: i32) -> bool {
        lib::rs_dbm_satisfies_i_LTE_j(
            self.matrix.as_mut_slice(),
            self.dimension,
            var_index_i,
            var_index_j,
            bound,
        )
    }

    pub fn satisfies_i_eq_j(&mut self, var_index_i: u32, var_index_j: u32) -> bool {
        lib::rs_dbm_satisfies_i_EQUAL_j(
            self.matrix.as_mut_slice(),
            self.dimension,
            var_index_i,
            var_index_j,
        )
    }

    pub fn satisfies_i_eq_j_bounds(
        &mut self,
        var_index_i: u32,
        var_index_j: u32,
        bound_i: i32,
        bound_j: i32,
    ) -> bool {
        lib::rs_dbm_satisfies_i_EQUAL_j_bounds(
            self.matrix.as_mut_slice(),
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
            var_index_j,
            var_index_i,
            constraint,
        )
    }

    pub fn add_lte_constraint(&mut self, var_index_i: u32, var_index_j: u32, bound: i32) -> bool {
        lib::rs_dbm_add_LTE_constraint(
            self.matrix.as_mut_slice(),
            self.dimension,
            var_index_j,
            var_index_i,
            bound,
        )
    }

    pub fn add_LT_constraint(&mut self, var_index_i: u32, var_index_j: u32, bound: i32) -> bool {
        lib::rs_dbm_add_LT_constraint(
            self.matrix.as_mut_slice(),
            self.dimension,
            var_index_j,
            var_index_i,
            bound,
        )
    }

    pub fn add_eq_constraint(&mut self, var_index_i: u32, var_index_j: u32) -> bool {
        lib::rs_dbm_add_EQ_constraint(
            self.matrix.as_mut_slice(),
            self.dimension,
            var_index_j,
            var_index_i,
        )
    }

    pub fn add_eq_const_constraint(&mut self, var_index: u32, bound: i32) -> bool {
        lib::rs_dbm_add_EQ_const_constraint(
            self.matrix.as_mut_slice(),
            self.dimension,
            var_index: u32,
            bound: i32,
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

    pub fn intersects(&mut self, other: &mut Self) -> bool {
        assert_eq!(
            self.dimension, other.dimension,
            "can not take intersection af two zones with differencing dimension"
        );

        lib::rs_dmb_intersection(
            self.matrix.as_mut_slice(),
            other.matrix.as_mut_slice(),
            self.dimension,
        )
    }

    pub fn update(&mut self, var_index: u32, value: i32) {
        lib::rs_dbm_update(self.matrix.as_mut_slice(), self.dimension, var_index, value)
    }

    pub fn is_subset_eq(&mut self, other: &mut Self) -> bool {
        assert_eq!(
            self.dimension, other.dimension,
            "can not take intersection af two zones with differencing dimension"
        );

        lib::rs_dbm_isSubsetEq(
            self.matrix.as_mut_slice(),
            other.matrix.as_mut_slice(),
            self.dimension,
        )
    }

    // TODO rs_dbm_fed_minus_fed

    // TODO rs_dbm_minus_dbm

    // TODO rs_dbm_extrapolateMaxBounds

    pub fn get_constraint(&mut self, var_index_i: u32, var_index_j: u32) -> i32 {
        lib::rs_dbm_get_constraint(
            self.matrix.as_mut_slice(),
            self.dimension,
            var_index_i,
            var_index_j,
        )
    }

    pub fn get_constraint_from_dbm_ptr(&mut self, var_index_i: u32, var_index_j: u32) -> i32 {
        todo!()
        //lib::rs_dbm_get_constraint_from_dbm_ptr(self.matrix.as_mut_slice(), self.dimension, var_index_i, var_index_j)
    }

    pub fn is_strict(constraint: i32) -> bool {
        lib::rs_raw_is_strict(constraint)
    }

    // TODO rs_raw_to_bound

    // TODO rs_vec_to_fed

    // TODO rs_fed_to_vec

    // TODO fed_to_vec

    pub fn up(&mut self) {
        lib::rs_dbm_up(self.matrix.as_mut_slice(), self.dimension)
    }

    pub fn zero(&mut self) {
        lib::rs_dbm_up(self.matrix.as_mut_slice(), self.dimension)
    }
}
