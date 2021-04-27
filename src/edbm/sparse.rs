use crate::edbm::dense::DenseDbm;
use crate::edbm::{DimensionIndex, DimensionValue};

pub struct Bound {
    clk1: DimensionIndex,
    clk2: DimensionIndex,
    val: DimensionValue,
}

pub struct SparseDbm {
    pub dimension: DimensionIndex,
    bounds: Vec<Bound>,
}

impl From<DenseDbm> for SparseDbm {
    fn from(_: DenseDbm) -> Self {
        todo!("convert into DenseDBM that is in canonical form")
    }
}
