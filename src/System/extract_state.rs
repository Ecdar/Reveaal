use edbm::zones::OwnedFederation;

use crate::extract_system_rep::SystemRecipe;
use crate::EdgeEval::constraint_applyer::apply_constraints_to_state;
use crate::ModelObjects::representations::{BoolExpression, QueryExpression};
use crate::ModelObjects::state::State;
use crate::TransitionSystems::{CompositionType, LocationID, LocationTree, TransitionSystemPtr};
use std::slice::Iter;

/// This function takes a [`QueryExpression`], the system recipe, and the transitionsystem -
/// to define a state from the [`QueryExpression`] which has clocks and locations.
/// The [`QueryExpression`] looks like this: `State(Vec<LocName>, Option<BoolExpression>)`.
/// `state_query` is the part of the query that describes the location and the clock constraints of the state.
/// `machine` defines which operators is used to define the transistion system.
/// `system` is the transition system.
pub fn get_state(
    state_query: &QueryExpression,
    machine: &SystemRecipe,
    system: &TransitionSystemPtr,
) -> Result<State, String> {
    if let QueryExpression::State(loc, clock) = state_query {
        let mut locations: Vec<&str> = Vec::new();

        for location in loc {
            match location.as_ref() {
                QueryExpression::LocName(name) => locations.push(name),
                _ => unreachable!(),
            };
        }
        let mut state = State::create(
            build_location_tree(&mut locations.iter(), machine, system)?,
            create_zone_given_constraints(clock.as_deref(), system)?,
        );
        state.apply_invariants();
        Ok(state)
    } else {
        Err(format!(
            "The following information \"{}\" could not be used to create a State",
            state_query.pretty_string()
        ))
    }
}

fn create_zone_given_constraints(
    constraints: Option<&BoolExpression>,
    system: &TransitionSystemPtr,
) -> Result<OwnedFederation, String> {
    constraints
        .map_or_else(
            || Ok(OwnedFederation::universe(system.get_dim())),
            |clock| {
                apply_constraints_to_state(
                    clock,
                    &system.get_combined_decls(),
                    OwnedFederation::universe(system.get_dim()),
                )
            },
        )
        .map_err(|clock| format!("Clock {} does not exist in the transition system", clock))
}

fn build_location_tree(
    locations: &mut Iter<&str>,
    machine: &SystemRecipe,
    system: &TransitionSystemPtr,
) -> Result<LocationTree, String> {
    match machine {
        SystemRecipe::Composition(left, right) => {
            let (left_system, right_system) = system.get_children();
            Ok(LocationTree::compose(
                &build_location_tree(locations, left, left_system)?,
                &build_location_tree(locations, right, right_system)?,
                CompositionType::Composition,
            ))
        }
        SystemRecipe::Conjunction(left, right) => {
            let (left_system, right_system) = system.get_children();
            Ok(LocationTree::compose(
                &build_location_tree(locations, left, left_system)?,
                &build_location_tree(locations, right, right_system)?,
                CompositionType::Conjunction,
            ))
        }
        SystemRecipe::Quotient(left, right, ..) => {
            let (left_system, right_system) = system.get_children();
            Ok(LocationTree::merge_as_quotient(
                &build_location_tree(locations, left, left_system)?,
                &build_location_tree(locations, right, right_system)?,
            ))
        }
        SystemRecipe::Component(component) => match locations.next().unwrap().trim() {
            // It is ensured .next() will not give a None, since the number of location is same as number of component. This is also being checked in validate_reachability_input function, that is called before get_state
            "_" => Ok(LocationTree::build_any_location_tree()),
            str => system
                .get_location(&LocationID::Simple(str.to_string()))
                .ok_or(format!(
                    "Location {} does not exist in the component {}",
                    str,
                    component.get_name()
                )),
        },
    }
}
