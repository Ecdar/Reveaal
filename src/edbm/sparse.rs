use crate::edbm::dense::DenseDBM;
use crate::edbm::{DimensionIndex, DimensionValue};

pub struct Bound {
    clk1: DimensionIndex,
    clk2: DimensionIndex,
    val: DimensionValue,
}

pub struct SparseDBM {
    pub dimension: DimensionIndex,
    bounds: Vec<Bound>,
}

impl From<DenseDBM> for SparseDBM {
    fn from(_: DenseDBM) -> Self {
        todo!("convert into DenseDBM that is in canonical form")
    }
}
