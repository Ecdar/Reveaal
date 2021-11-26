use serde::Deserialize;
use simple_error::bail;
use std::collections::HashMap;
use std::error::Error;

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
    pub fn swap_clock_names(
        &self,
        from_vars: &HashMap<String, u32>,
        to_vars: &HashMap<String, u32>,
    ) -> Result<BoolExpression, Box<dyn Error>> {
        match self {
            BoolExpression::AndOp(left, right) => Ok(BoolExpression::AndOp(
                Box::new(left.swap_clock_names(from_vars, to_vars)?),
                Box::new(right.swap_clock_names(from_vars, to_vars)?),
            )),
            BoolExpression::OrOp(left, right) => Ok(BoolExpression::OrOp(
                Box::new(left.swap_clock_names(from_vars, to_vars)?),
                Box::new(right.swap_clock_names(from_vars, to_vars)?),
            )),
            BoolExpression::LessEQ(left, right) => Ok(BoolExpression::LessEQ(
                Box::new(left.swap_clock_names(from_vars, to_vars)?),
                Box::new(right.swap_clock_names(from_vars, to_vars)?),
            )),
            BoolExpression::LessT(left, right) => Ok(BoolExpression::LessT(
                Box::new(left.swap_clock_names(from_vars, to_vars)?),
                Box::new(right.swap_clock_names(from_vars, to_vars)?),
            )),
            BoolExpression::EQ(left, right) => Ok(BoolExpression::EQ(
                Box::new(left.swap_clock_names(from_vars, to_vars)?),
                Box::new(right.swap_clock_names(from_vars, to_vars)?),
            )),
            BoolExpression::GreatEQ(left, right) => Ok(BoolExpression::GreatEQ(
                Box::new(left.swap_clock_names(from_vars, to_vars)?),
                Box::new(right.swap_clock_names(from_vars, to_vars)?),
            )),
            BoolExpression::GreatT(left, right) => Ok(BoolExpression::GreatT(
                Box::new(left.swap_clock_names(from_vars, to_vars)?),
                Box::new(right.swap_clock_names(from_vars, to_vars)?),
            )),

            BoolExpression::Parentheses(body) => {
                Ok(BoolExpression::Parentheses(Box::new(body.swap_clock_names(from_vars, to_vars)?)))
            }
            BoolExpression::Clock(_) => bail!("Did not expect clock index in boolexpression, cannot swap clock names in misformed BoolExpression"),
            BoolExpression::VarName(name) => {
                if let Some(index) = from_vars.get(name){
                    if let Some(new_name) = to_vars.iter()
                    .find_map(|(key, val)| if *val == *index { Some(key) } else { None }){
                        return Ok(BoolExpression::VarName(new_name.clone()));
                    }else{
                        bail!("Couldnt find index {} in to to_vars {:?}", index, to_vars);
                    }
                }else{
                    bail!("Couldnt find the name {} in the set from_vars {:?}", name, from_vars);
                }
            },
            BoolExpression::Bool(val) => Ok(BoolExpression::Bool(val.clone())),
            BoolExpression::Int(val) => Ok(BoolExpression::Int(val.clone())),
        }
    }

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
            BoolExpression::Clock(_) => [String::from("??")].concat(),
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

impl QueryExpression {
    pub fn pretty_string(&self) -> String {
        match self {
            QueryExpression::Refinement(left, right) => format!(
                "refinement: {} <= {}",
                left.pretty_string(),
                right.pretty_string()
            ),
            QueryExpression::Consistency(system) => {
                format!("consistency: {}", system.pretty_string())
            }
            QueryExpression::GetComponent(comp) => {
                format!("get-component: {}", comp.pretty_string())
            }
            QueryExpression::SaveAs(system, name) => {
                format!("{} save-as {}", system.pretty_string(), name.clone())
            }
            QueryExpression::Conjunction(left, right) => {
                format!("{} && {}", left.pretty_string(), right.pretty_string())
            }
            QueryExpression::Composition(left, right) => {
                format!("{} || {}", left.pretty_string(), right.pretty_string())
            }
            QueryExpression::Quotient(left, right) => {
                format!("{} \\\\ {}", left.pretty_string(), right.pretty_string())
            }
            QueryExpression::Prune(comp) => {
                format!("prune: {}", comp.pretty_string())
            }
            QueryExpression::Implementation(system) => {
                format!("implementation: {}", system.pretty_string())
            }
            QueryExpression::Determinism(system) => {
                format!("determinism: {}", system.pretty_string())
            }
            QueryExpression::Specification(system) => {
                format!("specification: {}", system.pretty_string())
            }
            QueryExpression::BisimMinimize(save_system) => {
                format!("bisim-minim: {}", save_system.pretty_string())
            }
            QueryExpression::Parentheses(system) => format!("({})", system.pretty_string()),
            QueryExpression::VarName(name) => name.clone(),
            QueryExpression::Possibly(bool_expr) => {
                format!("E<>{}", bool_expr.pretty_string())
            }
            QueryExpression::Invariantly(bool_expr) => {
                format!("A[]{}", bool_expr.pretty_string())
            }
            QueryExpression::EventuallyAlways(bool_expr) => {
                format!("E[]{}", bool_expr.pretty_string())
            }
            QueryExpression::Potentially(bool_expr) => {
                format!("A<>{}", bool_expr.pretty_string())
            }
            QueryExpression::AndOp(left, right) => {
                format!("({} ∧ {})", left.pretty_string(), right.pretty_string())
            }
            QueryExpression::OrOp(left, right) => {
                format!("({} ∨ {})", left.pretty_string(), right.pretty_string())
            }
            QueryExpression::LessEQ(left, right) => {
                format!("({} <= {})", left.pretty_string(), right.pretty_string())
            }
            QueryExpression::GreatEQ(left, right) => {
                format!("({} >= {})", left.pretty_string(), right.pretty_string())
            }
            QueryExpression::LessT(left, right) => {
                format!("({} < {})", left.pretty_string(), right.pretty_string())
            }
            QueryExpression::GreatT(left, right) => {
                format!("({} > {})", left.pretty_string(), right.pretty_string())
            }
            QueryExpression::Not(bool_expr) => {
                format!("!({})", bool_expr.pretty_string())
            }
            QueryExpression::Int(number) => {
                format!("{}", number)
            }
            QueryExpression::Bool(boolean) => {
                format!("{}", boolean)
            }
            _ => String::from("??"),
        }
    }
}
