use crate::ModelObjects::component::{DecoratedLocation, Location, SyncType, Transition};
use crate::ModelObjects::component_view::ComponentView;
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
    pub fn encode_expr(&self) -> String {
        match self {
            BoolExpression::AndOp(left, right) => [
                left.encode_expr(),
                String::from(" && "),
                right.encode_expr(),
            ]
            .concat(),
            BoolExpression::OrOp(left, right) => [
                left.encode_expr(),
                String::from(" || "),
                right.encode_expr(),
            ]
            .concat(),
            BoolExpression::LessEQ(left, right) => {
                [left.encode_expr(), String::from("<="), right.encode_expr()].concat()
            }
            BoolExpression::GreatEQ(left, right) => {
                [left.encode_expr(), String::from(">="), right.encode_expr()].concat()
            }
            BoolExpression::LessT(left, right) => {
                [left.encode_expr(), String::from("<"), right.encode_expr()].concat()
            }
            BoolExpression::GreatT(left, right) => {
                [left.encode_expr(), String::from(">"), right.encode_expr()].concat()
            }
            BoolExpression::EQ(left, right) => {
                [left.encode_expr(), String::from("=="), right.encode_expr()].concat()
            }
            BoolExpression::Parentheses(expr) => {
                [String::from("("), expr.encode_expr(), String::from(")")].concat()
            }
            BoolExpression::Clock(clock) => [String::from("??")].concat(),
            BoolExpression::VarName(var) => var.clone(),
            BoolExpression::Bool(boolean) => {
                format!("{}", boolean)
            }
            BoolExpression::Int(num) => {
                format!("{}", num)
            }
        }
    }

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

    pub fn add_component_id_to_vars(&mut self, comp_id: usize) {
        match self {
            BoolExpression::AndOp(left, right) => {
                left.add_component_id_to_vars(comp_id);
                right.add_component_id_to_vars(comp_id);
            }
            BoolExpression::OrOp(left, right) => {
                left.add_component_id_to_vars(comp_id);
                right.add_component_id_to_vars(comp_id);
            }
            BoolExpression::Parentheses(inner) => {
                inner.add_component_id_to_vars(comp_id);
            }
            BoolExpression::LessEQ(left, right) => {
                left.add_component_id_to_vars(comp_id);
                right.add_component_id_to_vars(comp_id);
            }
            BoolExpression::GreatT(left, right) => {
                left.add_component_id_to_vars(comp_id);
                right.add_component_id_to_vars(comp_id);
            }
            BoolExpression::GreatEQ(left, right) => {
                left.add_component_id_to_vars(comp_id);
                right.add_component_id_to_vars(comp_id);
            }
            BoolExpression::LessT(left, right) => {
                left.add_component_id_to_vars(comp_id);
                right.add_component_id_to_vars(comp_id);
            }
            BoolExpression::LessEQ(left, right) => {
                left.add_component_id_to_vars(comp_id);
                right.add_component_id_to_vars(comp_id);
            }
            BoolExpression::EQ(left, right) => {
                left.add_component_id_to_vars(comp_id);
                right.add_component_id_to_vars(comp_id);
            }
            BoolExpression::Clock(_) => {
                //Assuming ids are correctly offset we dont have to do anything here
            }
            BoolExpression::VarName(name) => {
                *name = format!("{}{}", *name, comp_id);
            }
            BoolExpression::Bool(_) => {}
            BoolExpression::Int(_) => {}
        }
    }

    pub fn swap_var_name(&mut self, from_name: &str, to_name: &str) {
        match self {
            BoolExpression::AndOp(left, right) => {
                left.swap_var_name(from_name, to_name);
                right.swap_var_name(from_name, to_name);
            }
            BoolExpression::OrOp(left, right) => {
                left.swap_var_name(from_name, to_name);
                right.swap_var_name(from_name, to_name);
            }
            BoolExpression::Parentheses(inner) => {
                inner.swap_var_name(from_name, to_name);
            }
            BoolExpression::LessEQ(left, right) => {
                left.swap_var_name(from_name, to_name);
                right.swap_var_name(from_name, to_name);
            }
            BoolExpression::GreatT(left, right) => {
                left.swap_var_name(from_name, to_name);
                right.swap_var_name(from_name, to_name);
            }
            BoolExpression::GreatEQ(left, right) => {
                left.swap_var_name(from_name, to_name);
                right.swap_var_name(from_name, to_name);
            }
            BoolExpression::LessT(left, right) => {
                left.swap_var_name(from_name, to_name);
                right.swap_var_name(from_name, to_name);
            }
            BoolExpression::LessEQ(left, right) => {
                left.swap_var_name(from_name, to_name);
                right.swap_var_name(from_name, to_name);
            }
            BoolExpression::EQ(left, right) => {
                left.swap_var_name(from_name, to_name);
                right.swap_var_name(from_name, to_name);
            }
            BoolExpression::Clock(_) => {
                //Assuming ids are correctly offset we dont have to do anything here
            }
            BoolExpression::VarName(name) => {
                if *name == from_name {
                    *name = to_name.to_string();
                }
            }
            BoolExpression::Bool(_) => {}
            BoolExpression::Int(_) => {}
        }
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

        0
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
    GetComponent(Box<QueryExpression>),
    Prune(Box<QueryExpression>),
    BisimMinimize(Box<QueryExpression>),
    SaveAs(Box<QueryExpression>, String),
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
pub enum SystemRepresentation<'a> {
    Composition(Box<SystemRepresentation<'a>>, Box<SystemRepresentation<'a>>),
    Conjunction(Box<SystemRepresentation<'a>>, Box<SystemRepresentation<'a>>),
    Parentheses(Box<SystemRepresentation<'a>>),
    Component(ComponentView<'a>),
}

impl<'a> SystemRepresentation<'a> {
    pub fn get_max_bounds(&self, dimensions: u32) -> MaxBounds {
        let mut bounds = MaxBounds::create(dimensions);

        match self {
            SystemRepresentation::Composition(left_side, right_side) => {
                bounds.add_bounds(&left_side.get_max_bounds(dimensions));
                bounds.add_bounds(&right_side.get_max_bounds(dimensions));
            }
            SystemRepresentation::Conjunction(left_side, right_side) => {
                bounds.add_bounds(&left_side.get_max_bounds(dimensions));
                bounds.add_bounds(&right_side.get_max_bounds(dimensions));
            }
            SystemRepresentation::Parentheses(rep) => {
                bounds.add_bounds(&rep.get_max_bounds(dimensions))
            }
            SystemRepresentation::Component(comp_view) => {
                bounds.add_bounds(&comp_view.get_max_bounds(dimensions));
            }
        }

        bounds
    }

    pub fn all_components<'b, F>(&'b self, predicate: &mut F) -> bool
    where
        F: FnMut(&'b ComponentView<'a>) -> bool,
    {
        match self {
            SystemRepresentation::Composition(left_side, right_side) => {
                left_side.all_components(predicate) && right_side.all_components(predicate)
            }
            SystemRepresentation::Conjunction(left_side, right_side) => {
                left_side.all_components(predicate) && right_side.all_components(predicate)
            }
            SystemRepresentation::Parentheses(rep) => rep.all_components(predicate),
            SystemRepresentation::Component(comp_view) => predicate(comp_view),
        }
    }

    pub fn collect_next_transitions<'b>(
        &'b self,
        locations: &[DecoratedLocation<'a>],
        index: &mut usize,
        action: &str,
        open_transitions: &mut Vec<Transition<'b>>,
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
            SystemRepresentation::Component(comp_view) => {
                let next_edges = comp_view.get_component().get_next_edges(
                    locations[*index].get_location(),
                    action,
                    *sync_type,
                );
                for e in next_edges {
                    open_transitions.push(Transition {
                        edges: vec![(comp_view, e, *index)],
                    });
                }

                *index += 1;
            }
        }
    }

    pub fn get_input_actions(&self, sys_decls: &SystemDeclarations) -> Vec<String> {
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
            SystemRepresentation::Component(comp_view) => {
                if let Some(inputs_res) = sys_decls
                    .get_declarations()
                    .get_input_actions()
                    .get(comp_view.get_name())
                {
                    vec.append(&mut inputs_res.clone());
                }
            }
        }
    }

    pub fn get_output_actions(&self, sys_decls: &SystemDeclarations) -> Vec<String> {
        let mut actions = vec![];

        self.all_components(&mut |comp: &ComponentView| -> bool {
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

    pub fn get_initial_locations<'b>(&'b self) -> Vec<DecoratedLocation<'b>> {
        let mut states = vec![];
        self.all_components(&mut |comp: &'b ComponentView| -> bool {
            let init_loc: &'b Location = comp.get_initial_location();

            let state = DecoratedLocation::create(init_loc, comp);
            states.push(state);

            true
        });

        states
    }

    pub fn precheck_sys_rep(&self) -> bool {
        self.all_components(&mut |comp_view: &ComponentView| -> bool {
            comp_view.get_component().check_consistency(true)
        })
    }
}
