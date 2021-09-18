#[cfg(test)]
mod test {
    use crate::DBMLib::dbm::{Federation, Zone};

    #[test]
    fn empty_federation_is_not_valid() {
        let fed = Federation::empty(3);

        assert!(!fed.is_valid());
    }

    #[test]
    fn no_clocks_fed_equal_itself() {
        let mut fed1 = Federation::full(1);
        let mut fed2 = Federation::full(1);

        assert!(fed1.is_subset_eq(&mut fed2));
    }

    #[test]
    fn delayed_empty_federation_check_zone() {
        let mut fed = Federation::zero(3);
        fed.up();

        let mut base_zone = Zone::new(3);
        base_zone.up();

        assert_eq!(fed.zones.len(), 1);

        assert!(fed.zones[0].is_subset_eq(&base_zone));
        assert!(base_zone.is_subset_eq(&fed.zones[0]));
    }

    #[test]
    fn updated_full_federation_check_zone() {
        let mut fed = Federation::full(3);
        fed.update(2, 0);

        let mut base_zone = Zone::init(3);
        base_zone.update(2, 0);

        assert_eq!(fed.zones.len(), 1);

        assert!(fed.zones[0].is_subset_eq(&base_zone));
        assert!(base_zone.is_subset_eq(&fed.zones[0]));
    }

    #[test]
    fn full_federations_equals() {
        let mut fed1 = Federation::full(3);
        let mut fed2 = Federation::full(3);

        assert!(fed1.is_subset_eq(&mut fed2));
        assert!(fed2.is_subset_eq(&mut fed1));
    }

    #[test]
    fn empty_is_subset_of_full_federation() {
        let mut empty = Federation::empty(3);
        let mut full = Federation::full(3);

        assert!(empty.is_subset_eq(&mut full));
        assert!(!full.is_subset_eq(&mut empty));
    }

    #[test]
    fn full_can_delay_indefinitely() {
        let mut full = Federation::full(3);

        assert!(full.can_delay_indefinitely());
    }

    #[test]
    fn delayed_zero_can_delay_indefinitely() {
        let mut fed = Federation::zero(3);
        fed.up();

        assert!(fed.can_delay_indefinitely());
    }

    #[test]
    fn empty_can_not_delay_indefinitely() {
        let mut empty = Federation::empty(3);
        assert!(!empty.can_delay_indefinitely());
    }

    #[test]
    fn contrain_full_fed() {
        let mut full = Federation::full(2);

        println!("\n{}", full);
        full.add_lt_constraint(0, 1, -20);

        println!("\n{}", full);
    }

    #[test]
    fn contrain_full_dbm() {
        let mut full = Zone::init(2);

        println!("\n{}", full);
        full.add_lt_constraint(0, 1, -20);

        println!("\n{}", full);
    }

    #[test]
    fn cant_delay_with_upper_bound() {
        let mut full = Federation::full(2);
        full.add_lt_constraint(1, 0, 50);

        assert!(!full.can_delay_indefinitely());
    }

    #[test]
    fn can_delay_with_lower_bound() {
        let mut full = Federation::full(2);
        full.add_lt_constraint(0, 1, -50);

        assert!(full.can_delay_indefinitely());
    }
}
