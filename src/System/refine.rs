use edbm::zones::OwnedFederation;
use log::{debug, info, log_enabled, trace, warn, Level};

use crate::DataTypes::{PassedStateList, PassedStateListExt, WaitingStateList};
use crate::ModelObjects::component::Transition;

use crate::ModelObjects::statepair::StatePair;
//use crate::ProtobufServer::services::query_response::{ConsistencyResult, DeterminismResult};
//use crate::ProtobufServer::services::query_response::{DeterminismResult, ConsistencyResult};
use crate::System::local_consistency::{ConsistencyResult, DeterminismResult};
use crate::TransitionSystems::{LocationTuple, TransitionSystemPtr};
use std::collections::HashSet;

#[allow(clippy::large_enum_variant)] //TODO: consider boxing the large fields to reduce the total size of the enum
pub enum RefinementResult {
    Success,
    Failure(RefinementFailure),
}

#[derive(Debug)]
pub enum RefinementFailure {
    NotDisjointAndNotSubset,
    NotDisjoint,
    NotSubset,                     //TODO: Add state that fails
    CutsDelaySolutions(StatePair), //TODO: StatePair or something else?
    InitialState(StatePair),
    EmptySpecification,
    EmptyImplementation,
    EmptyTransition2s(StatePair),
    NotEmptyResult(StatePair),
    Other, //TODO: More types of failures?
}

pub enum PreconditionsResult {
    Success,
    NotDisjointAndNotSubset,
    NotDisjoint,
    NotSubset,
    OtherFailure,
}

pub enum StatePairResult {
    Valid,
    EmptyTransition2s, //TODO: Better name? Based on description in comment
    NotEmptyResult,    //TODO: Better name? Based on description in comment
    CutsDelaySolutions,
}

/* impl StatePairResult {
    fn into(&self) -> RefinementResult {
        match self {
            StatePairResult::Valid => RefinementResult::Success,
            StatePairResult::EmptyTransition2s => RefinementResult::Failure(RefinementFailure::EmptyTransition2s),
            StatePairResult::NotEmptyResult => todo!(),
            StatePairResult::CutsDelaySolutions => todo!(),
        }
    }
} */

fn common_actions(
    sys1: &TransitionSystemPtr,
    sys2: &TransitionSystemPtr,
    is_input: bool,
) -> HashSet<String> {
    if is_input {
        sys2.get_input_actions()
    } else {
        sys1.get_output_actions()
    }
}

fn extra_actions(
    sys1: &TransitionSystemPtr,
    sys2: &TransitionSystemPtr,
    is_input: bool,
) -> HashSet<String> {
    if is_input {
        sys2.get_input_actions()
            .difference(&sys1.get_input_actions())
            .cloned()
            .collect()
    } else {
        sys1.get_output_actions()
            .difference(&sys2.get_output_actions())
            .cloned()
            .collect()
    }
}

struct RefinementContext<'a> {
    pub passed_list: PassedStateList,
    pub waiting_list: WaitingStateList,
    pub sys1: &'a TransitionSystemPtr,
    pub sys2: &'a TransitionSystemPtr,
}

impl<'a> RefinementContext<'a> {
    fn new(sys1: &'a TransitionSystemPtr, sys2: &'a TransitionSystemPtr) -> RefinementContext<'a> {
        RefinementContext {
            passed_list: PassedStateList::new(),
            waiting_list: WaitingStateList::new(),
            sys1,
            sys2,
        }
    }
}

