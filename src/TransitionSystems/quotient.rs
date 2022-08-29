use crate::DBMLib::dbm::Federation;

use crate::EdgeEval::updater::CompiledUpdate;
use crate::ModelObjects::component::Declarations;
use crate::ModelObjects::component::{Location, LocationType, State, Transition};
use crate::ModelObjects::max_bounds::MaxBounds;
use crate::ModelObjects::representations::BoolExpression;

use crate::TransitionSystems::{LocationTuple, TransitionSystem, TransitionSystemPtr};
use std::collections::hash_set::HashSet;

use super::CompositionType;

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
    }
}

impl TransitionSystem for Quotient {
    fn get_local_max_bounds(&self, loc: &LocationTuple) -> MaxBounds {
        if loc.is_universal() || loc.is_inconsistent() {
            MaxBounds::create(self.get_dim())
        } else {
            let (left, right) = self.get_children();
            let loc_l = loc.get_left();
            let loc_r = loc.get_right();
            let mut bounds_l = left.get_local_max_bounds(loc_l);
            let bounds_r = right.get_local_max_bounds(loc_r);
            bounds_l.add_bounds(&bounds_r);
            bounds_l
        }
    }

    fn next_transitions(&self, location: &LocationTuple, action: &str) -> Vec<Transition> {
        assert!(self.actions_contain(action));
        let is_input = self.inputs_contain(action);

        let mut transitions = vec![];

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
            transitions.push(transition);
            return transitions;
        }

        // As it is not universal or inconsistent it must be a quotient loc
        let loc_t = location.get_left();
        let loc_s = location.get_right();
        let t = self.T.next_transitions_if_available(loc_t, action);
        let s = self.S.next_transitions_if_available(loc_s, action);

        let inconsistent_location =
            LocationTuple::simple(&self.inconsistent_location, &self.decls, self.dim);
        let universal_location =
            LocationTuple::simple(&self.universal_location, &self.decls, self.dim);

        //Rule 1
        if self.S.actions_contain(action) && self.T.actions_contain(action) {
            for t_transition in &t {
                for s_transition in &s {
                    // In the following comments we use ϕ to symbolize the guard of the transition
                    // ϕ_T ∧ Inv(l2_t)[r |-> 0] ∧ Inv(l1_t)
                    let mut guard_zone = get_allowed_fed(&loc_t, t_transition);

                    // ϕ_T ∧ Inv(l2_t)[r |-> 0] ∧ Inv(l1_t) ∧ ϕ_S ∧ Inv(l2_s)[r |-> 0] ∧ Inv(l1_s)
                    guard_zone.intersect(&get_allowed_fed(&loc_s, s_transition));

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

        //Rule 2
        if self.S.actions_contain(action) && !self.T.actions_contain(action) {
            //Independent S
            for s_transition in &s {
                let guard_zone = get_allowed_fed(&loc_s, s_transition);

                let target_locations = merge(&loc_t, &s_transition.target_locations);
                let updates = s_transition.updates.clone();
                transitions.push(Transition {
                    guard_zone,
                    target_locations,
                    updates,
                });
            }
        }

        if self.S.get_output_actions().contains(action) {
            // new Rule 3 (includes rule 4 by de-morgan)
            let mut g_s = Federation::empty(self.dim);

            for s_transition in &s {
                let allowed_fed = get_allowed_fed(&loc_s, s_transition);
                g_s.add_fed(&allowed_fed);
            }

            // Rule 5 when Rule 3 applies
            let mut inv_l_s = Federation::full(self.dim);
            loc_s.apply_invariants(&mut inv_l_s);

            transitions.push(Transition {
                guard_zone: (!inv_l_s) + (!g_s),
                target_locations: universal_location.clone(),
                updates: vec![],
            });
        } else {
            // Rule 5 when Rule 3 does not apply
            let mut inv_l_s = Federation::full(self.dim);
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
            let mut g_t = Federation::empty(self.dim);
            for t_transition in &t {
                let allowed_fed = get_allowed_fed(&loc_t, t_transition);
                g_t.add_fed(&allowed_fed);
            }
            let inverse_g_t = !g_t;

            for s_transition in &s {
                // In the following comments we use ϕ to symbolize the guard of the transition
                // ϕ_S ∧ Inv(l2_s)[r |-> 0] ∧ Inv(l1_s)
                let mut guard_zone = get_allowed_fed(&loc_s, s_transition);

                // ϕ_S ∧ Inv(l2_s)[r |-> 0] ∧ Inv(l1_s) ∧ ¬G_T
                guard_zone.intersect(&inverse_g_t);

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
        if action == self.new_input_name {
            let inverse_t_invariant = !get_invariant(loc_t, self.dim);
            let s_invariant = get_invariant(loc_s, self.dim);
            let guard_zone = inverse_t_invariant.intersection(&s_invariant);

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
            for t_transition in &t {
                let mut guard_zone = get_allowed_fed(&loc_t, t_transition);

                loc_s.apply_invariants(&mut guard_zone);

                let target_locations = merge(&t_transition.target_locations, &loc_s);
                let updates = t_transition.updates.clone();

                transitions.push(Transition {
                    guard_zone,
                    target_locations,
                    updates,
                });
            }
        }

        transitions
            .into_iter()
            .filter(|e| e.guard_zone.is_valid())
            .collect()
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
        self.T.is_deterministic() && self.S.is_deterministic()
    }

    fn is_locally_consistent(&self) -> bool {
        self.T.is_locally_consistent() && self.S.is_locally_consistent()
    }

    fn get_initial_state(&self) -> Option<State> {
        let mut init_loc = self.get_initial_location()?;
        let zone = Federation::init(self.dim);
        Some(State {
            decorated_locations: init_loc,
            zone,
        })
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

fn merge(t: &LocationTuple, s: &LocationTuple) -> LocationTuple {
    LocationTuple::merge_as_quotient(t, s)
}

fn get_allowed_fed(from: &LocationTuple, transition: &Transition) -> Federation {
    let mut fed = transition.get_allowed_federation();
    from.apply_invariants(&mut fed);
    fed
}

fn get_invariant(loc: &LocationTuple, dim: u32) -> Federation {
    match loc.get_invariants() {
        Some(inv) => inv.clone(),
        None => Federation::full(dim),
    }
}
