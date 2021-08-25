use crate::input_enabler::build_guard_from_zone;
use crate::DBMLib::dbm::{Federation, Zone};
use crate::ModelObjects::component::{Component, Declarations, Location, State};
use crate::ModelObjects::max_bounds::MaxBounds;
use crate::TransitionSystems::LocationTuple;
use crate::TransitionSystems::TransitionSystem;

pub fn prune(comp: &Component, clocks: u32) -> Component {
    let mut new_comp = comp.clone().create_edge_io_split();
    let mut changed = false;

    println!("Begin prune with {} clocks", clocks);
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
    new_comp.create_edge_io_split()
}

fn prune_to_consistent_part(
    location: &Location,
    new_comp: &mut Component,
    decls: &Declarations,
    dimensions: u32,
) -> bool {
    let loc = LocationTuple::simple(location, decls);
    let mut zone = Zone::init(dimensions);
    loc.apply_invariants(&mut zone);
    let inv_fed = Federation::new(vec![zone], dimensions);

    let cons_fed = get_consistent_part(location, new_comp, dimensions);
    println!("Fed:");
    for zone in cons_fed.iter_zones() {
        println!("Zone: {}", zone);
    }

    // If cons strictly less than inv
    if cons_fed.is_subset_eq(&inv_fed) && !inv_fed.is_subset_eq(&cons_fed) {
        println!("Pruning...");
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
            return num_locs > num_locs2 || num_edges > num_edges2;
        }
    }

    false
}

fn get_consistent_part(location: &Location, comp: &Component, dimensions: u32) -> Federation {
    let loc = LocationTuple::simple(location, &comp.declarations);
    let mut zone = Zone::init(dimensions);

    loc.apply_invariants(&mut zone);
    if zone.canDelayIndefinitely() {
        return Federation::new(vec![zone], dimensions);
    }

    let mut federation = Federation::new(vec![], dimensions);
    for output in (comp as &dyn TransitionSystem).get_output_actions() {
        for transition in comp.next_outputs(&loc, &output) {
            if let Some(fed) = transition.get_guard_federation(&loc, dimensions) {
                for mut zone in fed.iter_zones().cloned() {
                    loc.apply_invariants(&mut zone);
                    federation.add(zone);
                }
            }
        }
    }

    federation
}
