#[cfg(test)]
pub mod save_comp_helper {
    use crate::tests::refinement::Helper;
    use crate::DataReader::parse_queries;
    use crate::ModelObjects::component_view::ComponentView;
    use crate::ModelObjects::representations::QueryExpression;
    use crate::ModelObjects::representations::SystemRepresentation;
    use crate::ModelObjects::system::UncachedSystem;
    use crate::System::extract_system_rep;
    use crate::System::input_enabler;
    use crate::System::refine;
    use crate::System::save_component::combine_components;

    pub fn json_reconstructed_component_refines_base_self(input_path: &str, system: &str) {
        let (components, mut decl) = Helper::json_setup(String::from(input_path));

        //This query is not executed but simply used to extract an UncachedSystem so the tests can just give system expressions
        let str_query = format!("get-component: {} save-as test", system);
        let query = parse_queries::parse(str_query.as_str()).remove(0);

        let mut clock_index: u32 = 0;
        let mut base_system = if let QueryExpression::GetComponent(expr) = &query {
            UncachedSystem::create(extract_system_rep::extract_side(
                expr.as_ref(),
                &components,
                &mut clock_index,
            ))
        } else {
            panic!("Failed to create system")
        };

        let new_comp = combine_components(&base_system.clone(), &decl.clone());
        let mut new_comp = new_comp.create_edge_io_split();
        decl.add_component(&new_comp);

        input_enabler::make_input_enabled(&mut new_comp, &decl);

        let mut new_system = UncachedSystem::create(SystemRepresentation::Component(
            ComponentView::create(&new_comp, clock_index),
        ));

        let base_is_consistent = base_system.check_consistency(&decl);

        if new_system.check_consistency(&decl) {
            assert!(
                refine::check_refinement(new_system.clone(), base_system.clone(), &decl).unwrap()
            );
            assert!(
                refine::check_refinement(base_system.clone(), new_system.clone(), &decl).unwrap()
            );
        } else {
            assert!(!base_is_consistent);
        }
    }
}
