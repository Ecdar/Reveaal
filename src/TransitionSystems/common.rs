macro_rules! default_composition {
    () => {
        fn get_max_bounds(&self, dim: u32) -> MaxBounds {
            let mut bounds = self.left.get_max_bounds(dim);
            bounds.add_bounds(&self.right.get_max_bounds(dim));
            bounds
        }
        fn get_input_actions(&self) -> Result<HashSet<String>> {
            Ok(self.inputs.clone())
        }
        fn get_output_actions(&self) -> Result<HashSet<String>> {
            Ok(self.outputs.clone())
        }
        fn get_num_clocks(&self) -> u32 {
            self.left.get_num_clocks() + self.right.get_num_clocks()
        }
        fn get_initial_location<'b>(&'b self) -> Option<LocationTuple<'b>> {
            if let Some(left) = self.left.get_initial_location() {
                if let Some(right) = self.right.get_initial_location() {
                    return Some(LocationTuple::compose(left, right));
                }
            }
            None
        }
        fn get_all_locations<'b>(&'b self) -> Vec<LocationTuple<'b>> {
            let mut location_tuples = vec![];
            let left = self.left.get_all_locations();
            let right = self.right.get_all_locations();
            for loc1 in left {
                for loc2 in &right {
                    location_tuples.push(LocationTuple::compose(loc1.clone(), loc2.clone()));
                }
            }
            location_tuples
        }

        fn get_components<'b>(&'b self) -> Vec<&'b Component> {
            let mut comps = self.left.get_components();
            comps.extend(self.right.get_components());
            comps
        }

        fn set_clock_indices(&mut self, index: &mut u32) {
            self.left.set_clock_indices(index);
            self.right.set_clock_indices(index);
        }

        fn get_max_clock_index(&self) -> u32 {
            std::cmp::max(
                self.left.get_max_clock_index(),
                self.right.get_max_clock_index(),
            )
        }

        fn precheck_sys_rep(&self, dim: u32) -> Result<bool> {
            if !self.is_deterministic(dim)? {
                println!("NOT DETERMINISTIC");
                return Ok(false);
            }

            if !self.is_locally_consistent(dim)? {
                println!("NOT CONSISTENT");
                return Ok(false);
            }

            Ok(true)
        }

        fn is_deterministic(&self, dim: u32) -> Result<bool> {
            Ok(self.left.is_deterministic(dim)? && self.right.is_deterministic(dim)?)
        }

        fn get_initial_state(&self, dimensions: u32) -> Result<State> {
            let init_loc = match self.get_initial_location() {
                Some(init_loc) => init_loc,
                None => bail!("Cannot create initial state as there is no initial location"),
            };
            let mut zone = Zone::init(dimensions);
            if !init_loc.apply_invariants(&mut zone)? {
                bail!("Invalid starting state");
            }

            Ok(State {
                decorated_locations: init_loc,
                zone,
            })
        }
    };
}
