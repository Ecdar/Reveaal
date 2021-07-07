use crate::ModelObjects::component::{
    Component, DecoratedLocation, LocationType, SyncType, Transition,
};
use crate::ModelObjects::max_bounds::MaxBounds;
use crate::ModelObjects::system_declarations::SystemDeclarations;
use serde::Deserialize;

/// This file contains the nested enums used to represent systems on each side of refinement as well as all guards, updates etc
/// note that the enum contains a box (pointer) to an object as they can only hold pointers to data on the heap

#[derive(Debug, Clone, Deserialize, std::cmp::PartialEq)]
pub enum BoolExpression {
    AndOp(Box<BoolExpression>, Box<BoolExpression>),
    OrOp(Box<BoolExpression>, Box<BoolExpression>),
    LessEQ(Box<BoolExpression>, Box<BoolExpression>),
    GreatEQ(Box<BoolExpression>, Box<BoolExpression>),
    LessT(Box<BoolExpression>, Box<BoolExpression>),
    GreatT(Box<BoolExpression>, Box<BoolExpression>),
    EQ(Box<BoolExpression>, Box<BoolExpression>),
    Parentheses(Box<BoolExpression>),
    Clock(u32),
    VarName(String),
    Bool(bool),
    Int(i32),
}

impl BoolExpression {
    pub fn get_max_constant(&self, clock: u32, clock_name: &str) -> i32 {
        let mut new_constraint = 0;

        self.iterate_constraints(&mut |left, right| {
            //Start by matching left and right operands to get constant, this might fail if it does we skip constraint defaulting to 0
            let constant = BoolExpression::get_constant(left, right, clock, clock_name);

            if new_constraint < constant {
                new_constraint = constant;
            }
        });

        //Should this be strict or not? I have set it to be strict as it has a smaller solution space
        new_constraint * 2 + 1
    }

    fn get_constant(left: &Self, right: &Self, clock: u32, clock_name: &str) -> i32 {
        match left {
            BoolExpression::Clock(clock_id) => {
                if *clock_id == clock {
                    if let BoolExpression::Int(constant) = right {
                        return *constant;
                    }
                }
            }
            BoolExpression::VarName(name) => {
                if name.eq(clock_name) {
                    if let BoolExpression::Int(constant) = right {
                        return *constant;
                    }
                }
            }
            BoolExpression::Int(constant) => match right {
                BoolExpression::Clock(clock_id) => {
                    if *clock_id == clock {
                        return *constant;
                    }
                }
                BoolExpression::VarName(name) => {
                    if name.eq(clock_name) {
                        return *constant;
                    }
                }
                _ => {}
            },
            _ => {}
        }

        return 0;
    }

