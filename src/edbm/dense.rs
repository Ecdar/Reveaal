use crate::edbm::sparse::SparseDbm;
use crate::edbm::{Dbm, DbmRelationOp, DimensionIndex, DimensionValue, RelationOps};
use std::fmt::{Display, Formatter, Write};

pub struct DenseDbm {
    pub dimension: u32,
    // Column-row order, index = col * dim + row
    // Least signification bit denotes `RelationOps`. O == Less, 1 == LessEq
    // Relation: row - col { n
    matrix: Vec<i32>,
}

impl DenseDbm {
    fn get_n(&self, row: DimensionIndex, col: DimensionIndex) -> DimensionValue {
        self.matrix[self.get_index(row, col)] / 2
    }

    fn get_rel(&self, row: DimensionIndex, col: DimensionIndex) -> RelationOps {
        let index = self.get_index(row, col);
        if index > (self.dimension * self.dimension - 1) as usize {
            println!("{}:{}", row, col);
        }
        let rel_bits = (self.matrix[index] % 2).abs();
        match rel_bits {
            0 => RelationOps::Less,
            1 => RelationOps::LessEq,
            _ => unreachable!(),
        }
    }

    #[inline]
    fn get_index(&self, row: DimensionIndex, col: DimensionIndex) -> usize {
        (col * self.dimension + row) as usize
    }

    fn new(dimension: u32, matrix: Vec<(i32, RelationOps)>) -> Self {
        let matrix = matrix
            .iter()
            .map(|(val, rel)| {
                val * 2
                    + match rel {
                        RelationOps::Less => 0,
                        RelationOps::LessEq => 1,
                    }
            })
            .collect();
        Self { dimension, matrix }
    }
}

impl Dbm for DenseDbm {
    fn consistent(&self) -> bool {
        todo!()
    }

    fn satisfied(&self, clk1: DimensionIndex, clk2: DimensionIndex, val: DimensionValue) {
        todo!()
    }

    fn up(&mut self) {
        todo!()
    }

    fn down(&mut self) {
        todo!()
    }

    fn free(&mut self, clk: DimensionIndex) {
        todo!()
    }

    fn reset(&mut self, clk: DimensionIndex, val: DimensionValue) {
        todo!()
    }

    fn copy(&mut self, clk1: DimensionIndex, clk2: DimensionIndex) {
        todo!()
    }

    fn and(&mut self, clk: DimensionIndex, val: DimensionValue) {
        todo!()
    }

    fn shift(&mut self, clk: DimensionIndex, offset: DimensionValue) {
        todo!()
    }

    fn make_canonical(&mut self) {
        // Based on the Floyd-Warshall algorithm as described in "DBM: Structures, Operations and Implementation" by Johan Bengtsson
        for k in 0..self.dimension {
            for col in 0..self.dimension {
                for row in 0..self.dimension {
                    // impl min(direct, step1 + step2) but with consideration of the relation operators

                    let direct = match self.get_rel(row, col) {
                        RelationOps::Less => self.get_n(row, col),
                        RelationOps::LessEq => self.get_n(row, col) - 1,
                    };
                    let step1 = match self.get_rel(row, k) {
                        RelationOps::Less => self.get_n(row, col),
                        RelationOps::LessEq => self.get_n(row, col) - 1,
                    };
                    let step2 = match self.get_rel(k, col) {
                        RelationOps::Less => self.get_n(row, col),
                        RelationOps::LessEq => self.get_n(row, col) - 1,
                    };
                    let index = self.get_index(row, col);
                    self.matrix[index] = std::cmp::min(direct, step1 + step2) * 2;
                }
            }
        }
    }

    fn norm(&mut self) {
        todo!()
    }
}

impl DbmRelationOp<DenseDbm> for DenseDbm {
    fn relation(&self, other: DenseDbm) -> bool {
        todo!()
    }
}

impl From<SparseDbm> for DenseDbm {
    fn from(_: SparseDbm) -> Self {
        todo!("convert into the minimal constraint system equivalent to the DenseDBM")
    }
}

impl Display for DenseDbm {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("DenseDBM[")?;
        for col in 0..self.dimension {
            f.write_char('[')?;
            for row in 0..self.dimension {
                let n = self.get_n(row, col);
                let rel = match self.get_rel(row, col) {
                    RelationOps::Less => "<",
                    RelationOps::LessEq => "<=",
                };
                f.write_fmt(format_args!("({}, {})", n, rel))?;
                if row != self.dimension - 1 {
                    f.write_char(',')?;
                }
            }
            f.write_char(']')?;
            if col != self.dimension - 1 {
                f.write_char(',')?;
            }
        }
        f.write_char(']')?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::edbm::dense::DenseDbm;
    use crate::edbm::{Dbm, RelationOps};

