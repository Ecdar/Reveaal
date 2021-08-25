#[cfg(test)]
pub mod save_comp_helper {
    use crate::tests::refinement::Helper;
    use crate::DataReader::parse_queries;
    use crate::ModelObjects::representations::QueryExpression;
    use crate::System::extract_system_rep;
    use crate::System::input_enabler;
    use crate::System::refine;
    use crate::System::save_component::combine_components;
    use crate::TransitionSystems::TransitionSystem;

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

        let mut new_comp = combine_components(&base_system.clone());
        new_comp.create_edge_io_split();
        let mut new_comp = Box::new(new_comp);
        decl.add_component(&new_comp);
        //input_enabler::make_input_enabled(&mut new_comp, &decl);

        let dimensions = 1 + new_comp.get_num_clocks() + base_system.get_num_clocks();

        let base_precheck = base_system.precheck_sys_rep(dimensions);
        let new_precheck = new_comp.precheck_sys_rep(dimensions);
        assert_eq!(base_precheck, new_precheck);
        new_comp.set_clock_indices(&mut clock_index);

        //Only do refinement check if both pass precheck
        if base_precheck && new_precheck {
            assert!(refine::check_refinement(new_comp.clone(), base_system.clone()).unwrap());
            assert!(refine::check_refinement(base_system.clone(), new_comp.clone()).unwrap());
        }
    }
}
