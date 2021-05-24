#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]

use crate::DBMLib::dbm::Federation;

//in DBM lib 0 is < and 1 is <=  here in regards to constraint_index parameter useds
const LT: i32 = 0;
const LTE: i32 = 1;
pub const DBM_INF: i32 = i32::MAX - 1;

// Taken from bindgen's definition of raw_t
type raw_t = i32;
// This type is unused, so we can just let it be the unit type
type dbm_fed_t = ();

/// Checks DBMS validity
/// returns true or false
///
/// # Arguments
///
/// * `dbm` - A mutable pointer to an array, which will be the dbm
/// * `dimension` - The dimension of the dbm
///
/// # Examples
///
/// ```
/// let mut dbm : [i32; 9] = [0; 9];
/// dbm_init(dbm.as_mut_ptr(), 3);
/// ```
pub fn rs_dbm_is_valid(_dbm: &mut [i32], _dimension: u32) -> bool {
    unimplemented!()
}

// pub fn rs_wrapped_dbm_is_valid(dbm: &mut[i32], dimension : u32) -> Result<bool, &'static str> {
//     match unsafe { let res = dbm_isValid(dbm.as_mut_ptr(), dimension); } {
//         res => Ok(true),
//     }
// }
/// Initializes a DBM with
/// * <= 0 on the diagonal and the first row
/// * <= infinity elsewhere
///
/// # Arguments
///
/// * `dbm` - A mutable pointer to an array, which will be the dbm
/// * `dimension` - The dimension of the dbm
///
/// # Examples
///
/// ```
/// let mut dbm : [i32; 9] = [0; 9];
/// dbm_init(dbm.as_mut_ptr(), 3);
/// ```
pub fn rs_dbm_init(_dbm: &mut [i32], _dimension: u32) {
    unimplemented!()
}

/// Checks if `x_i - x_j < bound` is satisfied.
/// It does not modify the DBM
///
/// # Arguments
///
/// * `dbm` - The DBM
/// * `dimension` - The dimension of the dbm
/// * `var_index_i` - The index of the variable representing the ith element
/// * `var_index_j` - The index of the variable representing the jth element
/// * `bound` - The value which bounds the expression
///
/// # Return
/// Bool indicating if the dbm satisfied the restriction
///
/// # Examples
///
/// ```
/// let mut dbm : [i32; 9] = [0; 9];
/// dbm_init(dbm.as_mut_ptr(), 3);
/// rs_dbm_satisfies_i_LT_j(&mut dbm, 3, 1, 2, 10);
/// ```
pub fn rs_dbm_satisfies_i_LT_j(
    _dbm: &mut [i32],
    _dimension: u32,
    _var_index_i: u32,
    _var_index_j: u32,
    _bound: i32,
) -> bool {
    unimplemented!()
}

/// Checks if `x_i - x_j <= bound` is satisfied.
/// It does not modify the DBM
///
/// # Arguments
///
/// * `dbm` - The DBM
/// * `dimension` - The dimension of the dbm
/// * `var_index_i` - The index of the variable representing the ith element
/// * `var_index_j` - The index of the variable representing the jth element
/// * `bound` - The value which bounds the expression
///
/// # Return
/// Bool indicating if the dbm satisfied the restriction
///
/// # Examples
///
/// ```
/// let mut dbm : [i32; 9] = [0; 9];
/// dbm_init(dbm.as_mut_ptr(), 3);
/// rs_dbm_satisfies_i_LTE_j(&mut dbm, 3, 1, 2, 10);
/// ```
pub fn rs_dbm_satisfies_i_LTE_j(
    _dbm: &mut [i32],
    _dimension: u32,
    _var_index_i: u32,
    _var_index_j: u32,
    _bound: i32,
) -> bool {
    unimplemented!()
}

/// Checks if `x_i - x_j = 0` and `x_j - x_i = 0` is satisfied.
/// It does not modify the DBM
///
/// # Arguments
///
/// * `dbm` - The DBM
/// * `dimension` - The dimension of the dbm
/// * `var_index_i` - The index of the variable representing the ith element
/// * `var_index_j` - The index of the variable representing the jth element
///
/// # Return
/// Bool indicating if the dbm satisfied the restriction
///
/// # Examples
///
/// ```
/// let mut dbm : [i32; 9] = [0; 9];
/// dbm_init(dbm.as_mut_ptr(), 3);
/// rs_dbm_satisfies_i_EQUAL_j(&mut dbm, 3, 1, 2);
/// ```
pub fn rs_dbm_satisfies_i_EQUAL_j(
    _dbm: &mut [i32],
    _dimension: u32,
    _var_index_i: u32,
    _var_index_j: u32,
) -> bool {
    unimplemented!()
}

