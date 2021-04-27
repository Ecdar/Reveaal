pub mod dense;
pub mod sparse;

type DimensionIndex = u32;
type DimensionValue = i32;

/// Used to represent unconstrained relation between clocks
pub const DIM_INF: DimensionValue = i32::MAX;

#[derive(Debug, Eq, PartialEq)]
enum RelationOps {
    /// Strictly less than
    Less,
    /// Less than, or equals
    LessEq,
}

trait Dbm {
    /* Property-Checking */

    fn consistent(&self) -> bool;

    // Relation is defined in the trait `DBMRelationOp`

    fn satisfied(&self, clk1: DimensionIndex, clk2: DimensionIndex, val: DimensionValue);

    /* Transformations */

    fn up(&mut self);

    fn down(&mut self);

    fn free(&mut self, clk: DimensionIndex);

    fn reset(&mut self, clk: DimensionIndex, val: DimensionValue);

    fn copy(&mut self, clk1: DimensionIndex, clk2: DimensionIndex);

    fn and(&mut self, clk: DimensionIndex, val: DimensionValue);

    fn shift(&mut self, clk: DimensionIndex, offset: DimensionValue);

    fn make_canonical(&mut self);

    /* Normalisation */

    fn norm(&mut self /*TODO arguments*/);
}

trait DbmRelationOp<O> {
    fn relation(&self, other: O) -> bool;
}
