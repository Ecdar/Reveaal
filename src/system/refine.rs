use edbm::zones::OwnedFederation;
use log::{debug, info, log_enabled, trace, Level};

use crate::model_objects::{
    PassedStateList, PassedStateListExt, StatePair, Transition, WaitingStateList,
};
use crate::system::query_failures::RefinementFailure;
use crate::transition_systems::TransitionSystemPtr;
use std::collections::HashSet;
use std::rc::Rc;

use super::query_failures::{ActionFailure, RefinementPrecondition, RefinementResult};

const SUCCESS: RefinementResult = Ok(());

enum StatePairResult {
    Valid,
    EmptyTransition2s,
    NotEmptyResult,
    CutsDelaySolutions,
}

impl StatePairResult {
    fn check(
        &self,
        sys1: &TransitionSystemPtr,
        sys2: &TransitionSystemPtr,
        action: &str,
        curr_pair: &StatePair,
    ) -> RefinementResult {
        match self {
            StatePairResult::Valid => Ok(()),
            StatePairResult::EmptyTransition2s | StatePairResult::NotEmptyResult => {
                RefinementFailure::cannot_match(sys1.as_ref(), sys2.as_ref(), action, curr_pair)
            }
            StatePairResult::CutsDelaySolutions => {
                RefinementFailure::cuts_delays(sys1.as_ref(), sys2.as_ref(), action, curr_pair)
            }
        }
    }
}

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

/// Checks if sys1 refines sys2
pub fn check_refinement(sys1: TransitionSystemPtr, sys2: TransitionSystemPtr) -> RefinementResult {
    let mut context = RefinementContext::new(&sys1, &sys2);
    let dimensions = sys1.get_dim();
    debug!("Dimensions: {}", dimensions);

    //Firstly we check the preconditions
    check_preconditions(&sys1, &sys2)?;

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

    if initial_locations_1.is_none() {
        if initial_locations_2.is_none() {
            // Both are empty, so trivially true
            return SUCCESS;
        }
        return RefinementFailure::empty_child(sys1.as_ref(), sys2.as_ref(), true);
    }

    if initial_locations_2.is_none() {
        //The empty automata cannot implement
        return RefinementFailure::empty_child(sys1.as_ref(), sys2.as_ref(), false);
    }

    let initial_locations_1 = initial_locations_1.unwrap();
    let initial_locations_2 = initial_locations_2.unwrap();

    let mut initial_pair = StatePair::from_locations(
        dimensions,
        Rc::clone(&initial_locations_1),
        Rc::clone(&initial_locations_2),
    );

    if initial_pair.ref_zone().is_empty() {
        return RefinementFailure::empty_initial(sys1.as_ref(), sys2.as_ref());
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
                vec![Transition::without_id(
                    curr_pair.get_locations2(),
                    dimensions,
                )]
            } else {
                sys2.next_outputs(curr_pair.get_locations2(), output)
            };

            has_valid_state_pairs(
                &output_transition1,
                &output_transition2,
                &curr_pair,
                &mut context,
                true,
            )
            .check(&sys1, &sys2, output, &curr_pair)?;
        }

        for input in &inputs {
            let extra = extra_inputs.contains(input);

            let input_transitions1 = if extra {
                vec![Transition::without_id(
                    curr_pair.get_locations1(),
                    dimensions,
                )]
            } else {
                sys1.next_inputs(curr_pair.get_locations1(), input)
            };

            let input_transitions2 = sys2.next_inputs(curr_pair.get_locations2(), input);

            has_valid_state_pairs(
                &input_transitions2,
                &input_transitions1,
                &curr_pair,
                &mut context,
                false,
            )
            .check(&sys1, &sys2, input, &curr_pair)?;
        }
    }
    info!("Refinement check passed");
    if log_enabled!(Level::Debug) {
        debug!("With relation:");
        print_relation(&context.passed_list);
    }

    SUCCESS
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
    //Creates DBM for that state pair
    let mut new_sp_zone = curr_pair.clone_zone();

    //Apply guards on both sides

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
    let (locations1, locations2) = (
        Rc::clone(&transition1.target_locations),
        Rc::clone(&transition2.target_locations),
    );

    // Apply invariants on the left side of relation
    let (left_loc, right_loc) = if is_state1 {
        (locations1, locations2)
    } else {
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
    if !s_invariant.subset_eq(&t_invariant) {
        return BuildResult::Failure;
    }

    let mut new_sp = StatePair::new(left_loc, right_loc, Rc::new(new_sp_zone));
    new_sp.extrapolate_max_bounds(context.sys1, context.sys2);

    if !context.passed_list.has(&new_sp) && !context.waiting_list.has(&new_sp) {
        debug!("New state {}", new_sp);

        context.waiting_list.put(new_sp);
    }

    BuildResult::Success
}