/// Checks if `x_i - x_j = bound_j-bound_i` and `x_j - x_i = bound_i-bound_j` is satisfied.
/// It does not modify the DBM
///
/// # Arguments
///
/// * `dbm` - The DBM
/// * `dimension` - The dimension of the dbm
/// * `var_index_i` - The index of the variable representing the ith element
/// * `var_index_j` - The index of the variable representing the jth element
/// * `bound_i` - The value affecting the variable i
/// * `bound_j` - The value affecting the variable j
///
/// # Return
/// Bool indicating if the dbm satisfied the restriction
///
/// # Examples
///
/// ```
/// let mut dbm : [i32; 9] = [0; 9];
/// dbm_init(dbm.as_mut_ptr(), 3);
/// rs_dbm_satisfies_i_EQUAL_j_bounds(&mut dbm, 3, 1, 2, 10, 4);
/// ```
pub fn rs_dbm_satisfies_i_EQUAL_j_bounds(
    _dbm: &mut [i32],
    _dimension: u32,
    _var_index_i: u32,
    _var_index_j: u32,
    _bound_i: i32,
    _bound_j: i32,
) -> bool {
    unimplemented!()
}

/// Contrain DBM with one constraint.
/// * DBM must be closed and non empty
/// * dim > 1 induced by i < dim & j < dim & i != j
/// * as a consequence: i>=0 & j>=0 & i!=j => (i or j) > 0 and dim > (i or j) > 0 => dim > 1
///
/// # Arguments
///
/// * `dbm` - The DBM
/// * `dimension` - The dimension of the dbm
/// * `var_index_i` - The index of the variable representing the ith element
/// * `var_index_j` - The index of the variable representing the jth element
/// * `constraint` - Constraint for x_i - x_j to use
///
/// # Return
/// Bool indicating if the constraint was applied sucessfully.
///
/// The resulting DBM is closed if it is non empty.
///
/// # Examples
///
/// ```
/// let mut dbm : [i32; 9] = [0; 9];
/// dbm_init(dbm.as_mut_ptr(), 3);
/// let constraint = dbm_boundbool2raw_exposed(10, false);
/// dbm_constrain1(dbm.as_mut_ptr(), 3, 1, 0, constraint);
/// ```
pub fn rs_dbm_constrain1(
    _dbm: &mut [i32],
    _dimension: u32,
    _var_index_i: u32,
    _var_index_j: u32,
    _constraint: i32,
) -> bool {
    unimplemented!()
}

/// Contrain DBM with one <= constraint based on the bound.
/// * DBM must be closed and non empty
/// * dim > 1 induced by i < dim & j < dim & i != j
/// * as a consequence: i>=0 & j>=0 & i!=j => (i or j) > 0 and dim > (i or j) > 0 => dim > 1
///
/// # Arguments
///
/// * `dbm` - The DBM
/// * `dimension` - The dimension of the dbm
/// * `var_index_i` - The index of the variable representing the ith element
/// * `var_index_j` - The index of the variable representing the jth element
/// * `bound` - The value which bounds the expression
///
/// # Return
/// Bool indicating if the constraint was applied sucessfully.
///
/// The resulting DBM is closed if it is non empty.
///
/// # Examples
///
/// ```use regex::internal::Input;

/// let mut dbm : [i32; 9] = [0; 9];
/// dbm_init(dbm.as_mut_ptr(), 3);
/// rs_dbm_add_LTE_constraint(dbm.as_mut_ptr(), 3, 1, 2, 3);
/// ```
pub fn rs_dbm_add_LTE_constraint(
    _dbm: &mut [i32],
    _dimension: u32,
    _var_index_i: u32,
    _var_index_j: u32,
    _bound: i32,
) -> bool {
    unimplemented!()
}

