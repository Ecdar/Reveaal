#[cfg(test)]
mod composition_tests {
    use crate::ModelObjects::component::{get_dummy_component, SyncType};

    use crate::TransitionSystems::{Composition, Conjunction, TransitionSystem};

    #[test]
    fn Test1() {
        let inputs = vec!["i1".to_string(), "i2".to_string()];
        let outputs = vec!["o1".to_string(), "o2".to_string()];

        let comp1 = get_dummy_component("Test1".to_string(), &inputs, &outputs);
        let comp2 = get_dummy_component("Test2".to_string(), &inputs, &outputs);
        let comp = Composition::new(Box::new(comp1), Box::new(comp2));

        println!("Locs: {:?}", comp.get_all_locations());
        println!("Transitions:");
        for t in comp.next_transitions(&comp.get_initial_location(), "i1", &SyncType::Input, &mut 0)
        {
            println!("{}", t);
        }
        assert!(false);
    }
}