    pub fn iterate_constraints<F>(&self, function: &mut F)
    where
        F: FnMut(&BoolExpression, &BoolExpression),
    {
        match self {
            BoolExpression::AndOp(left, right) => {
                left.iterate_constraints(function);
                right.iterate_constraints(function);
            }
            BoolExpression::OrOp(left, right) => {
                left.iterate_constraints(function);
                right.iterate_constraints(function);
            }
            BoolExpression::Parentheses(expr) => expr.iterate_constraints(function),
            BoolExpression::GreatEQ(left, right) => function(left, right),
            BoolExpression::LessEQ(left, right) => function(left, right),
            BoolExpression::LessT(left, right) => function(left, right),
            BoolExpression::GreatT(left, right) => function(left, right),
            BoolExpression::EQ(left, right) => function(left, right),
            _ => (),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub enum QueryExpression {
    Refinement(Box<QueryExpression>, Box<QueryExpression>),
    Consistency(Box<QueryExpression>),
    Implementation(Box<QueryExpression>),
    Determinism(Box<QueryExpression>),
    Specification(Box<QueryExpression>),
    Conjunction(Box<QueryExpression>, Box<QueryExpression>),
    Composition(Box<QueryExpression>, Box<QueryExpression>),
    Quotient(Box<QueryExpression>, Box<QueryExpression>),
    Possibly(Box<QueryExpression>),
    Invariantly(Box<QueryExpression>),
    EventuallyAlways(Box<QueryExpression>),
    Potentially(Box<QueryExpression>),
    Parentheses(Box<QueryExpression>),
    ComponentExpression(Box<QueryExpression>, Box<QueryExpression>),
    AndOp(Box<QueryExpression>, Box<QueryExpression>),
    OrOp(Box<QueryExpression>, Box<QueryExpression>),
    LessEQ(Box<QueryExpression>, Box<QueryExpression>),
    GreatEQ(Box<QueryExpression>, Box<QueryExpression>),
    LessT(Box<QueryExpression>, Box<QueryExpression>),
    GreatT(Box<QueryExpression>, Box<QueryExpression>),
    Not(Box<QueryExpression>),
    VarName(String),
    Bool(bool),
    Int(i32),
}

#[derive(Debug, Clone)]
pub enum SystemRepresentation {
    Composition(Box<SystemRepresentation>, Box<SystemRepresentation>),
    Conjunction(Box<SystemRepresentation>, Box<SystemRepresentation>),
    Parentheses(Box<SystemRepresentation>),
    Component(Component),
}

impl<'a> SystemRepresentation {
    pub fn get_max_bounds(&self, dimensions: u32) -> MaxBounds {
        let mut bounds = MaxBounds::create(dimensions);

        match self {
            SystemRepresentation::Composition(left_side, right_side) => {
                bounds.add_bounds(&mut left_side.get_max_bounds(dimensions));
                bounds.add_bounds(&mut right_side.get_max_bounds(dimensions));
            }
            SystemRepresentation::Conjunction(left_side, right_side) => {
                bounds.add_bounds(&mut left_side.get_max_bounds(dimensions));
                bounds.add_bounds(&mut right_side.get_max_bounds(dimensions));
            }
            SystemRepresentation::Parentheses(rep) => {
                bounds.add_bounds(&mut rep.get_max_bounds(dimensions))
            }
            SystemRepresentation::Component(comp) => {
                let mut comp_bounds = comp.get_max_bounds(dimensions);
                bounds.add_bounds(&comp_bounds);
            }
        }

        bounds
    }

    pub fn any_composition<F>(&'a self, predicate: &mut F) -> bool
    where
        F: FnMut(&'a Component) -> bool,
    {
        match self {
            SystemRepresentation::Composition(left_side, right_side) => {
                left_side.any_composition(predicate) || right_side.any_composition(predicate)
            }
            SystemRepresentation::Conjunction(left_side, right_side) => {
                left_side.any_composition(predicate) && right_side.any_composition(predicate)
            }
            SystemRepresentation::Parentheses(rep) => rep.any_composition(predicate),
            SystemRepresentation::Component(comp) => predicate(comp),
        }
    }

    pub fn all_components<F>(&'a self, predicate: &mut F) -> bool
    where
        F: FnMut(&'a Component) -> bool,
    {
        match self {
            SystemRepresentation::Composition(left_side, right_side) => {
                left_side.all_components(predicate) && right_side.all_components(predicate)
            }
            SystemRepresentation::Conjunction(left_side, right_side) => {
                left_side.all_components(predicate) && right_side.all_components(predicate)
            }
            SystemRepresentation::Parentheses(rep) => rep.all_components(predicate),
            SystemRepresentation::Component(comp) => predicate(comp),
        }
    }

    pub fn all_mut_components<F>(&'a mut self, predicate: &mut F) -> bool
    where
        F: FnMut(&'a mut Component) -> bool,
    {
        match self {
            SystemRepresentation::Composition(left_side, right_side) => {
                left_side.all_mut_components(predicate) && right_side.all_mut_components(predicate)
            }
            SystemRepresentation::Conjunction(left_side, right_side) => {
                left_side.all_mut_components(predicate) && right_side.all_mut_components(predicate)
            }
            SystemRepresentation::Parentheses(rep) => rep.all_mut_components(predicate),
            SystemRepresentation::Component(comp) => predicate(comp),
        }
    }

    pub fn collect_next_inputs(
        &'a self,
        locations: &[DecoratedLocation<'a>],
        action: &String,
    ) -> Result<Vec<Transition<'a>>, String> {
        let mut transitions = vec![];
        let mut index = 0;

        self.collect_next_transitions(
            locations,
            &mut index,
            action,
            &mut transitions,
            &SyncType::Input,
        );
        Ok(transitions)
    }

    pub fn collect_next_outputs(
        &'a self,
        locations: &[DecoratedLocation<'a>],
        action: &String,
    ) -> Result<Vec<Transition<'a>>, String> {
        let mut transitions = vec![];
        let mut index = 0;

        self.collect_next_transitions(
            locations,
            &mut index,
            action,
            &mut transitions,
            &SyncType::Output,
        );
        Ok(transitions)
    }

    fn collect_next_transitions(
        &'a self,
        locations: &[DecoratedLocation<'a>],
        index: &mut usize,
        action: &String,
        open_transitions: &mut Vec<Transition<'a>>,
        sync_type: &SyncType,
    ) {
        match self {
            SystemRepresentation::Composition(left_side, right_side) => {
                let mut left = vec![];
                let mut right = vec![];

                left_side.collect_next_transitions(locations, index, action, &mut left, sync_type);

                right_side
                    .collect_next_transitions(locations, index, action, &mut right, sync_type);
                // Independent actions
                if left.is_empty() || right.is_empty() {
                    open_transitions.append(&mut left);
                    open_transitions.append(&mut right);
                }
                // Synchronized actions
                else {
                    open_transitions.append(&mut Transition::combinations(&mut left, &mut right));
                }
            }
            SystemRepresentation::Conjunction(left_side, right_side) => {
                let mut left = vec![];
                let mut right = vec![];
                left_side.collect_next_transitions(locations, index, action, &mut left, sync_type);

                right_side
                    .collect_next_transitions(locations, index, action, &mut right, sync_type);

                open_transitions.append(&mut Transition::combinations(&mut left, &mut right));
            }
            SystemRepresentation::Parentheses(rep) => {
                rep.collect_next_transitions(locations, index, action, open_transitions, sync_type);
            }
            SystemRepresentation::Component(comp) => {
                let next_edges =
                    comp.get_next_edges(locations[*index].get_location(), action, *sync_type);
                for e in next_edges {
                    open_transitions.push(Transition {
                        edges: vec![(comp, e, *index)],
                    });
                }

                *index += 1;
            }
        }
    }

    pub fn get_input_actions(&'a self, sys_decls: &SystemDeclarations) -> Vec<String> {
        let mut actions = vec![];
        // Consider compositions as they may remove input actions
        self.collect_input_actions(sys_decls, &mut actions);
        actions
    }

    fn collect_input_actions(&'a self, sys_decls: &SystemDeclarations, vec: &mut Vec<String>) {
        match self {
            SystemRepresentation::Composition(left_side, right_side) => {
                let mut left_in = vec![];
                left_side.collect_input_actions(sys_decls, &mut left_in);
                let mut right_in = vec![];
                right_side.collect_input_actions(sys_decls, &mut right_in);
                let left_out = left_side.get_output_actions(sys_decls);
                let right_out = right_side.get_output_actions(sys_decls);
                for a in &left_in {
                    if !right_out.contains(a) {
                        vec.push(a.clone());
                    }
                }
                for a in &right_in {
                    if !left_out.contains(a) {
                        vec.push(a.clone());
                    }
                }
            }
            SystemRepresentation::Conjunction(left_side, right_side) => {
                left_side.collect_input_actions(sys_decls, vec);
                right_side.collect_input_actions(sys_decls, vec);
            }
            SystemRepresentation::Parentheses(rep) => {
                rep.collect_input_actions(sys_decls, vec);
            }
            SystemRepresentation::Component(comp) => {
                if let Some(inputs_res) = sys_decls
                    .get_declarations()
                    .get_input_actions()
                    .get(comp.get_name())
                {
                    vec.append(&mut inputs_res.clone());
                }
            }
        }
    }

    pub fn get_output_actions(&'a self, sys_decls: &SystemDeclarations) -> Vec<String> {
        let mut actions = vec![];

        self.all_components(&mut |comp: &Component| -> bool {
            if let Some(outputs_res) = sys_decls
                .get_declarations()
                .get_output_actions()
                .get(comp.get_name())
            {
                actions.append(&mut outputs_res.clone());
            }

            true
        });

        actions
    }

    pub fn find_matching_input(
        &self,
        sys_decls: &SystemDeclarations,
        inputs2: &[String],
    ) -> Vec<String> {
        let inputs1 = self.get_input_actions(&sys_decls);

        let mut matching_i: Vec<String> = vec![];
        for i2 in inputs2 {
            let mut found_match = false;
            for i1 in &inputs1 {
                if i1 == i2 {
                    found_match = true;
                }
            }
            if !found_match {
                matching_i.push(i2.clone());
            }
        }

        matching_i
    }

    pub fn find_matching_output(
        &self,
        sys_decls: &SystemDeclarations,
        outputs1: &[String],
    ) -> Vec<String> {
        let outputs2 = self.get_output_actions(&sys_decls);

        let mut matching_o: Vec<String> = vec![];
        for o1 in outputs1 {
            let mut found_match = false;
            for o2 in &outputs2 {
                if o1 == o2 {
                    found_match = true;
                }
            }
            if !found_match {
                matching_o.push(o1.clone());
            }
        }

        matching_o
    }

    pub fn get_initial_locations(&'a self) -> Vec<DecoratedLocation<'a>> {
        let mut states = vec![];
        self.all_components(&mut |comp: &Component| -> bool {
            let init_loc = comp
                .get_locations()
                .iter()
                .find(|location| location.get_location_type() == &LocationType::Initial);
            if let Some(init_loc) = init_loc {
                let state = DecoratedLocation::create(init_loc, comp.get_declarations().clone());
                states.push(state);
            }

            true
        });

        states
    }

    pub fn precheck_sys_rep(&mut self) -> bool {
        self.all_mut_components(&mut |comp: &mut Component| -> bool {
            let clock_clone = comp.get_declarations().get_clocks().clone();

            let len = comp.get_mut_declaration().get_clocks().len();
            comp.get_mut_declaration().dimension = 1 + len as u32;

            comp.get_mut_declaration().reset_clock_indices();

            let res = comp.check_consistency(true);
            comp.get_mut_declaration().clocks = clock_clone;
            res
        })
    }
}
