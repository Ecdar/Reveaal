#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]

macro_rules! trace {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        if (DEBUG) {
            let name = type_name_of(f);
            println!("{}", &name[..name.len() - 3]);
        }
    }};
}

use crate::DBMLib::dbm::{Federation, Relation, Zone};
use crate::ModelObjects::max_bounds::MaxBounds;
use std::{mem, ptr, slice};

use std::convert::TryInto;
mod UDBM {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}
//in DBM lib 0 is < and 1 is <=  here in regards to constraint_index parameter useds
const LT: i32 = 0;
const LTE: i32 = 1;
pub const DBM_INF: i32 = i32::MAX - 1;

pub type fed_raw = UDBM::dbm_fed_t;

const DEBUG: bool = false;

/// Max dim from dbm\build\UDBM\src\udbm\dbm\DBMAllocator.h:32:35
/// If exceeded causes segmentation fault in c code
const MAX_DIM: u32 = 256;

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
/// UDBM::dbm_init(dbm.as_mut_ptr(), 3);
/// ```

pub fn rs_dbm_is_valid(dbm: &UDBM::dbm_dbm_t, dimension: u32) -> bool {
    trace!();
    unsafe {
        let res = UDBM::dbm_check_validity(dbm);

        res
    }
}

// pub fn rs_wrapped_dbm_is_valid(dbm: &mut[i32], dimension : u32) -> Result<bool, &'static str> {
// trace!();
//     match unsafe { let res = UDBM::dbm_isValid(dbm.as_mut_ptr(), dimension); } {
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
/// UDBM::dbm_init(dbm.as_mut_ptr(), 3);
/// ```
pub fn rs_dbm_init(dbm: &mut [i32], dimension: u32) {
    trace!();
    unsafe {
        UDBM::dbm_init(dbm.as_mut_ptr(), dimension);
    }
}

