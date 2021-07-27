use crate::DBMLib::dbm::Zone;
use crate::ModelObjects::component::{DecoratedLocation, State, SyncType, Transition};
use crate::ModelObjects::component_view::ComponentView;
use crate::ModelObjects::max_bounds::MaxBounds;
use crate::ModelObjects::representations::SystemRepresentation;
use crate::ModelObjects::system_declarations::SystemDeclarations;
use crate::System::local_consistency;
use std::cell::RefCell;
use std::collections::HashSet;

#[derive(Clone)]
pub struct UncachedSystem<'a> {
    base_representation: SystemRepresentation<'a>,
}

#[allow(dead_code)]
impl<'a> UncachedSystem<'a> {
    pub fn create(base_representation: SystemRepresentation<'a>) -> Self {
        UncachedSystem {
            base_representation,
        }
    }

    pub fn cache(
        system: UncachedSystem<'a>,
        dimensions: u32,
        sys_decls: &SystemDeclarations,
    ) -> System<'a> {
        let max_bounds = system.get_max_bounds(dimensions);
        let input_actions = system.get_input_actions(sys_decls);
        let output_actions = system.get_output_actions(sys_decls);

        let cache = System {
            base_representation: system.move_represetation(),
            max_bounds,
            input_actions,
            output_actions,
            initial_locations: RefCell::default(),
        };

        cache
    }

    pub fn move_represetation(self) -> SystemRepresentation<'a> {
        self.base_representation
    }

    pub fn borrow_representation(&self) -> &SystemRepresentation<'a> {
        &self.base_representation
    }

    pub fn create_initial_state(&self) -> State {
        let dimensions = self.get_clock_count() + 1;
        let init_loc = self.get_initial_locations();
        let mut zone = Zone::init(dimensions);
        for loc in &init_loc {
            if !loc.apply_invariant(&mut zone) {
                panic!("Invalid initial state");
            }
        }

        State {
            decorated_locations: init_loc,
            zone,
        }
    }

    pub fn get_max_bounds(&self, dimensions: u32) -> MaxBounds {
        self.base_representation.get_max_bounds(dimensions)
    }

    pub fn collect_next_inputs(
        &self,
        locations: &[DecoratedLocation],
        action: &str,
    ) -> Vec<Transition> {
        let mut transitions: Vec<Transition> = vec![];
        let mut index = 0;

        self.base_representation.collect_next_transitions(
            locations,
            &mut index,
            action,
            &mut transitions,
            &SyncType::Input,
        );

        transitions
    }

    pub fn collect_next_outputs(
        &self,
        locations: &[DecoratedLocation],
        action: &str,
    ) -> Vec<Transition> {
        let mut transitions: Vec<Transition> = vec![];
        let mut index = 0;

        self.base_representation.collect_next_transitions(
            locations,
            &mut index,
            action,
            &mut transitions,
            &SyncType::Output,
        );

        transitions
    }

    pub fn get_input_actions(&self, sys_decls: &SystemDeclarations) -> HashSet<String> {
        self.base_representation.get_input_actions(sys_decls)
    }

    pub fn get_output_actions(&self, sys_decls: &SystemDeclarations) -> HashSet<String> {
        self.base_representation.get_output_actions(sys_decls)
    }

    pub fn find_matching_input(
        &self,
        sys_decls: &SystemDeclarations,
        inputs2: &[String],
    ) -> Vec<String> {
        self.base_representation
            .find_matching_input(sys_decls, inputs2)
    }

    pub fn find_matching_output(
        &self,
        sys_decls: &SystemDeclarations,
        outputs1: &[String],
    ) -> Vec<String> {
        self.base_representation
            .find_matching_output(sys_decls, outputs1)
    }

    pub fn get_initial_locations(&self) -> Vec<DecoratedLocation> {
        self.base_representation.get_initial_locations()
    }

    pub fn get_clock_count(&self) -> u32 {
        let mut clocks = 0;

        self.base_representation
            .all_components(&mut |comp_view: &ComponentView| {
                clocks += comp_view.clock_count() as u32;

                true
            });

        clocks
    }

    pub fn all_components_are_deterministic(&self) -> bool {
        self.base_representation
            .all_components(&mut |comp| comp.get_component().is_deterministic())
    }

    pub fn precheck_sys_rep(&self, dimensions: u32, sys_decls: &SystemDeclarations) -> bool {
        if !self.all_components_are_deterministic() {
            println!("NOT DETERMINISTIC");
            return false;
        }

        if !self
            .base_representation
            .consistency_check(dimensions, sys_decls)
        {
            println!("NOT LOCALLY CONSISTENT");
            return false;
        }

        true
    }
}