pub fn check_refinement(sys1: TransitionSystemPtr, sys2: TransitionSystemPtr) -> RefinementResult {
    let mut context = RefinementContext::new(&sys1, &sys2);
    let dimensions = sys1.get_dim();
    debug!("Dimensions: {}", dimensions);

    //Firstly we check the preconditions
    match check_preconditions(&sys1, &sys2) {
        //TODO: Match on the returned value from check_preconditions
        PreconditionsResult::Success => (),
        PreconditionsResult::NotDisjointAndNotSubset => {
            warn!("Refinement failed - Systems are not disjoint and not subset");
            return RefinementResult::Failure(RefinementFailure::NotDisjointAndNotSubset);
        }
        PreconditionsResult::NotDisjoint => {
            warn!("Refinement failed - Systems not disjoint");
            return RefinementResult::Failure(RefinementFailure::NotDisjoint);
        }
        PreconditionsResult::NotSubset => {
            warn!("Refinement failed - Systems are not subset");
            return RefinementResult::Failure(RefinementFailure::NotSubset);
        }
        PreconditionsResult::OtherFailure => {
            //TODO: Change this
            warn!("Refinement failed - Preconditions not met");
            return RefinementResult::Failure(RefinementFailure::Other);
        }
    }

    // Common inputs and outputs
    let inputs = common_actions(&sys1, &sys2, true);
    let outputs = common_actions(&sys1, &sys2, false);

    info!(
        "Left inputs: {:?}, Left outputs: {:?}",
        sys1.get_input_actions(),
        sys1.get_output_actions()
    );

    info!(
        "Right inputs: {:?}, Right outputs; {:?}",
        sys2.get_input_actions(),
        sys2.get_output_actions()
    );

    // Extra inputs and outputs are ignored by default
    let extra_inputs = extra_actions(&sys1, &sys2, true);
    let extra_outputs = extra_actions(&sys1, &sys2, false);

    let initial_locations_1 = sys1.get_initial_location();
    let initial_locations_2 = sys2.get_initial_location();

    debug!("Extra inputs {:?}", extra_inputs);
    debug!("Extra outputs {:?}", extra_outputs);

    if initial_locations_1 == None {
        if initial_locations_2 == None {
            // Both are empty, so trivially true
            return RefinementResult::Success;
        }
        return RefinementResult::Failure(RefinementFailure::EmptyImplementation);
    }

    if initial_locations_2 == None {
        //The empty automata cannot implement
        return RefinementResult::Failure(RefinementFailure::EmptySpecification);
    }

    let initial_locations_1 = initial_locations_1.unwrap();
    let initial_locations_2 = initial_locations_2.unwrap();

    let mut initial_pair = StatePair::create(
        dimensions,
        initial_locations_1.clone(),
        initial_locations_2.clone(),
    );

    if !prepare_init_state(&mut initial_pair, initial_locations_1, initial_locations_2) {
        return RefinementResult::Failure(RefinementFailure::InitialState(initial_pair));
    }
    initial_pair.extrapolate_max_bounds(context.sys1, context.sys2);

    debug!("Initial {}", initial_pair);
    context.waiting_list.put(initial_pair);

    while !context.waiting_list.is_empty() {
        let curr_pair = context.waiting_list.pop().unwrap();
        trace!("Checking {}", curr_pair);

        context.passed_list.put(curr_pair.clone());
        for output in &outputs {
            let extra = extra_outputs.contains(output);

            let output_transition1 = sys1.next_outputs(curr_pair.get_locations1(), output);
            let output_transition2 = if extra {
                vec![Transition::new(curr_pair.get_locations2(), dimensions)]
            } else {
                sys2.next_outputs(curr_pair.get_locations2(), output)
            };

            match has_valid_state_pairs(
                &output_transition1,
                &output_transition2,
                &curr_pair,
                &mut context,
                true,
            ) {
                StatePairResult::Valid => trace!("Created state pairs for input {}", output),
                StatePairResult::EmptyTransition2s => {
                    log_refinement_check_failure(
                        &output_transition1,
                        &output_transition2,
                        &curr_pair,
                        &context,
                        output,
                        false,
                    );
                    return RefinementResult::Failure(RefinementFailure::EmptyTransition2s(
                        curr_pair,
                    ));
                }
                StatePairResult::NotEmptyResult => {
                    log_refinement_check_failure(
                        &output_transition1,
                        &output_transition2,
                        &curr_pair,
                        &context,
                        output,
                        false,
                    );
                    return RefinementResult::Failure(RefinementFailure::NotEmptyResult(curr_pair));
                }
                StatePairResult::CutsDelaySolutions => {
                    log_refinement_check_failure(
                        &output_transition1,
                        &output_transition2,
                        &curr_pair,
                        &context,
                        output,
                        false,
                    );
                    return RefinementResult::Failure(RefinementFailure::CutsDelaySolutions(
                        curr_pair,
                    ));
                }
            }
        }

        for input in &inputs {
            let extra = extra_inputs.contains(input);

            let input_transitions1 = if extra {
                vec![Transition::new(curr_pair.get_locations1(), dimensions)]
            } else {
                sys1.next_inputs(curr_pair.get_locations1(), input)
            };

            let input_transitions2 = sys2.next_inputs(curr_pair.get_locations2(), input);

            match has_valid_state_pairs(
                &input_transitions2,
                &input_transitions1,
                &curr_pair,
                &mut context,
                false,
            ) {
                StatePairResult::Valid => trace!("Created state pairs for input {}", input),
                StatePairResult::EmptyTransition2s => {
                    log_refinement_check_failure(
                        &input_transitions1,
                        &input_transitions2,
                        &curr_pair,
                        &context,
                        input,
                        true,
                    );
                    return RefinementResult::Failure(RefinementFailure::EmptyTransition2s(
                        curr_pair,
                    ));
                }
                StatePairResult::NotEmptyResult => {
                    log_refinement_check_failure(
                        &input_transitions1,
                        &input_transitions2,
                        &curr_pair,
                        &context,
                        input,
                        true,
                    );
                    return RefinementResult::Failure(RefinementFailure::NotEmptyResult(curr_pair));
                }
                StatePairResult::CutsDelaySolutions => {
                    log_refinement_check_failure(
                        &input_transitions1,
                        &input_transitions2,
                        &curr_pair,
                        &context,
                        input,
                        true,
                    );
                    return RefinementResult::Failure(RefinementFailure::CutsDelaySolutions(
                        curr_pair,
                    ));
                }
            }
        }
    }
    info!("Refinement check passed");
    if log_enabled!(Level::Debug) {
        debug!("With relation:");
        print_relation(&context.passed_list);
    }

    RefinementResult::Success
}

