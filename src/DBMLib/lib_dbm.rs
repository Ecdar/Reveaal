#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]

use crate::DBMLib::dbm::{Federation, Zone};
use crate::ModelObjects::representations;
use std::ptr::slice_from_raw_parts;
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

//in DBM lib 0 is < and 1 is <=  here in regards to constraint_index parameter useds
const LT: i32 = 0;
const LTE: i32 = 1;
pub const DBM_INF: i32 = i32::MAX - 1;

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
pub fn rs_dbm_is_valid(dbm: &mut [i32], dimension: u32) -> bool {
    let first = dbm.get(0).unwrap();
    if first == &-1 {
        return false;
    }
    unsafe {
        let res = dbm_check_validity(dbm.as_mut_ptr(), dimension);

        return if 1 == res {
            true
        } else if 0 == res {
            false
        } else {
            panic!("Could not convert bool value from libary, found {:?}", res)
        };
    }
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
pub fn rs_dbm_init(dbm: &mut [i32], dimension: u32) {
    unsafe {
        dbm_init(dbm.as_mut_ptr(), dimension);
    }
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
    dbm: &mut [i32],
    dimension: u32,
    var_index_i: u32,
    var_index_j: u32,
    bound: i32,
) -> bool {
    unsafe {
        let constraint = dbm_boundbool2raw(bound, true);

        let res = dbm_satisfies(
            dbm.as_mut_ptr(),
            dimension,
            var_index_i,
            var_index_j,
            constraint,
        );
        return if BOOL_TRUE == res {
            true
        } else if BOOL_FALSE == res {
            false
        } else {
            panic!("Could not convert bool value from library, found {:?}", res)
        };
    }
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
    dbm: &mut [i32],
    dimension: u32,
    var_index_i: u32,
    var_index_j: u32,
    bound: i32,
) -> bool {
    unsafe {
        let constraint = dbm_boundbool2raw(bound, false);

        let res = dbm_satisfies(
            dbm.as_mut_ptr(),
            dimension,
            var_index_i,
            var_index_j,
            constraint,
        );
        return if BOOL_TRUE == res {
            true
        } else if BOOL_FALSE == res {
            false
        } else {
            panic!("Could not convert bool value from libary, found {:?}", res)
        };
    }
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
    dbm: &mut [i32],
    dimension: u32,
    var_index_i: u32,
    var_index_j: u32,
) -> bool {
    unsafe {
        let constraint = dbm_boundbool2raw(0, false);

        let res_i_minus_j = dbm_satisfies(
            dbm.as_mut_ptr(),
            dimension,
            var_index_i,
            var_index_j,
            constraint,
        );
        let res_j_minus_i = dbm_satisfies(
            dbm.as_mut_ptr(),
            dimension,
            var_index_j,
            var_index_i,
            constraint,
        );
        return if BOOL_TRUE == res_i_minus_j && BOOL_TRUE == res_j_minus_i {
            true
        } else if (BOOL_FALSE == res_i_minus_j && BOOL_TRUE == res_j_minus_i)
            || (BOOL_TRUE == res_i_minus_j && BOOL_FALSE == res_j_minus_i)
            || (BOOL_FALSE == res_i_minus_j && BOOL_FALSE == res_j_minus_i)
        {
            false
        } else {
            panic!(
                "Could not convert bool value from libary, found {:?} for i - j and {:?} for j - i",
                res_i_minus_j, res_j_minus_i
            )
        };
    }
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
    dbm: &mut [i32],
    dimension: u32,
    var_index_i: u32,
    var_index_j: u32,
    bound_i: i32,
    bound_j: i32,
) -> bool {
    unsafe {
        let constraint_i_minus_j = dbm_boundbool2raw(bound_j - bound_i, false);
        let constraint_j_minus_i = dbm_boundbool2raw(bound_i - bound_j, false);

        let res_i_minus_j = dbm_satisfies(
            dbm.as_mut_ptr(),
            dimension,
            var_index_i,
            var_index_j,
            constraint_i_minus_j,
        );
        let res_j_minus_i = dbm_satisfies(
            dbm.as_mut_ptr(),
            dimension,
            var_index_j,
            var_index_i,
            constraint_j_minus_i,
        );
        return if BOOL_TRUE == res_i_minus_j && BOOL_TRUE == res_j_minus_i {
            true
        } else if (BOOL_FALSE == res_i_minus_j && BOOL_TRUE == res_j_minus_i)
            || (BOOL_TRUE == res_i_minus_j && BOOL_FALSE == res_j_minus_i)
            || (BOOL_FALSE == res_i_minus_j && BOOL_FALSE == res_j_minus_i)
        {
            false
        } else {
            panic!(
                "Could not convert bool value from libary, found {:?} for i - j and {:?} for j - i",
                res_i_minus_j, res_j_minus_i
            )
        };
    }
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
/// let constraint = dbm_boundbool2raw(10, false);
/// dbm_constrain1(dbm.as_mut_ptr(), 3, 1, 0, constraint);
/// ```
pub fn rs_dbm_constrain1(
    dbm: &mut [i32],
    dimension: u32,
    var_index_i: u32,
    var_index_j: u32,
    constraint: i32,
) -> bool {
    unsafe {
        let res = dbm_constrain1(
            dbm.as_mut_ptr(),
            dimension,
            var_index_i,
            var_index_j,
            constraint,
        );
        return if BOOL_TRUE == res {
            true
        } else if BOOL_FALSE == res {
            false
        } else {
            panic!("Could not convert bool value from libary, found {:?}", res)
        };
    }
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
    dbm: &mut [i32],
    dimension: u32,
    var_index_i: u32,
    var_index_j: u32,
    bound: i32,
) -> bool {
    unsafe {
        let constraint = dbm_boundbool2raw(bound, false);
        rs_dbm_constrain1(dbm, dimension, var_index_i, var_index_j, constraint)
    }
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
    dbm: &mut [i32],
    dimension: u32,
    var_index_i: u32,
    var_index_j: u32,
    bound: i32,
) -> bool {
    unsafe {
        let constraint = dbm_boundbool2raw(bound, true);

        rs_dbm_constrain1(dbm, dimension, var_index_i, var_index_j, constraint)
    }
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
    dbm: &mut [i32],
    dimension: u32,
    var_index_i: u32,
    var_index_j: u32,
) -> bool {
    unsafe {
        let constraint = dbm_boundbool2raw(0, false);

        let res1 = rs_dbm_constrain1(dbm, dimension, var_index_i, var_index_j, constraint);
        let res2 = rs_dbm_constrain1(dbm, dimension, var_index_j, var_index_i, constraint);
        res1 && res2
    }
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
    dbm: &mut [i32],
    dimension: u32,
    var_index: u32,
    bound: i32,
) -> bool {
    unsafe {
        let constraint1 = dbm_boundbool2raw(bound, false);
        let constraint2 = dbm_boundbool2raw(-bound, false);

        let res1 = rs_dbm_constrain1(dbm, dimension, var_index, 0, constraint1);
        let res2 = rs_dbm_constrain1(dbm, dimension, 0, var_index, constraint2);
        res1 && res2
    }
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
/// let constraint1 = dbm_boundbool2raw(10, false);
/// let constraint2 = dbm_boundbool2raw(15, true);
/// rs_dbm_add_and_constraint(dbm.as_mut_ptr(), 3, 1, 2, constraint1, constraint2);
/// ```
pub fn rs_dbm_add_and_constraint(
    dbm: &mut [i32],
    dimension: u32,
    var_index_i: u32,
    var_index_j: u32,
    constraint1: i32,
    constraint2: i32,
) -> bool {
    let res1 = rs_dbm_constrain1(dbm, dimension, var_index_i, var_index_j, constraint1);
    let res2 = rs_dbm_constrain1(dbm, dimension, var_index_i, var_index_j, constraint2);
    res1 && res2
}

/// Constrain clock to == value, and return
/// * if the result is non empty.
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
    dbm: &mut [i32],
    dimension: u32,
    var_index: u32,
    value: i32,
) -> bool {
    unsafe {
        let res = dbm_constrainClock(dbm.as_mut_ptr(), dimension, var_index, value);
        return if BOOL_TRUE == res {
            true
        } else if BOOL_FALSE == res {
            false
        } else {
            panic!("Could not convert bool value from libary, found {:?}", res)
        };
    }
}

/// Used to check if dbms intersect and thus have overlapping moves
pub fn rs_dmb_intersection(dbm1: &mut [i32], dbm2: &mut [i32], dimension: u32) -> bool {
    unsafe {
        let res = dbm_intersection(dbm1.as_mut_ptr(), dbm2.as_mut_ptr(), dimension);
        return if BOOL_TRUE == res {
            true
        } else if BOOL_FALSE == res {
            false
        } else {
            panic!("Could not convert bool value from libary, found {:?}", res)
        };
    }
}

/// Former "reset" operation, properly called update.
/// * Implement the operation x := v, where x is a clock and v a positive integer.
///
/// # Arguments
///
/// * `dbm` - The DBM
/// * `dimension` - The dimension of the dbm
/// * `var_index` - The index of the clock
/// * `value` - the value to reset the clock to
///
pub fn rs_dbm_update(dbm: &mut [i32], dimension: u32, var_index: u32, value: i32) {
    unsafe {
        dbm_updateValue(dbm.as_mut_ptr(), dimension, var_index, value);
    }
}

/// Free operation. Remove all constraints (lower and upper bounds) for a given clock, i.e., set them to infinity
///
/// # Arguments
///
/// * `dbm` - The DBM
/// * `dimension` - The dimension of the dbm
/// * `var_index` - The index of the clock to free
///
pub fn rs_dbm_freeClock(dbm: &mut [i32], dimension: u32, var_index: u32) {
    unsafe {
        dbm_freeClock(dbm.as_mut_ptr(), dimension, var_index);
    }
}

/** Test only if dbm1 <= dbm2.
 * @param dbm1,dbm2: DBMs to be tested.
 * @param dim: dimension of the DBMs.
 * @pre
 * - dbm1 and dbm2 have the same dimension
 * - dbm_isValid for both DBMs
 * @return TRUE if dbm1 <= dbm2, FALSE otherwise.
 */
pub fn rs_dbm_isSubsetEq(dbm1: &mut [i32], dbm2: &mut [i32], dimension: u32) -> bool {
    unsafe { BOOL_TRUE == dbm_isSubsetEq(dbm1.as_mut_ptr(), dbm2.as_mut_ptr(), dimension) }
}

///  oda federation minus federation
pub fn rs_dbm_fed_minus_fed(
    dbm_vec1: &mut Vec<*mut raw_t>,
    dbm_vec2: &mut Vec<*mut raw_t>,
    dim: u32,
) -> Federation {
    unsafe {
        let mut res = dbm_fed_t::new(dim);
        dbm_fed_minus_fed(
            dbm_vec1.as_mut_ptr(),
            dbm_vec2.as_mut_ptr(),
            (dbm_vec1.len()) as u32,
            (dbm_vec2.len()) as u32,
            dim,
            &mut res,
        );

        fed_to_federation(&mut res, dim)
    }
}

fn fed_to_federation(fed: &mut dbm_fed_t, dim: u32) -> Federation {
    let result = rs_fed_to_vec(fed);

    // zone: [i32; dim * dim]
    let mut zones = Vec::with_capacity(result.len());
    for dbm_ptr in result.iter() {
        if dbm_ptr.is_null() {
            continue;
        }

        let mut zone_vec = Vec::with_capacity((dim * dim) as usize);
        for i in 0..dim {
            for j in 0..dim {
                zone_vec.push(rs_dbm_get_constraint_from_dbm_ptr(*dbm_ptr, dim, i, j))
            }
        }
        zones.push(Zone {
            dimension: dim,
            matrix: zone_vec,
        })
    }

    Federation::new(zones, dim)
}

/// currently unused
pub fn rs_dbm_minus_dbm(dbm1: &mut [i32], dbm2: &mut [i32], dim: u32) -> Federation {
    unsafe {
        let mut res = dbm_subtract1_exposed(dbm1.as_mut_ptr(), dbm2.as_mut_ptr(), dim);
        fed_to_federation(&mut res, dim)
    }
}

///Currently unused
pub fn rs_dbm_extrapolateMaxBounds(dbm1: &mut [i32], dim: u32, maxbounds: *const i32) {
    unsafe { dbm_extrapolateMaxBounds(dbm1.as_mut_ptr(), dim, maxbounds) }
}

pub fn rs_dbm_get_constraint(
    dbm: &mut [i32],
    dimension: u32,
    var_index_i: u32,
    var_index_j: u32,
) -> raw_t {
    unsafe { dbm_get_value(dbm.as_mut_ptr(), dimension, var_index_i, var_index_j) }
}

///used by input enabler to get the upper and lower bounds for each clocks so that constraints can be created
pub fn rs_dbm_get_constraint_from_dbm_ptr(
    dbm: *const i32,
    dimension: u32,
    var_index_i: u32,
    var_index_j: u32,
) -> raw_t {
    unsafe { dbm_get_value(dbm, dimension, var_index_i, var_index_j) }
}

/// used in input enabler to check if the constraint is strictly bound e.g strictly less than
pub fn rs_raw_is_strict(raw: raw_t) -> bool {
    unsafe { BOOL_TRUE == dbm_rawIsStrict(raw) }
}

///converts the bound from c++ to an usable rust type - used when input enabling
pub fn rs_raw_to_bound(raw: raw_t) -> i32 {
    unsafe { dbm_raw2bound(raw) }
}

pub fn rs_vec_to_fed(dbm_vec: &mut Vec<*mut raw_t>, dim: u32) -> dbm_fed_t {
    unsafe {
        let mut res = dbm_fed_t::new(dim);
        dbm_vec_to_fed(dbm_vec.as_mut_ptr(), (dbm_vec.len()) as u32, dim, &mut res);
        res
    }
}

/// takes a c++ federation and convert it to a vector of pointers
pub fn rs_fed_to_vec(fed: &mut dbm_fed_t) -> Vec<*const i32> {
    unsafe {
        let mut result: Vec<*const i32> = vec![];
        let fed_size = dbm_get_fed_size(fed);
        for i in 0..fed_size {
            let raw_data = dbm_get_ith_element_in_fed(fed, i);
            let dbm_slice = slice_from_raw_parts(raw_data, dbm_get_dbm_dimension(fed) as usize);
            let new_const_ptr: *const i32 = (&*dbm_slice).as_ptr();
            result.push(new_const_ptr);
        }

        result
    }
}
///does a dbm up operation
pub fn rs_dbm_up(dbm: &mut [i32], dimension: u32) {
    unsafe {
        dbm_up(dbm.as_mut_ptr(), dimension);
    }
}

///setup a slice to be a zero dbm
pub fn rs_dbm_zero(dbm: &mut [i32], dimension: u32) {
    unsafe {
        dbm_zero(dbm.as_mut_ptr(), dimension);
    }
}

/// test function taken from Jecdar
pub fn libtest() {
    let mut intArr = [0, 0, 0, 0, 0, 0, 0, 0, 0];
    let mut intArr2 = [0, 0, 0, 0, 0, 0, 0, 0, 0];
    let mut arr2 = [1, 1, 2147483646, 1];
    let dbm = &mut intArr;
    let dbm2 = &mut intArr2;
    unsafe {
        println!("dbm before init: {:?}", dbm);
        dbm_init(dbm.as_mut_ptr(), 3);
        println!("dbm after init: {:?}", dbm);

        dbm_init(dbm2.as_mut_ptr(), 3);

        dbm_init(arr2.as_mut_ptr(), 2);
        println!("dbm 2 after init: {:?}", arr2);

        let _testbool = dbm_constrain1(arr2.as_mut_ptr(), 2, 1, 0, 5);
        println!("{:?}", arr2);

        let _testbool = dbm_constrain1(arr2.as_mut_ptr(), 2, 0, 1, -2);
        println!("{:?}", arr2);

        dbm_updateValue(arr2.as_mut_ptr(), 2, 1, 0);

        println!("{:?}", arr2);

        let raw = 3;

        let bound = dbm_raw2bound(raw);
        println!("raw: {:?}, bound: {:?}", raw, bound);

        dbm_zero(arr2.as_mut_ptr(), 2);
        println!("{:?}", arr2);

        println!("dbm before constraint: {:?}", dbm);

        let constraint = dbm_boundbool2raw(0, true);

        rs_dbm_constrain1(dbm, 3, 1, 2, constraint);

        println!("dbm after constraint: {:?}", dbm);

        let res = rs_dbm_satisfies_i_LT_j(dbm, 3, 2, 1, 0);

        println!("Result of satisfies check: {:?}", res);
    }
}

/// test function taken from Jecdar
pub fn libtest2() {
    unsafe {
        let mut dbm: [i32; 9] = [0; 9];
        dbm_init(dbm.as_mut_ptr(), 3);
        println!("{:?}", dbm);

        rs_dbm_satisfies_i_LT_j(&mut dbm, 3, 1, 2, 10);

        dbm_constrain1(dbm.as_mut_ptr(), 3, 1, 0, 3);
        println!("{:?}", dbm);

        dbm_constrain1(dbm.as_mut_ptr(), 3, 2, 0, 2);
        println!("{:?}", dbm);

        dbm_updateValue(dbm.as_mut_ptr(), 3, 1, 0);
        println!("{:?}", dbm);

        dbm_up(dbm.as_mut_ptr(), 3);
        println!("{:?}", dbm);
    }
}
