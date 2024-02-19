use crate::model_objects::expressions::BoolExpression;
use crate::model_objects::{Component, Declarations, Location, LocationType, SyncType};
use crate::transition_systems::{LocationTree, TransitionSystemPtr};
use std::collections::HashMap;
use std::rc::Rc;

pub enum PruningStrategy {
    Reachable,
    NoPruning,
}

use crate::model_objects::Edge;
use edbm::util::constraints::ClockIndex;
use PruningStrategy::*;

pub fn combine_components(
    system: &TransitionSystemPtr,
    reachability: PruningStrategy,
) -> Component {
    let mut location_trees = vec![];
    let mut edges = vec![];
    let clocks = get_clock_map(system);
    match reachability {
        Reachable => {
            collect_reachable_edges_and_locations(system, &mut location_trees, &mut edges, &clocks)
        }
        NoPruning => {
            collect_all_edges_and_locations(system, &mut location_trees, &mut edges, &clocks)
        }
    };

    let locations = get_locations_from_trees(location_trees.as_slice(), &clocks);

    Component {
        name: "".to_string(),
        declarations: Declarations {
            ints: HashMap::new(),
            clocks,
        },
        locations,
        edges,
        special_id: None,
        clock_usages: Default::default(),
    }
}

pub fn get_locations_from_trees(
    location_trees: &[Rc<LocationTree>],
    clock_map: &HashMap<String, ClockIndex>,
) -> Vec<Location> {
    location_trees
        .iter()
        .cloned()
        .map(|loc_vec| {
            let invariant: Option<BoolExpression> = loc_vec.get_invariants().and_then(|fed| {
                BoolExpression::from_disjunction(&fed.minimal_constraints(), clock_map)
            });

            let location_type = if loc_vec.is_initial() {
                LocationType::Initial
            } else {
                LocationType::Normal
            };

            Location {
                id: loc_vec.id.to_string(),
                invariant,
                location_type,
                urgency: "NORMAL".to_string(), //TODO: Handle different urgencies eventually
            }
        })
        .collect()
}

pub fn get_clock_map(sysrep: &TransitionSystemPtr) -> HashMap<String, ClockIndex> {
    let mut clocks = HashMap::new();
    let decls = sysrep.get_decls();

    if decls.len() == 1 {
        return decls[0].clocks.clone();
    }
    for (comp_id, decl) in decls.into_iter().enumerate() {
        for (k, v) in &decl.clocks {
            if clocks.contains_key(k) {
                clocks.insert(format!("{}{}", k, comp_id), *v);
            } else {
                clocks.insert(k.clone(), *v);
            }
        }
    }

    clocks
}

fn collect_all_edges_and_locations(
    representation: &TransitionSystemPtr,
    locations: &mut Vec<Rc<LocationTree>>,
    edges: &mut Vec<Edge>,
    clock_map: &HashMap<String, ClockIndex>,
) {
    let l = representation.get_all_locations();
    locations.extend(l);
    for location in locations {
        collect_edges_from_location(Rc::clone(location), representation, edges, clock_map);
    }
}

fn collect_reachable_edges_and_locations(
    representation: &TransitionSystemPtr,
    locations: &mut Vec<Rc<LocationTree>>,
    edges: &mut Vec<Edge>,
    clock_map: &HashMap<String, ClockIndex>,
) {
    let l = representation.get_initial_location();

    if l.is_none() {
        return;
    }
    let l = l.unwrap();

    locations.push(l.clone());

    collect_reachable_locations(l, representation, locations);

    for loc in locations {
        collect_edges_from_location(Rc::clone(loc), representation, edges, clock_map);
    }
}

fn collect_reachable_locations(
    location: Rc<LocationTree>,
    representation: &TransitionSystemPtr,
    locations: &mut Vec<Rc<LocationTree>>,
) {
    for input in [true, false].iter() {
        for sync in if *input {
            representation.get_input_actions()
        } else {
            representation.get_output_actions()
        } {
            let transitions = representation.next_transitions(Rc::clone(&location), &sync);

            for transition in transitions {
                let target_location = transition.target_locations;

                if !locations.contains(&target_location) {
                    locations.push(Rc::clone(&target_location));
                    collect_reachable_locations(target_location, representation, locations);
                }
            }
        }
    }
}

