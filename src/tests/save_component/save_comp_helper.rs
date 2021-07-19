#[cfg(test)]
pub mod save_comp_helper {
    use crate::tests::refinement::Helper;
    use crate::DataReader::{parse_queries, xml_parser};
    use crate::ModelObjects::component_view::ComponentView;
    use crate::ModelObjects::queries::Query;
    use crate::ModelObjects::representations::QueryExpression;
    use crate::ModelObjects::representations::SystemRepresentation;
    use crate::ModelObjects::system::UncachedSystem;
    use crate::ModelObjects::system_declarations::SystemDeclarations;
    use crate::System::executable_query::QueryResult;
    use crate::System::extract_system_rep;
    use crate::System::refine;
    use crate::System::save_component::combine_components;

    pub fn json_reconstructed_component_refines_base_self(input_path: &str, system: &str) {
        let (components, mut decl) = Helper::json_setup(String::from(input_path));

        let str_query = format!("get-component: {} save-as test", system);
        let query = parse_queries::parse(str_query.as_str()).remove(0);

        let mut clock_index: u32 = 0;
        let base_system = if let QueryExpression::GetComponent(expr) = &query {
            if let QueryExpression::SaveAs(system_expr, _) = expr.as_ref() {
                UncachedSystem::create(extract_system_rep::extract_side(
                    expr.as_ref(),
                    &components,
                    &mut clock_index,
                ))
            } else {
                panic!("Failed to create system");
            }
        } else {
            panic!("Failed to create system")
        };

        let new_comp = combine_components(&base_system.clone(), &decl.clone());
        let new_comp = new_comp.create_edge_io_split();
        decl.add_component(&new_comp);

        let new_system = UncachedSystem::create(SystemRepresentation::Component(
            ComponentView::create(&new_comp, clock_index),
        ));

        assert!(new_system.precheck_sys_rep());
        assert!(base_system.precheck_sys_rep());

        assert!(refine::check_refinement(new_system.clone(), base_system.clone(), &decl).unwrap());
        assert!(refine::check_refinement(base_system.clone(), new_system.clone(), &decl).unwrap());
    }
}
