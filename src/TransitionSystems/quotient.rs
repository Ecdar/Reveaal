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

    dim: u32,
}

static INCONSISTENT_LOC_NAME: &str = "Inconsistent";
static UNIVERSAL_LOC_NAME: &str = "Universal";
impl Quotient {
    pub fn new(
        T: TransitionSystemPtr,
        S: TransitionSystemPtr,
        new_clock_index: u32,
        dim: u32,
    ) -> Result<TransitionSystemPtr, String> {
        if !S.get_output_actions().is_disjoint(&T.get_input_actions()) {
            return Err(format!(
                "s_out and t_in not disjoint in quotient! s_out: {:?} t_in {:?}",
                S.get_output_actions(),
                T.get_input_actions()
            ));
        }

        if !T.precheck_sys_rep() {
            return Err("T (left) must be least consistent for quotient".to_string());
        }

        if !S.precheck_sys_rep() {
            return Err("S (right) must be least consistent for quotient".to_string());
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

        let mut decls = Declarations::empty();
        decls
            .clocks
            .insert("quotient_xnew".to_string(), new_clock_index);

        println!("S//T Inputs: {inputs:?}, Outputs: {outputs:?}");
        println!(
            "S Inputs: {:?}, Outputs: {:?}",
            S.get_input_actions(),
            S.get_output_actions()
        );
        println!(
            "T Inputs: {:?}, Outputs: {:?}",
            T.get_input_actions(),
            T.get_output_actions()
        );

        let ts = Box::new(Quotient {
            T,
            S,
            inputs,
            outputs,
            universal_location,
            inconsistent_location,
            decls,
            new_clock_index,
            new_input_name,
            dim,
        });
        Ok(ts)

        //Ok(pruning::prune_system(ts, dim))
    }
}

impl TransitionSystem for Quotient {
    fn next_transitions(&self, location: &LocationTuple, action: &str) -> Vec<Transition> {
        //println!("Action: {}", action);
        assert!(self.actions_contain(action));
        let is_input = self.inputs_contain(action);

        let mut transitions = vec![];
        let mut reset_all = vec![];
        for clock in self
            .get_decls()
            .iter()
            .flat_map(|d| d.get_clocks().values())
            .copied()
        {
            reset_all.push(CompiledUpdate {
                clock_index: clock,
                value: 0,
            });
        }

        //Rules [universal] and [inconsistent]

        if location.is_inconsistent() {
            //Rule 10
            if is_input {
                let mut transition = Transition::new(location, self.dim);
                transition
                    .guard_zone
                    .add_eq_const_constraint(self.new_clock_index, 0);
                transitions.push(transition);
            }
            return transitions;
        } else if location.is_universal() {
            // Rule 9
            let mut transition = Transition::new(location, self.dim);
            /* This should be okay */
            transition.updates = reset_all.clone();
            transitions.push(transition);
            return transitions;
        }

        // As it is not universal or inconsistent it must be a quotient loc
        let loc_t = location.get_left();
        let loc_s = location.get_right();
        let t = self.T.next_transitions_if_available(loc_t, action);
        let s = self.S.next_transitions_if_available(loc_s, action);

        let mut inconsistent_location =
            LocationTuple::simple(&self.inconsistent_location, &self.decls, self.dim);
        let mut universal_location =
            LocationTuple::simple(&self.universal_location, &self.decls, self.dim);

        //Rule 1
        if self.S.actions_contain(action) && self.T.actions_contain(action) {
            for t_transition in &t {
                for s_transition in &s {
                    // ϕ_T
                    let mut guard_zone = t_transition.guard_zone.clone();

                    // Inv(l2_t)[r |-> 0] where r is in clock resets for s
                    apply_resetted_invariant(&t_transition, &mut guard_zone);

                    // ϕ_S
                    s_transition.apply_guards(&mut guard_zone);

                    // Inv(l1_s)
                    loc_s.apply_invariants(&mut guard_zone);

                    // Inv(l2_s)[r |-> 0] where r is in clock resets for t
                    apply_resetted_invariant(&s_transition, &mut guard_zone);

                    let mut target_locations = merge(
                        &t_transition.target_locations,
                        &s_transition.target_locations,
                    );

                    //Union of left and right updates
                    let mut updates = t_transition.updates.clone();
                    updates.append(&mut s_transition.updates.clone());

                    println!("Rule 1: {}", guard_zone);

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
            //Independent S
            let mut new_transitions = s.clone();
            for s_transition in &mut new_transitions {
                // Inv(l1_s)
                loc_s.apply_invariants(&mut s_transition.guard_zone);

                // Inv(l2_s)[r |-> 0] where r is in clock resets for s
                s_transition
                    .guard_zone
                    .intersect(&get_resetted_invariant(&s_transition, self.dim));

                println!("Rule 2: {}", s_transition.guard_zone);

                s_transition.target_locations = merge(&loc_t, &s_transition.target_locations);
            }

            transitions.append(&mut new_transitions);
        }

        if self.S.get_output_actions().contains(action) {
            // Rule 3
            let mut g_s = Federation::empty(self.dim);

            for s_transition in &s {
                g_s.add_fed(&s_transition.guard_zone);
            }

            // Rule 4
            let mut g = Federation::empty(self.dim);
            for s_transition in &s {
                g.add_fed(&get_resetted_invariant(&s_transition, self.dim));
            }

            // Rule 5
            let mut inv_l_s = Federation::full(self.dim);
            loc_s.apply_invariants(&mut inv_l_s);

            println!("Rule 3: {}", g_s.inverse());
            println!("Rule 4: {}", g.inverse());
            println!("Rule 5: {}", inv_l_s.inverse());

            // Rule 3 || Rule 4 || Rule 5
            // Combine the rules as they target the same location (universal)
            transitions.push(Transition {
                guard_zone: (!g_s) + (!g) + (!inv_l_s),
                target_locations: universal_location.clone(),
                updates: reset_all.clone(),
            });
        } else {
            // Rule 5
            let mut inv_l_s = Federation::full(self.dim);
            loc_s.apply_invariants(&mut inv_l_s);

            println!("Rule 5: {}", inv_l_s.inverse());

            transitions.push(Transition {
                guard_zone: !inv_l_s,
                target_locations: universal_location.clone(),
                updates: reset_all.clone(),
            });
        }

        //Rule 6
        if self.S.get_output_actions().contains(action)
            && self.T.get_output_actions().contains(action)
        {
            //Calculate inverse G_T
            let mut g_t = Federation::empty(self.dim);
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

                println!("Rule 6: {}", guard_zone);

                transitions.push(Transition {
                    guard_zone,
                    target_locations: inconsistent_location.clone(),
                    updates,
                })
            }
        }

        //Rule 7
        if action == self.new_input_name {
            let inverse_t_invariant = !get_invariant(loc_t, self.dim);
            println!("!inv_t: {}", inverse_t_invariant);
            let s_invariant = get_invariant(loc_s, self.dim);
            println!("inv_s: {}", s_invariant);
            let guard_zone = inverse_t_invariant.intersection(&s_invariant);

            let updates = vec![CompiledUpdate {
                clock_index: self.new_clock_index,
                value: 0,
            }];

            println!("Rule 7: {}", guard_zone);

            transitions.push(Transition {
                guard_zone,
                target_locations: inconsistent_location.clone(),
                updates,
            })
        }

        //Rule 8
        if self.T.actions_contain(action) && !self.S.actions_contain(action) {
            for mut t_transition in t {
                // TODO: This is new
                loc_s.apply_invariants(&mut t_transition.guard_zone);
                /*println!(
                    "Invariant_s: {}",
                    loc_s.get_invariants().unwrap_or(&Federation::full(1))
                );*/

                //Inv(l2_T)[r |-> 0] where r is in clock resets
                t_transition
                    .guard_zone
                    .intersect(&get_resetted_invariant(&t_transition, self.dim));

                t_transition.target_locations = merge(&t_transition.target_locations, &loc_s);

                println!("Rule 8: {}", t_transition.guard_zone);
                transitions.push(t_transition);
            }
        }

        transitions
            .into_iter()
            .filter(|e| e.guard_zone.is_valid())
            .collect()
    }

    fn is_locally_consistent(&self) -> bool {
        local_consistency::is_least_consistent(self)
    }

    fn get_all_locations(&self) -> Vec<LocationTuple> {
        let mut location_tuples = vec![];

        let left = self.T.get_all_locations();
        let right = self.S.get_all_locations();
        for loc_t in left {
            for loc_s in &right {
                let mut location = merge(&loc_t, &loc_s);
                location_tuples.push(location);
            }
        }

        let mut inconsistent =
            LocationTuple::simple(&self.inconsistent_location, &self.decls, self.dim);
        let mut universal = LocationTuple::simple(&self.universal_location, &self.decls, self.dim);

        location_tuples.push(inconsistent);
        location_tuples.push(universal);

        location_tuples
    }

    fn get_max_bounds(&self) -> MaxBounds {
        let mut bounds = self.T.get_max_bounds();
        bounds.add_bounds(&self.S.get_max_bounds());
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
        self.inputs.union(&self.outputs).cloned().collect()
    }
    fn get_initial_location(&self) -> Option<LocationTuple> {
        let (t, s) = self.get_children();
        Some(merge(
            &t.get_initial_location()?,
            &s.get_initial_location()?,
        ))
    }

    fn get_decls(&self) -> Vec<&Declarations> {
        let mut comps = self.T.get_decls();
        comps.extend(self.S.get_decls());
        comps.push(&self.decls);
        comps
    }

    fn precheck_sys_rep(&self) -> bool {
        if !self.is_deterministic() {
            println!("NOT DETERMINISTIC");
            return false;
        }

        if !self.is_locally_consistent() {
            println!("NOT CONSISTENT");
            return false;
        }

        true
    }

    fn is_deterministic(&self) -> bool {
        local_consistency::is_deterministic(self)
        //self.T.is_deterministic() && self.S.is_deterministic()
    }

    fn get_initial_state(&self) -> Option<State> {
        let mut init_loc = self.get_initial_location()?;
        let zone = Federation::init(self.dim);
        Some(State {
            decorated_locations: init_loc,
            zone,
        })
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

    fn get_dim(&self) -> u32 {
        self.dim
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

fn get_resetted_invariant3(transition: &Transition, dim: u32) -> Federation {
    match transition.target_locations.get_invariants() {
        Some(inv) => {
            let mut fed = inv.clone();
            transition.inverse_apply_updates(&mut fed);
            //transition.apply_updates(&mut fed);
            fed
        }
        None => Federation::full(dim),
    }
}

fn get_resetted_invariant2(transition: &Transition, dim: u32) -> Federation {
    match transition.target_locations.get_invariants() {
        Some(inv) => {
            let mut fed = inv.clone();
            //transition.inverse_apply_updates(&mut fed);
            transition.apply_updates(&mut fed);
            fed
        }
        None => Federation::full(dim),
    }
}

fn get_resetted_invariant1(transition: &Transition, dim: u32) -> Federation {
    match transition.target_locations.get_invariants() {
        Some(inv) => inv.inverse(),
        None => Federation::empty(dim),
    }
}

fn get_resetted_invariant(transition: &Transition, dim: u32) -> Federation {
    get_resetted_invariant3(transition, dim)
}

fn apply_resetted_invariant(transition: &Transition, fed: &mut Federation) {
    fed.intersect(&get_resetted_invariant(transition, fed.get_dimensions()));
}

fn get_invariant(loc: &LocationTuple, dim: u32) -> Federation {
    match loc.get_invariants() {
        Some(inv) => inv.clone(),
        None => Federation::full(dim),
    }
}

/*

*/

/*
fn next_transitions(&self, location: &LocationTuple, action: &str) -> Vec<Transition> {
       assert!(self.actions_contain(action));
       let is_input = self.inputs_contain(action);

       let mut transitions = vec![];

       //Rules [universal] and [inconsistent]

       if location.is_inconsistent() {
           //Rule 7
           if is_input {
               let mut transition = Transition::new(location, self.dim);
               transition
                   .guard_zone
                   .add_eq_const_constraint(self.new_clock_index, 0);
               transitions.push(transition);
           }
           return transitions;
       } else if location.is_universal() {
           // Rule 8
           println!("Adding universal");
           transitions.push(Transition::new(location, self.dim));
           return transitions;
       }

       // As it is not universal or inconsistent it must be a quotient loc
       let loc_t = location.get_left();
       let loc_s = location.get_right();
       let t = self.T.next_transitions_if_available(loc_t, action);
       let s = self.S.next_transitions_if_available(loc_s, action);

       let mut inconsistent_location =
           LocationTuple::simple(&self.inconsistent_location, &self.decls, self.dim);
       let mut universal_location =
           LocationTuple::simple(&self.universal_location, &self.decls, self.dim);

       //Rule 1
       {
           // inv_s
           let inv_s = get_invariant(loc_s, self.dim);
           // {x_new}
           let updates = vec![CompiledUpdate {
               clock_index: self.new_clock_index,
               value: 0,
           }];
           transitions.push(Transition {
               guard_zone: !inv_s,
               target_locations: universal_location.clone(),
               updates,
           });
       }

       //Rule 2
       if action == self.new_input_name {
           // inv_s
           let inv_s = get_invariant(loc_s, self.dim);

           // inv_t
           let inv_t = !get_invariant(loc_t, self.dim);

           // inv_s ∧ ¬inv_t
           let guard_zone = inv_s.intersection(&!inv_t);

           // {x_new}
           let updates = vec![CompiledUpdate {
               clock_index: self.new_clock_index,
               value: 0,
           }];
           transitions.push(Transition {
               guard_zone,
               target_locations: inconsistent_location.clone(),
               updates,
           });
       }

       //Rule 3
       if self.S.actions_contain(action) && self.T.actions_contain(action) {
           for t_transition in &t {
               for s_transition in &s {
                   // ϕ_T
                   let mut guard_zone = t_transition.guard_zone.clone();

                   // ϕ_T ∧ ϕ_S
                   s_transition.apply_guards(&mut guard_zone);

                   let target_locations = merge(
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

       //Rule 4
       if self.S.outputs_contain(action) {
           // Rule 4
           let mut G_T = Federation::empty(self.dim);
           for transition_t in &t {
               G_T.add_fed(&transition_t.guard_zone);
           }
           let neg_G_T = !G_T;

           for transition_s in &s {
               //ϕ_S
               let mut guard_zone = transition_s.guard_zone.clone();

               //ϕ_S ∧ ¬G_T
               guard_zone.intersect(&neg_G_T);

               // {x_new}
               let updates = vec![CompiledUpdate {
                   clock_index: self.new_clock_index,
                   value: 0,
               }];

               transitions.push(Transition {
                   guard_zone,
                   target_locations: inconsistent_location.clone(),
                   updates,
               });
           }
       }

       //Rule 5
       if self.T.actions_contain(action) && !self.S.actions_contain(action) {
           for mut transition_t in t.clone() {
               transition_t.target_locations = merge(&transition_t.target_locations, &loc_s);
               transitions.push(transition_t);
           }
       }

       //Rule 6
       if self.S.outputs_contain(action) {
           // Rule 6
           let mut G_S = Federation::empty(self.dim);
           for transition_s in &s {
               G_S.add_fed(&transition_s.guard_zone);
           }

           if !t.is_empty() {
               transitions.push(Transition {
                   guard_zone: !G_S,
                   target_locations: universal_location.clone(),
                   updates: vec![],
               });
           }
       }

       transitions
   }
*/
