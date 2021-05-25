#[cfg(test)]
mod samples {
    use crate::tests::refinement::Helper::setup;
    use crate::ModelObjects::component::Component;

    static CONJUNCTION_SAMPLE: &str = "samples/json/Conjunction";

    #[test]
    fn test_locations_T1() {
        let (automataList, _) = setup(CONJUNCTION_SAMPLE.to_string());
        let t1 = automataList.get("Test1").unwrap().clone();

        assert_eq!(t1.get_name(), "Test1");
        assert_eq!(t1.get_locations().len(), 2);
    }

    #[test]
    fn test_locations_T2() {
        let (automataList, _) = setup(CONJUNCTION_SAMPLE.to_string());
        let t2 = automataList.get("Test2").unwrap().clone();

        assert_eq!(t2.get_name(), "Test2");
        assert_eq!(t2.get_locations().len(), 2);
    }

    #[test]
    fn test_locations_T3() {
        let (automataList, _) = setup(CONJUNCTION_SAMPLE.to_string());
        let t3: Component = automataList.get("Test3").unwrap().clone();

        assert_eq!(t3.get_name(), "Test3");
        assert_eq!(t3.get_locations().len(), 3);
    }

    #[test]
    fn test_names_T1_through_T12() {
        let (automataList, _) = setup(CONJUNCTION_SAMPLE.to_string());

        for i in 1..12 {
            let t: Component = automataList
                .get(&format!("Test{}", i).to_string())
                .unwrap()
                .clone();

            assert_eq!(t.name, format!("Test{}", i));
        }
    }
}
