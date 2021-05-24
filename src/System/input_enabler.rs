use crate::DBMLib::lib;
use crate::EdgeEval::constraint_applyer;
use crate::ModelObjects::component;
use crate::ModelObjects::representations;
use crate::ModelObjects::system_declarations;
use std::collections::HashMap;
use std::ptr;

pub fn make_input_enabled(
    component: &mut component::Component,
    sys_decls: &system_declarations::SystemDeclarations,
) {
    let dimension = *(component.get_declarations().get_dimension()) + 1;
    let len = dimension * dimension;
    let mut new_edges: Vec<component::Edge> = vec![];
    if let Some(inputs) = sys_decls
        .get_declarations()
        .get_input_actions()
        .get(component.get_name())
    {
        for location in component.get_locations() {
            let mut zone = [0; 1000];
            let mut state = component::State {
                declarations: component.get_declarations().clone(),
                location,
            };

            lib::rs_dbm_init(&mut zone[0..len as usize], dimension);
            if let Some(invariant) = location.get_invariant() {
                constraint_applyer::apply_constraints_to_state(
                    invariant,
                    &mut state,
                    &mut zone[0..len as usize],
                    dimension,
                );
            }

            let mut full_federation_vec: Vec<*mut i32> = vec![];
            full_federation_vec.push(zone.as_mut_ptr());

            for input in inputs {
                let input_edges =
                    component.get_next_edges(location, input, component::SyncType::Input);
                let mut zones = vec![];

                for edge in input_edges {
                    let mut guard_zone = zone.clone();
                    let has_inv = if let Some(target_invariant) = component
                        .get_location_by_name(edge.get_target_location())
                        .get_invariant()
                    {
                        constraint_applyer::apply_constraints_to_state(
                            target_invariant,
                            &mut state,
                            &mut guard_zone[0..len as usize],
                            dimension,
                        )
                    } else {
                        false
                    };

                    if let Some(_) = edge.get_update() {
                        let update_clocks = edge.get_update_clocks();
                        for clock in update_clocks {
                            let clock_index =
                                component.get_declarations().clocks.get(clock).unwrap();
                            lib::rs_dbm_freeClock(
                                &mut guard_zone[0..len as usize],
                                dimension,
                                *clock_index,
                            );
                        }
                    }

                    let has_guard = if let Some(guard) = edge.get_guard() {
                        let res = constraint_applyer::apply_constraints_to_state(
                            guard,
                            &mut state,
                            &mut guard_zone[0..len as usize],
                            dimension,
                        );
                        res
                    } else {
                        false
                    };

                    if !has_inv && !has_guard {
                        zones.push(zone.clone());
                    } else {
                        zones.push(guard_zone);
                    }
                }

                let mut federation_vec = vec![];
                for zone in zones.iter_mut() {
                    federation_vec.push(zone.as_mut_ptr());
                }
                let result_federation_vec = lib::rs_dbm_fed_minus_fed(
                    &mut full_federation_vec,
                    &mut federation_vec,
                    dimension,
                );

                for fed_zone in result_federation_vec {
                    if fed_zone == ptr::null() {
                        continue;
                    }
                    new_edges.push(component::Edge {
                        source_location: location.get_id().to_string(),
                        target_location: location.get_id().to_string(),
                        sync_type: component::SyncType::Input,
                        guard: build_guard_from_zone(
                            fed_zone,
                            dimension,
                            component.get_declarations().get_clocks(),
                        ),
                        update: None,
                        sync: input.to_string(),
                    });
                }
            }
        }
    }
    component.add_input_edges(&mut new_edges);
}

fn build_guard_from_zone(
    zone: *const i32,
    dimension: u32,
    clocks: &HashMap<String, u32>,
) -> Option<representations::BoolExpression> {
    let mut guards: Vec<representations::BoolExpression> = vec![];

    for (_, index) in clocks {
        let raw_upper = lib::rs_dbm_get_constraint_from_dbm_ptr(zone, dimension, *index, 0);
        let raw_lower = lib::rs_dbm_get_constraint_from_dbm_ptr(zone, dimension, 0, *index);

        // lower bound must be different from 1 (==0)
        if raw_lower != 1 {
            if lib::rs_raw_is_strict(raw_lower) {
                guards.push(representations::BoolExpression::LessT(
                    Box::new(representations::BoolExpression::Int(
                        (-1) * lib::rs_raw_to_bound(raw_lower),
                    )),
                    Box::new(representations::BoolExpression::Clock(*index)),
                ));
            } else {
                guards.push(representations::BoolExpression::LessEQ(
                    Box::new(representations::BoolExpression::Int(
                        (-1) * lib::rs_raw_to_bound(raw_lower),
                    )),
                    Box::new(representations::BoolExpression::Clock(*index)),
                ));
            }
        }

        if raw_upper != lib::DBM_INF {
            if lib::rs_raw_is_strict(raw_upper) {
                guards.push(representations::BoolExpression::LessT(
                    Box::new(representations::BoolExpression::Clock(*index)),
                    Box::new(representations::BoolExpression::Int(lib::rs_raw_to_bound(
                        raw_upper,
                    ))),
                ));
            } else {
                guards.push(representations::BoolExpression::LessEQ(
                    Box::new(representations::BoolExpression::Clock(*index)),
                    Box::new(representations::BoolExpression::Int(lib::rs_raw_to_bound(
                        raw_upper,
                    ))),
                ));
            }
        }
    }

    let res = build_guard_from_zone_helper(&mut guards);
    return match res {
        representations::BoolExpression::Bool(false) => None,
        _ => Some(res),
    };
}

fn build_guard_from_zone_helper(
    guards: &mut Vec<representations::BoolExpression>,
) -> representations::BoolExpression {
    let num_guards = guards.len();

    return if let Some(guard) = guards.pop() {
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
    };
}

fn get_inv_clocks<'a>(
    invariant: &'a representations::BoolExpression,
    component: &component::Component,
    clock_vec: &mut Vec<&'a str>,
) {
    match invariant {
        representations::BoolExpression::AndOp(left, right)
        | representations::BoolExpression::OrOp(left, right)
        | representations::BoolExpression::LessEQ(left, right)
        | representations::BoolExpression::GreatEQ(left, right)
        | representations::BoolExpression::EQ(left, right)
        | representations::BoolExpression::LessT(left, right)
        | representations::BoolExpression::GreatT(left, right) => {
            get_inv_clocks(left, component, clock_vec);
            get_inv_clocks(right, component, clock_vec);
        }
        representations::BoolExpression::Parentheses(inner) => {
            get_inv_clocks(inner, component, clock_vec);
        }
        representations::BoolExpression::Clock(_)
        | representations::BoolExpression::Bool(_)
        | representations::BoolExpression::Int(_) => {}
        representations::BoolExpression::VarName(varname) => {
            if component
                .get_declarations()
                .get_clocks()
                .contains_key(varname)
            {
                clock_vec.push(varname);
            }
        }
    }
}
