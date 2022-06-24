#[cfg(test)]
pub mod save_comp_helper {
    use crate::DataReader::component_loader::JsonProjectLoader;
    use crate::DataReader::parse_queries;
    use crate::ModelObjects::representations::QueryExpression;
    use crate::System::extract_system_rep;
    use crate::System::extract_system_rep::SystemRecipe;
    use crate::System::refine;
    use crate::System::save_component::combine_components;

    pub fn json_reconstructed_component_refines_base_self(input_path: &str, system: &str) {
        let project_loader = JsonProjectLoader::new(String::from(input_path)).unwrap();
        let mut decl = project_loader.get_declarations().clone();

        //This query is not executed but simply used to extract an UncachedSystem so the tests can just give system expressions
        let str_query = format!("get-component: {} save-as test", system);
        let query = parse_queries::parse_to_expression_tree(str_query.as_str())
            .unwrap()
            .remove(0);

        let mut dim: u32 = 0;
        let (base_system, new_system) = if let QueryExpression::GetComponent(expr) = &query {
            let mut comp_loader = project_loader.to_comp_loader();
            (
                extract_system_rep::get_system_recipe(expr.as_ref(), &mut *comp_loader, &mut dim)
                    .unwrap(),
                extract_system_rep::get_system_recipe(expr.as_ref(), &mut *comp_loader, &mut dim)
                    .unwrap(),
            )
        } else {
            panic!("Failed to create system")
        };

        let new_comp = new_system.compile(dim);

        if let Err(_) = new_comp {
            return;
        }
        let new_comp = combine_components(&new_comp.unwrap()).unwrap();

        let new_comp = SystemRecipe::Component(Box::new(new_comp))
            .compile(dim)
            .unwrap();
        let base_system = base_system.compile(dim).unwrap();

        let base_precheck = base_system.precheck_sys_rep().unwrap();
        let new_precheck = new_comp.precheck_sys_rep().unwrap();
        assert_eq!(base_precheck, new_precheck);

        //Only do refinement check if both pass precheck
        if base_precheck && new_precheck {
            assert!(refine::check_refinement(new_comp.clone(), base_system.clone()).unwrap());
            assert!(refine::check_refinement(base_system.clone(), new_comp.clone()).unwrap());
        }
    }
}
