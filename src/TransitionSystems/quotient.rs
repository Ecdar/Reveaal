use crate::DBMLib::dbm::{Federation, Zone};
use crate::DataReader::parse_edge;
use crate::DataReader::parse_edge::Update;
use crate::EdgeEval::updater::updater;
use crate::ModelObjects::component::Declarations;
use crate::ModelObjects::component::{
    Component, DeclarationProvider, DecoratedLocation, Location, LocationType, State, SyncType,
    Transition,
};
use crate::ModelObjects::max_bounds::MaxBounds;
use crate::ModelObjects::representations::BoolExpression;
use crate::System::local_consistency;
use crate::TransitionSystems::{LocationTuple, TransitionSystem, TransitionSystemPtr};
use std::collections::hash_set::HashSet;
use std::collections::HashMap;

#[derive(Clone)]
pub struct Quotient {
    left: TransitionSystemPtr,
    right: TransitionSystemPtr,
    inputs: HashSet<String>,
    outputs: HashSet<String>,
    universal_location: Location,
    inconsistent_location: Location,
    decls: Declarations,
    xnew_clock_index: u32,
}

static INCONSISTENT_LOC_NAME: &str = "Inconsistent";
static UNIVERSAL_LOC_NAME: &str = "Universal";
impl Quotient {
    pub fn new(left: TransitionSystemPtr, right: TransitionSystemPtr) -> Box<Quotient> {
        let universal_location = Location {
            id: UNIVERSAL_LOC_NAME.to_string(),
            invariant: None,
            location_type: LocationType::Universal,
            urgency: "".to_string(),
        };

        let inconsistent_location = Location {
            id: INCONSISTENT_LOC_NAME.to_string(),
            // xnew <= 0
            invariant: Some(BoolExpression::LessEQ(
                Box::new(BoolExpression::VarName("quotient_xnew".to_string())),
                Box::new(BoolExpression::Int(0)),
            )),
            location_type: LocationType::Inconsistent,
            urgency: "".to_string(),
        };

        let mut inputs: HashSet<String> = left
            .get_input_actions()
            .union(&right.get_output_actions())
            .cloned()
            .collect();
        inputs.insert("quotient_inew".to_string());

        let output_dif: HashSet<String> = left
            .get_output_actions()
            .difference(&right.get_output_actions())
            .cloned()
            .collect();
        let input_dif: HashSet<String> = right
            .get_input_actions()
            .difference(&left.get_input_actions())
            .cloned()
            .collect();

        let outputs: HashSet<String> = output_dif.union(&input_dif).cloned().collect();

        let mut decls = Declarations::empty();
        decls.clocks.insert("quotient_xnew".to_string(), 0);

        Box::new(Quotient {
            left,
            right,
            inputs,
            outputs,
            universal_location,
            inconsistent_location,
            decls,
            xnew_clock_index: 0,
        })
    }
}

