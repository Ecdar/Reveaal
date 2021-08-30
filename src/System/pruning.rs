use crate::input_enabler::build_guard_from_zone;
use crate::system_declarations::{SystemDeclarations, SystemSpecification};
use crate::DBMLib::dbm::{Federation, Zone};
use crate::ModelObjects::component::{Component, Declarations, Location, LocationType, State};
use crate::ModelObjects::max_bounds::MaxBounds;
use crate::System::input_enabler;
use crate::System::save_component::combine_components;
use crate::TransitionSystems::LocationTuple;
use crate::TransitionSystems::{PrunedComponent, TransitionSystem, TransitionSystemPtr};
use std::collections::{HashMap, HashSet};

pub fn prune_system(ts: TransitionSystemPtr, clocks: u32) -> TransitionSystemPtr {
    let inputs = ts.get_input_actions();
    let outputs = ts.get_output_actions();
    let comp = combine_components(&ts);

    let mut input_map: HashMap<String, Vec<String>> = HashMap::new();
    input_map.insert(comp.get_name().clone(), inputs.iter().cloned().collect());

    let sys_decl = SystemDeclarations {
        name: "".to_string(),
        declarations: SystemSpecification {
            components: vec![comp.get_name().clone()],
            input_actions: input_map,
            output_actions: HashMap::new(),
        },
    };

    let result = Box::new(prune(&comp, clocks, inputs, outputs, &sys_decl));

    result
}

pub fn prune(
    comp: &Component,
    clocks: u32,
    inputs: HashSet<String>,
    outputs: HashSet<String>,
    decl: &SystemDeclarations,
) -> PrunedComponent {
    let mut new_comp = comp.clone();
    new_comp.create_edge_io_split();
    let mut changed = false;

    loop {
        changed = false;
        for location in new_comp.get_locations().clone() {
            changed |=
                prune_to_consistent_part(&location, &mut new_comp, &comp.declarations, clocks + 1);
        }
        if !changed {
            break;
        }
    }

    //TODO: Maybe we ought not input enable here?
    input_enabler::make_input_enabled(&mut new_comp, decl);

    PrunedComponent {
        component: Box::new(new_comp),
        inputs,
        outputs,
    }
}

fn prune_to_consistent_part(
    location: &Location,
    new_comp: &mut Component,
    decls: &Declarations,
    dimensions: u32,
) -> bool {
    let loc = LocationTuple::simple(location, decls);
    let mut zone = Zone::init(dimensions);
    let inv_fed = if loc.apply_invariants(&mut zone) {
        Federation::new(vec![zone], dimensions)
    } else {
        Federation::new(vec![], dimensions)
    };
    let cons_fed = get_consistent_part(location, new_comp, dimensions);
    // If cons strictly less than inv
    if cons_fed.is_subset_eq(&inv_fed) && !inv_fed.is_subset_eq(&cons_fed) {
        if cons_fed.num_zones() > 1 {
            panic!("Implementation cannot handle disjunct invariants")
        }
        if let Some(zone) = cons_fed.iter_zones().next() {
            if let Some(mut old_loc) = new_comp
                .get_mut_locations()
                .iter_mut()
                .find(|l| l.get_id() == location.get_id())
            {
                old_loc.invariant = build_guard_from_zone(zone, &decls.clocks);
            } else {
                panic!();
            }

            return true;
        } else {
            //TODO: check with Martijn & Ulrik whether we should prune the initial location

            //Remove the location / error state
            let (num_locs, num_edges) = (new_comp.edges.len(), new_comp.locations.len());
            new_comp
                .get_mut_locations()
                .retain(|l| l.get_id() != location.get_id());
            //Remove edges involving the error state
            new_comp.get_mut_edges().retain(|e| {
                e.target_location != *location.get_id() && e.source_location != *location.get_id()
            });
            let (num_locs2, num_edges2) = (new_comp.edges.len(), new_comp.locations.len());
            let changed = num_locs > num_locs2 || num_edges > num_edges2;

            if changed {
                new_comp.create_edge_io_split();
            }
            return changed;
        }
    }

    false
}

fn get_consistent_part(location: &Location, comp: &Component, dimensions: u32) -> Federation {
    let loc = LocationTuple::simple(location, &comp.declarations);
    let mut zone = Zone::init(dimensions);

    if !loc.apply_invariants(&mut zone) {
        return Federation::new(vec![], dimensions);
    }
    if zone.canDelayIndefinitely() {
        return Federation::new(vec![zone], dimensions);
    }

    let mut federation = Federation::new(vec![], dimensions);
    for output in (comp as &dyn TransitionSystem).get_output_actions() {
        for transition in comp.next_outputs(&loc, &output) {
            if let Some(fed) = transition.get_guard_federation(&loc, dimensions) {
                for mut zone in fed.iter_zones().cloned() {
                    zone.down();
                    if loc.apply_invariants(&mut zone) {
                        federation.add(zone);
                    }
                }
            }
        }
    }

    federation
}
