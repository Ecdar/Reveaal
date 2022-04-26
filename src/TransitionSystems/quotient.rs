use crate::DBMLib::dbm::Federation;
use crate::DataReader::parse_edge;
use crate::DataReader::parse_edge::Update;
use crate::EdgeEval::updater::{inverse_updater, updater};
use crate::ModelObjects::component::Declarations;
use crate::ModelObjects::component::{
    Component, DeclarationProvider, DecoratedLocation, Location, LocationType, State, SyncType,
    Transition,
};
use crate::ModelObjects::max_bounds::MaxBounds;
use crate::ModelObjects::representations::BoolExpression;
use crate::System::{local_consistency, pruning};
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
    new_clock_index: u32,
    new_input_name: String,
}

static INCONSISTENT_LOC_NAME: &str = "Inconsistent";
static UNIVERSAL_LOC_NAME: &str = "Universal";
impl Quotient {
    pub fn new(
        t: TransitionSystemPtr,
        s: TransitionSystemPtr,
        clock_index: &mut u32,
    ) -> TransitionSystemPtr {
        if !s.get_output_actions().is_disjoint(&t.get_input_actions()) {
            println!(
                "s_out and t_in not disjoint s_out: {:?} t_in {:?}",
                s.get_output_actions(),
                t.get_input_actions()
            );
            return Box::new(Component::invalid());
        }

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

        let mut inputs: HashSet<String> = t
            .get_input_actions()
            .union(&s.get_output_actions())
            .cloned()
            .collect();
        let mut i = 0;
        let new_input_name = loop {
            let test = format!("quotient_new_input{}", i);
            if !inputs.contains(&test) {
                break test;
            }
            i += 1;
        };

        inputs.insert(new_input_name.clone());

        let output_dif: HashSet<String> = t
            .get_output_actions()
            .difference(&s.get_output_actions())
            .cloned()
            .collect();
        let input_dif: HashSet<String> = s
            .get_input_actions()
            .difference(&t.get_input_actions())
            .cloned()
            .collect();

        let outputs: HashSet<String> = output_dif.union(&input_dif).cloned().collect();

        *clock_index += 1;

        let mut decls = Declarations::empty();
        decls
            .clocks
            .insert("quotient_xnew".to_string(), *clock_index);

        let ts = Box::new(Quotient {
            left: t,
            right: s,
            inputs,
            outputs,
            universal_location,
            inconsistent_location,
            decls,
            new_clock_index: *clock_index,
            new_input_name,
        });
        //ts
        let num_clocks = ts.get_max_clock_index();
        pruning::prune_system(ts, num_clocks)
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
        let mut t = self
            .left
            .next_transitions(location, action, sync_type, index, dim);
        let right_index = *index;
        let mut s = self
            .right
            .next_transitions(location, action, sync_type, index, dim);
        let quotient_index = *index;
        *index += 1;

        let is_input = *sync_type == SyncType::Input;
        let is_output = !is_input;

        // Inv(l_t)[r |-> 0]
        let t_resetted_invariant = |transition: &Transition| {
            get_resetted_invariant(
                left_index,
                right_index,
                &transition.target_locations,
                &transition.updates,
                dim,
            )
        };

        // Inv(l_r)[r |-> 0]
        let s_resetted_invariant = |transition: &Transition| {
            get_resetted_invariant(
                right_index,
                quotient_index,
                &transition.target_locations,
                &transition.updates,
                dim,
            )
        };

        // Inv(l_s)
        let apply_right_invariant = |location: &LocationTuple, fed: &mut Federation| {
            for i in right_index..quotient_index {
                apply_invariant_on_index(location, i, fed);
            }
        };

        // Inv(l_t)
        let apply_left_invariant = |location: &LocationTuple, fed: &mut Federation| {
            for i in left_index..right_index {
                apply_invariant_on_index(location, i, fed);
            }
        };

        for transition in t.iter_mut().chain(s.iter_mut()) {
            transition.target_locations.ignore_all_invariants();
        }

        //Rules [universal] and [incosistent]
        if let Some(quotient_loc) = location.try_get_location(quotient_index) {
            let quotient_state = quotient_loc.get_location_type();
            if *quotient_state == LocationType::Inconsistent {
                //Rule 10
                if is_input && self.inputs.contains(action) {
                    let mut transition = Transition::new(location.clone(), dim);
                    transition
                        .guard_zone
                        .add_eq_const_constraint(self.new_clock_index, 0);
                    transitions.push(transition);
                }
                return transitions;
            } else if *quotient_state == LocationType::Universal {
                // Rule 9
                transitions.push(Transition::new(location.clone(), dim));
                return transitions;
            }
        }

        let mut inconsistent_location =
            LocationTuple::simple_indexed(*index, &self.inconsistent_location, &self.decls);
        let mut universal_location =
            LocationTuple::simple_indexed(*index, &self.universal_location, &self.decls);
        for i in 0..location.len() {
            inconsistent_location.set_default_decl(i, location.get_decl(i).clone());
            universal_location.set_default_decl(i, location.get_decl(i).clone());
        }

        //Rule 1
        if self.right.actions_contain(action, sync_type)
            && self.left.actions_contain(action, sync_type)
        {
            for t_transition in &t {
                for s_transition in &s {
                    // Guard for edge
                    // P_t && P_s
                    let mut guard_zone = t_transition.guard_zone.clone();
                    guard_zone.intersect(&s_transition.guard_zone);

                    // Inv(l1_s)
                    apply_right_invariant(location, &mut guard_zone);

                    // Inv(l2_t)[r |-> 0] where r is in clock resets for t
                    guard_zone.intersect(&t_resetted_invariant(&t_transition));

                    // Inv(l2_s)[r |-> 0] where r is in clock resets for t
                    guard_zone.intersect(&s_resetted_invariant(&s_transition));

                    let mut target_locations = LocationTuple::merge(
                        t_transition.target_locations.clone(),
                        &s_transition.target_locations,
                    );
                    target_locations.set_default_decl(quotient_index, self.decls.clone());

                    //Union of left and right updates
                    let mut updates = t_transition.updates.clone();
                    for (index, r_updates) in &s_transition.updates {
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
        if self.right.actions_contain(action, sync_type)
            && !self.left.actions_contain(action, sync_type)
        {
            let mut new_transitions = s.clone();
            for s_transition in &mut new_transitions {
                // Inv(l1_s)
                apply_right_invariant(location, &mut s_transition.guard_zone);

                // Inv(l2_s)[r |-> 0] where r is in clock resets for s
                s_transition
                    .guard_zone
                    .intersect(&s_resetted_invariant(&s_transition));

                s_transition.target_locations =
                    LocationTuple::merge(location.clone(), &s_transition.target_locations);
            }

            transitions.append(&mut new_transitions);
        }

        if is_output && self.right.get_output_actions().contains(action) {
            //Rule 3
            let mut g_s = Federation::empty(dim);

            for s_transition in &s {
                g_s.add_fed(&s_transition.guard_zone);
            }

            //Rule 4
            let mut g = Federation::empty(dim);
            for s_transition in &s {
                g.add_fed(&s_resetted_invariant(&s_transition));
            }

            //Rule 3 || Rule 4
            transitions.append(&mut create_transitions(
                (!g_s) + (!g),
                &universal_location,
                &HashMap::new(),
            ));
        }

        //Rule 5
        if (is_input && self.inputs.contains(action))
            || (is_output && self.outputs.contains(action))
        {
            let mut inv_l_s = Federation::full(dim);
            apply_right_invariant(location, &mut inv_l_s);

            //println!("Action {action} fed {inv_l_s}");

            transitions.append(&mut create_transitions(
                !inv_l_s,
                &universal_location,
                &HashMap::new(),
            ));
        }

        //Rule 6
        if is_output
            && self.right.get_output_actions().contains(action)
            && self.left.get_output_actions().contains(action)
        {
            //Calculate inverse G_T
            let mut g_t = Federation::empty(dim);
            for t_transition in &t {
                let mut zone = t_transition.guard_zone.clone();

                //Inv(l2_T)[r |-> 0] where r is in clock resets
                zone.intersect(&t_resetted_invariant(&t_transition));
                g_t.add_fed(&zone)
            }
            let inverse_g_t = !g_t;

            for s_transition in &s {
                let mut s_guard_zone = s_transition.guard_zone.clone();

                //Inv(l2_s)[r |-> 0] where r is in clock resets
                s_guard_zone.intersect(&s_resetted_invariant(&s_transition));
                let mut guard_zone = inverse_g_t.clone();

                guard_zone.intersect(&s_guard_zone);

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

        //Rule 7
        if *sync_type == SyncType::Input && action == self.new_input_name {
            let t_invariant = get_invariant(left_index, right_index, location, dim);
            let s_invariant = get_invariant(right_index, quotient_index, location, dim);
            let inverse_t_invariant = t_invariant.inverse();
            let mut guard_zone = inverse_t_invariant;
            guard_zone.intersect(&s_invariant);

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

        //Rule 8
        if self.left.actions_contain(action, sync_type)
            && !self.right.actions_contain(action, sync_type)
        {
            for mut t_transition in t {
                //Inv(l2_T)[r |-> 0] where r is in clock resets
                t_transition
                    .guard_zone
                    .intersect(&t_resetted_invariant(&t_transition));

                t_transition.target_locations =
                    LocationTuple::merge(location.clone(), &t_transition.target_locations);

                transitions.push(t_transition);
            }
        }

        transitions
    }

    fn is_locally_consistent(&self, dimensions: u32) -> bool {
        local_consistency::is_least_consistent(self, dimensions)
    }

    fn get_all_locations<'b>(&'b self, index: &mut usize) -> Vec<LocationTuple<'b>> {
        let mut location_tuples = vec![];
        let lowest_index = *index;

        let left = self.left.get_all_locations(index);
        let right = self.right.get_all_locations(index);
        for loc1 in left {
            for loc2 in &right {
                let mut location = LocationTuple::merge(loc1.clone(), &loc2);
                location.set_location(*index, None, self.decls.clone());
                location.ignore_all_invariants();
                location_tuples.push(location);
            }
        }

        let mut inconsistent =
            LocationTuple::simple_indexed(*index, &self.inconsistent_location, &self.decls);
        let mut universal =
            LocationTuple::simple_indexed(*index, &self.universal_location, &self.decls);
        let location = &location_tuples[0];
        for i in lowest_index..*index {
            inconsistent.set_default_decl(i, location.get_decl(i).clone());
            universal.set_default_decl(i, location.get_decl(i).clone());
        }

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
        let mut loc_tuple = LocationTuple::compose_iter(locations);
        loc_tuple.set_default_decl(loc_tuple.locations.len(), self.decls.clone());

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

    fn get_initial_state(&self, dimensions: u32) -> Option<State> {
        let mut init_loc = self.get_initial_location()?;
        init_loc.ignore_all_invariants();
        let zone = Federation::init(dimensions);
        Some(State {
            decorated_locations: init_loc,
            zone,
        })
    }

    fn set_clock_indices(&mut self, index: &mut u32) {
        unimplemented!();
    }

    fn get_mut_children(&mut self) -> Vec<&mut TransitionSystemPtr> {
        vec![&mut self.left, &mut self.right]
    }

    fn get_children(&self) -> Vec<&TransitionSystemPtr> {
        vec![&self.left, &self.right]
    }
}

fn apply_invariant_on_index(location: &LocationTuple, index: usize, guard_zone: &mut Federation) {
    if let Some(loc) = location.try_get_location(index) {
        let dec_loc = DecoratedLocation::create(loc, location.get_decl(index));
        dec_loc.apply_invariant(guard_zone);
    }
}

fn create_transitions<'a>(
    fed: Federation,
    target_locations: &LocationTuple<'a>,
    updates: &HashMap<usize, Vec<parse_edge::Update>>,
) -> Vec<Transition<'a>> {
    let mut transitions = vec![];

    transitions.push(Transition {
        guard_zone: fed,
        target_locations: target_locations.clone(),
        updates: updates.clone(),
    });

    transitions
}

fn get_resetted_invariant(
    start: usize,
    end: usize,
    location: &LocationTuple,
    updates_map: &HashMap<usize, Vec<parse_edge::Update>>,
    dim: u32,
) -> Federation {
    let mut zone = Federation::full(dim);
    for i in start..end {
        if let Some(loc) = location.try_get_location(i) {
            let location = DecoratedLocation::create(loc, location.get_decl(i));
            location.apply_invariant(&mut zone);
            // For some reason nessecary with a check for None?
        }
        if let Some(updates) = updates_map.get(&i) {
            updater(&updates, location.get_decl(i), &mut zone);
        }
    }
    zone
}

fn get_invariant(start: usize, end: usize, location: &LocationTuple, dim: u32) -> Federation {
    let mut zone = Federation::full(dim);
    for i in start..end {
        apply_invariant_on_index(location, i, &mut zone);
    }
    zone
}