fn collect_edges_from_location(
    location: Rc<LocationTree>,
    representation: &TransitionSystemPtr,
    edges: &mut Vec<Edge>,
    clock_map: &HashMap<String, ClockIndex>,
) {
    collect_specific_edges_from_location(
        Rc::clone(&location),
        representation,
        edges,
        true,
        clock_map,
    );
    collect_specific_edges_from_location(
        Rc::clone(&location),
        representation,
        edges,
        false,
        clock_map,
    );
}

fn collect_specific_edges_from_location(
    location: Rc<LocationTree>,
    representation: &TransitionSystemPtr,
    edges: &mut Vec<Edge>,
    input: bool,
    clock_map: &HashMap<String, ClockIndex>,
) {
    for sync in if input {
        representation.get_input_actions()
    } else {
        representation.get_output_actions()
    } {
        let transitions = representation.next_transitions(Rc::clone(&location), &sync);
        for transition in transitions {
            let target_location_id = transition.target_locations.id.to_string();

            let guard = transition.get_renamed_guard_expression(clock_map);
            if let Some(BoolExpression::Bool(false)) = guard {
                continue;
            }

            let edge = Edge {
                id: transition.id.to_string(),
                source_location: location.id.to_string(),
                target_location: target_location_id,
                sync_type: if input {
                    SyncType::Input
                } else {
                    SyncType::Output
                },
                guard,
                update: transition.get_renamed_updates(clock_map),
                sync: sync.clone(),
            };
            edges.push(edge);
        }
    }
}

#[cfg(test)]
pub mod tests {
    use crate::data_reader::component_loader::JsonProjectLoader;
    use crate::data_reader::parse_queries;
    use crate::model_objects::expressions::QueryExpression;
    use crate::system::query_failures::ConsistencyResult;
    use crate::system::refine;
    use crate::system::save_component::combine_components;
    use crate::system::save_component::PruningStrategy;
    use crate::system::system_recipe::{get_system_recipe, SystemRecipe};
    use edbm::util::constraints::ClockIndex;

    const PATH: &str = "samples/json/Conjunction";
    const ECDAR_UNI: &str = "samples/json/EcdarUniversity";

    pub fn json_reconstructed_component_refines_base_self(input_path: &str, system: &str) {
        let project_loader =
            JsonProjectLoader::new_loader(String::from(input_path), crate::DEFAULT_SETTINGS);

        //This query is not executed but simply used to extract an UncachedSystem so the tests can just give system expressions
        let str_query = format!("get-component: {} save-as test", system);
        let query = parse_queries::parse_to_expression_tree(str_query.as_str())
            .unwrap()
            .remove(0);

        let mut dim: ClockIndex = 0;
        let (base_system, new_system) = if let QueryExpression::GetComponent(expr) = &query {
            let mut comp_loader = project_loader.to_comp_loader();
            (
                get_system_recipe(&expr.system, &mut *comp_loader, &mut dim, &mut None).unwrap(),
                get_system_recipe(&expr.system, &mut *comp_loader, &mut dim, &mut None).unwrap(),
            )
        } else {
            panic!("Failed to create system")
        };

        let new_comp = new_system.compile(dim);
        //TODO:: Return the SystemRecipeFailure if new_comp is a failure
        if new_comp.is_err() {
            return;
        }
        let new_comp = combine_components(&new_comp.unwrap(), PruningStrategy::NoPruning);

        let new_comp = SystemRecipe::Component(Box::new(new_comp))
            .compile(dim)
            .unwrap();
        //TODO:: if it can fail unwrap should be replaced.
        let base_system = base_system.compile(dim).unwrap();

        let base_precheck = base_system.precheck_sys_rep();
        let new_precheck = new_comp.precheck_sys_rep();
        assert_eq!(helper(&base_precheck), helper(&new_precheck));

        //Only do refinement check if both pass precheck
        if helper(&base_precheck) && helper(&new_precheck) {
            assert!(matches!(
                refine::check_refinement(new_comp.clone(), base_system.clone()),
                Ok(())
            ));
            assert!(matches!(
                refine::check_refinement(base_system.clone(), new_comp.clone()),
                Ok(())
            ));
        }
    }

    fn helper(a: &ConsistencyResult) -> bool {
        a.is_ok()
    }

