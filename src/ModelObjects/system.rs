use crate::DBMLib::dbm::{Federation, Zone};
use crate::ModelObjects::component::{DecoratedLocation, State, SyncType, Transition};
use crate::ModelObjects::component_view::ComponentView;
use crate::ModelObjects::max_bounds::MaxBounds;
use crate::ModelObjects::representations::SystemRepresentation;
use crate::ModelObjects::system_declarations::SystemDeclarations;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

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

    pub fn collect_open_input_transitions<'b>(
        &'b self,
        sys_decls: &SystemDeclarations,
        locations: &Vec<DecoratedLocation<'b>>,
        zone: &Zone,
    ) -> Vec<(Transition<'b>, (Vec<DecoratedLocation>, Zone))> {
        let mut input_transitions = vec![];
        for input in self.get_input_actions(sys_decls) {
            let transitions = self.collect_next_inputs(&locations, &input);
            for transition in transitions {
                let mut locations_clone = locations.clone();
                let mut zone_clone = zone.clone();
                if transition.apply_guards(&locations_clone, &mut zone_clone) {
                    transition.apply_updates(&locations_clone, &mut zone_clone);
                    transition.move_locations(&mut locations_clone);
                    if transition.apply_invariants(&locations_clone, &mut zone_clone) {
                        input_transitions.push((transition, (locations_clone, zone_clone)));
                    }
                }
            }
        }

        input_transitions
    }

    pub fn collect_open_output_transitions<'b>(
        &'b self,
        sys_decls: &SystemDeclarations,
        locations: &Vec<DecoratedLocation<'b>>,
        zone: &Zone,
    ) -> Vec<(Transition<'b>, (Vec<DecoratedLocation>, Zone))> {
        let mut output_transitions = vec![];
        for output in self.get_output_actions(sys_decls) {
            let transitions = self.collect_next_outputs(&locations, &output);
            for transition in transitions.into_iter() {
                let mut locations_clone = locations.clone();
                let mut zone_clone = zone.clone();
                if transition.apply_guards(&locations_clone, &mut zone_clone) {
                    transition.apply_updates(&locations_clone, &mut zone_clone);
                    transition.move_locations(&mut locations_clone);
                    if transition.apply_invariants(&locations_clone, &mut zone_clone) {
                        output_transitions.push((transition, (locations_clone, zone_clone)));
                    }
                }
            }
        }

        output_transitions
    }

    pub fn all_reachable_states<F>(
        &self,
        sys_decls: &SystemDeclarations,
        dim: u32,
        predicate: &mut F,
    ) -> bool
    where
        F: FnMut((&Vec<DecoratedLocation>, &Zone), Vec<Transition>, Vec<Transition>) -> bool,
    {
        let mut passed: Vec<(Vec<DecoratedLocation>, Zone)> = vec![];
        let mut waiting: Vec<(Vec<DecoratedLocation>, Zone)> = vec![];

        let max_bounds = self.get_max_bounds(dim);

        let init_loc = self.get_initial_locations();
        let mut zone = Zone::init(dim);

        for location in init_loc.iter() {
            if !location.apply_invariant(&mut zone) {
                panic!("Invariants led to bad zone");
            }
        }
        zone.extrapolate_max_bounds(&max_bounds);

        waiting.push((init_loc, zone));

        while !waiting.is_empty() {
            let (locations, zone) = waiting.pop().unwrap();

            let mut input_moves = self.collect_open_input_transitions(sys_decls, &locations, &zone);
            let mut output_moves =
                self.collect_open_output_transitions(sys_decls, &locations, &zone);

            for (_, (next_locations, next_zone)) in
                input_moves.iter_mut().chain(output_moves.iter_mut())
            {
                next_zone.up();
                for location in next_locations.iter() {
                    if !location.apply_invariant(next_zone) {
                        panic!("Invariants led to bad zone");
                    }
                }
                next_zone.extrapolate_max_bounds(&max_bounds);

                //Aggressive cloning can be reduced by changing to a State struct over tuple
                if !passed.contains(&(next_locations.clone(), next_zone.clone()))
                    && !waiting.contains(&(next_locations.clone(), next_zone.clone()))
                    && (&locations, &zone) != (next_locations, next_zone)
                {
                    waiting.push((next_locations.clone(), next_zone.clone()));
                }
            }

            let input_transitions = input_moves
                .into_iter()
                .map(|(transition, _)| transition)
                .collect();
            let output_transitions = output_moves
                .into_iter()
                .map(|(transition, _)| transition)
                .collect();

            if !predicate((&locations, &zone), input_transitions, output_transitions) {
                return false;
            }
            passed.push((locations, zone));
        }

        true
    }

    pub fn check_consistency(&mut self, sys_decls: &SystemDeclarations) -> bool {
        let dimensions = self.base_representation.get_dimensions();
        let old_offset = self.set_clock_offset(0);
        //Check if local consistency holds for all reachable states
        let is_consistent =
            self.all_reachable_states(sys_decls, dimensions, &mut |(_, zone), _, outputs| {
                !outputs.is_empty() || zone.canDelayIndefinitely()
            });
        let is_deterministic = self.all_components_are_deterministic();

        self.set_clock_offset(old_offset);

        is_deterministic && is_consistent
    }

    pub fn check_determinism(&self, sys_decls: &SystemDeclarations) -> bool {
        let dimensions = self.base_representation.get_dimensions();

        self.all_reachable_states(
            sys_decls,
            dimensions,
            &mut |(locations, zone), inputs, outputs| {
                let mut zones: HashMap<&String, Federation> = HashMap::new();

                for input_transition in &inputs {
                    if zones.contains_key(input_transition.get_action().unwrap()) {
                        let fed = zones
                            .get_mut(input_transition.get_action().unwrap())
                            .unwrap();
                        let guard_fed = input_transition
                            .get_guard_federation(&locations, zone.dimension)
                            .unwrap();

                        for guard_fed_zone in guard_fed.move_zones() {
                            for fed_zone in fed.iter_zones() {
                                if guard_fed_zone.clone().intersection(fed_zone) {
                                    return false;
                                }
                            }
                            fed.add(guard_fed_zone);
                        }
                    } else {
                        let guard_fed = input_transition
                            .get_guard_federation(&locations, zone.dimension)
                            .unwrap();
                        zones.insert(input_transition.get_action().unwrap(), guard_fed);
                    }
                }

                for output_transition in &outputs {
                    if zones.contains_key(output_transition.get_action().unwrap()) {
                        let fed = zones
                            .get_mut(output_transition.get_action().unwrap())
                            .unwrap();
                        let guard_fed = output_transition
                            .get_guard_federation(&locations, zone.dimension)
                            .unwrap();

                        for guard_fed_zone in guard_fed.move_zones() {
                            for fed_zone in fed.iter_zones() {
                                if guard_fed_zone.clone().intersection(fed_zone) {
                                    return false;
                                }
                            }
                            fed.add(guard_fed_zone);
                        }
                    } else {
                        let guard_fed = output_transition
                            .get_guard_federation(&locations, zone.dimension)
                            .unwrap();
                        zones.insert(output_transition.get_action().unwrap(), guard_fed);
                    }
                }

                true
            },
        )
    }

    //Sets a new clock offset and returns the old one
    pub fn set_clock_offset(&mut self, offset: u32) -> u32 {
        let mut lowest: Option<u32> = None;
        let mut clock_index = offset;
        self.base_representation
            .all_mut_components(&mut |comp_view: &mut ComponentView| {
                let next = Some(comp_view.get_clock_offset());
                comp_view.set_clock_offset(clock_index);
                clock_index += comp_view.clock_count();
                if lowest.is_none() || lowest > next {
                    lowest = next;
                }
                true
            });

        lowest.unwrap()
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
}