impl TransitionSystem<'static> for Quotient {
    fn next_transitions<'b>(
        &'b self,
        location: &LocationTuple<'b>,
        action: &str,
        sync_type: &SyncType,
        index: &mut usize,
        dim: u32,
    ) -> Vec<Transition<'b>> {
        let mut transitions = vec![];
        let left_index = *index;
        let mut left = self
            .left
            .next_transitions(location, action, sync_type, index, dim);
        let right_index = *index;
        let mut right = self
            .right
            .next_transitions(location, action, sync_type, index, dim);
        let quotient_index = *index;
        *index += 1;

        for transition in left.iter_mut().chain(right.iter_mut()) {
            transition.target_locations.ignore_all_invariants();
        }

        //Rules [universal] and [incosistent]
        if let Some(quotient_loc) = location.try_get_location(quotient_index) {
            let quotient_state = quotient_loc.get_location_type();
            if *quotient_state == LocationType::Inconsistent {
                //Rule 10
                if self.inputs.contains(action) {
                    let mut transition = Transition::new(location.clone(), dim);
                    transition
                        .guard_zone
                        .add_eq_const_constraint(self.xnew_clock_index, 0);
                    transitions.push(transition);
                }
                return transitions;
            } else if *quotient_state == LocationType::Universal {
                // Rule 9
                transitions.push(Transition::new(location.clone(), dim));
                return transitions;
            }
        }

        //Reused target locations
        let mut universal_location = LocationTuple::create_empty();
        universal_location.set_location(
            quotient_index,
            Some(&self.universal_location),
            self.decls.clone(),
        );
        let mut inconsistent_location = LocationTuple::create_empty();
        inconsistent_location.set_location(
            quotient_index,
            Some(&self.inconsistent_location),
            self.decls.clone(),
        );

        //Rule 1
        if self.right.get_actions().contains(action) && self.left.get_actions().contains(action) {
            for left_transition in &left {
                for right_transition in &right {
                    // Guard for edge
                    // P_t && P_s
                    let mut guard_zone = left_transition.guard_zone.clone();
                    guard_zone.intersection(&right_transition.guard_zone);

                    // Inv(l1_s)
                    for i in right_index..quotient_index {
                        let dec_loc = DecoratedLocation::create(
                            location.get_location(i),
                            location.get_decl(i),
                        );
                        dec_loc.apply_invariant(&mut guard_zone);
                    }

                    // Inv(l2_t)[r |-> 0] where r is in clock resets for t
                    guard_zone.intersection(&get_resetted_invariant(
                        left_index,
                        right_index,
                        &left_transition.target_locations,
                        &left_transition.updates,
                        dim,
                    ));

                    // Inv(l2_s)[r |-> 0] where r is in clock resets for t
                    guard_zone.intersection(&get_resetted_invariant(
                        right_index,
                        quotient_index,
                        &right_transition.target_locations,
                        &right_transition.updates,
                        dim,
                    ));

                    let mut target_locations = LocationTuple::merge(
                        left_transition.target_locations.clone(),
                        &right_transition.target_locations,
                    );
                    target_locations.set_location(quotient_index, None, Declarations::empty());

                    //Union of left and right updates
                    let mut updates = left_transition.updates.clone();
                    for (index, r_updates) in &right_transition.updates {
                        match updates.get_mut(index) {
                            Some(update_list) => {
                                update_list.append(&mut r_updates.clone());
                            }
                            None => {
                                updates.insert(*index, r_updates.clone());
                            }
                        };
                    }

                    transitions.push(Transition {
                        guard_zone,
                        target_locations,
                        updates,
                    });
                }
            }
        }

        //Rule 2
        if self.right.get_actions().contains(action) && !self.left.get_actions().contains(action) {
            let mut new_transitions = right.clone();
            for transition in &mut new_transitions {
                // Inv(l1_s)
                for i in right_index..quotient_index {
                    let dec_loc =
                        DecoratedLocation::create(location.get_location(i), location.get_decl(i));
                    dec_loc.apply_invariant(&mut transition.guard_zone);
                }

                // Inv(l2_s)[r |-> 0] where r is in clock resets for s
                transition.guard_zone.intersection(&get_resetted_invariant(
                    left_index,
                    right_index,
                    &transition.target_locations,
                    &transition.updates,
                    dim,
                ));

                transition.target_locations =
                    LocationTuple::merge(location.clone(), &transition.target_locations);
            }

            transitions.append(&mut new_transitions);
        }

        if self.right.get_output_actions().contains(action) {
            //Rule 3
            let fed = Federation::new(right.iter().map(|t| &t.guard_zone).cloned().collect(), dim)
                .inverse(dim);
            transitions.append(&mut create_transitions(
                fed,
                &universal_location,
                &HashMap::new(),
            ));

            //Rule 4
            let mut zones = vec![];
            for transition in &right {
                let mut zone = Zone::init(dim);
                for (opt_location, decl) in transition.target_locations.iter_values() {
                    if let Some(location) = opt_location {
                        let dec_loc = DecoratedLocation::create(location, decl);
                        dec_loc.apply_invariant(&mut zone);
                    }
                }
                for (&index, updates) in &transition.updates {
                    //Assumes all updates are free operations i.e [x -> 0]
                    updater(
                        updates,
                        self.right.get_components()[index].get_declarations(),
                        &mut zone,
                    );
                }
                zones.push(zone);
            }
            let fed = Federation::new(zones, dim).inverse(dim);
            transitions.append(&mut create_transitions(
                fed,
                &universal_location,
                &HashMap::new(),
            ));
        }

        //Rule 5
        if self.inputs.contains(action) || self.outputs.contains(action) {
            let mut zone = Zone::init(dim);
            let mut tmp_index = right_index;
            for _ in self.right.get_children() {
                let dec_loc = DecoratedLocation::create(
                    location.get_location(tmp_index),
                    location.get_decl(tmp_index),
                );
                dec_loc.apply_invariant(&mut zone);

                tmp_index += 1;
            }
            let fed = Federation::new(vec![zone], dim).inverse(dim);
            transitions.append(&mut create_transitions(
                fed,
                &universal_location,
                &HashMap::new(),
            ));
        }

        //Rule 6
        if self.right.get_output_actions().contains(action)
            && self.left.get_output_actions().contains(action)
        {
            //Calculate inverse G_T
            let mut fed = Federation::new(vec![], dim);
            for transition in &left {
                let mut zone = transition.guard_zone.clone();

                //Inv(l2_T)[r |-> 0] where r is in clock resets
                zone.intersection(&get_resetted_invariant(
                    left_index,
                    right_index,
                    location,
                    &transition.updates,
                    dim,
                ));
                fed.zones.push(zone)
            }
            fed = fed.inverse(dim);

            for transition in &right {
                let mut right_guard_zone = transition.guard_zone.clone();

                //Inv(l2_s)[r |-> 0] where r is in clock resets
                right_guard_zone.intersection(&get_resetted_invariant(
                    right_index,
                    quotient_index,
                    location,
                    &transition.updates,
                    dim,
                ));

                for mut guard_zone in fed.zones.iter().cloned() {
                    guard_zone.intersection(&right_guard_zone);

                    let mut xnew_reset = HashMap::new();
                    xnew_reset.insert(
                        quotient_index,
                        vec![Update {
                            variable: "quotient_xnew".to_string(),
                            expression: BoolExpression::Int(0),
                        }],
                    );

                    transitions.push(Transition {
                        guard_zone,
                        target_locations: inconsistent_location.clone(),
                        updates: xnew_reset,
                    })
                }
            }
        }

        //Rule 7
        if *sync_type == SyncType::Input && action == "quotient_inew" {
            let t_invariant = get_invariant(left_index, right_index, location, dim);
            let s_invariant = get_invariant(right_index, quotient_index, location, dim);
            let inverse_t_invariant = Federation::new(vec![t_invariant], dim).inverse(dim);

            for mut guard_zone in inverse_t_invariant.zones {
                guard_zone.intersection(&s_invariant);

                let mut xnew_reset = HashMap::new();
                xnew_reset.insert(
                    quotient_index,
                    vec![Update {
                        variable: "quotient_xnew".to_string(),
                        expression: BoolExpression::Int(0),
                    }],
                );

                transitions.push(Transition {
                    guard_zone,
                    target_locations: inconsistent_location.clone(),
                    updates: xnew_reset,
                })
            }
        }

        //Rule 8
        if self.left.get_actions().contains(action) && !self.right.get_actions().contains(action) {
            for mut transition in left {
                //Inv(l2_T)[r |-> 0] where r is in clock resets
                transition.guard_zone.intersection(&get_resetted_invariant(
                    left_index,
                    right_index,
                    location,
                    &transition.updates,
                    dim,
                ));

                transition.target_locations =
                    LocationTuple::merge(location.clone(), &transition.target_locations);

                transitions.push(transition);
            }
        }

        transitions
    }

    fn is_locally_consistent(&self, dimensions: u32) -> bool {
        local_consistency::is_least_consistent(self, dimensions)
    }

    fn get_all_locations<'b>(&'b self, index: &mut usize) -> Vec<LocationTuple<'b>> {
        let mut location_tuples = vec![];
        let left = self.left.get_all_locations(index);
        let right = self.right.get_all_locations(index);
        for loc1 in left {
            for loc2 in &right {
                let mut location = LocationTuple::merge(loc1.clone(), &loc2);
                location.set_location(*index, None, Declarations::empty());
                location.ignore_all_invariants();
                location_tuples.push(location);
            }
        }

        let inconsistent =
            LocationTuple::simple_indexed(*index, &self.inconsistent_location, &self.decls);
        let universal =
            LocationTuple::simple_indexed(*index, &self.universal_location, &self.decls);
        *index += 1;

        location_tuples.push(inconsistent);
        location_tuples.push(universal);

        location_tuples
    }

    fn get_max_bounds(&self, dim: u32) -> MaxBounds {
        let mut bounds = self.left.get_max_bounds(dim);
        bounds.add_bounds(&self.right.get_max_bounds(dim));
        //Potentially add xnew bound might save something
        bounds
    }
    fn get_input_actions(&self) -> HashSet<String> {
        self.inputs.clone()
    }
    fn get_output_actions(&self) -> HashSet<String> {
        self.outputs.clone()
    }
    fn get_actions(&self) -> HashSet<String> {
        self.inputs
            .union(&self.outputs)
            .map(|action| action.to_string())
            .collect()
    }
    fn get_num_clocks(&self) -> u32 {
        self.get_children()
            .iter()
            .fold(1, |accumulator, child| accumulator + child.get_num_clocks())
    }
    fn get_initial_location<'b>(&'b self) -> Option<LocationTuple<'b>> {
        let mut locations = vec![];

        for child in self.get_children() {
            locations.push(child.get_initial_location()?);
        }
        let loc_tuple = LocationTuple::compose_iter(locations);

        Some(loc_tuple)
    }

    fn get_components<'b>(&'b self) -> Vec<&'b Component> {
        let mut comps = self.left.get_components();
        comps.extend(self.right.get_components());
        comps
    }

    fn get_max_clock_index(&self) -> u32 {
        std::cmp::max(
            self.left.get_max_clock_index(),
            self.right.get_max_clock_index(),
        ) + 1
    }

    fn precheck_sys_rep(&self, dim: u32) -> bool {
        if !self.is_deterministic(dim) {
            println!("NOT DETERMINISTIC");
            return false;
        }

        if !self.is_locally_consistent(dim) {
            println!("NOT CONSISTENT");
            return false;
        }

        true
    }

    fn is_deterministic(&self, dim: u32) -> bool {
        self.left.is_deterministic(dim) && self.right.is_deterministic(dim)
    }

    fn get_initial_state(&self, dimensions: u32) -> State {
        let mut init_loc = self.get_initial_location().unwrap();
        init_loc.ignore_all_invariants();
        let zone = Zone::init(dimensions);
        State {
            decorated_locations: init_loc,
            zone,
        }
    }

    fn set_clock_indices(&mut self, index: &mut u32) {
        self.xnew_clock_index = *index;
        self.decls
            .clocks
            .insert("quotient_xnew".to_string(), self.xnew_clock_index);
        *index += 1;
    }

    fn get_mut_children(&mut self) -> Vec<&mut TransitionSystemPtr> {
        vec![&mut self.left, &mut self.right]
    }

    fn get_children(&self) -> Vec<&TransitionSystemPtr> {
        vec![&self.left, &self.right]
    }
}

