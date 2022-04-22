#[cfg(test)]
pub mod test {
    use crate::DBMLib::dbm::Federation;

    const DIM: u32 = 5;

    #[test]
    pub fn fed_is_empty() {
        assert!(Federation::empty(DIM).is_empty());
    }

    #[test]
    fn fed_with_added_fed() {
        let fed1 = Federation::zero(DIM);
        let fed2 = !Federation::zero(DIM);

        let fed3 = fed1.with_added_fed(&fed2);
        // fed1 and fed2 remain unchanged

        println!("fed1: {}", fed1);
        println!("fed2: {}", fed2);
        println!("fed3: {}", fed3);

        assert_eq!(Federation::full(DIM), fed3);
        assert_eq!(fed3, fed1 + fed2);
    }

    #[test]
    fn fed_add_fed() {
        let mut fed1 = Federation::zero(DIM);
        let fed2 = !Federation::zero(DIM);

        fed1.add_fed(&fed2);
        // fed1 is changed and fed2 remains unchanged

        println!("fed1: {}", fed1);
        println!("fed2: {}", fed2);

        assert_eq!(Federation::full(DIM), fed1);
    }
    #[test]
    fn fed_inverse() {
        let fed1 = Federation::full(DIM);
        let fed2 = fed1.inverse();
        // fed1 remains unchanged

        println!("fed1: {}", fed1);
        println!("fed2: {}", fed2);

        assert_eq!(Federation::empty(DIM), fed2);
        assert_eq!(fed2, !fed1);
    }

    #[test]
    fn fed_invert() {
        let mut fed1 = Federation::full(DIM);
        fed1.invert();
        // fed1 is changed to contain its inverse

        println!("fed1: {}", fed1);

        assert_eq!(Federation::empty(DIM), fed1);
    }

    #[test]
    fn fed_intersect() {
        let mut fed1 = Federation::full(DIM);
        let fed2 = Federation::zero(DIM);

        fed1.intersect(&fed2);
        // fed1 is changed and fed2 remains unchanged

        println!("fed1: {}", fed1);
        println!("fed2: {}", fed2);

        assert_eq!(Federation::zero(DIM), fed1);
    }

    #[test]
    fn fed_intersection() {
        let fed1 = Federation::full(DIM);
        let fed2 = Federation::zero(DIM);

        let fed3 = fed1.intersection(&fed2);
        // fed1 and fed2 remain unchanged

        println!("fed1: {}", fed1);
        println!("fed2: {}", fed2);
        println!("fed3: {}", fed3);

        assert_eq!(Federation::zero(DIM), fed3);
        assert_eq!(fed3, fed1 & fed2);
    }

    #[test]
    fn fed_subtract() {
        let mut fed1 = Federation::zero(DIM);
        let fed2 = Federation::full(DIM);

        fed1.subtract(&fed2);
        // fed1 is changed and fed2 remains unchanged

        println!("fed1: {}", fed1);
        println!("fed2: {}", fed2);

        assert_eq!(Federation::empty(DIM), fed1);
    }

    #[test]
    fn fed_subtraction() {
        let fed1 = Federation::zero(DIM);
        let fed2 = Federation::full(DIM);

        let fed3 = fed1.subtraction(&fed2);
        // fed1 and fed2 remain unchanged

        println!("fed1: {}", fed1);
        println!("fed2: {}", fed2);
        println!("fed3: {}", fed3);

        assert_eq!(Federation::empty(DIM), fed3);
        assert_eq!(fed3, fed1 - fed2);
    }
}
