use edbm::zones::OwnedFederation;

use crate::EdgeEval::constraint_applyer;
use crate::ModelObjects::component;
use crate::ModelObjects::component::DeclarationProvider;
use crate::ModelObjects::edge::{Edge, SyncType};
use crate::ModelObjects::representations::BoolExpression;

pub fn make_input_enabled(component: &mut component::Component, inputs: &[String]) {
    let dimension = component.declarations.get_clock_count() + 1;
    let mut new_edges: Vec<Edge> = vec![];
    let input_edges = component
        .get_edges()
        .iter()
        .filter(|edge| *edge.get_sync_type() == SyncType::Input);
    for location in component.get_locations() {
        let mut location_inv_zone = OwnedFederation::universe(dimension);

        if let Some(invariant) = location.get_invariant() {
            location_inv_zone = constraint_applyer::apply_constraints_to_state(
                invariant,
                component.get_declarations(),
                location_inv_zone,
            )
            .unwrap();
        }

        // No constraints on any clocks
        let full_federation = location_inv_zone.clone();
        let location_edges = input_edges
            .clone()
            .filter(|edge| edge.get_source_location() == location.get_id());

        for input in inputs {
            let specific_edges = location_edges
                .clone()
                .filter(|edge| *edge.get_sync() == *input || *edge.get_sync() == "*");
            let mut zones_federation = OwnedFederation::empty(dimension);

            for edge in specific_edges {
                let mut guard_zone = OwnedFederation::universe(dimension);
                if let Some(target_invariant) = component
                    .get_location_by_name(edge.get_target_location())
                    .get_invariant()
                {
                    guard_zone = constraint_applyer::apply_constraints_to_state(
                        target_invariant,
                        component.get_declarations(),
                        guard_zone,
                    )
                    .unwrap();
                }

                if let Some(updates) = edge.get_update() {
                    for update in updates {
                        let cu = update.compiled(component.get_declarations());
                        guard_zone = cu.apply_as_guard(guard_zone);
                        guard_zone = cu.apply_as_free(guard_zone);
                    }
                }

                if let Some(guard) = edge.get_guard() {
                    guard_zone = constraint_applyer::apply_constraints_to_state(
                        guard,
                        component.get_declarations(),
                        guard_zone,
                    )
                    .unwrap();
                }

                zones_federation += guard_zone.intersection(&location_inv_zone);
            }

            let result_federation = full_federation.clone().subtraction(&zones_federation);

            if result_federation.is_empty() {
                continue;
            }

            new_edges.push(Edge {
                id: format!("input_{}_{}", location.get_id(), input),
                source_location: location.get_id().to_string(),
                target_location: location.get_id().to_string(),
                sync_type: SyncType::Input,
                guard: BoolExpression::from_disjunction(
                    &result_federation.minimal_constraints(),
                    component.get_declarations().get_clocks(),
                ),
                update: None,
                sync: input.to_string(),
            });
        }
    }

    component.add_edges(&mut new_edges);
}
