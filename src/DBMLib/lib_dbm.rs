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
use std::convert::TryInto;

mod UDBM {
    #![allow(non_snake_case)]
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(dead_code)]
    #![allow(improper_ctypes)]
    #![allow(deref_nullptr)]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

use lazy_static::lazy_static; // 1.4.0
use std::sync::{Mutex, RwLock};
const DEBUG: bool = false;

/// Max dim from dbm\build\UDBM\src\udbm\dbm\DBMAllocator.h:32:35
/// If exceeded causes segmentation fault in c code
const MAX_DIM: u32 = 256;

lazy_static! {
    // static ref LIBRARY_LOCK: RwLock<()> = RwLock::new(());
    static ref LIBRARY_LOCK: Mutex<i32> = Mutex::new(0);
}

/*
unsafe fn syncr<T>(f: &mut dyn Fn() -> T) -> T {
    let _l = LIBRARY_LOCK.read().unwrap();
    f()
}*/

unsafe fn syncw<T>(f: &mut dyn FnMut() -> T) -> T {
    let _l = LIBRARY_LOCK.lock().unwrap();
    f()
}

#[cfg(feature = "single-threaded")]
macro_rules! sync {
    ($arg:expr) => {
        $arg
    };
}

#[cfg(not(feature = "single-threaded"))]
macro_rules! sync {
    ($arg:expr) => {
        syncw(&mut || $arg)
    };
}

pub const DBM_INF: i32 = i32::MAX - 1;

pub type FedRaw = UDBM::dbm_fed_t;

/// used in input enabler to check if the constraint is strictly bound e.g strictly less than
pub fn rs_raw_is_strict(raw: UDBM::raw_t) -> bool {
    unsafe { UDBM::dbm_rawIsStrict_wrapper(raw) }
}

///converts the bound from c++ to an usable rust type - used when input enabling
pub fn rs_raw_to_bound(raw: UDBM::raw_t) -> i32 {
    unsafe { UDBM::dbm_raw2bound_wrapper(raw) }
}

fn rs_bound_strict_to_raw(bound: i32, is_strict: bool) -> UDBM::raw_t {
    unsafe { UDBM::dbm_boundbool2raw_wrapper(bound, is_strict) }
}

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
        sync!(UDBM::fed_get_dbm_vec(
            &fed.raw,
            out.as_mut_ptr(),
            len as u64
        ));
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
        sync!(UDBM::fed_intersection(&mut fed1.raw, &fed2.raw));
    }
    //rs_fed_reduce(&mut fed1, true);
}

pub fn rs_fed_predt(good: &mut Federation, bad: &Federation) {
    trace!();
    unsafe {
        sync!(UDBM::fed_predt(&mut good.raw, &bad.raw));
    }
}

///  oda federation minus federation
pub fn rs_fed_subtract(fed1: &mut Federation, fed2: &Federation) {
    trace!();
    //let mut result = fed1.clone();
    unsafe {
        sync!(UDBM::fed_subtraction(&mut fed1.raw, &fed2.raw));
    }
    //May want to only do this optionally/not expensively?
    rs_fed_reduce(fed1, true);
}

pub fn rs_fed_is_valid(fed: &Federation) -> bool {
    trace!();
    unsafe { !sync!(UDBM::fed_is_empty(&fed.raw)) }
}

pub fn rs_fed_is_empty(fed: &Federation) -> bool {
    trace!();
    unsafe { sync!(UDBM::fed_is_empty(&fed.raw)) }
}

pub fn rs_fed_up(fed: &mut Federation) {
    trace!();
    unsafe {
        sync!(UDBM::fed_up(&mut fed.raw));
    };
}

pub fn rs_fed_down(fed: &mut Federation) {
    trace!();
    unsafe {
        sync!(UDBM::fed_down(&mut fed.raw));
    };
}

pub fn rs_fed_init(fed: &mut Federation) {
    trace!();
    unsafe {
        sync!(UDBM::fed_init(&mut fed.raw));
    };
}

pub fn rs_fed_zero(fed: &mut Federation) {
    trace!();
    unsafe {
        sync!(UDBM::fed_zero(&mut fed.raw));
    };
}

