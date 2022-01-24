use crate::DBMLib::dbm::{Federation, Zone};
use crate::EdgeEval::constraint_applyer;
use crate::ModelObjects::component;
use crate::ModelObjects::component::DeclarationProvider;
use crate::ModelObjects::representations;
use crate::TransitionSystems::TransitionSystem;
use std::collections::HashMap;

pub fn make_input_enabled(component: &mut component::Component, inputs: &[String]) {
    let dimension = (component as &dyn TransitionSystem).get_max_clock_index() + 1;
    let mut new_edges: Vec<component::Edge> = vec![];

    for location in component.get_locations() {
        let mut location_inv_zone = Zone::init(dimension);

        if let Some(invariant) = location.get_invariant() {
            constraint_applyer::apply_constraints_to_state_declarations(
                invariant,
                component.get_declarations(),
                &mut location_inv_zone,
            );
        }

        // No constraints on any clocks
        let full_federation =
            Federation::new(vec![location_inv_zone.clone()], location_inv_zone.dimension);

        for input in inputs {
            let input_edges = component.get_next_edges(location, input, component::SyncType::Input);
            let mut zones = vec![];

            for edge in input_edges {
                let mut guard_zone = location_inv_zone.clone();
                let has_inv = if let Some(target_invariant) = component
                    .get_location_by_name(edge.get_target_location())
                    .get_invariant()
                {
                    constraint_applyer::apply_constraints_to_state_declarations(
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
                    constraint_applyer::apply_constraints_to_state_declarations(
                        guard,
                        component.get_declarations(),
                        &mut guard_zone,
                    )
                } else {
                    false
                };

                if !has_inv && !has_guard {
                    zones.push(location_inv_zone.clone());
                } else {
                    zones.push(guard_zone);
                }
            }

            let zones_federation = Federation::new(zones, location_inv_zone.dimension);
            let result_federation = full_federation.minus_fed(&zones_federation);

            for fed_zone in result_federation.iter_zones() {
                new_edges.push(component::Edge {
                    source_location: location.get_id().to_string(),
                    target_location: location.get_id().to_string(),
                    sync_type: component::SyncType::Input,
                    guard: build_guard_from_zone(
                        &fed_zone,
                        component.get_declarations().get_clocks(),
                    ),
                    update: None,
                    sync: input.to_string(),
                });
            }
        }
    }

    component.add_input_edges(&mut new_edges);
}

pub fn build_guard_from_zone(
    zone: &Zone,
    clocks: &HashMap<String, u32>,
) -> Option<representations::BoolExpression> {
    let mut guards: Vec<representations::BoolExpression> = vec![];

    for clock in clocks.keys() {
        let index = clocks.get(clock).unwrap();
        let (upper_is_strict, upper_val) = zone.get_constraint(*index, 0);
        let (lower_is_strict, lower_val) = zone.get_constraint(0, *index);

        // lower bound must be different from 1 (==0)
        if lower_is_strict || lower_val != 0 {
            if lower_is_strict {
                guards.push(representations::BoolExpression::LessT(
                    Box::new(representations::BoolExpression::Int(-lower_val)),
                    Box::new(representations::BoolExpression::VarName(clock.clone())),
                ));
            } else {
                guards.push(representations::BoolExpression::LessEQ(
                    Box::new(representations::BoolExpression::Int(-lower_val)),
                    Box::new(representations::BoolExpression::VarName(clock.clone())),
                ));
            }
        }

        if !zone.is_constraint_infinity(*index, 0) {
            if upper_is_strict {
                guards.push(representations::BoolExpression::LessT(
                    Box::new(representations::BoolExpression::VarName(clock.clone())),
                    Box::new(representations::BoolExpression::Int(upper_val)),
                ));
            } else {
                guards.push(representations::BoolExpression::LessEQ(
                    Box::new(representations::BoolExpression::VarName(clock.clone())),
                    Box::new(representations::BoolExpression::Int(upper_val)),
                ));
            }
        }
    }

    let res = build_guard_from_zone_helper(&mut guards);
    match res {
        representations::BoolExpression::Bool(false) => None,
        _ => Some(res),
    }
}

fn build_guard_from_zone_helper(
    guards: &mut Vec<representations::BoolExpression>,
) -> representations::BoolExpression {
    let num_guards = guards.len();

    if let Some(guard) = guards.pop() {
        if num_guards == 1 {
            guard
        } else {
            representations::BoolExpression::AndOp(
                Box::new(guard),
                Box::new(build_guard_from_zone_helper(guards)),
            )
        }
    } else {
        representations::BoolExpression::Bool(false)
    }
}
