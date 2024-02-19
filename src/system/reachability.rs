use edbm::util::bounds::Bounds;
use edbm::zones::OwnedFederation;

use super::query_failures::PathFailure;
use super::specifics::SpecificPath;
use crate::model_objects::{Decision, State, Transition};
use crate::transition_systems::{LocationID, TransitionSystemPtr};
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

use super::query_failures::PathResult;

/// This holds the result of a reachability query
#[derive(Debug, Clone)]
pub struct Path {
    pub path: Vec<Decision>,
}

/// This holds which transition from which state (the `destination_state` of the `previous_sub_path`) it took to reach this state
struct SubPath {
    previous_sub_path: Option<Rc<SubPath>>,
    destination_state: State,
    transition: Option<(Transition, String)>,
}

fn is_trivially_unreachable(start_state: &State, end_state: &State) -> bool {
    // If any of the zones are empty
    if start_state.ref_zone().is_empty() || end_state.ref_zone().is_empty() {
        return true;
    }

    // If the end location has invariants and these do not have an intersection (overlap) with the zone of the end state of the query
    if let Some(invariants) = end_state.decorated_locations.get_invariants() {
        if !end_state.ref_zone().has_intersection(invariants) {
            return true;
        }
    }

    false
}

///# Find path
///
/// Returns a path from a start state to an end state in a transition system.
///
/// If it is reachable, it returns a path.
///
/// If it is not reachable, it returns None.
///
/// The start state can be omitted with None to use the start state of the transition system.
///
///## Checking if a state can reach another:
/// ```ignore
/// let is_reachable: bool = match find_path(Some(start_state), end_state, transition_system) {
///    Ok(result) => match result {
///        Some(path) => true,
///        None => false,
///    },
///    Err(string) => panic!(string),
/// };
/// ```
///
///## Omitting start state:
/// ```ignore
/// let is_reachable: bool = match find_path(None, end_state, transition_system) {
///    Ok(result) => match result {
///        Some(path) => true,
///        None => false,
///    },
///    Err(string) => panic!(string),
/// };
/// ```
pub fn find_path(
    start_state: State,
    end_state: State,
    system: &TransitionSystemPtr,
) -> Result<Path, PathFailure> {
    if is_trivially_unreachable(&start_state, &end_state) {
        return Err(PathFailure::Unreachable);
    }

    reachability_search(&start_state, &end_state, system)
}

pub fn find_specific_path(
    start_state: State,
    end_state: State,
    system: &TransitionSystemPtr,
) -> PathResult {
    find_path(start_state, end_state, system).map(|p| SpecificPath::from_path(&p, system.as_ref()))
}

/// Currently runs a BFS search on the transition system.
/// BFS is preferable to a DFS, as it reduces the chance of "Mistakes", meaning
/// having to revisit a state with a larger zone, forcing it to be readded ot the frontier.
/// Inspired from http://link.springer.com/10.1007/978-3-319-22975-1_9, see article for possible optimizations and more explanation.
fn reachability_search(
    start_state: &State,
    end_state: &State,
    system: &TransitionSystemPtr,
) -> Result<Path, PathFailure> {
    // Apply the invariant of the start state to the start state
    let mut start_state = start_state.clone();
    start_state.apply_invariants();

    // hashmap linking every location to all its current zones
    let mut visited_states: HashMap<LocationID, Vec<OwnedFederation>> = HashMap::new();

    // List of states that are to be visited
    let mut frontier_states: VecDeque<Rc<SubPath>> = VecDeque::new();

    let mut actions: Vec<String> = system.get_actions().into_iter().collect();
    actions.sort();

    // Push start state to visited state
    visited_states.insert(
        start_state.decorated_locations.id.clone(),
        vec![start_state.ref_zone().clone()],
    );

    // Push initial state to frontier
    frontier_states.push_back(Rc::new(SubPath {
        previous_sub_path: None,
        destination_state: start_state.clone(),
        transition: None,
    }));

    let target_bounds = end_state.ref_zone().get_bounds();

    // Take the first state from the frontier and explore it
    while let Some(sub_path) = frontier_states.pop_front() {
        if reached_end_state(&sub_path.destination_state, end_state) {
            return Ok(make_path(sub_path, start_state));
        }

        for action in &actions {
            for transition in &system.next_transitions(
                Rc::clone(&sub_path.destination_state.decorated_locations),
                action,
            ) {
                take_transition(
                    &sub_path,
                    transition,
                    &mut frontier_states,
                    &mut visited_states,
                    system,
                    action,
                    &target_bounds,
                );
            }
        }
    }
    // If nothing has been found, it is not reachable
    Err(PathFailure::Unreachable)
}