pub fn rs_fed_new(dim: UDBM::cindex_t) -> Federation {
    trace!();
    assert!(dim > 0);
    // Max dim from dbm\build\UDBM\src\udbm\dbm\DBMAllocator.h:32:35
    // If exceeded causes segmentation fault in c code
    assert!(dim < MAX_DIM);
    let raw = unsafe { sync!(UDBM::dbm_fed_t::new(dim)) };
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
    unsafe {
        sync!(UDBM::fed_constrain(
            &mut fed.raw,
            var_index_i,
            var_index_j,
            bound,
            isStrict
        ))
    }
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
    let constraint = rs_bound_strict_to_raw(0, false);
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
        sync!(UDBM::fed_update(&mut fed.raw, x, y, v));
    }
}

pub fn rs_fed_free_clock(fed: &mut Federation, x: UDBM::cindex_t) {
    trace!();
    unsafe {
        sync!(UDBM::fed_free_clock(&mut fed.raw, x));
    }
}

pub fn rs_fed_subset_eq(fed1: &Federation, fed2: &Federation) -> bool {
    trace!();
    unsafe { sync!(UDBM::fed_subset_eq(&fed1.raw, &fed2.raw)) }
}

pub fn rs_fed_intersects(fed1: &Federation, fed2: &Federation) -> bool {
    trace!();
    unsafe { sync!(UDBM::fed_intersects(&fed1.raw, &fed2.raw)) }
}

pub fn rs_fed_relation(fed1: &Federation, fed2: &Federation) -> Relation {
    trace!();
    let rel: UDBM::relation_t = unsafe { sync!(UDBM::fed_exact_relation(&fed1.raw, &fed2.raw)) };

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
    unsafe { sync!(UDBM::fed_exact_eq(&fed1.raw, &fed2.raw)) }
}

pub fn rs_fed_reduce(fed: &mut Federation, expensive: bool) {
    trace!();
    unsafe {
        if expensive {
            sync!(UDBM::fed_reduce(&mut fed.raw));
        } else {
            sync!(UDBM::fed_expensive_reduce(&mut fed.raw));
        }
    }
}

pub fn rs_fed_can_delay_indef(fed: &Federation) -> bool {
    trace!();
    unsafe { sync!(UDBM::fed_can_delay_indef(&fed.raw)) }
}

pub fn rs_fed_extrapolate_max_bounds(fed: &mut Federation, bounds: &MaxBounds) {
    trace!();
    //assert_eq!(fed.get_dimensions(), bounds.get_dimensions());
    unsafe {
        sync!(UDBM::fed_extrapolate_max_bounds(
            &mut fed.raw,
            bounds.clock_bounds.as_ptr()
        ));
    }
}

pub fn rs_fed_diagonal_extrapolate_max_bounds(fed: &mut Federation, bounds: &MaxBounds) {
    trace!();
    assert_eq!(fed.get_dimensions(), bounds.get_dimensions());
    unsafe {
        sync!(UDBM::fed_diagonal_extrapolate_max_bounds(
            &mut fed.raw,
            bounds.clock_bounds.as_ptr()
        ));
    }
}

pub fn rs_fed_add_fed(fed: &mut Federation, other: &Federation) {
    trace!();
    assert_eq!(fed.get_dimensions(), other.get_dimensions());
    unsafe {
        sync!(UDBM::fed_add_fed(&mut fed.raw, &other.raw));
    }
}

pub fn rs_fed_invert(fed: &mut Federation) {
    trace!();
    unsafe {
        sync!(UDBM::fed_invert(&mut fed.raw));
    }
}

pub fn rs_fed_size(fed: &Federation) -> usize {
    trace!();
    unsafe { sync!(UDBM::fed_size(&fed.raw)) }
        .try_into()
        .unwrap()
}

pub unsafe fn rs_fed_destruct(fed: &mut Federation) {
    sync!(UDBM::fed_destruct(&mut fed.raw));
}

pub fn rs_fed_copy(fed: &Federation) -> Federation {
    trace!();
    let raw = unsafe { sync!(UDBM::dbm_fed_t::new1(&fed.raw)) };
    Federation { raw }
}

pub fn rs_fed_dimensions(fed: &Federation) -> UDBM::cindex_t {
    trace!();
    unsafe { sync!(UDBM::fed_dimension(&fed.raw)) }
}

/*
pub fn rs_crash(dim: u32) {
    unsafe {
        sync!( UDBM::fed_crash(dim));
    }
}*/

/* END FEDERATION METHODS */
