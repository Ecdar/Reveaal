#[cfg(test)]
mod samples {
    use crate::DataReader::component_loader::JsonProjectLoader;
    use crate::DataReader::parse_queries;
    use crate::ModelObjects::representations::QueryExpression;
    use crate::System::extract_system_rep;
    use crate::TransitionSystems::TransitionSystemPtr;

    pub fn get_transition_system_from(input_path: &str, system: &str) -> TransitionSystemPtr {
        let project_loader = JsonProjectLoader::new(String::from(input_path));

        //This query is not executed but simply used to extract a TransitionsSystem
        let str_query = format!("get-component: {} save-as test", system);
        let query = parse_queries::parse_to_expression_tree(str_query.as_str()).remove(0);

        let mut clock_index: u32 = 0;
        if let QueryExpression::GetComponent(expr) = &query {
            let mut comp_loader = project_loader.to_comp_loader();
            extract_system_rep::extract_side(expr.as_ref(), &mut *comp_loader, &mut clock_index)
        } else {
            panic!("Failed to create system")
        }
    }
    static PATH: &str = "samples/json/Quotient";

    #[test]
    fn s1_t1_has_6_locations() {
        let system = get_transition_system_from(PATH, "S1//T1");

        assert_eq!(system.get_all_locations(&mut 0).len(), 6);
    }

    #[test]
    fn s1_t1_has_inconsistent_location() {
        let system = get_transition_system_from(PATH, "S1//T1");

        let locations = system.get_all_locations(&mut 0);
        let has_inconsistent = locations
            .iter()
            .any(|loc| loc.to_string() == "(Inconsistent)");

        assert!(has_inconsistent);
    }

    #[test]
    fn s1_t1_has_universal_location() {
        let system = get_transition_system_from(PATH, "S1//T1");

        let locations = system.get_all_locations(&mut 0);
        let has_inconsistent = locations
            .iter()
            .any(|loc| loc.to_string() == "(Inconsistent)");

        assert!(has_inconsistent);
    }
}