/// Contrain DBM with one < constraint based on the bound.
/// * DBM must be closed and non empty
/// * dim > 1 induced by i < dim & j < dim & i != j
/// * as a consequence: i>=0 & j>=0 & i!=j => (i or j) > 0 and dim > (i or j) > 0 => dim > 1
///
/// # Arguments
///
/// * `dbm` - The DBM
/// * `dimension` - The dimension of the dbm
/// * `var_index_i` - The index of the variable representing the ith element
/// * `var_index_j` - The index of the variable representing the jth element
/// * `bound` - The value which bounds the expression
///
/// # Return
/// Bool indicating if the constraint was applied sucessfully.
///
/// The resulting DBM is closed if it is non empty.
///
/// # Examples
///
/// ```
/// let mut dbm : [i32; 9] = [0; 9];
/// dbm_init(dbm.as_mut_ptr(), 3);
/// rs_dbm_add_LT_constraint(dbm.as_mut_ptr(), 3, 1, 2, 3);
/// ```
pub fn rs_dbm_add_LT_constraint(
    _dbm: &mut [i32],
    _dimension: u32,
    _var_index_i: u32,
    _var_index_j: u32,
    _bound: i32,
) -> bool {
    unimplemented!()
}

/// Contrain DBM with one constraint based on the bound in both directions with bound 0.
///
/// `x_i-x_j <= 0` and `x_j-x_i <= 0`
/// * DBM must be closed and non empty
/// * dim > 1 induced by i < dim & j < dim & i != j
/// * as a consequence: i>=0 & j>=0 & i!=j => (i or j) > 0 and dim > (i or j) > 0 => dim > 1
///
/// # Arguments
///
/// * `dbm` - The DBM
/// * `dimension` - The dimension of the dbm
/// * `var_index_i` - The index of the variable representing the ith element
/// * `var_index_j` - The index of the variable representing the jth element
///
/// # Return
/// Bool indicating if the constraint was applied sucessfully.
///
/// The resulting DBM is closed if it is non empty.
///
/// # Examples
///
/// ```
/// let mut dbm : [i32; 9] = [0; 9];
/// dbm_init(dbm.as_mut_ptr(), 3);
/// rs_dbm_add_EQ_constraint(dbm.as_mut_ptr(), 3, 1, 2);
/// ```
pub fn rs_dbm_add_EQ_constraint(
    _dbm: &mut [i32],
    _dimension: u32,
    _var_index_i: u32,
    _var_index_j: u32,
) -> bool {
    unimplemented!()
}

/// Contrain DBM with one constraint based on the bound
///
/// `x_i-0 <= 5` and `0-x_i <= -5`
/// * DBM must be closed and non empty
/// * dim > 1 induced by i < dim & j < dim & i != j
/// * as a consequence: i>=0 & j>=0 & i!=j => (i or j) > 0 and dim > (i or j) > 0 => dim > 1
///
/// # Arguments
///
/// * `dbm` - The DBM
/// * `dimension` - The dimension of the dbm
/// * `var_index` - The index of the variable representing the ith element
/// * `bound` - The constant bound the clock is set equal to
///
/// # Return
/// Bool indicating if the constraint was applied sucessfully.
///
/// The resulting DBM is closed if it is non empty.
pub fn rs_dbm_add_EQ_const_constraint(
    _dbm: &mut [i32],
    _dimension: u32,
    _var_index: u32,
    _bound: i32,
) -> bool {
    unimplemented!()
}

/// Contrain DBM with two constraints both applied to the same variables.
/// * DBM must be closed and non empty
/// * dim > 1 induced by i < dim & j < dim & i != j
/// * as a consequence: i>=0 & j>=0 & i!=j => (i or j) > 0 and dim > (i or j) > 0 => dim > 1
///
/// # Arguments
///
/// * `dbm` - The DBM
/// * `dimension` - The dimension of the dbm
/// * `var_index_i` - The index of the variable representing the ith element
/// * `var_index_j` - The index of the variable representing the jth element
/// * `constraint1` - First constraint for x_i - x_j to use
/// * `constraint2` - Second constraint for x_i - x_j to use
///
/// # Return
/// Bool indicating if the constraint was applied sucessfully.
///
/// The resulting DBM is closed if it is non empty.
///
/// # Examples
///
/// ```
/// let mut dbm : [i32; 9] = [0; 9];
/// dbm_init(dbm.as_mut_ptr(), 3);
/// let constraint1 = dbm_boundbool2raw_exposed(10, false);
/// let constraint2 = dbm_boundbool2raw_exposed(15, true);
/// rs_dbm_add_and_constraint(dbm.as_mut_ptr(), 3, 1, 2, constraint1, constraint2);
/// ```
pub fn rs_dbm_add_and_constraint(
    _dbm: &mut [i32],
    _dimension: u32,
    _var_index_i: u32,
    _var_index_j: u32,
    _constraint1: i32,
    _constraint2: i32,
) -> bool {
    unimplemented!()
}

