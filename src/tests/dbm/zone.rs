#[cfg(test)]
mod test {
    use crate::DBMLib::dbm::{Federation, Zone};
    use crate::DBMLib::lib;

    #[test]
    fn testZoneValid1() {
        assert!(Zone::from(vec![1, 1, lib::DBM_INF, 1]).is_valid());
    }

    #[test]
    fn testZoneValid2() {
        assert!(Zone::from(vec![1, 1, 1, 1]).is_valid());
    }

    #[test]
    fn testZoneValid3() {
        assert!(Zone::from(vec![1, -3, 11, 1]).is_valid());
    }

    #[test]
    fn testZoneInit() {
        Zone::init(10);
    }

    #[test]
    fn testZoneNew() {
        Zone::new(10);
    }

    #[test]
    fn testZoneGetConstraint1() {
        let mut zone = Zone::from(vec![1, -3, 11, 1]).clone();
        assert_eq!(zone.get_constraint(0, 0), (false, 0));
        assert_eq!(zone.get_constraint(0, 1), (false, -2));
        assert_eq!(zone.get_constraint(1, 0), (false, 5));
        assert_eq!(zone.get_constraint(1, 1), (false, 0));
    }

    #[test]
    fn testZoneUp() {
        let mut zone = Zone::init(10);
        //zone.zero();
        zone.up();
        for i in 1..10 {
            for j in 0..10 {
                // not the diagonal
                if i != j {
                    assert!(zone.is_constraint_infinity(i, j));
                }
            }
        }
    }

    #[ignore]
    #[test]
    fn testDbmMinusDbm() {
        let mut zone1 = Zone::init(10);
        zone1.up();
        let mut zone2 = Zone::init(10);
        zone2.up();
        zone2.add_lte_constraint(1, 0, 10);

        zone1.dbm_minus_dbm(&mut zone2);
    }
}
