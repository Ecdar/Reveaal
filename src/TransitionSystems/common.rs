macro_rules! default_composition {
    () => {
        fn get_dim(&self) -> ClockIndex {
            self.dim
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

        fn get_local_max_bounds(&self, loc: &LocationTuple) -> Bounds {
            if loc.is_universal() || loc.is_inconsistent() {
                Bounds::new(self.get_dim())
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

        fn get_initial_location(&self) -> Option<LocationTuple> {
            let (left, right) = self.get_children();
            let l = left.get_initial_location()?;
            let r = right.get_initial_location()?;

            Some(LocationTuple::compose(&l, &r, self.get_composition_type()))
        }

        fn get_decls(&self) -> Vec<&Declarations> {
            let mut comps = self.left.get_decls();
            comps.extend(self.right.get_decls());
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
            //local_consistency::is_deterministic(self)
            self.left.is_deterministic() && self.right.is_deterministic()
        }

        fn get_initial_state(&self) -> Option<State> {
            let init_loc = self.get_initial_location().unwrap();
            let mut zone = OwnedFederation::init(self.dim);

            zone = init_loc.apply_invariants(zone);
            if zone.is_empty() {
                println!("Empty initial state");
                return None;
            }

            Some(State::create(init_loc, zone))
        }
    };
}