/// Contrain DBM by setting variable to value.
///
/// # Arguments
///
/// * `dbm` - The DBM
/// * `dimension` - The dimension of the dbm
/// * `var_index` - The index of the variable
/// * `value` - the value to set the variable to
///
/// # Return
/// Bool indicating if the constraint was applied sucessfully.
///
/// The resulting DBM is closed if it is non empty.
///
/// # Examples
///
/// ```
/// let mut dbm : [i32; 9] = [0; 9];
/// dbm_init(dbm.as_mut_ptr(), 3);
/// rs_dbm_constrain_var_to_val(dbm.as_mut_ptr(), 3, 1, 0);
/// ```
pub fn rs_dbm_constrain_var_to_val(
    _dbm: &mut [i32],
    _dimension: u32,
    _var_index: u32,
    _value: i32,
) -> bool {
    unimplemented!()
}

/// Used to check if dbms intersect and thus have overlapping moves
pub fn rs_dmb_intersection(_dbm1: &mut [i32], _dbm2: &mut [i32], _dimension: u32) -> bool {
    unimplemented!()
}

pub fn rs_dbm_update(_dbm: &mut [i32], _dimension: u32, _var_index: u32, _value: i32) {
    unimplemented!()
}

/// Free operation. Remove all constraints (lower and upper bounds) for a given clock, i.e., set them to infinity
///
/// # Arguments
///
/// * `dbm` - The DBM
/// * `dimension` - The dimension of the dbm
/// * `var_index` - The index of the clock to free
///
pub fn rs_dbm_freeClock(_dbm: &mut [i32], _dimension: u32, _var_index: u32) {
    unimplemented!()
}

/** Test only if dbm1 <= dbm2.
 * @param dbm1,dbm2: DBMs to be tested.
 * @param dim: dimension of the DBMs.
 * @pre
 * - dbm1 and dbm2 have the same dimension
 * - dbm_isValid for both DBMs
 * @return TRUE if dbm1 <= dbm2, FALSE otherwise.
 */
pub fn rs_dbm_isSubsetEq(_dbm1: &mut [i32], _dbm2: &mut [i32], _dimension: u32) -> bool {
    unimplemented!()
}

///  oda federation minus federation
pub fn rs_dbm_fed_minus_fed(
    _dbm_vec1: &mut Vec<*mut raw_t>,
    _dbm_vec2: &mut Vec<*mut raw_t>,
    _dim: u32,
) -> Federation {
    unimplemented!()
}

/// currently unused
pub fn rs_dbm_minus_dbm(_dbm1: &mut [i32], _dbm2: &mut [i32], _dim: u32) -> Federation {
    unimplemented!()
}

///Currently unused
pub fn rs_dbm_extrapolateMaxBounds(_dbm1: &mut [i32], _dim: u32, _maxbounds: *const i32) {
    unimplemented!()
}

pub fn rs_dbm_get_constraint(
    _dbm: &mut [i32],
    _dimension: u32,
    _var_index_i: u32,
    _var_index_j: u32,
) -> raw_t {
    unimplemented!()
}

///used by input enabler to get the upper and lower bounds for each clocks so that constraints can be created
pub fn rs_dbm_get_constraint_from_dbm_ptr(
    _dbm: *const i32,
    _dimension: u32,
    _var_index_i: u32,
    _var_index_j: u32,
) -> raw_t {
    unimplemented!()
}

/// used in input enabler to check if the constraint is strictly bound e.g strictly less than
pub fn rs_raw_is_strict(_raw: raw_t) -> bool {
    unimplemented!()
}

///converts the bound from c++ to an usable rust type - used when input enabling
pub fn rs_raw_to_bound(_raw: raw_t) -> i32 {
    unimplemented!()
}

pub fn rs_vec_to_fed(_dbm_vec: &mut Vec<*mut raw_t>, _dim: u32) -> dbm_fed_t {
    unimplemented!()
}

/// takes a c++ federation and convert it to a vector of pointers
pub fn rs_fed_to_vec(_fed: &mut dbm_fed_t) -> Vec<*const i32> {
    unimplemented!()
}
///does a dbm up operation
pub fn rs_dbm_up(_dbm: &mut [i32], _dimension: u32) {
    unimplemented!()
}

///setup a slice to be a zero dbm
pub fn rs_dbm_zero(_dbm: &mut [i32], _dimension: u32) {
    unimplemented!()
}

/// test function taken from Jecdar
pub fn libtest() {
    unimplemented!()
}

/// test function taken from Jecdar
pub fn libtest2() {
    unimplemented!()
}
