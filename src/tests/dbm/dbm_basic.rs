#[cfg(test)]
mod test {
    use super::super::super::super::DBMLib::lib;

    #[test]
    fn testDbmValid0() {
        let mut intArr = [0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut intArr2 = [0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut _arr2 = [1, 1, 2147483646, 1];

        let dbm = &mut intArr;
        let _dbm2 = &mut intArr2;

        println!("dbm before init: {:?}", dbm);
        lib::rs_dbm_init(dbm, 3);
        println!("dbm after init: {:?}", dbm);
    }

    #[test]
    fn testDbmValid1() {
        assert!(lib::rs_dbm_is_valid([1, 1, lib::DBM_INF, 1].as_mut(), 2));
    }

    #[test]
    fn testDbmValid2() {
        assert!(lib::rs_dbm_is_valid([1, 1, 1, 1].as_mut(), 2));
    }

    #[test]
    fn testDbmValid3() {
        assert!(lib::rs_dbm_is_valid([1, -3, 11, 1].as_mut(), 2));
    }

    #[test]
    fn testDbmNotValid1() {
        assert!(!lib::rs_dbm_is_valid([0, 0, 0, 0].as_mut(), 2));
    }

    #[test]
    fn testDbmNotValid2() {
        //TODO returns true even tho should false
        //assert!(!lib::rs_dbm_is_valid([-1, 0, 0, 0].as_mut(), 2));
    }

    #[test]
    fn testRaw2Bound1() {
        assert_eq!(0, lib::rs_raw_to_bound(1));
    }

    #[test]
    fn testRaw2Bound2() {
        assert_eq!(1073741823, lib::rs_raw_to_bound(lib::DBM_INF));
    }

    //TOD O add bound to raw
    //     @Test
    //     public void testBound2Raw1() {
    // assertEquals(1, DBMLib.boundbool2raw(0, false));
    // }
    //
    //     @Test
    //     public void testBound2Raw2() {
    // assertEquals(2147483647, DBMLib.boundbool2raw(1073741823, false));
    // }
    #[test]
    fn testDbmInit1() {
        let mut intArr = [0, 0, 0, 0];
        let dbm = &mut intArr;
        lib::rs_dbm_init(dbm, 2);
        assert_eq!([1, 1, lib::DBM_INF, 1].as_mut(), dbm);
    }

    #[test]
    fn testDbmInit2() {
        let mut intArr = [0, 0, 0, 0, 0, 0, 0, 0, 0];
        let dbm = &mut intArr;
        lib::rs_dbm_init(dbm, 3);
        assert_eq!(
            [
                1,
                1,
                1,
                lib::DBM_INF,
                1,
                lib::DBM_INF,
                lib::DBM_INF,
                lib::DBM_INF,
                1
            ]
            .as_mut(),
            dbm
        );
    }

    #[test]
    fn testDbmConstrain1() {
        let mut intArr = [1, 1, lib::DBM_INF, 1];
        let dbm = &mut intArr;
        lib::rs_dbm_add_LTE_constraint(dbm, 2, 1, 0, 5);
        assert_eq!([1, 1, 11, 1].as_mut(), dbm);
    }

    #[test]
    fn testDbmConstrain2() {
        let mut intArr = [1, 1, 11, 1];
        let dbm = &mut intArr;
        lib::rs_dbm_add_LTE_constraint(dbm, 2, 0, 1, -2);
        assert_eq!([1, -3, 11, 1].as_mut(), dbm);
    }

    #[test]
    fn testDbmConstrain4() {
        //DBM With two dependent clocks from 0 to inf both
        let mut intArr = [1, 1, 1, lib::DBM_INF, 1, 1, lib::DBM_INF, 1, 1];
        let dbm = &mut intArr;
        //Apply constraint on the first clock <= 5 in raw value 11
        lib::rs_dbm_add_LTE_constraint(dbm, 3, 1, 0, 5);
        //Both clocks are bounded by 11 but no differences change
        assert_eq!([1, 1, 1, 11, 1, 1, 11, 1, 1].as_mut(), dbm);
    }

    #[test]
    fn testDbmConstrain3() {
        //DBM with two independent clocks from 0 to inf
        let mut intArr = [1, 1, 1, 1, 1, 1, 1, 1, 1];
        let dbm = &mut intArr;
        lib::rs_dbm_init(dbm, 3);
        lib::rs_dbm_add_LTE_constraint(dbm, 3, 1, 0, 4);
        //the clock and difference gets bounded by 9 but no other clocks change
        assert_eq!(
            [1, 1, 1, 9, 1, 9, lib::DBM_INF, lib::DBM_INF, 1].as_mut(), 
            dbm
        );
    }

    #[test]
    fn testDbmReset1() {
        //TODO Check why it fails whilel it shouldn't
        let mut intArr = [1, -3, 11, 1];
        let dbm = &mut intArr;
        lib::rs_dbm_update(dbm, 2, 1, 0);
        assert_eq!([1, 1, 1, 1].as_mut(), dbm);
    }

    #[test]
    fn testDbmReset2() {
        let mut intArr = [1, 1, 1, 7, 1, 7, 5, 5, 1];
        let dbm = &mut intArr;
        lib::rs_dbm_update(dbm, 3, 1, 0);
        assert_eq!([1, 1, 1, 1, 1, 1, 5, 5, 1].as_mut(), dbm);
    }

    #[test]
    fn testDbmFuture1() {
        let mut intArr = [1, 1, 1, 1];
        let dbm = &mut intArr;
        lib::rs_dbm_up(dbm, 2);
        assert_eq!([1, 1, lib::DBM_INF, 1].as_mut(), dbm);
    }

    #[test]
    fn testDbmFuture2() {
        let mut intArr = [1, -3, 11, 1];
        let dbm = &mut intArr;
        lib::rs_dbm_up(dbm, 2);
        assert_eq!([1, -3, lib::DBM_INF, 1].as_mut(), dbm);
    }

    #[test]
    fn testDbmMinusDbm() {
        println!("testDbmMinusDbm");
        let dim = 3;
        let mut intArr1 = [
            1,
            1,
            1,
            lib::DBM_INF,
            1,
            lib::DBM_INF,
            lib::DBM_INF,
            lib::DBM_INF,
            1,
        ];
        let _dbm1 = &mut intArr1;
        let mut intArr2 = [
            1,
            1,
            1,
            lib::DBM_INF,
            1,
            lib::DBM_INF,
            lib::DBM_INF,
            lib::DBM_INF,
            1,
        ];
        let dbm2 = &mut intArr2;

        lib::rs_dbm_add_LTE_constraint(dbm2, dim, 0, 1, -2);
        println!("{:?}", dbm2);
        lib::rs_dbm_add_LTE_constraint(dbm2, dim, 0, 2, -3);
        lib::rs_dbm_add_LTE_constraint(dbm2, dim, 1, 0, 4);
        lib::rs_dbm_add_LTE_constraint(dbm2, dim, 2, 0, 5);
        println!("{:?}", dbm2);
        //TODO check DBM - DBM error
        //let mut arr1 = lib::rs_dbm_minus_dbm(dbm1, dbm2, dim);

        // let mut fed1 = Federation(arr1);
        //         let mut intArr3 = [1, 1, 1, lib::DBM_INF, 1, lib::DBM_INF, lib::DBM_INF, lib::DBM_INF, 1];
        //         let dbm3 = &mut intArr3;
        //
        //         lib::rs_dbm_add_LTE_constraint(dbm3, dim, 0, 1, 0);
        //         lib::rs_dbm_add_LTE_constraint(dbm3, dim, 0, 2, 0);
        //         lib::rs_dbm_add_LTE_constraint(dbm3, dim, 1, 0, 1);
        //         lib::rs_dbm_add_LTE_constraint(dbm3, dim, 2, 0, 1);
        //         let fedP = &mut arr1;
        //let mut arr2 = lib::rs_dbm_fed_minus_fed(fedP, dbm3, dim);

        //    assert_eq!(arr1.len(), 4);
        //assert_eq!(arr2.size(), 5);
    }
}