fn log_refinement_check_failure(
    transitions1: &Vec<Transition>,
    transitions2: &Vec<Transition>,
    curr_pair: &StatePair,
    context: &RefinementContext,
    action: &String,
    is_input: bool,
) {
    let action_type = if is_input {
        String::from("Input")
    } else {
        String::from("Output")
    };
    info!("Refinement check failed for {} {:?}", action_type, action);
    if log_enabled!(Level::Debug) {
        debug!("Transitions1:");
        for t in transitions1 {
            debug!("{}", t);
        }
        debug!("Transitions2:");
        for t in transitions2 {
            debug!("{}", t);
        }
        debug!("Current pair: {}", curr_pair);
        debug!("Relation:");
        print_relation(&context.passed_list);
    }
}

fn print_relation(passed_list: &PassedStateList) {
    let verbose = false;

    let mut sorted_keys: Vec<_> = passed_list.keys().collect();
    sorted_keys.sort_by_key(|(a, b)| format!("1:{}, 2:{}", a, b));
    for (id1, id2) in sorted_keys {
        let zones = passed_list.zones(&(id1.clone(), id2.clone()));

        debug!(
            "{}",
            if zones.len() != 1 {
                format!("1:{} 2:{} {} zones", id1, id2, zones.len())
            } else if verbose {
                format!("1:{} 2:{} {} zone", id1, id2, zones[0])
            } else {
                format!("1:{} 2:{}", id1, id2)
            }
        );
    }
}

