use crate::DBMLib::dbm::Federation;
use crate::DataReader::parse_edge;
use crate::DataReader::parse_edge::Update;
use crate::EdgeEval::updater::CompiledUpdate;
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

use super::transition_system::CompositionType;

#[derive(Clone)]
pub struct Quotient {
    T: TransitionSystemPtr,
    S: TransitionSystemPtr,
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
        T: TransitionSystemPtr,
        S: TransitionSystemPtr,
        clock_index: &mut u32,
    ) -> TransitionSystemPtr {
        if !S.get_output_actions().is_disjoint(&T.get_input_actions()) {
            println!(
                "s_out and t_in not disjoint s_out: {:?} t_in {:?}",
                S.get_output_actions(),
                T.get_input_actions()
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

        let mut inputs: HashSet<String> = T
            .get_input_actions()
            .union(&S.get_output_actions())
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

        let output_dif: HashSet<String> = T
            .get_output_actions()
            .difference(&S.get_output_actions())
            .cloned()
            .collect();
        let input_dif: HashSet<String> = S
            .get_input_actions()
            .difference(&T.get_input_actions())
            .cloned()
            .collect();

        let outputs: HashSet<String> = output_dif.union(&input_dif).cloned().collect();

        *clock_index += 1;

        let mut decls = Declarations::empty();
        decls
            .clocks
            .insert("quotient_xnew".to_string(), *clock_index);

        let ts = Box::new(Quotient {
            T,
            S,
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

impl TransitionSystem for Quotient {
    fn next_transitions(
        &self,
        location: &LocationTuple,
        action: &str,
        sync_type: &SyncType,
        index: &mut usize,
        dim: u32,
    ) -> Vec<Transition> {
        let mut transitions = vec![];

        if !match sync_type {
            SyncType::Input => &self.inputs,
            SyncType::Output => &self.outputs,
        }
        .contains(action)
        {
            return vec![];
        }

        //Rules [universal] and [inconsistent]

        if location.is_inconsistent() {
            //Rule 10
            if self.inputs.contains(action) {
                let mut transition = Transition::new(location.clone(), dim);
                transition
                    .guard_zone
                    .add_eq_const_constraint(self.new_clock_index, 0);
                transitions.push(transition);
            }
            return transitions;
        } else if location.is_universal() {
            // Rule 9
            transitions.push(Transition::new(location.clone(), dim));
            return transitions;
        }

        // As it is not universal or inconsistent it must be a quotient loc
        let loc_t = location.get_left();
        let loc_s = location.get_right();
        let t = self
            .T
            .next_transitions(loc_t, action, sync_type, index, dim);
        let s = self
            .S
            .next_transitions(loc_s, action, sync_type, index, dim);

        let mut inconsistent_location =
            LocationTuple::simple(&self.inconsistent_location, &self.decls, dim);
        let mut universal_location =
            LocationTuple::simple(&self.universal_location, &self.decls, dim);

        //Rule 1
        if self.S.actions_contain(action) && self.T.actions_contain(action) {
            for t_transition in &t {
                for s_transition in &s {
                    // ϕ_T
                    let mut guard_zone = t_transition.guard_zone.clone();

                    // Inv(l2_s)[r |-> 0] where r is in clock resets for s
                    apply_resetted_invariant(&s_transition, &mut guard_zone);

                    // ϕ_S
                    s_transition.apply_guards(&mut guard_zone);

                    // Inv(l1_s)
                    loc_s.apply_invariants(&mut guard_zone);

                    // Inv(l2_t)[r |-> 0] where r is in clock resets for t
                    apply_resetted_invariant(&t_transition, &mut guard_zone);

                    let mut target_locations = merge(
                        &t_transition.target_locations,
                        &s_transition.target_locations,
                    );

                    //Union of left and right updates
                    let mut updates = t_transition.updates.clone();
                    updates.append(&mut s_transition.updates.clone());

                    transitions.push(Transition {
                        guard_zone,
                        target_locations,
                        updates,
                    });
                }
            }
        }

        //Rule 2
        if self.S.actions_contain(action) && !self.T.actions_contain(action) {
            let mut new_transitions = s.clone();
            for s_transition in &mut new_transitions {
                // Inv(l1_s)
                loc_s.apply_invariants(&mut s_transition.guard_zone);

                // Inv(l2_s)[r |-> 0] where r is in clock resets for s
                s_transition
                    .guard_zone
                    .intersect(&get_resetted_invariant(&s_transition, dim));

                s_transition.target_locations = merge(&loc_t, &s_transition.target_locations);
            }

            transitions.append(&mut new_transitions);
        }

        if self.S.get_output_actions().contains(action) {
            //Rule 3
            let mut g_s = Federation::empty(dim);

            for s_transition in &s {
                g_s.add_fed(&s_transition.guard_zone);
            }

            //Rule 4
            let mut g = Federation::empty(dim);
            for s_transition in &s {
                g.add_fed(&get_resetted_invariant(&s_transition, dim));
            }

            //Rule 3 || Rule 4
            transitions.push(Transition {
                guard_zone: (!g_s) + (!g),
                target_locations: universal_location.clone(),
                updates: vec![],
            });
        }

        //Rule 5
        if self.inputs.contains(action) || self.outputs.contains(action) {
            let mut inv_l_s = Federation::full(dim);
            loc_s.apply_invariants(&mut inv_l_s);

            transitions.push(Transition {
                guard_zone: !inv_l_s,
                target_locations: universal_location.clone(),
                updates: vec![],
            });
        }

        //Rule 6
        if self.S.get_output_actions().contains(action)
            && self.T.get_output_actions().contains(action)
        {
            //Calculate inverse G_T
            let mut g_t = Federation::empty(dim);
            for t_transition in &t {
                let mut zone = t_transition.guard_zone.clone();

                //Inv(l2_T)[r |-> 0] where r is in clock resets
                apply_resetted_invariant(&t_transition, &mut zone);
                g_t.add_fed(&zone)
            }
            let inverse_g_t = !g_t;

            for s_transition in &s {
                let mut s_guard_zone = s_transition.guard_zone.clone();

                //Inv(l2_s)[r |-> 0] where r is in clock resets
                apply_resetted_invariant(&s_transition, &mut s_guard_zone);

                let mut guard_zone = inverse_g_t.clone();

                guard_zone.intersect(&s_guard_zone);

                let updates = vec![CompiledUpdate {
                    clock_index: self.new_clock_index,
                    value: 0,
                }];

                transitions.push(Transition {
                    guard_zone,
                    target_locations: inconsistent_location.clone(),
                    updates,
                })
            }
        }

        //Rule 7
        if *sync_type == SyncType::Input && action == self.new_input_name {
            let t_invariant = get_invariant(loc_t, dim);
            let s_invariant = get_invariant(loc_s, dim);
            let inverse_t_invariant = t_invariant.inverse();
            let mut guard_zone = inverse_t_invariant;
            guard_zone.intersect(&s_invariant);

            let updates = vec![CompiledUpdate {
                clock_index: self.new_clock_index,
                value: 0,
            }];

            transitions.push(Transition {
                guard_zone,
                target_locations: inconsistent_location.clone(),
                updates,
            })
        }

        //Rule 8
        if self.T.actions_contain(action) && !self.S.actions_contain(action) {
            for mut t_transition in t {
                //Inv(l2_T)[r |-> 0] where r is in clock resets
                t_transition
                    .guard_zone
                    .intersect(&get_resetted_invariant(&t_transition, dim));

                t_transition.target_locations = merge(&t_transition.target_locations, &loc_s);

                transitions.push(t_transition);
            }
        }

        transitions
    }

    fn is_locally_consistent(&self, dimensions: u32) -> bool {
        local_consistency::is_least_consistent(self, dimensions)
    }

    fn get_all_locations(&self, dim: u32) -> Vec<LocationTuple> {
        let mut location_tuples = vec![];

        let left = self.T.get_all_locations(dim);
        let right = self.S.get_all_locations(dim);
        for loc_t in left {
            for loc_s in &right {
                let mut location = merge(&loc_t, &loc_s);
                location_tuples.push(location);
            }
        }

        let mut inconsistent = LocationTuple::simple(&self.inconsistent_location, &self.decls, dim);
        let mut universal = LocationTuple::simple(&self.universal_location, &self.decls, dim);

        location_tuples.push(inconsistent);
        location_tuples.push(universal);

        location_tuples
    }

    fn get_max_bounds(&self, dim: u32) -> MaxBounds {
        let mut bounds = self.T.get_max_bounds(dim);
        bounds.add_bounds(&self.S.get_max_bounds(dim));
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
        let (left, right) = self.get_children();
        1 + left.get_num_clocks() + right.get_num_clocks()
    }
    fn get_initial_location(&self, dim: u32) -> Option<LocationTuple> {
        let (t, s) = self.get_children();
        Some(merge(
            &t.get_initial_location(dim)?,
            &s.get_initial_location(dim)?,
        ))
    }

    fn get_decls(&self) -> Vec<&Declarations> {
        let mut comps = self.T.get_decls();
        comps.extend(self.S.get_decls());
        comps.push(&self.decls);
        comps
    }

    fn get_max_clock_index(&self) -> u32 {
        std::cmp::max(self.T.get_max_clock_index(), self.S.get_max_clock_index()) + 1
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
        self.T.is_deterministic(dim) && self.S.is_deterministic(dim)
    }

    fn get_initial_state(&self, dimensions: u32) -> Option<State> {
        let mut init_loc = self.get_initial_location(dimensions)?;
        let zone = Federation::init(dimensions);
        Some(State {
            decorated_locations: init_loc,
            zone,
        })
    }

    fn set_clock_indices(&mut self, index: &mut u32) {
        unimplemented!();
    }

    fn get_mut_children(&mut self) -> (&mut TransitionSystemPtr, &mut TransitionSystemPtr) {
        (&mut self.T, &mut self.S)
    }

    fn get_children(&self) -> (&TransitionSystemPtr, &TransitionSystemPtr) {
        (&self.T, &self.S)
    }

    fn get_composition_type(&self) -> CompositionType {
        CompositionType::Quotient
    }
}
/*
fn create_transitions(
    fed: Federation,
    target_locations: &LocationTuple,
    updates: &HashMap<usize, Vec<parse_edge::Update>>,
) -> Vec<Transition> {
    let mut transitions = vec![];

    transitions.push(Transition {
        guard_zone: fed,
        target_locations: target_locations.clone(),
        updates: updates.clone(),
    });

    transitions
}*/

fn merge(t: &LocationTuple, s: &LocationTuple) -> LocationTuple {
    LocationTuple::merge(t, s, CompositionType::Quotient)
}

fn get_resetted_invariant(transition: &Transition, dim: u32) -> Federation {
    match transition.target_locations.get_invariants() {
        Some(inv) => {
            let mut fed = inv.clone();
            transition.inverse_apply_updates(&mut fed);
            fed
        }
        None => Federation::full(dim),
    }
}

fn apply_resetted_invariant(transition: &Transition, fed: &mut Federation) {
    if let Some(inv) = transition.target_locations.get_invariants() {
        fed.intersect(&inv);
        transition.inverse_apply_updates(fed);
    };
}

fn get_invariant(loc: &LocationTuple, dim: u32) -> Federation {
    match loc.get_invariants() {
        Some(inv) => inv.clone(),
        None => Federation::full(dim),
    }
}