fn reached_end_state(cur_state: &State, end_state: &State) -> bool {
    cur_state
        .decorated_locations
        .compare_partial_locations(Rc::clone(&end_state.decorated_locations))
        && cur_state.ref_zone().has_intersection(end_state.ref_zone())
}

fn take_transition(
    sub_path: &Rc<SubPath>,
    transition: &Transition,
    frontier_states: &mut VecDeque<Rc<SubPath>>,
    visited_states: &mut HashMap<LocationID, Vec<OwnedFederation>>,
    system: &TransitionSystemPtr,
    action: &str,
    target_bounds: &Bounds,
) {
    let mut new_state = sub_path.destination_state.clone();
    if transition.use_transition(&mut new_state) {
        // Extrapolation ensures the bounds cant grow indefinitely, avoiding infinite loops
        // We must take the added bounds from the target state into account to ensure correctness
        new_state.extrapolate_max_bounds_with_extra_bounds(system.as_ref(), target_bounds);
        let new_location_id = &new_state.decorated_locations.id;
        let existing_zones = visited_states.entry(new_location_id.clone()).or_default();
        // If this location has not already been reached (explored) with a larger zone
        if !zone_subset_of_existing_zones(new_state.ref_zone(), existing_zones) {
            // Remove the smaller zones for this location in visited_states
            remove_existing_subsets_of_zone(new_state.ref_zone(), existing_zones);
            // Add the new zone to the list of zones for this location in visited_states
            visited_states
                .get_mut(new_location_id)
                .unwrap()
                .push(new_state.ref_zone().clone());
            // Add the new state to the frontier
            frontier_states.push_back(Rc::new(SubPath {
                previous_sub_path: Some(Rc::clone(sub_path)),
                destination_state: new_state,
                transition: Some((transition.clone(), action.to_string())),
            }));
        }
    }
}

/// Checks if this zone is redundant by being a subset of any other zone
fn zone_subset_of_existing_zones(
    new_state: &OwnedFederation,
    existing_states: &[OwnedFederation],
) -> bool {
    existing_states
        .iter()
        .any(|existing_state| new_state.subset_eq(existing_state))
}

/// Removes everything in existing_zones that is a subset of zone
fn remove_existing_subsets_of_zone(
    new_zone: &OwnedFederation,
    existing_zones: &mut Vec<OwnedFederation>,
) {
    existing_zones.retain(|existing_zone| !existing_zone.subset_eq(new_zone));
}
/// Makes the path from the last subpath
fn make_path(mut sub_path: Rc<SubPath>, start_state: State) -> Path {
    let mut path: Vec<(Transition, String)> = Vec::new();
    // Traverse the subpaths to make the path (from end location to start location)
    while sub_path.previous_sub_path.is_some() {
        path.push(sub_path.transition.clone().unwrap());
        sub_path = Rc::clone(sub_path.previous_sub_path.as_ref().unwrap());
    }
    // Reverse the path since the transitions are in reverse order (now from start location to end location)
    path.reverse();
    let mut state = start_state;

    let mut decisions = Vec::new();

    for (transition, action) in path {
        let decision = Decision::from_state_transition(state.clone(), &transition, action)
            .expect("If the transition is in a path, it should lead to a non-empty state");

        decisions.push(decision);

        transition.use_transition(&mut state);
    }

    Path { path: decisions }
}

#[cfg(test)]
mod tests {
    use crate::extract_system_rep::ExecutableQueryError;
    use crate::model_objects::{Declarations, Location, LocationType};
    use crate::system::query_failures::QueryResult;
    use crate::test_helpers::json_run_query;
    use crate::transition_systems::{CompositionType, LocationTree, TransitionID};
    use crate::{extract_system_rep, parse_queries, JsonProjectLoader};
    use std::rc::Rc;
    use test_case::test_case;

    const FOLDER_PATH: &str = "samples/json/EcdarUniversity";
    const PATH2: &str = "samples/json/AutomatonTestReachability";

    fn build_location_tree_helper(id: &str, location_type: LocationType) -> Rc<LocationTree> {
        LocationTree::simple(
            &Location {
                id: id.to_string(),
                invariant: None,
                location_type,
                urgency: "".to_string(),
            },
            &Declarations::empty(),
            0,
        )
    }

