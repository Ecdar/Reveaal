use crate::DBMLib::dbm::Zone;
use crate::ModelObjects::component::{Component, DecoratedLocation, Edge, State};
use crate::ModelObjects::representations::SystemRepresentation;
use crate::ModelObjects::system::UncachedSystem;
use crate::ModelObjects::system_declarations::SystemDeclarations;

pub fn consistency_check(system: &mut UncachedSystem, sys_decls: &SystemDeclarations) -> bool {
    if let SystemRepresentation::Composition(_, _) = system.borrow_representation() {
        check_consistency_repr(system.borrow_representation(), sys_decls)
    } else {
        let old_offset = system.set_clock_offset(0);

        let faulty_states = find_inconsistent_states(system, sys_decls);
        let result = reachability_analysis(system, sys_decls, faulty_states);

        system.set_clock_offset(old_offset);

        result
    }
}

fn check_consistency_repr(
    system_repr: &SystemRepresentation,
    sys_decls: &SystemDeclarations,
) -> bool {
    //Apply optimisation: If A and B are consistent in system  A || B = C then C is also consistent
    if let SystemRepresentation::Composition(left, right) = system_repr {
        check_consistency_repr(left.as_ref(), sys_decls)
            && check_consistency_repr(right.as_ref(), sys_decls)
    } else {
        let mut system = UncachedSystem::create(system_repr.clone());
        system.set_clock_offset(0);

        let faulty_states = find_inconsistent_states(&system, sys_decls);
        reachability_analysis(&system, sys_decls, faulty_states)
    }
}

fn find_inconsistent_states<'a>(
    system: &'a UncachedSystem,
    sys_decls: &SystemDeclarations,
) -> Vec<State<'a>> {
    let mut inconsistent_states = vec![];
    let dimensions = system.borrow_representation().get_dimensions();

    'location_for: for location in system.get_locations() {
        let mut zone = Zone::init(dimensions);

        for comp_loc in &location {
            if !comp_loc.apply_invariant(&mut zone) {
                continue 'location_for; //Skip invalid zones
            }
        }

        let state = State {
            decorated_locations: location,
            zone,
        };

        let cannot_delay_forever = !state.zone.canDelayIndefinitely();
        let no_open_output = system
            .collect_open_output_transitions(sys_decls, &state)
            .is_empty();

        if no_open_output && cannot_delay_forever {
            inconsistent_states.push(state);
        }
    }

    inconsistent_states
}

fn reachability_analysis<'a>(
    system: &'a UncachedSystem,
    sys_decls: &SystemDeclarations,
    mut faulty_states: Vec<State<'a>>,
) -> bool {
    let dimensions = system.borrow_representation().get_dimensions();
    let max_bounds = system.get_max_bounds(dimensions);
    println!("There are {} initially bad states", faulty_states.len());

    let start_state = State::from_location(system.get_initial_locations(), dimensions).unwrap();

    let mut index = 0;
    while index < faulty_states.len() {
        let state = faulty_states[index].clone();
        index += 1;

        println!("{}", state.zone);

        //If we can reach the error state from the initial state we fail the consistency check
        if start_state.is_subset_of(&state) {
            return false;
        }

        let inputs = system.collect_previous_inputs(sys_decls, &state);

        'input_for: for input in &inputs {
            //Apply the transition backwards and move from state to previous_state
            let mut previous_state = state.clone();
            input.apply_updates(
                &previous_state.decorated_locations,
                &mut previous_state.zone,
            );
            if input.apply_guards(
                &previous_state.decorated_locations,
                &mut previous_state.zone,
            ) {
                input.move_locations_backwards(&mut previous_state.decorated_locations);
                if input.apply_invariants(
                    &mut previous_state.decorated_locations,
                    &mut previous_state.zone,
                ) {
                    //Ignore invalid states
                    continue;
                }
            }

            //check if strengthend (Guard or invariant)?! can exclude transition

            //If previous_state has already been deemed faulty skip this transition
            for faulty_state in &faulty_states {
                if previous_state.is_subset_of(faulty_state) {
                    continue 'input_for;
                }
            }

            //If this is a new faulty state, expand the state's zone to include all times that can reach this fault
            previous_state.zone.down();
            if !input.apply_invariants(
                &mut previous_state.decorated_locations,
                &mut previous_state.zone,
            ) {
                panic!("This should never fail as we already checked for earlier");
            }
            previous_state.zone.extrapolate_max_bounds(&max_bounds);
            faulty_states.push(previous_state);
        }
    }

    true
}