fn has_valid_state_pairs(
    transitions1: &[Transition],
    transitions2: &[Transition],
    curr_pair: &StatePair,
    context: &mut RefinementContext,
    is_state1: bool,
) -> StatePairResult {
    let (fed1, fed2) = get_guard_fed_for_sides(transitions1, transitions2, curr_pair, is_state1);

    // If there are no valid transition1s, continue
    if fed1.is_empty() {
        return StatePairResult::Valid;
    }

    // If there are (valid) transition1s but no transition2s there are no valid pairs
    if fed2.is_empty() {
        trace!("Empty transition2s");
        return StatePairResult::EmptyTransition2s;
    };

    let result_federation = fed1.subtraction(&fed2);

    // If the entire zone of transition1s cannot be matched by transition2s
    if !result_federation.is_empty() {
        return StatePairResult::NotEmptyResult;
    }

    // Finally try to create the pairs
    let res = try_create_new_state_pairs(transitions1, transitions2, curr_pair, context, is_state1);

    match res {
        BuildResult::Success => StatePairResult::Valid,
        //TODO: Should we do something with this?
        BuildResult::Failure => StatePairResult::CutsDelaySolutions,
    }
}

fn get_guard_fed_for_sides(
    transitions1: &[Transition],
    transitions2: &[Transition],
    curr_pair: &StatePair,
    is_state1: bool,
) -> (OwnedFederation, OwnedFederation) {
    let dim = curr_pair.ref_zone().dim();

    let pair_zone = curr_pair.ref_zone();
    trace!("Zone: {}", pair_zone);
    //create guard zones left
    let mut feds = OwnedFederation::empty(dim);
    trace!("{}", if is_state1 { "Left:" } else { "Right:" });
    for transition in transitions1 {
        trace!("{}", transition);
        feds += transition.get_allowed_federation();
    }
    let fed1 = feds.intersection(pair_zone);
    trace!("{}", fed1);

    trace!("{}", if is_state1 { "Right:" } else { "Left:" });
    //Create guard zones right
    let mut feds = OwnedFederation::empty(dim);
    for transition in transitions2 {
        trace!("{}", transition);
        feds += transition.get_allowed_federation();
    }
    let fed2 = feds.intersection(pair_zone);
    trace!("{}", fed2);

    (fed1, fed2)
}

enum BuildResult {
    Success,
    Failure,
}

/// Returns a failure if the new state pairs cut delay solutions, otherwise returns success
fn try_create_new_state_pairs(
    transitions1: &[Transition],
    transitions2: &[Transition],
    curr_pair: &StatePair,
    context: &mut RefinementContext,
    is_state1: bool,
) -> BuildResult {
    for transition1 in transitions1 {
        for transition2 in transitions2 {
            if let BuildResult::Failure =
                build_state_pair(transition1, transition2, curr_pair, context, is_state1)
            {
                return BuildResult::Failure;
            }
        }
    }

    BuildResult::Success
}

