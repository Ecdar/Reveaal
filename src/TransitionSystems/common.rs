macro_rules! default_composition {
    () => {
        fn get_max_bounds(&self, dim: u32) -> MaxBounds {
            let mut bounds = self.left.get_max_bounds(dim);
            bounds.add_bounds(&self.right.get_max_bounds(dim));
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
                .fold(0, |accumulator, child| accumulator + child.get_num_clocks())
        }
        fn get_initial_location<'b>(&'b self) -> Option<LocationTuple<'b>> {
            let mut locations = vec![];

            for child in self.get_children() {
                locations.push(child.get_initial_location()?);
            }
            Some(LocationTuple::compose_iter(locations))
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
            )
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
            let init_loc = self.get_initial_location().unwrap();
            let mut zone = Federation::init(dimensions);
            if !init_loc.apply_invariants(&mut zone) {
                println!("Empty initial state");
                return None;
            }

            Some(State {
                decorated_locations: init_loc,
                zone,
            })
        }

        fn set_clock_indices(&mut self, index: &mut u32) {
            for child in self.get_mut_children() {
                child.set_clock_indices(index);
            }
        }
    };
}
