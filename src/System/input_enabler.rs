use crate::DBMLib::dbm::Federation;
use crate::EdgeEval::constraint_applyer;
use crate::ModelObjects::component;
use crate::ModelObjects::component::DeclarationProvider;
use crate::TransitionSystems::TransitionSystem;

pub fn make_input_enabled(component: &mut component::Component, inputs: &[String]) {
    let dimension = (component as &dyn TransitionSystem).get_max_clock_index() + 1;
    let mut new_edges: Vec<component::Edge> = vec![];

    for location in component.get_locations() {
        let mut location_inv_zone = Federation::full(dimension);

        if let Some(invariant) = location.get_invariant() {
            constraint_applyer::apply_constraints_to_state(
                invariant,
                component.get_declarations(),
                &mut location_inv_zone,
            );
        }

        // No constraints on any clocks
        let full_federation = location_inv_zone.clone();

        for input in inputs {
            let input_edges = component.get_next_edges(location, input, component::SyncType::Input);
            let mut zones_federation = Federation::empty(location_inv_zone.get_dimensions());

            for edge in input_edges {
                let mut guard_zone = location_inv_zone.clone();
                let has_inv = if let Some(target_invariant) = component
                    .get_location_by_name(edge.get_target_location())
                    .get_invariant()
                {
                    constraint_applyer::apply_constraints_to_state(
                        target_invariant,
                        component.get_declarations(),
                        &mut guard_zone,
                    )
                } else {
                    false
                };

                if edge.get_update().is_some() {
                    let update_clocks = edge.get_update_clocks();
                    for clock in update_clocks {
                        let clock_index = component.get_declarations().clocks.get(clock).unwrap();
                        guard_zone.free_clock(*clock_index);
                    }
                }

                let has_guard = if let Some(guard) = edge.get_guard() {
                    constraint_applyer::apply_constraints_to_state(
                        guard,
                        component.get_declarations(),
                        &mut guard_zone,
                    )
                } else {
                    false
                };

                if !has_inv && !has_guard {
                    zones_federation.add_fed(&location_inv_zone);
                } else {
                    zones_federation.add_fed(&guard_zone);
                }
            }

            //let zones_federation = Federation::new(zones, location_inv_zone.dimension);
            let result_federation = full_federation.subtraction(&zones_federation);

            if result_federation.is_empty() {
                continue;
            }

            //for fed_zone in result_federation.iter_zones() {
            new_edges.push(component::Edge {
                source_location: location.get_id().to_string(),
                target_location: location.get_id().to_string(),
                sync_type: component::SyncType::Input,
                guard: result_federation
                    .as_boolexpression(Some(component.get_declarations().get_clocks())), //build_guard_from_zone(
                //    &fed_zone,
                //    Some(component.get_declarations().get_clocks()),
                //)
                update: None,
                sync: input.to_string(),
            });
            //}
        }
    }

    component.add_input_edges(&mut new_edges);
}