/*
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
/// UDBM::dbm_init(dbm.as_mut_ptr(), 3);
/// rs_dbm_satisfies_i_LT_j(&mut dbm, 3, 1, 2, 10);
/// ```
pub fn rs_dbm_satisfies_i_LT_j(
    dbm: &[i32],
    dimension: u32,
    var_index_i: u32,
    var_index_j: u32,
    bound: i32,
) -> bool {
    trace!();
    unsafe {
        let constraint = UDBM::dbm_boundbool2raw_wrapper(bound, true);

        let res = UDBM::dbm_satisfies_wrapper(
            dbm.as_ptr(),
            dimension,
            var_index_i,
            var_index_j,
            constraint,
        );

        res
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
/// UDBM::dbm_init(dbm.as_mut_ptr(), 3);
/// rs_dbm_satisfies_i_LTE_j(&mut dbm, 3, 1, 2, 10);
/// ```
pub fn rs_dbm_satisfies_i_LTE_j(
    dbm: &[i32],
    dimension: u32,
    var_index_i: u32,
    var_index_j: u32,
    bound: i32,
) -> bool {
    trace!();
    unsafe {
        let constraint = UDBM::dbm_boundbool2raw_wrapper(bound, false);

        let res = UDBM::dbm_satisfies_wrapper(
            dbm.as_ptr(),
            dimension,
            var_index_i,
            var_index_j,
            constraint,
        );

        res
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
/// UDBM::dbm_init(dbm.as_mut_ptr(), 3);
/// rs_dbm_satisfies_i_EQUAL_j(&mut dbm, 3, 1, 2);
/// ```
pub fn rs_dbm_satisfies_i_EQUAL_j(
    dbm: &[i32],
    dimension: u32,
    var_index_i: u32,
    var_index_j: u32,
) -> bool {
    trace!();
    unsafe {
        let constraint = UDBM::dbm_boundbool2raw_wrapper(0, false);

        let res_i_minus_j = UDBM::dbm_satisfies_wrapper(
            dbm.as_ptr(),
            dimension,
            var_index_i,
            var_index_j,
            constraint,
        );
        let res_j_minus_i = UDBM::dbm_satisfies_wrapper(
            dbm.as_ptr(),
            dimension,
            var_index_j,
            var_index_i,
            constraint,
        );
        return if res_i_minus_j && res_j_minus_i {
            true
        } else if (!res_i_minus_j && res_j_minus_i)
            || (res_i_minus_j && !res_j_minus_i)
            || (!res_i_minus_j && !res_j_minus_i)
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
/// UDBM::dbm_init(dbm.as_mut_ptr(), 3);
/// rs_dbm_satisfies_i_EQUAL_j_bounds(&mut dbm, 3, 1, 2, 10, 4);
/// ```
pub fn rs_dbm_satisfies_i_EQUAL_j_bounds(
    dbm: &[i32],
    dimension: u32,
    var_index_i: u32,
    var_index_j: u32,
    bound_i: i32,
    bound_j: i32,
) -> bool {
    trace!();
    unsafe {
        let constraint_i_minus_j = UDBM::dbm_boundbool2raw_wrapper(bound_j - bound_i, false);
        let constraint_j_minus_i = UDBM::dbm_boundbool2raw_wrapper(bound_i - bound_j, false);

        let res_i_minus_j = UDBM::dbm_satisfies_wrapper(
            dbm.as_ptr(),
            dimension,
            var_index_i,
            var_index_j,
            constraint_i_minus_j,
        );
        let res_j_minus_i = UDBM::dbm_satisfies_wrapper(
            dbm.as_ptr(),
            dimension,
            var_index_j,
            var_index_i,
            constraint_j_minus_i,
        );
        return if res_i_minus_j && res_j_minus_i {
            true
        } else if (!res_i_minus_j && res_j_minus_i)
            || (res_i_minus_j && !res_j_minus_i)
            || (!res_i_minus_j && !res_j_minus_i)
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
*/

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
/// UDBM::dbm_init(dbm.as_mut_ptr(), 3);
/// let constraint = UDBM::dbm_boundbool2raw(10, false);
/// UDBM::dbm_constrain1(dbm.as_mut_ptr(), 3, 1, 0, constraint);
/// ```
pub fn rs_dbm_constrain1(
    dbm: &mut [i32],
    dimension: u32,
    var_index_i: u32,
    var_index_j: u32,
    constraint: i32,
) -> bool {
    trace!();
    unsafe {
        let res = UDBM::dbm_constrain1(
            dbm.as_mut_ptr(),
            dimension,
            var_index_i,
            var_index_j,
            constraint,
        );
        return if true == res {
            true
        } else if false == res {
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
/// UDBM::dbm_init(dbm.as_mut_ptr(), 3);
/// rs_dbm_add_LTE_constraint(dbm.as_mut_ptr(), 3, 1, 2, 3);
/// ```
pub fn rs_dbm_add_LTE_constraint(
    dbm: &mut [i32],
    dimension: u32,
    var_index_i: u32,
    var_index_j: u32,
    bound: i32,
) -> bool {
    trace!();
    unsafe {
        let constraint = UDBM::dbm_boundbool2raw_wrapper(bound, false);
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
/// UDBM::dbm_init(dbm.as_mut_ptr(), 3);
/// rs_dbm_add_LT_constraint(dbm.as_mut_ptr(), 3, 1, 2, 3);
/// ```
pub fn rs_dbm_add_LT_constraint(
    dbm: &mut [i32],
    dimension: u32,
    var_index_i: u32,
    var_index_j: u32,
    bound: i32,
) -> bool {
    trace!();
    unsafe {
        let constraint = UDBM::dbm_boundbool2raw_wrapper(bound, true);

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
/// UDBM::dbm_init(dbm.as_mut_ptr(), 3);
/// rs_dbm_add_EQ_constraint(dbm.as_mut_ptr(), 3, 1, 2);
/// ```
pub fn rs_dbm_add_EQ_constraint(
    dbm: &mut [i32],
    dimension: u32,
    var_index_i: u32,
    var_index_j: u32,
) -> bool {
    trace!();
    unsafe {
        let constraint = UDBM::dbm_boundbool2raw_wrapper(0, false);

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
    trace!();
    unsafe {
        let constraint1 = UDBM::dbm_boundbool2raw_wrapper(bound, false);
        let constraint2 = UDBM::dbm_boundbool2raw_wrapper(-bound, false);

        let res1 = rs_dbm_constrain1(dbm, dimension, var_index, 0, constraint1);
        let res2 = rs_dbm_constrain1(dbm, dimension, 0, var_index, constraint2);
        res1 && res2
    }
}

pub fn rs_dbm_close(dbm: &mut [i32], dimension: u32) {
    trace!();
    unsafe {
        UDBM::dbm_close(dbm.as_mut_ptr(), dimension);
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
/// UDBM::dbm_init(dbm.as_mut_ptr(), 3);
/// let constraint1 = UDBM::dbm_boundbool2raw(10, false);
/// let constraint2 = UDBM::dbm_boundbool2raw(15, true);
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
    trace!();
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
/// UDBM::dbm_init(dbm.as_mut_ptr(), 3);
/// rs_dbm_constrain_var_to_val(dbm.as_mut_ptr(), 3, 1, 0);
/// ```
pub fn rs_dbm_constrain_var_to_val(
    dbm: &mut [i32],
    dimension: u32,
    var_index: u32,
    value: i32,
) -> bool {
    trace!();
    unsafe {
        let res = UDBM::dbm_constrainClock(dbm.as_mut_ptr(), dimension, var_index, value);
        return if true == res {
            true
        } else if false == res {
            false
        } else {
            panic!("Could not convert bool value from libary, found {:?}", res)
        };
    }
}

pub fn rs_dbm_extrapolateMaxBounds(dbm1: &mut [i32], dim: u32, maxbounds: *const i32) {
    trace!();
    unsafe { UDBM::dbm_extrapolateMaxBounds(dbm1.as_mut_ptr(), dim, maxbounds) }
}
/*
pub fn rs_dbm_get_constraint(
    dbm: &[i32],
    dimension: u32,
    var_index_i: u32,
    var_index_j: u32,
) -> raw_t {
    //trace!();
    unsafe {
        return UDBM::dbm_get_value(dbm.as_ptr(), dimension, var_index_i, var_index_j);
    }
}*/

/// used in input enabler to check if the constraint is strictly bound e.g strictly less than
pub fn rs_raw_is_strict(raw: UDBM::raw_t) -> bool {
    //trace!();
    unsafe { UDBM::dbm_rawIsStrict_wrapper(raw) }
}

///converts the bound from c++ to an usable rust type - used when input enabling
pub fn rs_raw_to_bound(raw: UDBM::raw_t) -> i32 {
    //trace!();
    unsafe { UDBM::dbm_raw2bound_wrapper(raw) }
}

/*
/// test function taken from Jecdar
pub fn libtest() {
    let mut intArr = [0, 0, 0, 0, 0, 0, 0, 0, 0];
    let mut intArr2 = [0, 0, 0, 0, 0, 0, 0, 0, 0];
    let mut arr2 = [1, 1, 2147483646, 1];
    let dbm = &mut intArr;
    let dbm2 = &mut intArr2;
    unsafe {
        println!("dbm before init: {:?}", dbm);
        UDBM::dbm_init(dbm.as_mut_ptr(), 3);
        println!("dbm after init: {:?}", dbm);

        UDBM::dbm_init(dbm2.as_mut_ptr(), 3);

        UDBM::dbm_init(arr2.as_mut_ptr(), 2);
        println!("dbm 2 after init: {:?}", arr2);

        let _testbool = UDBM::dbm_constrain1(arr2.as_mut_ptr(), 2, 1, 0, 5);
        println!("{:?}", arr2);

        let _testbool = UDBM::dbm_constrain1(arr2.as_mut_ptr(), 2, 0, 1, -2);
        println!("{:?}", arr2);

        UDBM::dbm_updateValue(arr2.as_mut_ptr(), 2, 1, 0);

        println!("{:?}", arr2);

        let raw = 3;

        let bound = UDBM::dbm_raw2bound_wrapper(raw);
        println!("raw: {:?}, bound: {:?}", raw, bound);

        UDBM::dbm_zero_wrapper(arr2.as_mut_ptr(), 2);
        println!("{:?}", arr2);

        println!("dbm before constraint: {:?}", dbm);

        let constraint = UDBM::dbm_boundbool2raw_wrapper(0, true);

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
        UDBM::dbm_init(dbm.as_mut_ptr(), 3);
        println!("{:?}", dbm);

        rs_dbm_satisfies_i_LT_j(&mut dbm, 3, 1, 2, 10);

        UDBM::dbm_constrain1(dbm.as_mut_ptr(), 3, 1, 0, 3);
        println!("{:?}", dbm);

        UDBM::dbm_constrain1(dbm.as_mut_ptr(), 3, 2, 0, 2);
        println!("{:?}", dbm);

        UDBM::dbm_updateValue(dbm.as_mut_ptr(), 3, 1, 0);
        println!("{:?}", dbm);

        UDBM::dbm_up(dbm.as_mut_ptr(), 3);
        println!("{:?}", dbm);
    }
}*/

pub fn rs_dbm_boundbool2raw(bound: i32, is_strict: bool) -> i32 {
    trace!();
    unsafe { UDBM::dbm_boundbool2raw_wrapper(bound, is_strict) }
}

/*
fn rs_zone_from_dbm(dbm_ptr: UDBM::dbm_dbm_t, dim: u32) -> Zone {
    trace!();
    let mut zone_vec = Vec::with_capacity((dim * dim) as usize);
    for i in 0..dim {
        for j in 0..dim {
            zone_vec.push(unsafe { UDBM::dbm_get_value(&dbm_ptr, i, j) }.clone())
        }
    }

    Zone {
        matrix: zone_vec,
        dimension: dim,
    }
}*/

/* BEGIN FEDERATION METHODS*/

pub fn rs_fed_get_zones(fed: &Federation) -> Vec<Zone> {
    trace!();
    let dim = fed.get_dimensions();
    let d = dim as usize;
    let zones = fed.num_zones();
    let len = zones * d * d;

    //let out = vec![0; len];
    if len == 0 {
        return vec![];
    }
    let mut out = Vec::<UDBM::raw_t>::with_capacity(len);
    unsafe {
        UDBM::fed_get_dbm_vec(&fed.raw, out.as_mut_ptr(), len as u64);
        out.set_len(len);
    }

    out.chunks(d * d)
        .into_iter()
        .map(|v| Zone {
            dimension: dim,
            matrix: v.to_vec(),
        })
        .collect()
}

pub fn rs_fed_intersect(fed1: &mut Federation, fed2: &Federation) {
    trace!();
    unsafe {
        UDBM::fed_intersection(&mut fed1.raw, &fed2.raw);
    }
    //rs_fed_reduce(&mut fed1, true);
}

///  oda federation minus federation
pub fn rs_fed_subtract(fed1: &mut Federation, fed2: &Federation) {
    trace!();
    //let mut result = fed1.clone();
    unsafe {
        UDBM::fed_subtraction(&mut fed1.raw, &fed2.raw);
    }
    //May want to only do this optionally/not expensively?
    rs_fed_reduce(fed1, true);
}

pub fn rs_fed_is_valid(fed: &Federation) -> bool {
    trace!();
    unsafe { !UDBM::fed_is_empty(&fed.raw) }
}

pub fn rs_fed_is_empty(fed: &Federation) -> bool {
    trace!();
    unsafe { UDBM::fed_is_empty(&fed.raw) }
}

pub fn rs_fed_up(fed: &mut Federation) {
    trace!();
    unsafe {
        UDBM::fed_up(&mut fed.raw);
    };
}

pub fn rs_fed_down(fed: &mut Federation) {
    trace!();
    unsafe {
        UDBM::fed_down(&mut fed.raw);
    };
}

pub fn rs_fed_init(fed: &mut Federation) {
    trace!();
    unsafe {
        UDBM::fed_init(&mut fed.raw);
    };
}

pub fn rs_fed_zero(fed: &mut Federation) {
    trace!();
    unsafe {
        UDBM::fed_zero(&mut fed.raw);
    };
}

pub fn rs_fed_new(dim: UDBM::cindex_t) -> Federation {
    trace!();
    assert!(dim > 0);

    // Max dim from dbm\build\UDBM\src\udbm\dbm\DBMAllocator.h:32:35
    // If exceeded causes segmentation fault in c code
    assert!(dim < MAX_DIM);

    let raw = unsafe {
        UDBM::dbm_fed_t::new(dim)
        //UDBM::fed_new(dim)
        /*let mut ptr = ::std::mem::MaybeUninit::uninit();
        UDBM::fed_new(ptr.as_mut_ptr(), dim);
        ptr.assume_init()*/
    };
    Federation { raw }
}

/// Contrain Federation with one constraint.
/// * Federation must be non empty
/// * dim > 1 induced by i < dim & j < dim & i != j
/// * as a consequence: i>=0 & j>=0 & i!=j => (i or j) > 0 and dim > (i or j) > 0 => dim > 1
///
/// # Arguments
///
/// * `fed` - The Federation
/// * `var_index_i` - The index of the variable representing the ith element
/// * `var_index_j` - The index of the variable representing the jth element
/// * `bound` - The value which bounds the expression
/// * `isStrict` - Whether the inequality is strict (<) or not (<=)
///
/// # Return
/// Bool indicating if the constraint was applied sucessfully.
///
pub fn rs_fed_constrain(
    fed: &mut Federation,
    var_index_i: u32,
    var_index_j: u32,
    bound: i32,
    isStrict: bool,
) -> bool {
    unsafe { UDBM::fed_constrain(&mut fed.raw, var_index_i, var_index_j, bound, isStrict) }
}

/// Contrain Federation with one <= constraint based on the bound.
/// * Federation must be non empty
/// * dim > 1 induced by i < dim & j < dim & i != j
/// * as a consequence: i>=0 & j>=0 & i!=j => (i or j) > 0 and dim > (i or j) > 0 => dim > 1
///
/// # Arguments
///
/// * `fed` - The Federation
/// * `var_index_i` - The index of the variable representing the ith element
/// * `var_index_j` - The index of the variable representing the jth element
/// * `bound` - The value which bounds the expression
///
/// # Return
/// Bool indicating if the constraint was applied sucessfully.
///
pub fn rs_fed_add_LTE_constraint(
    fed: &mut Federation,
    var_index_i: u32,
    var_index_j: u32,
    bound: i32,
) -> bool {
    trace!();
    rs_fed_constrain(fed, var_index_i, var_index_j, bound, false)
}

/// Contrain Federation with one < constraint based on the bound.
/// * Federation must be non empty
/// * dim > 1 induced by i < dim & j < dim & i != j
/// * as a consequence: i>=0 & j>=0 & i!=j => (i or j) > 0 and dim > (i or j) > 0 => dim > 1
///
/// # Arguments
///
/// * `fed` - The Federation
/// * `var_index_i` - The index of the variable representing the ith element
/// * `var_index_j` - The index of the variable representing the jth element
/// * `bound` - The value which bounds the expression
///
/// # Return
/// Bool indicating if the constraint was applied sucessfully.
///
pub fn rs_fed_add_LT_constraint(
    fed: &mut Federation,
    var_index_i: u32,
    var_index_j: u32,
    bound: i32,
) -> bool {
    trace!();
    rs_fed_constrain(fed, var_index_i, var_index_j, bound, true)
}

/// Contrain Federation such that clock[var_index_i] == clock[var_index_j]
/// * Federation must be non empty
/// * dim > 1 induced by i < dim & j < dim & i != j
/// * as a consequence: i>=0 & j>=0 & i!=j => (i or j) > 0 and dim > (i or j) > 0 => dim > 1
///
/// # Arguments
///
/// * `fed` - The Federation
/// * `var_index_i` - The index of the variable representing the ith element
/// * `var_index_j` - The index of the variable representing the jth element
///
/// # Return
/// Bool indicating if the constraint was applied sucessfully.
///
pub fn rs_fed_add_EQ_constraint(fed: &mut Federation, var_index_i: u32, var_index_j: u32) -> bool {
    trace!();
    let constraint = unsafe { UDBM::dbm_boundbool2raw_wrapper(0, false) };
    let res1 = rs_fed_constrain(fed, var_index_i, var_index_j, 0, false);
    let res2 = rs_fed_constrain(fed, var_index_j, var_index_i, 0, false);
    res1 && res2
}

/// Contrain Federation such that clock[var_index_i] == bound
/// * Federation must be non empty
/// * dim > 1 induced by i < dim & j < dim & i != j
/// * as a consequence: i>=0 & j>=0 & i!=j => (i or j) > 0 and dim > (i or j) > 0 => dim > 1
///
/// # Arguments
///
/// * `fed` - The Federation
/// * `var_index` - The index of the variable representing the ith element
/// * `bound` - The constant bound the clock is set equal to
///
/// # Return
/// Bool indicating if the constraint was applied sucessfully.
///
pub fn rs_fed_add_EQ_const_constraint(fed: &mut Federation, var_index: u32, bound: i32) -> bool {
    trace!();
    let res1 = rs_fed_constrain(fed, var_index, 0, bound, false);
    let res2 = rs_fed_constrain(fed, 0, var_index, -bound, false);
    res1 && res2
}

pub fn rs_fed_update_clock(fed: &mut Federation, x: UDBM::cindex_t, v: i32) {
    trace!();
    rs_fed_update(fed, x, 0, v);
}

pub fn rs_fed_update(fed: &mut Federation, x: UDBM::cindex_t, y: UDBM::cindex_t, v: i32) {
    trace!();
    unsafe {
        UDBM::fed_update(&mut fed.raw, x, y, v);
    }
}

pub fn rs_fed_free_clock(fed: &mut Federation, x: UDBM::cindex_t) {
    trace!();
    unsafe {
        UDBM::fed_free_clock(&mut fed.raw, x);
    }
}

pub fn rs_fed_subset_eq(fed1: &Federation, fed2: &Federation) -> bool {
    trace!();
    unsafe { UDBM::fed_subset_eq(&fed1.raw, &fed2.raw) }
}

pub fn rs_fed_intersects(fed1: &Federation, fed2: &Federation) -> bool {
    trace!();
    unsafe { UDBM::fed_intersects(&fed1.raw, &fed2.raw) }
}

pub fn rs_fed_relation(fed1: &Federation, fed2: &Federation) -> Relation {
    trace!();
    let rel: UDBM::relation_t = unsafe { UDBM::fed_exact_relation(&fed1.raw, &fed2.raw) };

    match rel {
        0 => Relation::Different,
        1 => Relation::Superset,
        2 => Relation::Subset,
        3 => Relation::Equal,
        _ => panic!("Unknown relation {}", rel),
    }
}

pub fn rs_fed_equals(fed1: &Federation, fed2: &Federation) -> bool {
    trace!();
    unsafe { UDBM::fed_exact_eq(&fed1.raw, &fed2.raw) }
}

pub fn rs_fed_reduce(fed: &mut Federation, expensive: bool) {
    trace!();
    unsafe {
        if expensive {
            UDBM::fed_reduce(&mut fed.raw);
        } else {
            UDBM::fed_expensive_reduce(&mut fed.raw);
        }
    }
}

pub fn rs_fed_can_delay_indef(fed: &Federation) -> bool {
    trace!();
    unsafe { UDBM::fed_can_delay_indef(&fed.raw) }
}

pub fn rs_fed_extrapolate_max_bounds(fed: &mut Federation, bounds: &MaxBounds) {
    trace!();
    assert_eq!(fed.get_dimensions(), bounds.get_dimensions());
    unsafe {
        UDBM::fed_extrapolate_max_bounds(&mut fed.raw, bounds.clock_bounds.as_ptr());
    }
}

pub fn rs_fed_add_fed(fed: &mut Federation, other: &Federation) {
    trace!();
    assert_eq!(fed.get_dimensions(), other.get_dimensions());
    unsafe {
        UDBM::fed_add_fed(&mut fed.raw, &other.raw);
    }
}

pub fn rs_fed_invert(fed: &mut Federation) {
    trace!();
    unsafe {
        UDBM::fed_invert(&mut fed.raw);
    }
}

pub fn rs_fed_size(fed: &Federation) -> usize {
    trace!();
    unsafe { UDBM::fed_size(&fed.raw) }.try_into().unwrap()
}

pub unsafe fn rs_fed_destruct(fed: &mut Federation) {
    //fed.raw.nil();
    //fed.raw.destruct();
    UDBM::fed_destruct(&mut fed.raw);
}

pub fn rs_fed_copy(fed: &Federation) -> Federation {
    trace!();
    let raw = unsafe {
        UDBM::dbm_fed_t::new1(&fed.raw)
        //UDBM::fed_copy(&fed.raw)
    };
    Federation { raw }
}

pub fn rs_fed_dimensions(fed: &Federation) -> UDBM::cindex_t {
    trace!();
    unsafe { UDBM::fed_dimension(&fed.raw) }
}

/*
pub fn rs_crash(dim: u32) {
    unsafe {
        UDBM::fed_crash(dim);
    }
}*/

/* END FEDERATION METHODS */