fn create_transitions<'a>(
    fed: Federation,
    target_locations: &LocationTuple<'a>,
    updates: &HashMap<usize, Vec<parse_edge::Update>>,
) -> Vec<Transition<'a>> {
    let mut transitions = vec![];
    for zone in fed.zones {
        transitions.push(Transition {
            guard_zone: zone,
            target_locations: target_locations.clone(),
            updates: updates.clone(),
        });
    }
    transitions
}

fn get_resetted_invariant(
    start: usize,
    end: usize,
    location: &LocationTuple,
    updates_map: &HashMap<usize, Vec<parse_edge::Update>>,
    dim: u32,
) -> Zone {
    let mut zone = Zone::init(dim);
    for i in start..end {
        let location = DecoratedLocation::create(location.get_location(i), location.get_decl(i));
        location.apply_invariant(&mut zone);
        // For some reason nessecary with a check for None?
        if let Some(updates) = updates_map.get(&i) {
            updater(&updates, location.get_declarations(), &mut zone);
        }
    }
    zone
}

fn get_invariant(start: usize, end: usize, location: &LocationTuple, dim: u32) -> Zone {
    let mut zone = Zone::init(dim);
    for i in start..end {
        let location = DecoratedLocation::create(location.get_location(i), location.get_decl(i));
        location.apply_invariant(&mut zone);
    }
    zone
}
