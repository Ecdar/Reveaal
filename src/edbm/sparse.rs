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

impl Into<DenseDBM> for SparseDBM {
    fn into(self) -> DenseDBM {
        todo!("convert into DenseDBM that is in canonical form")
    }
}