#[derive(Clone)]
pub struct System<'a> {
    base_representation: SystemRepresentation<'a>,
    max_bounds: MaxBounds,
    input_actions: HashSet<String>,
    output_actions: HashSet<String>,
    initial_locations: RefCell<Option<Vec<DecoratedLocation<'a>>>>,
}

#[allow(dead_code)]
impl<'a> System<'a> {
    pub fn create(
        base_representation: SystemRepresentation<'a>,
        dimensions: u32,
        sys_decls: &SystemDeclarations,
    ) -> System<'a> {
        let system = UncachedSystem {
            base_representation,
        };

        UncachedSystem::cache(system, dimensions, sys_decls)
    }

    pub fn precheck_sys_rep(&self, dimensions: u32, sys_decls: &SystemDeclarations) -> bool {
        if !self.all_components_are_deterministic() {
            println!("NOT DETERMINISTIC");
            return false;
        }

        if !self
            .base_representation
            .consistency_check(dimensions, sys_decls)
        {
            println!("NOT LOCALLY CONSISTENT");
            return false;
        }

        true
    }

    pub fn collect_next_inputs(
        &self,
        locations: &[DecoratedLocation],
        action: &str,
    ) -> Vec<Transition> {
        let mut transitions: Vec<Transition> = vec![];
        let mut index = 0;

        self.base_representation.collect_next_transitions(
            locations,
            &mut index,
            action,
            &mut transitions,
            &SyncType::Input,
        );

        transitions
    }

    pub fn collect_next_outputs(
        &self,
        locations: &[DecoratedLocation],
        action: &str,
    ) -> Vec<Transition> {
        let mut transitions: Vec<Transition> = vec![];
        let mut index = 0;

        self.base_representation.collect_next_transitions(
            locations,
            &mut index,
            action,
            &mut transitions,
            &SyncType::Output,
        );

        transitions
    }

    pub fn create_initial_state(&self, dimensions: u32) -> State {
        let init_loc = self.base_representation.get_initial_locations();
        let mut zone = Zone::init(dimensions);
        for loc in &init_loc {
            if !loc.apply_invariant(&mut zone) {
                panic!("Invalid initial state");
            }
        }

        State {
            decorated_locations: init_loc,
            zone,
        }
    }

    pub fn get_clock_count(&self) -> u32 {
        let mut clocks = 0;

        self.base_representation
            .all_components(&mut |comp_view: &ComponentView| {
                clocks += comp_view.clock_count() as u32;

                true
            });

        clocks
    }

    pub fn move_represetation(self) -> SystemRepresentation<'a> {
        self.base_representation
    }

    pub fn get_max_bounds(&self) -> &MaxBounds {
        &self.max_bounds
    }

    pub fn get_input_actions(&self) -> &HashSet<String> {
        &self.input_actions
    }

    pub fn get_output_actions(&self) -> &HashSet<String> {
        &self.output_actions
    }

    pub fn get_initial_locations(&'a self) -> Vec<DecoratedLocation> {
        self.initial_locations
            .borrow_mut()
            .get_or_insert_with(|| self.base_representation.get_initial_locations())
            .clone()
    }

    pub fn all_components_are_deterministic(&self) -> bool {
        self.base_representation
            .all_components(&mut |comp| comp.get_component().is_deterministic())
    }
}