fn check_preconditions(
    sys1: &TransitionSystemPtr,
    sys2: &TransitionSystemPtr,
) -> Result<(), Box<RefinementPrecondition>> {
    sys1.precheck_sys_rep()
        .map_err(|e| e.to_precondition(sys1.as_ref(), sys2.as_ref()))?;
    sys2.precheck_sys_rep()
        .map_err(|e| e.to_precondition(sys1.as_ref(), sys2.as_ref()))?;

    let s_outputs = sys1.get_output_actions();
    let t_outputs = sys2.get_output_actions();

    let s_inputs = sys1.get_input_actions();
    let t_inputs = sys2.get_input_actions();

    if !s_inputs.is_disjoint(&t_outputs) {
        ActionFailure::not_disjoint((sys1.as_ref(), s_inputs), (sys2.as_ref(), t_outputs))
            .map_err(|e| e.to_precondition(sys1.as_ref(), sys2.as_ref()))
    } else if !t_inputs.is_disjoint(&s_outputs) {
        ActionFailure::not_disjoint((sys2.as_ref(), t_inputs), (sys1.as_ref(), s_outputs))
            .map_err(|e| e.to_precondition(sys1.as_ref(), sys2.as_ref()))
    } else if !s_inputs.is_subset(&t_inputs) {
        ActionFailure::not_subset((sys1.as_ref(), s_inputs), (sys2.as_ref(), t_inputs))
            .map_err(|e| e.to_precondition(sys1.as_ref(), sys2.as_ref()))
    } else if !t_outputs.is_subset(&s_outputs) {
        ActionFailure::not_subset((sys2.as_ref(), t_outputs), (sys1.as_ref(), s_outputs))
            .map_err(|e| e.to_precondition(sys1.as_ref(), sys2.as_ref()))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::logging::setup_logger;
    use crate::system::query_failures::QueryResult;
    use crate::test_helpers::{json_run_query, xml_run_query};
    use test_case::test_case;

    const AG_PATH: &str = "samples/json/AG";
    const BIG_PATH: &str = "samples/json/BigRefinement";
    const CONJUNCTION_PATH: &str = "samples/json/Conjunction";
    const DELAY_PATH: &str = "samples/json/DelayAdd";
    const UNI_PATH: &str = "samples/json/EcdarUniversity";
    const UNSPEC_PATH: &str = "samples/json/Unspec";

    fn json_refinement_check(path: &str, query: &str) -> bool {
        #[cfg(feature = "logging")]
        let _ = setup_logger();

        let q = format!("refinement: {}", query);
        match json_run_query(path, q.as_str()).unwrap() {
            QueryResult::Refinement(Ok(())) => true,
            QueryResult::Refinement(Err(_)) => false,
            QueryResult::CustomError(err) => panic!("{}", err),
            _ => panic!("Not a refinement check"),
        }
    }

    #[test_case(AG_PATH, "A <= A"; "A refines itself (ag)")]
    #[test_case(AG_PATH, "G <= G"; "G refines itself")]
    #[test_case(AG_PATH, "Q <= Q"; "Q refines itself")]
    #[test_case(AG_PATH, "Imp <= Imp"; "Imp refines itself")]
    #[test_case(AG_PATH, "AA <= AA"; "AA refines itself (ag)")]
    #[test_case(AG_PATH, "Imp <= G"; "Imp refines G")]
    #[test_case(AG_PATH, "G <= Q"; "G refines Q")]
    #[test_case(AG_PATH, "Q <= G"; "Q refines G")]
    #[test_case(BIG_PATH, "Ref1 <= Ref1"; "Ref 1 refines itself")]
    #[test_case(BIG_PATH, "Comp1 <= Comp1"; "Comp 1 refines itself")]
    #[test_case(CONJUNCTION_PATH, "Test1 <= Test1"; "Test 1 refines itself")]
    #[test_case(CONJUNCTION_PATH, "Test2 <= Test2"; "Test 2 refines itself")]
    #[test_case(CONJUNCTION_PATH, "Test3 <= Test3"; "Test 3 refines itself")]
    #[test_case(CONJUNCTION_PATH, "Test4 <= Test4"; "Test 4 refines itself")]
    #[test_case(CONJUNCTION_PATH, "Test5 <= Test5"; "Test 5 refines itself")]
    #[test_case(CONJUNCTION_PATH, "Test1 && Test2 <= Test3"; "Test 1 and 2 refines Test 3")]
    #[test_case(CONJUNCTION_PATH, "Test2 && Test3 <= Test1"; "Test 2 and 3 refines Test 1")]
    #[test_case(CONJUNCTION_PATH, "Test1 && Test3 <= Test2"; "Test 1 and 3 refines Test 2")]
    #[test_case(CONJUNCTION_PATH, "Test1 && Test2 && Test4 <= Test5"; "Test 1, 2, and 4 refines Test 5")]
    #[test_case(CONJUNCTION_PATH, "Test3 && Test4 <= Test5"; "Test 3 and 4 refines Test 5")]
    #[test_case(CONJUNCTION_PATH, "Test6 && Test7 <= Test8"; "Test 6 and 7 refines Test 8")]
    #[test_case(CONJUNCTION_PATH, "Test9 && Test10 && Test11 <= Test12"; "Test 9, 10, and 11 refines Test 12")]
    #[test_case(UNI_PATH, "Adm2 <= Adm2"; "Adm2 refines self")]
    #[test_case(UNI_PATH, "HalfAdm1 <= HalfAdm1"; "HalfAdm1 refines self")]
    #[test_case(UNI_PATH, "HalfAdm2 <= HalfAdm2"; "HalfAdm2 refines self")]
    #[test_case(UNI_PATH, "Administration <= Administration"; "Administration refines self")]
    #[test_case(UNI_PATH, "Machine <= Machine"; "Machine refines self")]
    #[test_case(UNI_PATH, "Researcher <= Researcher"; "Researcher refines self")]
    #[test_case(UNI_PATH, "Spec <= Spec"; "Spec refines self")]
    #[test_case(UNI_PATH, "Machine3 <= Machine3"; "Machine3 refines self")]
    #[test_case(UNI_PATH, "Machine3 <= Machine"; "Machine3 refines Machine")]
    #[test_case(UNI_PATH, "Administration || Researcher || Machine <= Spec"; "Adm || Researcher || Machine refines spec")]
    #[test_case(UNI_PATH, "Administration <= Spec // Researcher // Machine"; "Adm refines big quotient")]
    #[test_case(UNI_PATH, "Researcher <= Spec // Administration // Machine"; "Researcher refines big quotient (adm)")]
    #[test_case(UNI_PATH, "Machine <= Spec // Administration // Researcher"; "Machine refines big quotient (adm)")]
    #[test_case(UNI_PATH, "Administration || Researcher <= Spec // Machine"; "Adm || researcher refines spec, machine quotient")]
    #[test_case(UNI_PATH, "Researcher || Machine <= Spec // Administration"; "Researcher || machine refines spec, adm quotient refines both halves")]
    #[test_case(UNI_PATH, "Machine || Administration <= Spec // Researcher"; "Machine || Adm refines spec, researcher quotient")]
    #[test_case(UNI_PATH, "Administration || Researcher || Machine <=  Administration || Researcher || Machine"; "Large comp refines self")]
    #[test_case(UNI_PATH, "HalfAdm1 && HalfAdm2 <= Adm2"; "Both halves refines Adm2")]
    #[test_case(UNI_PATH, "Adm2 <= HalfAdm1 && HalfAdm2"; "Adm2 refines both halves")]
    #[test_case(UNSPEC_PATH, "A <= A"; "A refines itself (unspec)")]
    #[test_case(UNSPEC_PATH, "AA <= AA"; "AA refines itself (unspec)")]
    #[test_case(UNSPEC_PATH, "B <= B"; "B refines itself")]
    fn test_refinement(path: &str, query: &str) {
        assert!(json_refinement_check(path, query));
    }

    #[test_case(AG_PATH, "A||G <= A||Imp"; "A||G not refines A||Imp")]
    #[test_case(AG_PATH, "G <= Imp"; "G not refines Imp")]
    #[test_case(AG_PATH, "Q <= Imp"; "Q not refines Imp")]
    #[test_case(BIG_PATH, "Ref1 <= Comp1"; "Ref 1 not refine Comp 1")]
    #[test_case(BIG_PATH, "Comp1 <= Ref1"; "Comp 1 not refine Ref 1")]
    #[test_case(DELAY_PATH, "A1 || A2 <= B"; "Both A's not refine B")]
    #[test_case(DELAY_PATH, "C1 <= C2"; "C1 not refine C2")]
    #[test_case(DELAY_PATH, "D1 <= D2"; "D1 not refine D2")]
    #[test_case(UNI_PATH, "Spec <= Administration"; "Spec not refine Administration")]
    #[test_case(UNI_PATH, "Spec <= Machine"; "Spec not refine Machine")]
    #[test_case(UNI_PATH, "Spec <= Researcher"; "Spec not refine Researcher")]
    #[test_case(UNI_PATH, "Spec <= Machine3"; "Spec not refine Machine3")]
    #[test_case(UNI_PATH, "Machine <= Spec // Adm2 // Researcher"; "Machine not refine big quotient (adm2)")]
    #[test_case(UNI_PATH, "Machine <= Administration"; "Machine not refine Administration")]
    #[test_case(UNI_PATH, "Machine <= Researcher"; "Machine not refine Researcher")]
    #[test_case(UNI_PATH, "Machine <= Spec"; "Machine not refine Spec")]
    #[test_case(UNI_PATH, "Machine <= Machine3"; "Machine not refine Machine3")]
    #[test_case(UNI_PATH, "Machine || Adm2 <= Spec // Researcher"; "Machine || Adm2 refines spec, researcher quotient not refine quotient")]
    #[test_case(UNI_PATH, "(HalfAdm1 && HalfAdm2) || Researcher || Machine <= Spec"; "Both halves of adm and other components not refine spec")]
    #[test_case(UNI_PATH, "Machine3 <= Administration"; "Machine3 not refine Administration")]
    #[test_case(UNI_PATH, "Machine3 <= Researcher"; "Machine3 not refine researcher")]
    #[test_case(UNI_PATH, "Machine3 <= Spec"; "Machine3 not refine spec")]
    #[test_case(UNI_PATH, "Researcher <= Spec"; "Researcher not refines Spec")]
    #[test_case(UNI_PATH, "Researcher <= Machine"; "Researcher not refines Machine")]
    #[test_case(UNI_PATH, "Researcher <= Machine3"; "Researcher not refines Machine3")]
    #[test_case(UNI_PATH, "Researcher <= Spec // Adm2 // Machine"; "Researcher not refines big quotient (adm2)")]
    #[test_case(UNI_PATH, "Researcher || Machine <= Spec // Adm2"; "Researcher || Machine not refines spec, adm2 quotient")]
    #[test_case(UNI_PATH, "Administration <= Spec"; "Administration not refines Spec")]
    #[test_case(UNI_PATH, "Administration <= Machine"; "Administration not refines Machine")]
    #[test_case(UNI_PATH, "Administration <= Machine3"; "Administration not refines Machine3")]
    #[test_case(UNI_PATH, "Administration <= Researcher"; "Administration not refines Researcher")]
    #[test_case(UNI_PATH, "Adm2 || Researcher <= Spec // Machine"; "Adm2 || Researcher not refines spec, machine quotient")]
    #[test_case(UNI_PATH, "Adm2 <= Spec // Researcher // Machine"; "Adm2 not refines big quotient")]
    fn test_not_refinement(path: &str, query: &str) {
        assert!(!json_refinement_check(path, query));
    }

    const DELAY_PATH_XML: &str = "samples/xml/delayRefinement.xml";
    const LOOP_PATH_XML: &str = "samples/xml/loop.xml";
    const CONJUNCTION_PATH_XML: &str = "samples/xml/conjun.xml";
    const EXTRAPOLATE_PATH_XML: &str = "samples/xml/extrapolation_test.xml";
    const MISC_PATH_XML: &str = "samples/xml/misc_test.xml";

    fn xml_refinement_check(path: &str, query: &str) -> bool {
        #[cfg(feature = "logging")]
        let _ = setup_logger();

        let q = format!("refinement: {}", query);

        match xml_run_query(path, q.as_str()) {
            QueryResult::Refinement(Ok(())) => true,
            QueryResult::Refinement(Err(_)) => false,
            QueryResult::CustomError(err) => panic!("{}", err),
            _ => panic!("Not a refinement check"),
        }
    }

    #[test_case(MISC_PATH_XML, "GuardParan <= GuardParan"; "GuardParan refines itself")]
    #[test_case(EXTRAPOLATE_PATH_XML, "Inf <= Inf"; "Inf refines itself")]
    #[test_case(LOOP_PATH_XML, "SelfloopNonZeno <= SelfloopNonZeno"; "SelfLoop refines itself")]
    #[test_case(DELAY_PATH_XML, "T0 <= T0"; "T0 refines itself")]
    #[test_case(DELAY_PATH_XML, "T1 <= T1"; "T1 refines itself")]
    #[test_case(DELAY_PATH_XML, "T2 <= T2"; "T2 refines itself")]
    #[test_case(DELAY_PATH_XML, "T3 <= T3"; "T3 refines itself")]
    #[test_case(DELAY_PATH_XML, "T4 <= T4"; "T4 refines itself")]
    #[test_case(DELAY_PATH_XML, "T5 <= T5"; "T5 refines itself")]
    #[test_case(DELAY_PATH_XML, "T6 <= T6"; "T6 refines itself")]
    #[test_case(DELAY_PATH_XML, "T7 <= T7"; "T7 refines itself")]
    #[test_case(DELAY_PATH_XML, "T8 <= T8"; "T8 refines itself")]
    #[test_case(DELAY_PATH_XML, "T9 <= T9"; "T9 refines itself")]
    #[test_case(DELAY_PATH_XML, "T10 <= T10"; "T10 refines itself")]
    #[test_case(DELAY_PATH_XML, "T11 <= T11"; "T11 refines itself")]
    #[test_case(DELAY_PATH_XML, "C1 <= C1"; "C1 refines itself")]
    #[test_case(DELAY_PATH_XML, "C2 <= C2"; "C2 refines itself")]
    #[test_case(DELAY_PATH_XML, "F1 <= F1"; "F1 refines itself")]
    #[test_case(DELAY_PATH_XML, "F2 <= F2"; "F2 refines itself")]
    #[test_case(DELAY_PATH_XML, "F3 <= F3"; "F3 refines itself")]
    #[test_case(DELAY_PATH_XML, "N1 <= N1"; "N1 refines itself")]
    #[test_case(DELAY_PATH_XML, "N2 <= N2"; "N2 refines itself")]
    #[test_case(DELAY_PATH_XML, "N3 <= N3"; "N3 refines itself")]
    #[test_case(DELAY_PATH_XML, "N4 <= N4"; "N4 refines itself")]
    #[test_case(DELAY_PATH_XML, "D1 <= D1"; "D1 refines itself")]
    #[test_case(DELAY_PATH_XML, "D2 <= D2"; "D2 refines itself")]
    #[test_case(DELAY_PATH_XML, "K1 <= K1"; "K1 refines itself")]
    #[test_case(DELAY_PATH_XML, "K2 <= K2"; "K2 refines itself")]
    #[test_case(DELAY_PATH_XML, "K3 <= K3"; "K3 refines itself")]
    #[test_case(DELAY_PATH_XML, "K4 <= K4"; "K4 refines itself")]
    #[test_case(DELAY_PATH_XML, "K5 <= K5"; "K5 refines itself")]
    #[test_case(DELAY_PATH_XML, "K6 <= K6"; "K6 refines itself")]
    #[test_case(DELAY_PATH_XML, "P0 <= P0"; "P0 refines itself")]
    #[test_case(DELAY_PATH_XML, "P1 <= P1"; "P1 refines itself")]
    #[test_case(DELAY_PATH_XML, "P2 <= P2"; "P2 refines itself")]
    #[test_case(DELAY_PATH_XML, "P3 <= P3"; "P3 refines itself")]
    #[test_case(DELAY_PATH_XML, "P4 <= P4"; "P4 refines itself")]
    #[test_case(DELAY_PATH_XML, "P5 <= P5"; "P5 refines itself")]
    #[test_case(DELAY_PATH_XML, "P6 <= P6"; "P6 refines itself")]
    #[test_case(DELAY_PATH_XML, "P7 <= P7"; "P7 refines itself")]
    #[test_case(DELAY_PATH_XML, "L1 <= L1"; "L1 refines itself")]
    #[test_case(DELAY_PATH_XML, "L2 <= L2"; "L2 refines itself")]
    #[test_case(DELAY_PATH_XML, "L3 <= L3"; "L3 refines itself")]
    #[test_case(DELAY_PATH_XML, "L4 <= L4"; "L4 refines itself")]
    #[test_case(DELAY_PATH_XML, "L5 <= L5"; "L5 refines itself")]
    #[test_case(DELAY_PATH_XML, "L6 <= L6"; "L6 refines itself")]
    #[test_case(DELAY_PATH_XML, "L7 <= L7"; "L7 refines itself")]
    #[test_case(DELAY_PATH_XML, "Z1 <= Z1"; "Z1 refines itself")]
    #[test_case(DELAY_PATH_XML, "Z2 <= Z2"; "Z2 refines itself")]
    #[test_case(DELAY_PATH_XML, "Z3 <= Z3"; "Z3 refines itself")]
    #[test_case(DELAY_PATH_XML, "Z4 <= Z4"; "Z4 refines itself")]
    #[test_case(DELAY_PATH_XML, "Z5 <= Z5"; "Z5 refines itself")]
    #[test_case(DELAY_PATH_XML, "Z6 <= Z6"; "Z6 refines itself")]
    #[test_case(DELAY_PATH_XML, "Z7 <= Z7"; "Z7 refines itself")]
    #[test_case(DELAY_PATH_XML, "T0||T1||T2 <= T3"; "T0, 1, and 2 refines T3")]
    #[test_case(DELAY_PATH_XML, "T1||T2 <= T3"; "T1 and 2 refines T3")]
    #[test_case(DELAY_PATH_XML, "T4 <= T3"; "T4 refines T3")]
    #[test_case(DELAY_PATH_XML, "T6 <= T5"; "T6 refines T5")]
    #[test_case(DELAY_PATH_XML, "C1 <= C2"; "C1 refines C2")]
    #[test_case(DELAY_PATH_XML, "C2 <= C1"; "C2 refines C1")]
    #[test_case(DELAY_PATH_XML, "F1||F2 <= F3"; "F1 and 2 refines F3")]
    #[test_case(DELAY_PATH_XML, "N1 <= N2"; "N1 refines N2")]
    #[test_case(DELAY_PATH_XML, "D2 <= D1"; "D2 refines D1")]
    #[test_case(DELAY_PATH_XML, "P0 <= P1"; "P0 refines P1")]
    #[test_case(DELAY_PATH_XML, "P4 <= P5"; "P4 refines P5")]
    #[test_case(DELAY_PATH_XML, "P6 <= P7"; "P6 refines P7")]
    #[test_case(DELAY_PATH_XML, "Z1 <= Z2"; "Z1 refines Z2")]
    #[test_case(DELAY_PATH_XML, "Z3 <= Z4"; "Z3 refines Z4")]
    fn test_refinement_xml(path: &str, query: &str) {
        assert!(xml_refinement_check(path, query,));
    }

    #[test_case(CONJUNCTION_PATH_XML, "P0 && P1 <= P2"; "P0 and 1 not refine P2")]
    #[test_case(CONJUNCTION_PATH_XML, "P7 && P8 && P9 <= P10"; "P7, 8, and 9 not refine P10")]
    #[test_case(CONJUNCTION_PATH_XML, "P11 && P12 <= P13"; "P11 and 12 not refine P13")]
    #[test_case(DELAY_PATH_XML, "D1 <= D2"; "D1 not refine D2")]
    #[test_case(DELAY_PATH_XML, "K1 <= K2"; "K1 not refine K2")]
    #[test_case(DELAY_PATH_XML, "K3 <= K4"; "K3 not refine K4")]
    #[test_case(DELAY_PATH_XML, "K5 <= K6"; "K5 not refine K6")]
    #[test_case(DELAY_PATH_XML, "P2 <= P3"; "P2 not refine P3")]
    #[test_case(DELAY_PATH_XML, "L1||L2 <= L3"; "L1 and 2 not refine L3")]
    #[test_case(DELAY_PATH_XML, "Q1 <= Q2"; "Q1 not refine Q2")]
    #[test_case(DELAY_PATH_XML, "Q2 <= Q1"; "Q2 not refine Q1")]
    #[test_case(DELAY_PATH_XML, "T10 <= T11"; "T10 refines T11")]
    #[test_case(DELAY_PATH_XML, "T7 <= T8"; "T7 not refine T8")]
    #[test_case(DELAY_PATH_XML, "T9 <= T8"; "T9 not refine T8")]
    fn test_not_refinement_xml(path: &str, query: &str) {
        assert!(!xml_refinement_check(path, query,));
    }
}
