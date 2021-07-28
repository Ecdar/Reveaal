#[cfg(test)]
pub mod save_comp_helper {
    use crate::tests::refinement::Helper;
    use crate::DataReader::parse_queries;
    use crate::ModelObjects::representations::QueryExpression;
    use crate::System::extract_system_rep;
    use crate::System::refine;
    use crate::System::save_component::combine_components;

    pub fn json_reconstructed_component_refines_base_self(input_path: &str, system: &str) {
        let (components, mut decl) = Helper::json_setup(String::from(input_path));

        //This query is not executed but simply used to extract an UncachedSystem so the tests can just give system expressions
        let str_query = format!("get-component: {} save-as test", system);
        let query = parse_queries::parse(str_query.as_str()).remove(0);

        let mut clock_index: u32 = 0;
        let base_system = if let QueryExpression::GetComponent(expr) = &query {
            extract_system_rep::extract_side(expr.as_ref(), &components, &mut clock_index)
        } else {
            panic!("Failed to create system")
        };

        let new_comp = combine_components(&base_system.clone());
        let new_comp = new_comp.create_edge_io_split();
        decl.add_component(&new_comp);

        let new_system = Box::new(new_comp);
        assert!(refine::check_refinement(new_system.clone(), base_system.clone()).unwrap());
        assert!(refine::check_refinement(base_system.clone(), new_system.clone()).unwrap());
    }
}
