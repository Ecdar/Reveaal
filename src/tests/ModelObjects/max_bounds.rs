#[cfg(test)]
mod max_bounds {
    use crate::{DBMLib::dbm::Federation, ModelObjects::max_bounds::MaxBounds};

    const DIM: u32 = 10;
    #[test]
    fn extrapolate_test1() {
        let lt = 10;
        let bound = 9;

        let mut fed = Federation::init(DIM);

        fed.add_lt_constraint(1, 0, lt);
        let pre_fed = fed.clone();

        println!("Before {}", pre_fed);

        let mut bounds = MaxBounds::create(DIM);

        println!("Bound {}", bound);
        for i in 1..DIM {
            bounds.add_bound(i, bound);
        }

        let mut post_fed = fed;

        post_fed.extrapolate_max_bounds(&bounds);
        println!("After {}", post_fed);

        assert!(post_fed == Federation::init(DIM));
    }

    #[test]
    fn extrapolate_test2() {
        let lt = 9;
        let bound = 10;

        let mut fed = Federation::init(DIM);

        fed.add_lt_constraint(1, 0, lt);
        let pre_fed = fed.clone();

        println!("Before {}", pre_fed);

        let mut bounds = MaxBounds::create(DIM);

        println!("Bound {}", bound);
        for i in 1..DIM {
            bounds.add_bound(i, bound);
        }

        let mut post_fed = fed;

        post_fed.extrapolate_max_bounds(&bounds);
        println!("After {}", post_fed);

        assert!(post_fed == pre_fed);
    }
}
