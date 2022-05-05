#[cfg(test)]
pub mod save_comp_helper {
    use crate::DataReader::component_loader::JsonProjectLoader;
    use crate::DataReader::parse_queries;
    use crate::ModelObjects::component::DeclarationProvider;
    use crate::ModelObjects::representations::QueryExpression;
    use crate::System::extract_system_rep;
    use crate::System::input_enabler;
    use crate::System::refine;
    use crate::System::save_component::combine_components;
    use crate::TransitionSystems::CompiledComponent;
    use crate::TransitionSystems::TransitionSystem;

    pub fn json_reconstructed_component_refines_base_self(input_path: &str, system: &str) {
        let project_loader = JsonProjectLoader::new(String::from(input_path));
        let mut decl = project_loader.get_declarations().clone();

        //This query is not executed but simply used to extract an UncachedSystem so the tests can just give system expressions
        let str_query = format!("get-component: {} save-as test", system);
        let query = parse_queries::parse_to_expression_tree(str_query.as_str()).remove(0);

        let mut dim: u32 = 0;
        let base_system = if let QueryExpression::GetComponent(expr) = &query {
            let mut comp_loader = project_loader.to_comp_loader();
            extract_system_rep::get_system_recipe(expr.as_ref(), &mut *comp_loader, &mut dim)
        } else {
            panic!("Failed to create system")
        };

        let mut new_comp = combine_components(&base_system.clone().compile(dim));
        new_comp.set_clock_indices(&mut dim);
        let base_system = base_system.compile(dim);
        let new_comp = CompiledComponent::compile(new_comp, dim + 1);

        // let opt_inputs = decl.get_component_inputs(new_comp.get_name());
        // if opt_inputs.is_some() {
        //     input_enabler::make_input_enabled(&mut new_comp, opt_inputs.unwrap());
        // }

        let base_precheck = base_system.precheck_sys_rep();
        let new_precheck = new_comp.precheck_sys_rep();
        assert_eq!(base_precheck, new_precheck);

        //Only do refinement check if both pass precheck
        if base_precheck && new_precheck {
            assert!(refine::check_refinement(new_comp.clone(), base_system.clone()).unwrap());
            assert!(refine::check_refinement(base_system.clone(), new_comp.clone()).unwrap());
        }
    }
}
