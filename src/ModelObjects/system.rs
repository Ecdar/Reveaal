use crate::ModelObjects::component::{
    Component, DecoratedLocation, LocationType, SyncType, Transition,
};
use crate::ModelObjects::component_view::ComponentView;
use crate::ModelObjects::max_bounds::MaxBounds;
use crate::ModelObjects::representations::SystemRepresentation;
use crate::ModelObjects::system_declarations::SystemDeclarations;
use std::cell::RefCell;

#[derive(Clone)]
pub struct UncachedSystem<'a> {
    base_representation: SystemRepresentation<'a>,
}

impl<'a> UncachedSystem<'a> {
    pub fn create(base_representation: SystemRepresentation<'a>) -> Self {
        UncachedSystem {
            base_representation,
        }
    }

    pub fn cache<'b>(
        system: UncachedSystem<'a>,
        dimensions: u32,
        sys_decls: &SystemDeclarations,
    ) -> System<'a> {
        let max_bounds = system.get_max_bounds(dimensions).clone();
        let input_actions = system.get_input_actions(sys_decls).clone();
        let output_actions = system.get_output_actions(sys_decls).clone();

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

    pub fn get_max_bounds(&self, dimensions: u32) -> MaxBounds {
        self.base_representation.get_max_bounds(dimensions)
    }

    pub fn collect_next_inputs<'b>(
        &'b self,
        locations: &[DecoratedLocation<'a>],
        action: &str,
    ) -> Vec<Transition> {
        let mut transitions: Vec<Transition<'b>> = vec![];
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

    pub fn collect_next_outputs<'b>(
        &'b self,
        locations: &[DecoratedLocation<'a>],
        action: &str,
    ) -> Vec<Transition> {
        let mut transitions: Vec<Transition<'b>> = vec![];
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

    pub fn get_input_actions(&self, sys_decls: &SystemDeclarations) -> Vec<String> {
        self.base_representation.get_input_actions(sys_decls)
    }

    pub fn get_output_actions(&self, sys_decls: &SystemDeclarations) -> Vec<String> {
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

    pub fn get_clock_count(&mut self) -> u32 {
        let mut clocks = 0;

        self.base_representation
            .all_components(&mut |comp_view: &ComponentView| {
                clocks += comp_view.clock_count() as u32;

                true
            });

        clocks
    }

    pub fn precheck_sys_rep(&self) -> bool {
        self.base_representation.precheck_sys_rep()
    }

    pub fn all_components_are_deterministic(&self) -> bool {
        self.base_representation
            .all_components(&mut |comp| comp.get_component().is_deterministic())
    }
}

#[derive(Clone)]
pub struct System<'a> {
    base_representation: SystemRepresentation<'a>,
    max_bounds: MaxBounds,
    input_actions: Vec<String>,
    output_actions: Vec<String>,
    initial_locations: RefCell<Option<Vec<DecoratedLocation<'a>>>>,
}

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

    pub fn precheck_sys_rep(&self) -> bool {
        self.base_representation.precheck_sys_rep()
    }

    pub fn collect_next_inputs<'b>(
        &'b self,
        locations: &[DecoratedLocation<'a>],
        action: &str,
    ) -> Vec<Transition> {
        let mut transitions: Vec<Transition<'b>> = vec![];
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

    pub fn collect_next_outputs<'b>(
        &'b self,
        locations: &[DecoratedLocation<'a>],
        action: &str,
    ) -> Vec<Transition> {
        let mut transitions: Vec<Transition<'b>> = vec![];
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

    pub fn move_represetation(self) -> SystemRepresentation<'a> {
        self.base_representation
    }

    pub fn get_max_bounds(&self) -> &MaxBounds {
        &self.max_bounds
    }

    pub fn get_input_actions(&self) -> &Vec<String> {
        &self.input_actions
    }

    pub fn get_output_actions(&self) -> &Vec<String> {
        &self.output_actions
    }

    pub fn get_initial_locations(&'a self) -> Vec<DecoratedLocation> {
        self.initial_locations
            .borrow_mut()
            .get_or_insert_with(|| self.base_representation.get_initial_locations())
            .clone()
    }
}