    #[test_case(TransitionID::Conjunction(
    Box::new(TransitionID::Simple("a".to_string())),
    Box::new(TransitionID::Simple("b".to_string()))
    ),
    vec![vec!(TransitionID::Simple("a".to_string())), vec!(TransitionID::Simple("b".to_string()))];
    "Simple conjunction")]
    #[test_case(TransitionID::Composition(
    Box::new(TransitionID::Simple("a".to_string())),
    Box::new(TransitionID::Simple("b".to_string()))
    ),
    vec![vec!(TransitionID::Simple("a".to_string())), vec!(TransitionID::Simple("b".to_string()))];
    "Simple composition")]
    #[test_case(TransitionID::Conjunction(
    Box::new(TransitionID::Conjunction(
    Box::new(TransitionID::Simple("a".to_string())),
    Box::new(TransitionID::Simple("b".to_string()))
    )),
    Box::new(TransitionID::Simple("c".to_string()))
    ),
    vec![vec!(TransitionID::Simple("a".to_string())), vec!(TransitionID::Simple("b".to_string())), vec!(TransitionID::Simple("c".to_string()))];
    "Simple nesting")]
    #[test_case(TransitionID::Composition(
    Box::new(TransitionID::Conjunction(
    Box::new(TransitionID::Simple("a".to_string())),
    Box::new(TransitionID::Composition(
    Box::new(TransitionID::Simple("b".to_string())),
    Box::new(TransitionID::Simple("c".to_string()))
    ))
    )),
    Box::new(TransitionID::Composition(
    Box::new(TransitionID::Simple("d".to_string())),
    Box::new(TransitionID::Simple("e".to_string()))
    ))
    ),
    vec![
    vec!(TransitionID::Simple("a".to_string())),
    vec!(TransitionID::Simple("b".to_string())),
    vec!(TransitionID::Simple("c".to_string())),
    vec!(TransitionID::Simple("d".to_string())),
    vec!(TransitionID::Simple("e".to_string()))];
    "Multiple conjunction and composition")]
    #[test_case(TransitionID::Quotient(
    vec!(TransitionID::Simple("a".to_string())),
    vec!(TransitionID::Simple("b".to_string()))
    ),
    vec![vec!(TransitionID::Simple("a".to_string())), vec!(TransitionID::Simple("b".to_string()))];
    "simple quotient")]
    #[test_case(TransitionID::Quotient(
    vec!(TransitionID::Simple("a".to_string()), TransitionID::Simple("b".to_string())),
    vec!(TransitionID::Simple("c".to_string()), TransitionID::Simple("d".to_string()), TransitionID::Simple("e".to_string()))
    ),
    vec![
    vec!(TransitionID::Simple("a".to_string()), TransitionID::Simple("b".to_string())),
    vec!(TransitionID::Simple("c".to_string()), TransitionID::Simple("d".to_string()), TransitionID::Simple("e".to_string()))];
    "quotient with vec")]
    #[test_case(
    TransitionID::Conjunction(
    Box::new(
    TransitionID::Quotient(
    vec![
    TransitionID::Conjunction(
    Box::new(TransitionID::Simple("a".to_string())),
    Box::new(TransitionID::Simple("b".to_string())),
    ),
    TransitionID::Conjunction(
    Box::new(TransitionID::Simple("c".to_string())),
    Box::new(TransitionID::Simple("d".to_string())),
    )
    ],
    vec![TransitionID::Simple("e".to_string()), TransitionID::Simple("f".to_string())]
    )
    ),
    Box::new(TransitionID::Simple("g".to_string()))
    ),
    vec![
    vec!(TransitionID::Simple("a".to_string()), TransitionID::Simple("c".to_string())),
    vec!(TransitionID::Simple("b".to_string()), TransitionID::Simple("d".to_string())),
    vec!(TransitionID::Simple("e".to_string()), TransitionID::Simple("f".to_string())),
    vec!(TransitionID::Simple("g".to_string()))];
    "Complex quotient")]
    fn get_leaves_returns_correct_vector(id: TransitionID, expected: Vec<Vec<TransitionID>>) {
        assert_eq!(id.get_leaves(), expected);
    }

    #[test_case("reachability: Adm2 @ true -> Adm2.L20";
    "partial start state and one component")]
    #[test_case("reachability: Adm2[1] && Adm2[2] @ Adm2[1].L21 -> Adm2[1].L20 && Adm2[2].L21";
    "partial start state and two components")]
    #[test_case("reachability: Adm2[1] && Adm2[2] && Adm2[3] && Adm2[4] && Adm2[5] @ Adm2[1].L20 -> Adm2[2].L21";
    "partial start state and complex composition")]
    fn query_parser_reject_partial_start(parser_input: &str) {
        let mut comp_loader =
            JsonProjectLoader::new_loader(String::from(FOLDER_PATH), crate::DEFAULT_SETTINGS)
                .to_comp_loader();
        // Make query:
        let q = parse_queries::parse_to_query(parser_input);
        let queries = q.first().unwrap();

        let result = extract_system_rep::create_executable_query(queries, &mut *comp_loader);
        if let Err(e) = result {
            assert_eq!(
                e,
                ExecutableQueryError::Custom(
                    "Start state is a partial state, which it must not be".to_string()
                )
            );
        } else {
            panic!("No error was returned")
        }
    }

    #[test_case(LocationTree::build_any_location_tree(),
    build_location_tree_helper("L9", LocationType::Normal);
    "_ == L9")]
    #[test_case(build_location_tree_helper("L0", LocationType::Initial),
    LocationTree::build_any_location_tree();
    "L0 == _")]
    #[test_case(build_location_tree_helper("L5", LocationType::Normal),
    build_location_tree_helper("L5", LocationType::Normal);
    "L5 == L5")]
    #[test_case(LocationTree::merge_as_quotient(build_location_tree_helper("L5", LocationType::Normal), LocationTree::build_any_location_tree()),
    LocationTree::merge_as_quotient(build_location_tree_helper("L5", LocationType::Normal), build_location_tree_helper("L1", LocationType::Normal));
    "L5//_ == L5//L1")]
    #[test_case(LocationTree::compose(build_location_tree_helper("L5", LocationType::Normal), LocationTree::build_any_location_tree(), CompositionType::Conjunction),
    LocationTree::compose(LocationTree::build_any_location_tree(), build_location_tree_helper("L1", LocationType::Normal), CompositionType::Conjunction);
    "L5&&_ == _&&L1")]
    #[test_case(LocationTree::compose(build_location_tree_helper("L7", LocationType::Normal), LocationTree::build_any_location_tree(), CompositionType::Composition),
    LocationTree::compose(build_location_tree_helper("L7", LocationType::Normal), build_location_tree_helper("L1", LocationType::Normal), CompositionType::Composition);
    "L7||_ == L7||L1")]
    #[test_case(LocationTree::compose(LocationTree::build_any_location_tree(), LocationTree::build_any_location_tree(), CompositionType::Composition),
    LocationTree::compose(build_location_tree_helper("L2", LocationType::Normal), build_location_tree_helper("L1", LocationType::Normal), CompositionType::Composition);
    "_||_ == L2||L1")]
    #[test_case(LocationTree::compose(LocationTree::compose(LocationTree::build_any_location_tree(), LocationTree::build_any_location_tree(), CompositionType::Composition),build_location_tree_helper("L2", LocationType::Normal), CompositionType::Composition),
    LocationTree::compose(LocationTree::compose(build_location_tree_helper("L2", LocationType::Normal), build_location_tree_helper("L1", LocationType::Normal), CompositionType::Composition),build_location_tree_helper("L2", LocationType::Normal), CompositionType::Composition);
    "_||_||L2 == L2||L1||L2")]
    #[test_case(build_location_tree_helper("L_35", LocationType::Normal),
    build_location_tree_helper("L_35", LocationType::Normal);
    "L_35 == L_35")]
    fn checks_cmp_locations_returns_true(loc1: Rc<LocationTree>, loc2: Rc<LocationTree>) {
        assert!(loc1.compare_partial_locations(loc2));
    }

    #[test_case(LocationTree::compose(build_location_tree_helper("L2", LocationType::Normal), build_location_tree_helper("L5", LocationType::Normal), CompositionType::Composition),
    LocationTree::compose(build_location_tree_helper("L2", LocationType::Normal), build_location_tree_helper("L1", LocationType::Normal), CompositionType::Composition);
    "L2||L5 != L2||L1")]
    #[test_case(LocationTree::merge_as_quotient(build_location_tree_helper("L2", LocationType::Normal), build_location_tree_helper("L6", LocationType::Normal)),
    LocationTree::compose(build_location_tree_helper("L2", LocationType::Normal), build_location_tree_helper("L1", LocationType::Normal), CompositionType::Composition);
    "L2//L6 != L2||L1")]
    #[test_case(LocationTree::merge_as_quotient(build_location_tree_helper("L7", LocationType::Normal), build_location_tree_helper("L6", LocationType::Normal)),
    LocationTree::compose(build_location_tree_helper("L2", LocationType::Normal), build_location_tree_helper("L1", LocationType::Normal), CompositionType::Conjunction);
    "L7//L6 != L2&&L1")]
    #[test_case(LocationTree::merge_as_quotient(build_location_tree_helper("L8", LocationType::Normal), LocationTree::build_any_location_tree()),
    LocationTree::compose(build_location_tree_helper("L2", LocationType::Normal), build_location_tree_helper("L1", LocationType::Normal), CompositionType::Conjunction);
    "L8//_ != L2&&L1")]
    #[test_case(LocationTree::build_any_location_tree(),
    LocationTree::compose(build_location_tree_helper("L6", LocationType::Normal), build_location_tree_helper("L1", LocationType::Normal), CompositionType::Conjunction);
    "_ != L6&&L1")]
    #[test_case(LocationTree::build_any_location_tree(),
    LocationTree::compose(LocationTree::build_any_location_tree(), LocationTree::build_any_location_tree(), CompositionType::Conjunction);
    "anylocation _ != _&&_")]
    #[test_case(LocationTree::compose(build_location_tree_helper("L2", LocationType::Normal), build_location_tree_helper("L4", LocationType::Normal), CompositionType::Conjunction),
    LocationTree::merge_as_quotient(build_location_tree_helper("L2", LocationType::Normal), build_location_tree_helper("L4", LocationType::Normal));
    "L2&&L4 != L2\\L4")]
    #[test_case(LocationTree::compose(LocationTree::compose(LocationTree::build_any_location_tree(), LocationTree::build_any_location_tree(), CompositionType::Composition),build_location_tree_helper("L2", LocationType::Normal), CompositionType::Conjunction),
    LocationTree::compose(LocationTree::compose(build_location_tree_helper("L2", LocationType::Normal), build_location_tree_helper("L1", LocationType::Normal), CompositionType::Composition),build_location_tree_helper("L2", LocationType::Normal), CompositionType::Composition);
    "_||_&&L2 == L2||L1||L2")]
    #[test_case(LocationTree::compose(LocationTree::compose(build_location_tree_helper("L2", LocationType::Normal), LocationTree::build_any_location_tree(), CompositionType::Composition),build_location_tree_helper("L2", LocationType::Normal), CompositionType::Conjunction),
    LocationTree::compose(LocationTree::build_any_location_tree(), LocationTree::build_any_location_tree(), CompositionType::Conjunction);
    "L2||_&&L2 == _&&_")]
    #[test_case(build_location_tree_helper("L7", LocationType::Normal),
    build_location_tree_helper("L5", LocationType::Normal);
    "L7 != L5")]
    #[test_case(LocationTree::merge_as_quotient(LocationTree::build_any_location_tree(), LocationTree::build_any_location_tree()),
    LocationTree::compose(build_location_tree_helper("L6", LocationType::Normal), build_location_tree_helper("L25", LocationType::Normal), CompositionType::Conjunction);
    "_//_ != L6&&L25")]
    #[test_case(build_location_tree_helper("_L1", LocationType::Normal),
    build_location_tree_helper("L1", LocationType::Normal);
    "_L1 != L1")]
    #[test_case(build_location_tree_helper("__", LocationType::Normal),
    build_location_tree_helper("L7", LocationType::Normal);
    "__ != L7")]
    fn checks_cmp_locations_returns_false(loc1: Rc<LocationTree>, loc2: Rc<LocationTree>) {
        assert!(!loc1.compare_partial_locations(loc2));
    }

    #[test_case(FOLDER_PATH, "reachability: Machine @ Machine.L5 && Machine.y<6 -> Machine.L4 && Machine.y<=6", true; "Existing states and with right clocks")]
    #[test_case(FOLDER_PATH, "reachability: Machine @ Machine.L5 -> Machine.L4 && Machine.y>7", false; "Exisiting locations but not possible with the clocks")]
    #[test_case(FOLDER_PATH, "reachability: Machine @ Machine.L4 && Machine.y<=6 -> Machine.L5 && Machine.y>=4", true; "Switched the two states and with right clocks")]
    #[test_case(FOLDER_PATH, "reachability: Machine @ Machine.L5 && Machine.y<1 -> Machine.L5 && Machine.y<2", true; "Same location, different clocks")]
    #[test_case(FOLDER_PATH, "reachability: Machine @ Machine.L5 -> Machine.L5", true; "Same location, no clocks")]
    #[test_case(FOLDER_PATH, "reachability: Machine @ Machine.L5 -> true", true; "Trivially reachable because the end state is true which means any location")]
    #[test_case(FOLDER_PATH, "reachability: Machine || Researcher @ Machine.L5 && Researcher.L6 -> Machine.L4 && Researcher.L9", true; "Composition between Machine & Researcher, with existing locations and not clocks")]
    #[test_case(FOLDER_PATH, "reachability: Machine || Researcher @  Machine.L5 && Researcher.U0 -> Machine.L5 && Researcher.L7", false; "No valid path from the two states")]
    #[test_case(FOLDER_PATH, "reachability: Researcher @ Researcher.U0 -> Researcher.L7", false; "No possible path between to locations, locations exists in Researcher")]
    #[test_case(FOLDER_PATH, "reachability: Machine || Researcher @ Machine.L5 && Researcher.L6 -> Machine.L4", true; "Machine || Researcher with Partial end state")]
    #[test_case(FOLDER_PATH, "reachability: Machine || Researcher @ Machine.L5 && Researcher.L6 -> Researcher.L9", true; "Machine || Researcher with Partial end state 2")]
    #[test_case(FOLDER_PATH, "reachability: Machine || Researcher @ Machine.L5 && Researcher.U0 -> Machine.L5", true; "Machine || Researcher reachable with partial end state")]
    #[test_case(FOLDER_PATH, "reachability: Machine || Researcher @ Machine.L5 && Researcher.U0 -> Machine.L4", true; "Machine || Researcher reachable with partial end state 2")]
    #[test_case(FOLDER_PATH, "reachability: Machine || Researcher @ Machine.L5 && Researcher.U0 -> Researcher.L7", false; "Machine || Researcher not reachable with partial end state")]
    #[test_case(FOLDER_PATH, "reachability: Researcher[1] && Researcher[2] @ init -> Researcher[1].L7", true; "Machine || Researcher with partial state reachable from intial")]
    #[test_case(FOLDER_PATH, "reachability: Researcher[1] && Researcher[2] @ Researcher[1].U0 && Researcher[2].U0 -> Researcher[1].U0 && Researcher[2].U0", true; "Trivially reachable")]
    #[test_case(FOLDER_PATH, "reachability: Researcher[1] && Researcher[2] @ Researcher[1].U0 && Researcher[2].U0 -> Researcher[1].U0 && Researcher[2].U0 && Researcher[1].x>5", true; "Trivially reachable but with clocks")]
    #[test_case(FOLDER_PATH, "reachability: Researcher[1] && Researcher[2] @ Researcher[1].U0 && Researcher[2].U0 -> Researcher[1].L6 && Researcher[2].U0", false; "Trivially unreachable")]
    #[test_case(FOLDER_PATH, "reachability: Researcher[1] && Researcher[2] @ Researcher[1].U0 && Researcher[2].U0 -> Researcher[2].U0", true; "Trivially reachable because _ is U0")]
    fn search_algorithm_returns_result_university(path: &str, query: &str, expected: bool) {
        match json_run_query(path, query).unwrap() {
            QueryResult::Reachability(path) => assert_eq!(path.is_ok(), expected),
            _ => panic!("Inconsistent query result, expected Reachability"),
        }
    }

    #[test_case(PATH2, "reachability: Component1 @ Component1.L1 -> Component1.L3", false; "False due to invariants")]
    #[test_case(PATH2, "reachability: Component2 @ Component2.L4 -> Component2.L5", false; "False due to invariants, like the other")]
    #[test_case(PATH2, "reachability: Component3 @ Component3.L6 -> Component3.L8", false; "False due to guards on the last transition")]
    #[test_case(PATH2, "reachability: Component1 @ Component1.L0 -> Component1.L2", true; "It is possible to travel from L0 to L2 without specifiying guards")]
    #[test_case(PATH2, "reachability: Component4 @ Component4.L9 -> Component4.L10", false; "False due to start state invariant and guard")]
    #[test_case(PATH2, "reachability: Component3 @ Component3.L6 -> Component3.L7", true; "It is possible to travel from L6 to L7 without specifiying guards")]
    #[test_case(PATH2, "reachability: Component3 @ Component3.L7 -> Component3.L8", true; "It is possible to travel from L7 to L8 without specifiying guards")]
    #[test_case(PATH2, "reachability: Component3 @ Component3.L6 -> Component3.L7 && Component3.x<5", false; "It is not possible to travel from L6 to L7 due to specified guards")]
    #[test_case(PATH2, "reachability: Component3 @ Component3.L7 && Component3.x>4 -> Component3.L8", false; "It is not possible to travel from L7 to L8 due to specified guards")]
    #[test_case(PATH2, "reachability: Component5 @ Component5.L11 -> Component5.L12", true; "It is possible to travel from L11 to L12 due to update")]
    #[test_case(PATH2, "reachability: Component6 @ Component6.L13 -> Component6.L15", true; "It is possible to travel from L13 to L15 due to the updates at L14")]
    #[test_case(PATH2, "reachability: Component7 @ Component7.L16 -> Component7.L19", true; "Overwrite state of location once to reach end state")]
    #[test_case(PATH2, "reachability: Component8 @ Component8.L20 -> Component8.L22", true; "Reset clock to reach end state")]
    #[test_case(PATH2, "reachability: Component7 @ Component7.L16 -> Component7.L19 && Component7.y<2", false; "Unreachable due to second clock")]
    #[test_case(PATH2, "reachability: Component3[1] && Component3[2] @ Component3[1].L6 && Component3[2].L6 -> Component3[1].L7 && Component3[2].L7", true; "Simple conjunction")]
    fn search_algorithm_returns_result(path: &str, query: &str, expected: bool) {
        match json_run_query(path, query).unwrap() {
            QueryResult::Reachability(path) => {
                assert_eq!(path.is_ok(), expected, "Final state is not reachable")
            }
            _ => panic!("Inconsistent query result, expected Reachability"),
        }
    }

    #[test_case(PATH2, "reachability: Component1 @ Component1.L0 -> Component1.L2", vec!["E3", "E2"]; "Path in Component1 from L0 to L2")]
    #[test_case(PATH2, "reachability: Component3 @ Component3.L6 -> Component3.L7", vec!["E5"]; "Path in Component3 from L6 to L7")]
    #[test_case(PATH2, "reachability: Component3 @ Component3.L7 -> Component3.L8", vec!["E6"]; "Path in Component3 from L7 to L8")]
    #[test_case(PATH2, "reachability: Component5 @ Component5.L11 -> Component5.L12", vec!["E8"]; "Path in Component5 from L11 to L12")]
    #[test_case(PATH2, "reachability: Component6 @ Component6.L13 -> Component6.L15", vec!["E12", "E11", "E9", "E10", "E13"]; "Path in Component6 from L13 to L15")]
    #[test_case(PATH2, "reachability: Component7 @ Component7.L16 -> Component7.L19", vec!["E11", "E12", "E10"]; "Path in Component7 from L16 to L19")]
    #[test_case(PATH2, "reachability: Component8 @ Component8.L20 -> Component8.L22", vec!["E13", "E15", "E14"]; "Path in Component8 from L20 to L22")]
    #[test_case(PATH2, "reachability: Component9 @ Component9.L23 && Component9.x>5 -> Component9.L26", vec!["E17", "E18"]; "Path in Component9 from L23 x gt 5 to L26")]
    #[test_case(PATH2, "reachability: Component9 @ Component9.L23 && Component9.x<5 -> Component9.L26", vec!["E16", "E19"]; "Path in Component9 from L23 x lt 5 to L26")]
    fn path_gen_test_correct_path(folder_path: &str, query: &str, expected_path: Vec<&str>) {
        match json_run_query(folder_path, query).unwrap() {
            QueryResult::Reachability(actual_path) => {
                let actual_path = actual_path.unwrap_or_else(|_| {
                    panic!(
                        "Query: {}\nEnd state is not reachable from start state \n",
                        query
                    )
                });
                let path = actual_path.path;
                assert_eq!(expected_path.len(), path.len(), "Query: {}\nThe length of the actual and expected are not the same.\nexpected_path.len = {}\nactual_path.len = {} \n", query, expected_path.len(), path.len());
                for i in 0..path.len() {
                    let edges: Vec<_> = path[i].edges.iter().map(|e| e.edge_id.clone()).collect();
                    assert_eq!(
                        1,
                        edges.len(),
                        "Query: {}\nThere should only be one edge in the path \n",
                        query
                    );
                    assert_eq!(
                        expected_path[i], edges[0],
                        "Query: {}\nThe actual and expected is not the same \n",
                        query
                    );
                }
            }
            _ => panic!("Inconsistent query result, expected Reachability"),
        }
    }

    #[test_case(PATH2, "reachability: Component3[1] && Component3[2] @ Component3[1].L6 && Component3[2].L6  -> Component3[1].L7 && Component3[2].L7", vec![vec!["E5","E5"]]; "Path in Component3 && Component3 from L6 && L6 to L7 && L7")]
    #[test_case(PATH2, "reachability: Component3[1] && Component3[2] && Component3[3] @ Component3[1].L6 && Component3[2].L6 && Component3[3].L6  -> Component3[1].L7 && Component3[2].L7 && Component3[3].L7", vec![vec!["E5","E5", "E5"]]; "Path in Component3 && Component3 && Component3 from L6 && L6 && L6 to L7 && L7 && L7")]
    #[test_case(FOLDER_PATH, "reachability: Researcher[1] && Researcher[2] @ Researcher[1].U0 && Researcher[2].U0  -> Researcher[2].U0", vec![]; "Path in Researcher && Researcher from universal state to partial universal state")]
    fn path_gen_test_correct_path_vecvec(
        folder_path: &str,
        query: &str,
        expected_path: Vec<Vec<&str>>,
    ) {
        match json_run_query(folder_path, query).unwrap() {
            QueryResult::Reachability(actual_path) => {
                let actual_path = actual_path.unwrap_or_else(|_| {
                    panic!(
                        "Query: {}\nEnd state is not reachable from start state \n",
                        query
                    )
                });
                let path = actual_path.path;
                assert_eq!(expected_path.len(), path.len(), "Query: {}\nThe length of the actual and expected are not the same.\nexpected_path.len = {}\nactual_path.len = {} \n", query, expected_path.len(), path.len());
                for i in 0..path.len() {
                    let edges: Vec<_> = path[i].edges.iter().map(|e| e.edge_id.clone()).collect();
                    assert_eq!(
                        expected_path[i].len(),
                        edges.len(),
                        "Query: {}\nThere should only be one edge in the path \n",
                        query
                    );
                    assert_eq!(
                        expected_path[i], edges,
                        "Query: {}\nThe actual and expected is not the same \n",
                        query
                    );
                }
            }
            _ => panic!("Inconsistent query result, expected Reachability"),
        }
    }
    #[test_case(
    vec![],
    vec![];
    "Empty path")]
    #[test_case(
    vec![TransitionID::Simple("a".to_string())],
    vec![vec![vec![TransitionID::Simple("a".to_string())]]];
    "Simplest path")]
    #[test_case(
    vec![
    TransitionID::Simple("a".to_string()),
    TransitionID::None
    ],
    vec![
    // component 1
    vec![
    // transition 1
    vec![TransitionID::Simple("a".to_string())],
    vec![]
    ]
    ];
    "Has none")]
    #[test_case(
    vec![
    TransitionID::Conjunction(
    Box::new(TransitionID::Simple("a".to_string())),
    Box::new(TransitionID::Simple("b".to_string()))
    )
    ],
    vec![
    // component 1
    vec![
    vec![TransitionID::Simple("a".to_string())]
    ],
    // component 2
    vec![
    vec![TransitionID::Simple("b".to_string())]
    ]
    ];
    "One conjunction")]
    #[test_case(
    vec![
    TransitionID::Conjunction(
    Box::new(TransitionID::Simple("a".to_string())),
    Box::new(TransitionID::Simple("b".to_string()))
    ),
    TransitionID::Conjunction(
    Box::new(TransitionID::Simple("c".to_string())),
    Box::new(TransitionID::Simple("d".to_string()))
    ),
    TransitionID::Conjunction(
    Box::new(TransitionID::Simple("e".to_string())),
    Box::new(TransitionID::Simple("f".to_string()))
    )
    ],
    vec![
    // component 1
    vec![
    vec![TransitionID::Simple("a".to_string())],
    vec![TransitionID::Simple("c".to_string())],
    vec![TransitionID::Simple("e".to_string())]
    ],
    // component 2
    vec![
    vec![TransitionID::Simple("b".to_string())],
    vec![TransitionID::Simple("d".to_string())],
    vec![TransitionID::Simple("f".to_string())]
    ]
    ];
    "Path")]
    fn split_component_test(path: Vec<TransitionID>, expected: Vec<Vec<Vec<TransitionID>>>) {
        assert_eq!(
            TransitionID::split_into_component_lists(&path),
            Ok(expected)
        );
    }

    #[test_case(
    vec![
    TransitionID::Simple("a".to_string()),
    TransitionID::Conjunction(
    Box::new(TransitionID::Simple("b".to_string())),
    Box::new(TransitionID::Simple("c".to_string()))
    )
    ];
    "Different structures")]
    #[test_case(
    vec![
    TransitionID::Conjunction(
    Box::new(TransitionID::Simple("b".to_string())),
    Box::new(TransitionID::Simple("c".to_string()))
    ),
    TransitionID::Simple("a".to_string())
    ];
    "Different structures 2")]
    fn split_component_invalid_input(path: Vec<TransitionID>) {
        assert!(
            TransitionID::split_into_component_lists(&path).is_err(),
            "Expected error"
        )
    }
}