    #[test]
    fn t1_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test1");
    }
    #[test]
    fn t2_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test2");
    }
    #[test]
    fn t3_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test3");
    }
    #[test]
    fn t4_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test4");
    }
    #[test]
    fn t5_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test5");
    }
    #[test]
    fn t6_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test6");
    }
    #[test]
    fn t7_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test7");
    }
    #[test]
    fn t8_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test8");
    }
    #[test]
    fn t9_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test9");
    }
    #[test]
    fn t10_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test10");
    }
    #[test]
    fn t11_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test11");
    }
    #[test]
    fn t12_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test12");
    }

    #[test]
    fn adm_2_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Adm2");
    }

    #[test]
    fn administration_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Administration");
    }

    #[test]
    fn half_adm1saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "HalfAdm1");
    }

    #[test]
    fn half_adm_2_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "HalfAdm2");
    }

    #[test]
    fn machine_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Machine");
    }

    #[test]
    fn machine_2_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Machine2");
    }

    #[test]
    fn machine_3_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Machine3");
    }

    #[test]
    fn researcher_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Researcher");
    }

    #[test]
    fn spec_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Spec");
    }

    #[test]
    fn test1_and_test1_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test1 && Test1");
    }

    #[test]
    fn test1_and_test2_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test1 && Test2");
    }

    #[test]
    fn test1_and_test3_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test1 && Test3");
    }

    #[test]
    fn test1_and_test4_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test1 && Test4");
    }

    #[test]
    fn test1_and_test5_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test1 && Test5");
    }

    #[test]
    fn test1_and_test6_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test1 && Test6");
    }

    #[test]
    fn test1_and_test7_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test1 && Test7");
    }

    #[test]
    fn test1_and_test8_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test1 && Test8");
    }

    #[test]
    fn test1_and_test9_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test1 && Test9");
    }

    #[test]
    fn test1_and_test10_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test1 && Test10");
    }

    #[test]
    fn test1_and_test11_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test1 && Test11");
    }

    #[test]
    fn test1_and_test12_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test1 && Test12");
    }

    #[test]
    fn test2_and_test2_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test2 && Test2");
    }

    #[test]
    fn test2_and_test3_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test2 && Test3");
    }

    #[test]
    fn test2_and_test4_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test2 && Test4");
    }

    #[test]
    fn test2_and_test5_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test2 && Test5");
    }

    #[test]
    fn test2_and_test6_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test2 && Test6");
    }

    #[test]
    fn test2_and_test7_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test2 && Test7");
    }

    #[test]
    fn test2_and_test8_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test2 && Test8");
    }

    #[test]
    fn test2_and_test9_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test2 && Test9");
    }

    #[test]
    fn test2_and_test10_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test2 && Test10");
    }

    #[test]
    fn test2_and_test11_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test2 && Test11");
    }

    #[test]
    fn test2_and_test12_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test2 && Test12");
    }

    #[test]
    fn test3_and_test3_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test3 && Test3");
    }

    #[test]
    fn test3_and_test4_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test3 && Test4");
    }

    #[test]
    fn test3_and_test5_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test3 && Test5");
    }

    #[test]
    fn test3_and_test6_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test3 && Test6");
    }

    #[test]
    fn test3_and_test7_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test3 && Test7");
    }

    #[test]
    fn test3_and_test8_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test3 && Test8");
    }

    #[test]
    fn test3_and_test9_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test3 && Test9");
    }

    #[test]
    fn test3_and_test10_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test3 && Test10");
    }

    #[test]
    fn test3_and_test11_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test3 && Test11");
    }

    #[test]
    fn test3_and_test12_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test3 && Test12");
    }

    #[test]
    fn test4_and_test4_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test4 && Test4");
    }

    #[test]
    fn test4_and_test5_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test4 && Test5");
    }

    #[test]
    fn test4_and_test6_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test4 && Test6");
    }

    #[test]
    fn test4_and_test7_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test4 && Test7");
    }

    #[test]
    fn test4_and_test8_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test4 && Test8");
    }

    #[test]
    fn test4_and_test9_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test4 && Test9");
    }

    #[test]
    fn test4_and_test10_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test4 && Test10");
    }

    #[test]
    fn test4_and_test11_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test4 && Test11");
    }

    #[test]
    fn test4_and_test12_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test4 && Test12");
    }

    #[test]
    fn test5_and_test5_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test5 && Test5");
    }

    #[test]
    fn test5_and_test6_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test5 && Test6");
    }

    #[test]
    fn test5_and_test7_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test5 && Test7");
    }

    #[test]
    fn test5_and_test8_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test5 && Test8");
    }

    #[test]
    fn test5_and_test9_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test5 && Test9");
    }

    #[test]
    fn test5_and_test10_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test5 && Test10");
    }

    #[test]
    fn test5_and_test11_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test5 && Test11");
    }

    #[test]
    fn test5_and_test12_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test5 && Test12");
    }

    #[test]
    fn test6_and_test6_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test6 && Test6");
    }

    #[test]
    fn test6_and_test7_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test6 && Test7");
    }

    #[test]
    fn test6_and_test8_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test6 && Test8");
    }

    #[test]
    fn test6_and_test9_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test6 && Test9");
    }

    #[test]
    fn test6_and_test10_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test6 && Test10");
    }

    #[test]
    fn test6_and_test11_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test6 && Test11");
    }

    #[test]
    fn test6_and_test12_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test6 && Test12");
    }

    #[test]
    fn test7_and_test7_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test7 && Test7");
    }

    #[test]
    fn test7_and_test8_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test7 && Test8");
    }

    #[test]
    fn test7_and_test9_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test7 && Test9");
    }

    #[test]
    fn test7_and_test10_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test7 && Test10");
    }

    #[test]
    fn test7_and_test11_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test7 && Test11");
    }

    #[test]
    fn test7_and_test12_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test7 && Test12");
    }

    #[test]
    fn test8_and_test8_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test8 && Test8");
    }

    #[test]
    fn test8_and_test9_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test8 && Test9");
    }

    #[test]
    fn test8_and_test10_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test8 && Test10");
    }

    #[test]
    fn test8_and_test11_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test8 && Test11");
    }

    #[test]
    fn test8_and_test12_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test8 && Test12");
    }

    #[test]
    fn test9_and_test9_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test9 && Test9");
    }

    #[test]
    fn test9_and_test10_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test9 && Test10");
    }

    #[test]
    fn test9_and_test11_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test9 && Test11");
    }

    #[test]
    fn test9_and_test12_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test9 && Test12");
    }

    #[test]
    fn test10_and_test10_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test10 && Test10");
    }

    #[test]
    fn test10_and_test11_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test10 && Test11");
    }

    #[test]
    fn test10_and_test12_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test10 && Test12");
    }

    #[test]
    fn test11_and_test11_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test11 && Test11");
    }

    #[test]
    fn test11_and_test12_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test11 && Test12");
    }

    #[test]
    fn test12_and_test12_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test12 && Test12");
    }

    #[test]
    fn adm2_and_adm2_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Adm2 && Adm2");
    }

    #[test]
    fn adm2_and_administration_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Adm2 && Administration");
    }

    #[test]
    fn adm2_and_half_adm1_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Adm2 && HalfAdm1");
    }

    #[test]
    fn adm2_and_half_adm2_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Adm2 && HalfAdm2");
    }

    #[test]
    fn adm2_and_machine_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Adm2 && Machine");
    }

    #[test]
    fn adm2_and_machine2_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Adm2 && Machine2");
    }

    #[test]
    fn adm2_and_machine3_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Adm2 && Machine3");
    }

    #[test]
    fn adm2_and_researcher_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Adm2 && Researcher");
    }

    #[test]
    fn adm2_and_spec_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Adm2 && Spec");
    }

    #[test]
    fn administration_and_administration_saved_refines_self() {
        json_reconstructed_component_refines_base_self(
            ECDAR_UNI,
            "Administration && Administration",
        );
    }

    #[test]
    fn administration_and_half_adm1saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Administration && HalfAdm1");
    }

    #[test]
    fn administration_and_half_adm2saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Administration && HalfAdm2");
    }

    #[test]
    fn administration_and_machine_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Administration && Machine");
    }

    #[test]
    fn administration_and_machine2_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Administration && Machine2");
    }

    #[test]
    fn administration_and_machine3_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Administration && Machine3");
    }

    #[test]
    fn administration_and_researcher_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Administration && Researcher");
    }

    #[test]
    fn administration_and_spec_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Administration && Spec");
    }

    #[test]
    fn half_adm1and_half_adm1_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "HalfAdm1 && HalfAdm1");
    }

    #[test]
    fn half_adm1and_half_adm2_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "HalfAdm1 && HalfAdm2");
    }

    #[test]
    fn half_adm1and_machine_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "HalfAdm1 && Machine");
    }

    #[test]
    fn half_adm1and_machine2_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "HalfAdm1 && Machine2");
    }

    #[test]
    fn half_adm1and_machine3_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "HalfAdm1 && Machine3");
    }

    #[test]
    fn half_adm1and_researcher_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "HalfAdm1 && Researcher");
    }

    #[test]
    fn half_adm1and_spec_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "HalfAdm1 && Spec");
    }

    #[test]
    fn half_adm2_and_half_adm2_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "HalfAdm2 && HalfAdm2");
    }

    #[test]
    fn half_adm2_and_machine_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "HalfAdm2 && Machine");
    }

    #[test]
    fn half_adm2_and_machine2_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "HalfAdm2 && Machine2");
    }

    #[test]
    fn half_adm2_and_machine3_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "HalfAdm2 && Machine3");
    }

    #[test]
    fn half_adm2_and_researcher_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "HalfAdm2 && Researcher");
    }

    #[test]
    fn half_adm2_and_spec_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "HalfAdm2 && Spec");
    }

    #[test]
    fn machine_and_machine_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Machine && Machine");
    }

    #[test]
    fn machine_and_machine2_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Machine && Machine2");
    }

    #[test]
    fn machine_and_machine3_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Machine && Machine3");
    }

    #[test]
    fn machine_and_researcher_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Machine && Researcher");
    }

    #[test]
    fn machine_and_spec_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Machine && Spec");
    }

    #[test]
    fn machine2_and_machine2_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Machine2 && Machine2");
    }

    #[test]
    fn machine2_and_machine3_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Machine2 && Machine3");
    }

    #[test]
    fn machine2_and_researcher_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Machine2 && Researcher");
    }

    #[test]
    fn machine2_and_spec_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Machine2 && Spec");
    }

    #[test]
    fn machine3_and_machine3_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Machine3 && Machine3");
    }

    #[test]
    fn machine3_and_researcher_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Machine3 && Researcher");
    }

    #[test]
    fn machine3_and_spec_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Machine3 && Spec");
    }

    #[test]
    fn researcher_and_researcher_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Researcher && Researcher");
    }

    #[test]
    fn researcher_and_spec_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Researcher && Spec");
    }

    #[test]
    fn spec_and_spec_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Spec && Spec");
    }

    #[test]
    fn adm_2_machine_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Adm2 || Machine");
    }

    #[test]
    fn adm_2_machine_2_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Adm2 || Machine2");
    }

    #[test]
    fn adm_2_machine_3_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Adm2 || Machine3");
    }

    #[test]
    fn adm_2_researcher_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Adm2 || Researcher");
    }

    #[test]
    fn administration_machine_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Administration || Machine");
    }

    #[test]
    fn administration_machine_2_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Administration || Machine2");
    }

    #[test]
    fn administration_machine_3_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Administration || Machine3");
    }

    #[test]
    fn administration_researcher_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Administration || Researcher");
    }

    #[test]
    fn half_adm_1_machine_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "HalfAdm1 || Machine");
    }

    #[test]
    fn half_adm_1_machine_2_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "HalfAdm1 || Machine2");
    }

    #[test]
    fn half_adm_1_machine_3_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "HalfAdm1 || Machine3");
    }

    #[test]
    fn half_adm_1_researcher_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "HalfAdm1 || Researcher");
    }

    #[test]
    fn half_adm_2_machine_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "HalfAdm2 || Machine");
    }

    #[test]
    fn half_adm_2_machine_2_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "HalfAdm2 || Machine2");
    }

    #[test]
    fn half_adm_2_machine_3_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "HalfAdm2 || Machine3");
    }

    #[test]
    fn half_adm_2_researcher_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "HalfAdm2 || Researcher");
    }

    #[test]
    fn machine_researcher_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Machine || Researcher");
    }

    #[test]
    fn machine_spec_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Machine || Spec");
    }

    #[test]
    fn machine_2_researcher_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Machine2 || Researcher");
    }

    #[test]
    fn machine_2_spec_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Machine2 || Spec");
    }

    #[test]
    fn machine_3_researcher_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Machine3 || Researcher");
    }

    #[test]
    fn machine_3_spec_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Machine3 || Spec");
    }

    #[test]
    fn researcher_spec_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Researcher || Spec");
    }
}