fn build_state_pair(
    transition1: &Transition,
    transition2: &Transition,
    curr_pair: &StatePair,
    context: &mut RefinementContext,
    is_state1: bool,
) -> BuildResult {
    //Creates new state pair
    let mut new_sp: StatePair = curr_pair.clone();
    //Creates DBM for that state pair
    let mut new_sp_zone = new_sp.take_zone();
    //Apply guards on both sides
    let (locations1, locations2) = new_sp.get_mut_states(is_state1);

    //Applies the left side guards and checks if zone is valid
    new_sp_zone = transition1.apply_guards(new_sp_zone);
    //Applies the right side guards and checks if zone is valid
    new_sp_zone = transition2.apply_guards(new_sp_zone);

    // Continue to the next transition pair if the zone is empty
    if new_sp_zone.is_empty() {
        return BuildResult::Success;
    }

    //Apply updates on both sides
    new_sp_zone = transition1.apply_updates(new_sp_zone);
    new_sp_zone = transition2.apply_updates(new_sp_zone);

    //Perform a delay on the zone after the updates were applied
    new_sp_zone = new_sp_zone.up();

    //Update locations in states

    transition1.move_locations(locations1);
    transition2.move_locations(locations2);

    // Apply invariants on the left side of relation
    let (left_loc, right_loc) = if is_state1 {
        //(locations2, locations1)
        (locations1, locations2)
    } else {
        //(locations1, locations2)
        (locations2, locations1)
    };

    new_sp_zone = left_loc.apply_invariants(new_sp_zone);

    // Clone the zone before applying right side invariants
    let s_invariant = new_sp_zone.clone();

    // Apply right side invariants on the zone
    new_sp_zone = right_loc.apply_invariants(new_sp_zone);

    // Continue to the next transition pair if the newly built zones are empty
    if new_sp_zone.is_empty() || s_invariant.is_empty() {
        return BuildResult::Success;
    }

    // inv_s = x<10, inv_t = x>2 -> t cuts solutions but not delays, so it is fine and we can call down:
    let t_invariant = new_sp_zone.clone().down();

    // Check if the invariant of T (right) cuts delay solutions from S (left) and if so, report failure
    if !(s_invariant.subset_eq(&t_invariant)) {
        return BuildResult::Failure;
    }

    new_sp.set_zone(new_sp_zone);

    new_sp.extrapolate_max_bounds(context.sys1, context.sys2);

    if !context.passed_list.has(&new_sp) && !context.waiting_list.has(&new_sp) {
        debug!("New state {}", new_sp);

        context.waiting_list.put(new_sp);
    }

    BuildResult::Success
}

fn prepare_init_state(
    initial_pair: &mut StatePair,
    initial_locations_1: LocationTuple,
    initial_locations_2: LocationTuple,
) -> bool {
    let mut sp_zone = initial_pair.take_zone();
    sp_zone = initial_locations_1.apply_invariants(sp_zone);
    sp_zone = initial_locations_2.apply_invariants(sp_zone);

    initial_pair.set_zone(sp_zone);

    !initial_pair.ref_zone().is_empty()
}

fn check_preconditions(
    sys1: &TransitionSystemPtr,
    sys2: &TransitionSystemPtr,
) -> PreconditionsResult {
    if let (ConsistencyResult::Failure(_), DeterminismResult::Failure(_)) = sys2.precheck_sys_rep(){
        return PreconditionsResult::OtherFailure;
    }else if let (ConsistencyResult::Failure(_), DeterminismResult::Empty) = sys2.precheck_sys_rep(){
        return PreconditionsResult::OtherFailure;
    }else if let (ConsistencyResult::Failure(_), DeterminismResult::Empty) = sys1.precheck_sys_rep(){
        return  PreconditionsResult::OtherFailure;
    }else if let (ConsistencyResult::Failure(_), DeterminismResult::Failure(_)) = sys1.precheck_sys_rep(){
        return  PreconditionsResult::OtherFailure;
    }
    let s_outputs = sys1.get_output_actions();
    let t_outputs = sys2.get_output_actions();

    let s_inputs = sys1.get_input_actions();
    let t_inputs = sys2.get_input_actions();

    let disjoint = s_inputs.is_disjoint(&t_outputs) && t_inputs.is_disjoint(&s_outputs);

    let subset = s_inputs.is_subset(&t_inputs) && t_outputs.is_subset(&s_outputs);

    debug!("Disjoint {disjoint}, subset {subset}");
    debug!("S i:{s_inputs:?} o:{s_outputs:?}");
    debug!("T i:{t_inputs:?} o:{t_outputs:?}");

    if !disjoint && !subset {
        PreconditionsResult::NotDisjointAndNotSubset
    } else if !subset {
        PreconditionsResult::NotSubset
    } else if !disjoint {
        PreconditionsResult::NotDisjoint
    } else {
        PreconditionsResult::Success
    }
}
