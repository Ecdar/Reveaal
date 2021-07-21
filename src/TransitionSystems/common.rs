macro_rules! default_composition {
    () => {
        fn get_max_bounds(&self) -> MaxBounds {
            panic!["Not implemented"];
        }
        fn get_input_actions(&self) -> HashSet<String> {
            self.inputs.clone()
        }
        fn get_output_actions(&self) -> HashSet<String> {
            self.outputs.clone()
        }
        fn get_num_clocks(&self) -> u32 {
            self.left.get_num_clocks() + self.right.get_num_clocks()
        }
        fn get_initial_location<'b>(&'b self) -> LocationTuple<'b> {
            LocationTuple::compose(
                self.left.get_initial_location(),
                self.right.get_initial_location(),
            )
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
    };
}
