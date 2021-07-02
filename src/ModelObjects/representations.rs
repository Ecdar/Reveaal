use crate::ModelObjects::component::{Component, DecoratedLocation, Edge, LocationType, SyncType};
use crate::ModelObjects::max_bounds::MaxBounds;
use crate::ModelObjects::system_declarations::SystemDeclarations;
use serde::Deserialize;
use std::collections::HashMap;

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
    pub fn get_highest_constraints(&self) -> MaxBounds {
        let mut max_bounds = MaxBounds::create();

        self.iterate_constraints(&mut |left, right, op| {
            let clock: u32;
            let constant: i32;

            //Start by matching left and right operands as clock and constant, this might fail if it does we skip constraint
            if let BoolExpression::Clock(clock_ref) = left {
                if let BoolExpression::Int(constant_ref) = right {
                    constant = *constant_ref;
                    clock = *clock_ref;
                } else {
                    return;
                }
            } else if let BoolExpression::Clock(clock_ref) = right {
                if let BoolExpression::Int(constant_ref) = left {
                    constant = *constant_ref;
                    clock = *clock_ref;
                } else {
                    return;
                }
            } else {
                return;
            }

            let mut new_constraint = -1;
            if op(constant - 1, constant) {
                new_constraint = constant - 1
            } else if op(constant, constant) {
                new_constraint = constant
            } else if op(constant + 1, constant) {
                new_constraint = constant + 1
            }

            max_bounds.add_bound(clock, new_constraint);
        });

        max_bounds
    }

    pub fn iterate_constraints<F>(&self, function: &mut F)
    where
        F: FnMut(&BoolExpression, &BoolExpression, &mut Fn(i32, i32) -> bool),
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
            BoolExpression::GreatEQ(left, right) => function(left, right, &mut |x, y| x >= y),
            BoolExpression::LessEQ(left, right) => function(left, right, &mut |x, y| x <= y),
            BoolExpression::LessT(left, right) => function(left, right, &mut |x, y| x < y),
            BoolExpression::GreatT(left, right) => function(left, right, &mut |x, y| x > y),
            BoolExpression::EQ(left, right) => function(left, right, &mut |x, y| x == y),
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

    pub fn collect_open_inputs(
        &'a self,
        locations: &[DecoratedLocation<'a>],
        action: &str,
    ) -> Result<Vec<(&'a Component, Vec<&'a Edge>, usize)>, String> {
        let mut edges = vec![];
        let mut index = 0;

        if self.collect_open_edges(locations, &mut index, action, &mut edges, &SyncType::Input) {
            Ok(edges)
        } else {
            Err("Conjunction rules on output not satisfied".to_string())
        }
    }

    pub fn collect_open_outputs(
        &'a self,
        locations: &[DecoratedLocation<'a>],
        action: &str,
    ) -> Result<Vec<(&'a Component, Vec<&'a Edge>, usize)>, String> {
        let mut edges = vec![];
        let mut index = 0;

        if self.collect_open_edges(locations, &mut index, action, &mut edges, &SyncType::Output) {
            Ok(edges)
        } else {
            Err("Conjunction rules on input not satisfied".to_string())
        }
    }

    fn collect_open_edges(
        &'a self,
        locations: &[DecoratedLocation<'a>],
        index: &mut usize,
        action: &str,
        open_edges: &mut Vec<(&'a Component, Vec<&'a Edge>, usize)>,
        sync_type: &SyncType,
    ) -> bool {
        match self {
            SystemRepresentation::Composition(left_side, right_side) => {
                left_side.collect_open_edges(locations, index, action, open_edges, sync_type)
                    || right_side
                        .collect_open_edges(locations, index, action, open_edges, sync_type)
            }
            SystemRepresentation::Conjunction(left_side, right_side) => {
                let open_edges_len = open_edges.len();
                if left_side.collect_open_edges(locations, index, action, open_edges, sync_type) {
                    let left_found_transitions = open_edges_len != open_edges.len();
                    if right_side
                        .collect_open_edges(locations, index, action, open_edges, sync_type)
                    {
                        let right_found_transitions = open_edges_len != open_edges.len();
                        return left_found_transitions == right_found_transitions;
                    }
                }
                false
            }
            SystemRepresentation::Parentheses(rep) => {
                rep.collect_open_edges(locations, index, action, open_edges, sync_type)
            }
            SystemRepresentation::Component(comp) => {
                let next_edges =
                    comp.get_next_edges(locations[*index].get_location(), action, *sync_type);

                if !next_edges.is_empty() {
                    open_edges.push((comp, next_edges, *index));
                }

                *index += 1;
                true
            }
        }
    }

    pub fn get_input_actions(&'a self, sys_decls: &SystemDeclarations) -> Vec<String> {
        let mut actions = vec![];

        self.all_components(&mut |comp: &Component| -> bool {
            if let Some(inputs_res) = sys_decls
                .get_declarations()
                .get_input_actions()
                .get(comp.get_name())
            {
                actions.append(&mut inputs_res.clone());
            }

            true
        });

        actions
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