    /// Checks that a zeroed DBM does changed when changed to canonical form
    #[test]
    fn canonical_1() {
        const DIMENSION: usize = 3;
        let matrix = vec![
            (0, RelationOps::LessEq),
            (0, RelationOps::LessEq),
            (0, RelationOps::LessEq),
            (0, RelationOps::LessEq),
            (0, RelationOps::LessEq),
            (0, RelationOps::LessEq),
            (0, RelationOps::LessEq),
            (0, RelationOps::LessEq),
            (0, RelationOps::LessEq),
        ];
        let mut dbm = DenseDbm::new(3, matrix);

        dbm.make_canonical();

        assert_eq!(0, dbm.get_n(0, 0));
        assert_eq!(RelationOps::Less, dbm.get_rel(0, 0));
        assert_eq!(0, dbm.get_n(0, 0));
        assert_eq!(RelationOps::Less, dbm.get_rel(1, 0));
        assert_eq!(0, dbm.get_n(0, 0));
        assert_eq!(RelationOps::Less, dbm.get_rel(2, 0));

        assert_eq!(0, dbm.get_n(0, 0));
        assert_eq!(RelationOps::Less, dbm.get_rel(0, 1));
        assert_eq!(0, dbm.get_n(0, 0));
        assert_eq!(RelationOps::Less, dbm.get_rel(1, 1));
        assert_eq!(0, dbm.get_n(0, 0));
        assert_eq!(RelationOps::Less, dbm.get_rel(2, 1));

        assert_eq!(0, dbm.get_n(0, 0));
        assert_eq!(RelationOps::Less, dbm.get_rel(0, 2));
        assert_eq!(0, dbm.get_n(0, 0));
        assert_eq!(RelationOps::Less, dbm.get_rel(1, 2));
        assert_eq!(0, dbm.get_n(0, 0));
        assert_eq!(RelationOps::Less, dbm.get_rel(2, 2));
    }

    /// Checks that bound on clock _ is tightened according to the bounds between _ and 0, and  between _ and _
    #[test]
    #[ignore]
    fn canonical_2() {
        const DIMENSION: usize = 3;
        let matrix = vec![
            (0, RelationOps::LessEq),   // 0, 0
            (0, RelationOps::LessEq),   // x, 0
            (0, RelationOps::LessEq),   // y, 0
            (20, RelationOps::Less),    // 0, x
            (0, RelationOps::LessEq),   // x, x
            (-10, RelationOps::LessEq), // y, x
            (20, RelationOps::LessEq),  // y, 0
            (10, RelationOps::LessEq),  // y, x
            (0, RelationOps::LessEq),   // y, y
        ];
        let mut dbm = DenseDbm::new(3, matrix);

        dbm.make_canonical();

        todo!("Compute the canonical form, and paste it into the test");

        assert_eq!(0, dbm.get_n(0, 0));
        assert_eq!(RelationOps::Less, dbm.get_rel(0, 0));
        assert_eq!(0, dbm.get_n(0, 0));
        assert_eq!(RelationOps::Less, dbm.get_rel(1, 0));
        assert_eq!(0, dbm.get_n(0, 0));
        assert_eq!(RelationOps::Less, dbm.get_rel(2, 0));

        assert_eq!(0, dbm.get_n(0, 0));
        assert_eq!(RelationOps::Less, dbm.get_rel(0, 1));
        assert_eq!(0, dbm.get_n(0, 0));
        assert_eq!(RelationOps::Less, dbm.get_rel(1, 1));
        assert_eq!(0, dbm.get_n(0, 0));
        assert_eq!(RelationOps::Less, dbm.get_rel(2, 1));

        assert_eq!(0, dbm.get_n(0, 0));
        assert_eq!(RelationOps::Less, dbm.get_rel(0, 2));
        assert_eq!(0, dbm.get_n(0, 0));
        assert_eq!(RelationOps::Less, dbm.get_rel(1, 2));
        assert_eq!(0, dbm.get_n(0, 0));
        assert_eq!(RelationOps::Less, dbm.get_rel(2, 2));
    }

    #[test]
    fn encode_decode() {
        const DIMENSION: usize = 2;
        let matrix = vec![
            (5, RelationOps::LessEq),   // row 0, col 0
            (-10, RelationOps::Less),   // row 1, col 0
            (40, RelationOps::Less),    // row 0, col 1
            (-23, RelationOps::LessEq), // row 0, col 1
        ];
        let mut dbm = DenseDbm::new(3, matrix);

        assert_eq!(5, dbm.get_n(0, 0));
        assert_eq!(RelationOps::LessEq, dbm.get_rel(0, 0));
        assert_eq!(-10, dbm.get_n(1, 0));
        assert_eq!(RelationOps::Less, dbm.get_rel(1, 0));

        assert_eq!(40, dbm.get_n(0, 1));
        assert_eq!(RelationOps::Less, dbm.get_rel(0, 1));
        assert_eq!(-23, dbm.get_n(1, 1));
        assert_eq!(RelationOps::LessEq, dbm.get_rel(1, 1));
    }
}
