use crate::DBMLib::lib;
use crate::ModelObjects::max_bounds::MaxBounds;
use crate::ModelObjects::representations::{build_guard_from_zone, BoolExpression};
use colored::Colorize;
use std::cmp::{Ordering, PartialOrd};
use std::collections::HashMap;
use std::convert::TryInto;
use std::fmt::{Display, Formatter};
use std::ops;
#[derive(Clone, Debug, std::cmp::PartialEq)]
pub struct Zone {
    pub(crate) dimension: u32,
    pub(in crate::DBMLib) matrix: Vec<i32>,
}

pub enum Relation {
    Different,
    Superset,
    Subset,
    Equal,
}

impl Zone {
    pub fn get_constraint(&self, var_index_i: u32, var_index_j: u32) -> (bool, i32) {
        let index: usize = (var_index_i * self.dimension + var_index_j)
            .try_into()
            .unwrap();
        let raw = self.matrix[index];

        (lib::rs_raw_is_strict(raw), lib::rs_raw_to_bound(raw))
    }

    pub fn is_constraint_infinity(&self, var_index_i: u32, var_index_j: u32) -> bool {
        let index: usize = (var_index_i * self.dimension + var_index_j)
            .try_into()
            .unwrap();
        let raw = self.matrix[index];
        raw == lib::DBM_INF
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

#[derive(Debug)]
pub struct Federation {
    pub(in crate::DBMLib) raw: lib::fed_raw,
}

impl Federation {
    /// Get a new federation of a given dimension with the constraints that all clocks are equal to 0
    pub fn zero(dimension: u32) -> Self {
        let mut fed = lib::rs_fed_new(dimension);
        lib::rs_fed_zero(&mut fed);
        fed
    }

    /// Get a new empty federation of a given dimension, representing no possible clock valuations
    pub fn empty(dimension: u32) -> Self {
        lib::rs_fed_new(dimension)
    }

    /// Get a new federation of a given dimension without any constraints, representing all possible clock valuation
    pub fn full(dimension: u32) -> Self {
        let mut fed = lib::rs_fed_new(dimension);
        lib::rs_fed_init(&mut fed);
        fed
    }

    // Get a new federation of a given dimension with the constraints that all clocks are equal (zero(); up())
    pub fn init(dimension: u32) -> Self {
        let mut fed = Federation::zero(dimension);
        fed.up();
        fed
    }

    /// Get a federation containing the subtraction of the federations
    ///
    /// `self` is unchanged, for change of `self` see `subtract`
    /// Can be called with operator `-` consuming both operands
    ///
    /// [`subtract`]: Federation::subtract
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// let fed1 = Federation::zero();
    /// let fed2 = Federation::init();
    ///
    /// let fed3 = fed1.subtraction(&fed2);
    /// // fed1 and fed2 remain unchanged
    ///
    /// assert_eq!(Federation::empty(), fed3);
    /// ```
    pub fn subtraction(&self, other: &Self) -> Federation {
        let mut result = self.clone();
        lib::rs_fed_subtract(&mut result, &other);
        result
    }

    /// Update the federation to contain the subtraction of the federations
    ///
    /// `self` is changed, for unchanged `self` see  [`subtraction`].
    /// Can be called with operator `-=` consuming the other federation
    ///
    /// [`subtraction`]: Federation::subtraction
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// let mut fed1 = Federation::zero();
    /// let fed2 = Federation::init();
    ///
    /// fed1.subtract(&fed2);
    /// // fed1 is changed and fed2 remains unchanged
    ///
    /// assert_eq!(Federation::empty(), fed1);
    /// ```
    pub fn subtract(&mut self, other: &Self) {
        lib::rs_fed_subtract(self, &other);
    }

    pub fn is_subset_eq(&self, other: &Self) -> bool {
        lib::rs_fed_subset_eq(self, other)
    }

    /// Get a federation containing the intersection of the federations
    ///
    /// `self` is unchanged, for change of `self` see `intersect`
    /// Can be called with operator `-` consuming both operands
    ///
    /// [`intersect`]: Federation::intersect
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// let fed1 = Federation::init();
    /// let fed2 = Federation::zero();
    ///
    /// let fed3 = fed1.intersection(&fed2);
    /// // fed1 and fed2 remain unchanged
    ///
    /// assert_eq!(Federation::zero(), fed3);
    /// ```
    pub fn intersection(&self, other: &Self) -> Federation {
        let mut result = self.clone();
        lib::rs_fed_intersect(&mut result, &other);
        result
    }

    /// Update the federation to contain the intersection of the federations
    ///
    /// `self` is changed, for unchanged `self` see  [`intersection`].
    /// Can be called with operator `&=` consuming the other federation
    ///
    /// [`intersection`]: Federation::intersection
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// let mut fed1 = Federation::init();
    /// let fed2 = Federation::zero();
    ///
    /// fed1.intersect(&fed2);
    /// // fed1 is changed and fed2 remains unchanged
    ///
    /// assert_eq!(Federation::zero(), fed1);
    /// ```
    pub fn intersect(&mut self, other: &Self) {
        //assert_eq!(self.dimension, other.dimension);
        lib::rs_fed_intersect(self, other);
    }

    /// Returns whether the intersection of the federations is non-empty
    pub fn intersects(&self, other: &Self) -> bool {
        lib::rs_fed_intersects(self, other)
    }

    /// Update the federation to contain its inverse
    ///
    /// `self` is changed, for unchanged `self` see  [`inverse`].
    /// Can be called with operator `!` consuming self
    ///
    /// [`inverse`]: Federation::inverse
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// let mut fed1 = Federation::init();
    /// fed1.invert();
    /// fed1 is changed to contain its inverse
    ///
    /// assert_eq!(Federation::empty(), fed1);
    /// ```
    pub fn invert(&mut self) {
        lib::rs_fed_invert(self)
    }

    /// Get a federation containing the inverse of the federation
    ///
    /// `self` is unchanged, for change of `self` see [`invert`].
    /// Can be called with operator `!` consuming self
    ///
    /// [`invert`]: Federation::invert
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// let fed1 = Federation::init();
    /// let fed2 = fed1.inverse();
    /// fed1 remains unchanged
    ///
    /// assert_eq!(Federation::empty(), fed2);
    /// ```
    pub fn inverse(&self) -> Federation {
        let mut result = self.clone();
        lib::rs_fed_invert(&mut result);
        result
    }

    /// Updates the federation to contain the (non-convex) union of the federations
    ///
    /// `self` is changed, for unchanged `self` see `with_added_fed`
    /// Can be called with operator `+=` consuming the other federation
    ///
    /// [`with_added_fed`]: Federation::with_added_fed
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// let mut fed1 = Federation::zero();
    /// let fed2 = !Federation::zero();
    ///
    /// fed1.add_fed(&fed2);
    /// // fed1 is changed and fed2 remains unchanged
    ///
    /// assert_eq!(Federation::init(), fed1);
    /// ```
    pub fn add_fed(&mut self, fed: &Federation) {
        lib::rs_fed_add_fed(self, fed);
    }

    /// Get a federation containing the (non-convex) union of the federations
    ///
    /// `self` is unchanged, for change of `self` see `add_fed`
    /// Can be called with operator `+` consuming both operands
    ///
    /// [`add_fed`]: Federation::add_fed
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// let fed1 = Federation::zero();
    /// let fed2 = !Federation::zero();
    ///
    /// let fed3 = fed1.with_added_fed(&fed2);
    /// // fed1 and fed2 remain unchanged
    /// assert_eq!(Federation::init(), fed3);
    /// ```
    pub fn with_added_fed(&self, fed: &Federation) -> Federation {
        let mut result = self.clone();
        lib::rs_fed_add_fed(&mut result, fed);
        result
    }

    /// Checks whether the federation is empty
    ///
    /// Return `true` if all contained DBMs are empty, or there are no DBMs
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// assert!(Federation::empty().is_empty());
    /// assert!(!Federation::zero().is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        lib::rs_fed_is_empty(self)
    }

    /*
    fn iter_zones(&self) -> impl Iterator<Item = Zone> + '_ {
        self.get_zones().into_iter()
    }*/

    pub fn get_zones(&self) -> Vec<Zone> {
        let zones = lib::rs_fed_get_zones(self);
        assert_eq!(zones.len(), self.num_zones());
        zones
    }

    /// Get the number of zones in the federation
    pub fn num_zones(&self) -> usize {
        lib::rs_fed_size(self)
    }

    /// Get the dimension of the federation
    pub fn get_dimensions(&self) -> u32 {
        lib::rs_fed_dimensions(self)
    }

    pub fn reduce(&mut self, expensive: bool) {
        lib::rs_fed_reduce(self, expensive);
    }

    /// Get the constraints represented by the federation as a boolean expression
    pub fn as_boolexpression(
        &self,
        clocks: Option<&HashMap<String, u32>>,
    ) -> Option<BoolExpression> {
        let mut clone = self.clone();
        clone.reduce(true);
        let zones = clone.get_zones();
        let mut guard = BoolExpression::Bool(false);

        for zone in zones {
            let zone_guard = build_guard_from_zone(&zone, clocks);
            let g = match zone_guard {
                Some(g) => g,
                None => BoolExpression::Bool(true),
            };
            guard = BoolExpression::OrOp(Box::new(guard), Box::new(g));
        }
        guard.simplify();
        Some(guard)
    }

    /// Check whether the federation is valid (non-empty)
    pub fn is_valid(&self) -> bool {
        lib::rs_fed_is_valid(self)
    }

    /// Perform the 'up' operation on all DBMs in the federation
    pub fn up(&mut self) {
        lib::rs_fed_up(self);
    }

    /// Perform the 'down' operation on all DBMs in the federation
    pub fn down(&mut self) {
        lib::rs_fed_down(self);
    }

    /// Extrapolate max bounds on all DBMs in the federation
    pub fn extrapolate_max_bounds(&mut self, max_bounds: &MaxBounds) {
        lib::rs_fed_extrapolate_max_bounds(self, max_bounds);
    }

    /// Check whether the any DBM in the federation can delay indefinitely
    pub fn canDelayIndefinitely(&self) -> bool {
        lib::rs_fed_can_delay_indef(self)
    }

    /// Sets the clock (clocks[clock_index]) to an integer value in all DBMs in the federation
    pub fn update(&mut self, clock_index: u32, value: i32) {
        lib::rs_fed_update_clock(self, clock_index, value);
    }

    /// Removes the bounds on the clock (clocks[clock_index]) in all DBMs in the federation
    pub fn free_clock(&mut self, clock_index: u32) {
        lib::rs_fed_free_clock(self, clock_index);
    }

    pub fn constrain(
        &mut self,
        var_index_i: u32,
        var_index_j: u32,
        bound: i32,
        isStrict: bool,
    ) -> bool {
        lib::rs_fed_constrain(self, var_index_i, var_index_j, bound, isStrict)
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

    pub fn constrain_var_to_val(&mut self, var_index: u32, value: i32) -> bool {
        unimplemented!()
    }
}

impl ops::Add for Federation {
    type Output = Self;

    /// Get a federation containing the (non-convex) union of the federations
    fn add(mut self, other: Self) -> Self {
        self.add_fed(&other);
        self
    }
}

impl ops::Add for &Federation {
    type Output = Federation;

    /// Get a federation containing the (non-convex) union of the federations
    fn add(self, other: &Federation) -> Federation {
        self.with_added_fed(other)
    }
}

impl ops::Sub for Federation {
    type Output = Self;

    /// Get a federation containing the subtraction of the federations
    fn sub(mut self, other: Self) -> Self {
        self.subtract(&other);
        self
    }
}

impl ops::Sub for &Federation {
    type Output = Federation;

    /// Get a federation containing the subtraction of the federations
    fn sub(self, other: &Federation) -> Federation {
        self.subtraction(other)
    }
}

impl ops::BitAnd for Federation {
    type Output = Self;

    /// Get a federation containing the intersection of the federations
    fn bitand(mut self, other: Self) -> Self {
        self.intersect(&other);
        self
    }
}

impl ops::BitAnd for &Federation {
    type Output = Federation;

    /// Get a federation containing the intersection of the federations
    fn bitand(self, other: &Federation) -> Federation {
        self.intersection(other)
    }
}

impl ops::AddAssign for Federation {
    fn add_assign(&mut self, other: Federation) {
        self.add_fed(&other);
    }
}

impl ops::SubAssign for Federation {
    fn sub_assign(&mut self, other: Federation) {
        self.subtract(&other);
    }
}

impl ops::BitAndAssign for Federation {
    fn bitand_assign(&mut self, other: Self) {
        self.intersect(&other);
    }
}

impl PartialOrd for Federation {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match lib::rs_fed_relation(self, other) {
            Relation::Superset => Some(Ordering::Greater),
            Relation::Subset => Some(Ordering::Less),
            Relation::Equal => Some(Ordering::Equal),
            Relation::Different => None,
        }
    }
}

impl ops::Not for Federation {
    type Output = Self;

    /// Get a federation containing the inverse of the federation
    fn not(mut self) -> Self {
        self.invert();
        self
    }
}

impl PartialEq for Federation {
    fn eq(&self, other: &Self) -> bool {
        lib::rs_fed_equals(self, other)
    }
}

impl Drop for Federation {
    fn drop(&mut self) {
        unsafe {
            lib::rs_fed_destruct(self);
        }
    }
}

impl Clone for Federation {
    fn clone(&self) -> Federation {
        lib::rs_fed_copy(self)
    }
}

impl Display for Federation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        //return Ok(());
        write!(
            f,
            "Federation{{{}}}",
            self.as_boolexpression(None)
                .unwrap_or(BoolExpression::Bool(true))
        )?;
        /*      for zone in self.get_zones() {
                    write!(f, "\n{}", zone)?;
                }
                write!(f, "}}")?;
        */
        Ok(())
    }
}
