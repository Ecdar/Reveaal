use crate::DBMLib::dbm::Zone;
use colored::Colorize;
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::ops;

/// This file contains the nested enums used to represent systems on each side of refinement as well as all guards, updates etc
/// note that the enum contains a box (pointer) to an object as they can only hold pointers to data on the heap

#[derive(Debug, Clone, Deserialize, std::cmp::PartialEq, std::cmp::Eq)]
pub enum BoolExpression {
    AndOp(Box<BoolExpression>, Box<BoolExpression>),
    OrOp(Box<BoolExpression>, Box<BoolExpression>),
    LessEQ(Box<BoolExpression>, Box<BoolExpression>),
    GreatEQ(Box<BoolExpression>, Box<BoolExpression>),
    LessT(Box<BoolExpression>, Box<BoolExpression>),
    GreatT(Box<BoolExpression>, Box<BoolExpression>),
    EQ(Box<BoolExpression>, Box<BoolExpression>),
    Parentheses(Box<BoolExpression>),
    Difference(Box<BoolExpression>, Box<BoolExpression>),
    Addition(Box<BoolExpression>, Box<BoolExpression>),
    Multiplication(Box<BoolExpression>, Box<BoolExpression>),
    Division(Box<BoolExpression>, Box<BoolExpression>),
    Modulo(Box<BoolExpression>, Box<BoolExpression>),
    Clock(u32),
    VarName(String),
    Bool(bool),
    Int(i32),
}

impl BoolExpression {
    pub fn swap_clock_names(
        &self,
        from_vars: &HashMap<String, u32>,
        to_vars: &HashMap<u32, String>,
    ) -> BoolExpression {
        match self {
            BoolExpression::AndOp(left, right) => BoolExpression::AndOp(
                Box::new(left.swap_clock_names(from_vars, to_vars)),
                Box::new(right.swap_clock_names(from_vars, to_vars)),
            ),
            BoolExpression::OrOp(left, right) => BoolExpression::OrOp(
                Box::new(left.swap_clock_names(from_vars, to_vars)),
                Box::new(right.swap_clock_names(from_vars, to_vars)),
            ),
            BoolExpression::LessEQ(left, right) => BoolExpression::LessEQ(
                Box::new(left.swap_clock_names(from_vars, to_vars)),
                Box::new(right.swap_clock_names(from_vars, to_vars)),
            ),
            BoolExpression::LessT(left, right) => BoolExpression::LessT(
                Box::new(left.swap_clock_names(from_vars, to_vars)),
                Box::new(right.swap_clock_names(from_vars, to_vars)),
            ),
            BoolExpression::EQ(left, right) => BoolExpression::EQ(
                Box::new(left.swap_clock_names(from_vars, to_vars)),
                Box::new(right.swap_clock_names(from_vars, to_vars)),
            ),
            BoolExpression::GreatEQ(left, right) => BoolExpression::GreatEQ(
                Box::new(left.swap_clock_names(from_vars, to_vars)),
                Box::new(right.swap_clock_names(from_vars, to_vars)),
            ),
            BoolExpression::GreatT(left, right) => BoolExpression::GreatT(
                Box::new(left.swap_clock_names(from_vars, to_vars)),
                Box::new(right.swap_clock_names(from_vars, to_vars)),
            ),
            BoolExpression::Difference(left, right) => BoolExpression::Difference(
                Box::new(left.swap_clock_names(from_vars, to_vars)),
                Box::new(right.swap_clock_names(from_vars, to_vars)),
            ),
            BoolExpression::Addition(left, right) => BoolExpression::Addition(
                Box::new(left.swap_clock_names(from_vars, to_vars)),
                Box::new(right.swap_clock_names(from_vars, to_vars)),
            ),
            BoolExpression::Multiplication(left, right) => BoolExpression::Multiplication(
                Box::new(left.swap_clock_names(from_vars, to_vars)),
                Box::new(right.swap_clock_names(from_vars, to_vars)),
            ),
            BoolExpression::Division(left, right) => BoolExpression::Division(
                Box::new(left.swap_clock_names(from_vars, to_vars)),
                Box::new(right.swap_clock_names(from_vars, to_vars)),
            ),
            BoolExpression::Modulo(left, right) => BoolExpression::Modulo(
                Box::new(left.swap_clock_names(from_vars, to_vars)),
                Box::new(right.swap_clock_names(from_vars, to_vars)),
            ),


            BoolExpression::Parentheses(body) => {
                BoolExpression::Parentheses(Box::new(body.swap_clock_names(from_vars, to_vars)))
            }
            BoolExpression::Clock(_) => panic!("Did not expect clock index in boolexpression, cannot swap clock names in misformed bexpr"),
            BoolExpression::VarName(name) => {
                let index = from_vars.get(name).unwrap();
                let new_name = to_vars[index].clone();
                BoolExpression::VarName(new_name)
            },
            BoolExpression::Bool(val) => BoolExpression::Bool(val.clone()),
            BoolExpression::Int(val) => BoolExpression::Int(val.clone()),
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
            BoolExpression::Difference(left, right) => {
                [left.encode_expr(), String::from("-"), right.encode_expr()].concat()
            }
            BoolExpression::Addition(left, right) => {
                [left.encode_expr(), String::from("+"), right.encode_expr()].concat()
            }
            BoolExpression::Multiplication(left, right) => {
                [left.encode_expr(), String::from("*"), right.encode_expr()].concat()
            }
            BoolExpression::Division(left, right) => {
                [left.encode_expr(), String::from("/"), right.encode_expr()].concat()
            }
            BoolExpression::Modulo(left, right) => {
                [left.encode_expr(), String::from("%"), right.encode_expr()].concat()
            }
            BoolExpression::Clock(_) => [String::from("??")].concat(),
            BoolExpression::VarName(var) => var.clone(),
            BoolExpression::Bool(boolean) => boolean.to_string(),
            BoolExpression::Int(num) => num.to_string(),
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

        new_constraint // * 2 + 1 // This should not actually be a dbm_raw, as it is converted from bound to raw in the c code
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
            BoolExpression::Difference(left, right) => {
                left.swap_var_name(from_name, to_name);
                right.swap_var_name(from_name, to_name);
            }
            BoolExpression::Addition(left, right) => {
                left.swap_var_name(from_name, to_name);
                right.swap_var_name(from_name, to_name);
            }
            BoolExpression::Multiplication(left, right) => {
                left.swap_var_name(from_name, to_name);
                right.swap_var_name(from_name, to_name);
            }
            BoolExpression::Division(left, right) => {
                left.swap_var_name(from_name, to_name);
                right.swap_var_name(from_name, to_name);
            }
            BoolExpression::Modulo(left, right) => {
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

    pub fn conjunction(guards: &mut Vec<BoolExpression>) -> BoolExpression {
        let num_guards = guards.len();

        if let Some(guard) = guards.pop() {
            if num_guards == 1 {
                guard
            } else {
                BoolExpression::AndOp(
                    Box::new(guard),
                    Box::new(BoolExpression::conjunction(guards)),
                )
            }
        } else {
            BoolExpression::Bool(false)
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

    pub fn simplify(&mut self) {
        while self.simplify_helper() {}
    }

    fn simplify_helper(&mut self) -> bool {
        let mut changed = false;
        let mut value = None;
        match self {
            BoolExpression::AndOp(left, right) => {
                changed |= left.simplify_helper();
                changed |= right.simplify_helper();
                match **left {
                    BoolExpression::Bool(false) => value = Some(BoolExpression::Bool(false)),
                    BoolExpression::Bool(true) => value = Some((**right).clone()),
                    _ => {}
                }
                match **right {
                    BoolExpression::Bool(false) => value = Some(BoolExpression::Bool(false)),
                    BoolExpression::Bool(true) => value = Some((**left).clone()),
                    _ => {}
                }
            }
            BoolExpression::OrOp(left, right) => {
                changed |= left.simplify_helper();
                changed |= right.simplify_helper();
                match **left {
                    BoolExpression::Bool(true) => value = Some(BoolExpression::Bool(true)),
                    BoolExpression::Bool(false) => value = Some((**right).clone()),
                    _ => {}
                }
                match **right {
                    BoolExpression::Bool(true) => value = Some(BoolExpression::Bool(true)),
                    BoolExpression::Bool(false) => value = Some((**left).clone()),
                    _ => {}
                }
            }
            BoolExpression::Parentheses(inner) => {
                value = Some((**inner).clone());
            }
            _ => {}
        }

        if let Some(new_val) = value {
            *self = new_val;
            true
        } else {
            changed
        }
    }

    pub fn BLessEQ(left: BoolExpression, right: BoolExpression) -> BoolExpression {
        BoolExpression::LessEQ(Box::new(left), Box::new(right))
    }
    pub fn BLessT(left: BoolExpression, right: BoolExpression) -> BoolExpression {
        BoolExpression::LessT(Box::new(left), Box::new(right))
    }
    pub fn BGreatEQ(left: BoolExpression, right: BoolExpression) -> BoolExpression {
        BoolExpression::GreatEQ(Box::new(left), Box::new(right))
    }
    pub fn BGreatT(left: BoolExpression, right: BoolExpression) -> BoolExpression {
        BoolExpression::GreatT(Box::new(left), Box::new(right))
    }
    pub fn BEQ(left: BoolExpression, right: BoolExpression) -> BoolExpression {
        BoolExpression::EQ(Box::new(left), Box::new(right))
    }
    pub fn BPar(inner: BoolExpression) -> BoolExpression {
        inner
    }

    pub fn BDif(left: BoolExpression, right: BoolExpression) -> BoolExpression {
        if let BoolExpression::Int(0) = right {
            return left;
        }

        if let BoolExpression::Int(i) = left {
            if let BoolExpression::Int(j) = right {
                return BoolExpression::Int(i - j);
            }
        }

        BoolExpression::Difference(Box::new(left), Box::new(right))
    }
}

impl ops::BitAnd for BoolExpression {
    type Output = Self;

    fn bitand(self, other: Self) -> Self {
        BoolExpression::AndOp(Box::new(self), Box::new(other))
    }
}

impl ops::BitOr for BoolExpression {
    type Output = Self;

    fn bitor(self, other: Self) -> Self {
        BoolExpression::OrOp(Box::new(self), Box::new(other))
    }
}

fn get_op(exp: &Box<BoolExpression>) -> Option<String> {
    match exp.as_ref() {
        BoolExpression::EQ(_, _) => Some("=".to_string()),
        BoolExpression::LessEQ(_, _) => Some("≤".to_string()),
        BoolExpression::LessT(_, _) => Some("<".to_string()),
        _ => None,
    }
}

impl Display for BoolExpression {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BoolExpression::AndOp(left, right) => {
                // And(eq(a,b), And(eq(b,c), And(eq(c,d), And(...)))) -> a=b=c=d
                match &**left {
                    BoolExpression::EQ(a, b)
                    | BoolExpression::LessEQ(a, b)
                    | BoolExpression::LessT(a, b) => match &**right {
                        BoolExpression::AndOp(op, _) => {
                            if let BoolExpression::EQ(b1, _c)
                            | BoolExpression::LessEQ(b1, _c)
                            | BoolExpression::LessT(b1, _c) = &**op
                            {
                                if **b == **b1 {
                                    write!(f, "{}{}{}", a, get_op(left).unwrap(), right)?;
                                    return Ok(());
                                }
                            }
                        }
                        BoolExpression::EQ(b1, _c)
                        | BoolExpression::LessEQ(b1, _c)
                        | BoolExpression::LessT(b1, _c) => {
                            if **b == **b1 {
                                write!(f, "{}{}{}", a, get_op(left).unwrap(), right)?;
                                return Ok(());
                            }
                        }
                        _ => {}
                    },
                    _ => {}
                }

                let l_clone = left.clone();
                let l = match **left {
                    BoolExpression::OrOp(_, _) => BoolExpression::Parentheses(l_clone),
                    _ => *l_clone,
                };
                let r_clone = right.clone();
                let r = match **right {
                    BoolExpression::OrOp(_, _) => BoolExpression::Parentheses(r_clone),
                    _ => *r_clone,
                };
                write!(f, "{} && {}", l, r)?;
            }
            BoolExpression::OrOp(left, right) => {
                let l_clone = left.clone();
                let l = match **left {
                    BoolExpression::AndOp(_, _) => BoolExpression::Parentheses(l_clone),
                    _ => *l_clone,
                };
                let r_clone = right.clone();
                let r = match **right {
                    BoolExpression::AndOp(_, _) => BoolExpression::Parentheses(r_clone),
                    _ => *r_clone,
                };
                write!(f, "{} || {}", l, r)?;
            }
            BoolExpression::Parentheses(expr) => {
                let l_par = "(".to_string().yellow();
                let r_par = ")".to_string().yellow();
                write!(f, "{}{}{}", l_par, expr, r_par)?;
            }
            BoolExpression::GreatEQ(left, right) => {
                write!(f, "{}≥{}", left, right)?;
            }
            BoolExpression::LessEQ(left, right) => {
                write!(f, "{}≤{}", left, right)?;
            }
            BoolExpression::LessT(left, right) => {
                write!(f, "{}<{}", left, right)?;
            }
            BoolExpression::GreatT(left, right) => {
                write!(f, "{}>{}", left, right)?;
            }
            BoolExpression::EQ(left, right) => {
                write!(f, "{}={}", left, right)?;
            }
            BoolExpression::Clock(id) => {
                write!(f, "{}", format!("c:{}", id).to_string().magenta())?;
            }
            BoolExpression::VarName(name) => {
                write!(f, "{}", name.to_string().blue())?;
            }
            BoolExpression::Bool(val) => {
                if *val {
                    write!(f, "{}", val.to_string().green())?;
                } else {
                    write!(f, "{}", val.to_string().red())?;
                }
            }
            BoolExpression::Int(num) => {
                write!(f, "{}", num)?;
            }
            BoolExpression::Difference(left, right) => {
                write!(f, "{}-{}", left, right)?;
            }
            BoolExpression::Addition(left, right) => {
                write!(f, "{}+{}", left, right)?;
            }
            BoolExpression::Multiplication(left, right) => {
                write!(f, "{}*{}", left, right)?;
            }
            BoolExpression::Division(left, right) => {
                write!(f, "{}/{}", left, right)?;
            }
            BoolExpression::Modulo(left, right) => {
                write!(f, "{}%{}", left, right)?;
            }
        }
        Ok(())
    }
}

fn var_from_index(
    index: u32,
    clocks: &Option<&HashMap<String, u32>>,
) -> Option<Box<BoolExpression>> {
    let var = if let Some(c) = clocks {
        //If the index exists in dbm it must be in the map, so we unwrap
        let clock = c.keys().find(|&x| *c.get(x).unwrap() == index);

        match clock {
            Some(c) => Some(Box::new(BoolExpression::VarName(c.clone()))),
            None => None,
        }
    } else {
        Some(Box::new(BoolExpression::Clock(index)))
    };
    var
}

fn get_groups_from_zone(zone: &Zone, clocks: &Option<&HashMap<String, u32>>) -> Vec<Vec<u32>> {
    let mut groups: Vec<Vec<u32>> = vec![];
    let mut grouped: Vec<u32> = vec![];
    for index_i in 1..zone.dimension {
        if grouped.contains(&index_i) {
            continue;
        }

        if var_from_index(index_i, &clocks).is_none() {
            continue;
        }

        let mut group = vec![index_i];

        // Find next equal
        for index_j in index_i + 1..zone.dimension {
            if var_from_index(index_j, &clocks).is_none() {
                continue;
            }
            if is_equal(zone, index_i, index_j) {
                group.push(index_j);
                grouped.push(index_j);
            }
        }

        groups.push(group);
    }
    groups
}

pub fn build_guard_from_zone(
    zone: &Zone,
    clocks: Option<&HashMap<String, u32>>,
) -> Option<BoolExpression> {
    let mut guards: Vec<BoolExpression> = vec![];
    let groups = get_groups_from_zone(zone, &clocks);

    for group in &groups {
        let first = *group.first().unwrap();
        let last = *group.last().unwrap();
        let first_var = var_from_index(first, &clocks).unwrap();
        let last_var = var_from_index(last, &clocks).unwrap();

        let (upper_is_strict, upper_val) = zone.get_constraint(first, 0);
        let (lower_is_strict, lower_val) = zone.get_constraint(0, first);

        // if lower bound is different from (>=, 0)
        if lower_is_strict || lower_val != 0 {
            if lower_is_strict {
                guards.push(BoolExpression::LessT(
                    Box::new(BoolExpression::Int(-lower_val)),
                    first_var,
                ));
            } else {
                guards.push(BoolExpression::LessEQ(
                    Box::new(BoolExpression::Int(-lower_val)),
                    first_var,
                ));
            }
        }

        for index in 0..group.len() - 1 {
            let (a, b) = (group[index], group[index + 1]);
            let (a, b) = (
                var_from_index(a, &clocks).unwrap(),
                var_from_index(b, &clocks).unwrap(),
            );
            guards.push(BoolExpression::EQ(a, b));
        }

        // Upper bound
        if !zone.is_constraint_infinity(last, 0) {
            if upper_is_strict {
                guards.push(BoolExpression::LessT(
                    last_var,
                    Box::new(BoolExpression::Int(upper_val)),
                ));
            } else {
                guards.push(BoolExpression::LessEQ(
                    last_var,
                    Box::new(BoolExpression::Int(upper_val)),
                ));
            }
        }

        for other_group in &groups {
            let other_first = *other_group.first().unwrap();
            if other_first == first {
                continue;
            }

            add_diagonal_constraints(
                zone,
                other_first,
                first,
                var_from_index(other_first, &clocks).unwrap(),
                var_from_index(first, &clocks).unwrap(),
                &mut guards,
            );
        }
    }
    guards.reverse();

    let res = build_guard_from_zone_helper(&mut guards);
    Some(res)
}

fn add_diagonal_constraints(
    zone: &Zone,
    index_i: u32,
    index_j: u32,
    var_i: Box<BoolExpression>,
    var_j: Box<BoolExpression>,
    guards: &mut Vec<BoolExpression>,
) {
    if !zone.is_constraint_infinity(index_i, index_j) {
        if is_constraint_unnecessary(zone, index_i, index_j) {
            return;
        }
        // i-j <= c
        let (is_strict, val) = zone.get_constraint(index_i, index_j);
        /*if val == 0 {
            if is_strict {
                guards.push(BoolExpression::BLessT(*var_i, *var_j))
            } else {
                guards.push(BoolExpression::BLessEQ(*var_i, *var_j))
            }
        } else*/
        {
            if is_strict {
                guards.push(BoolExpression::BLessT(
                    BoolExpression::Difference(var_i, var_j),
                    BoolExpression::Int(val),
                ))
            } else {
                guards.push(BoolExpression::BLessEQ(
                    BoolExpression::Difference(var_i, var_j),
                    BoolExpression::Int(val),
                ))
            }
        }
    }
}

fn is_equal(zone: &Zone, index_i: u32, index_j: u32) -> bool {
    let d1 = zone.get_constraint(index_i, index_j);
    let d2 = zone.get_constraint(index_j, index_i);

    const EQ_ZERO: (bool, i32) = (false, 0);

    d1 == EQ_ZERO && d2 == EQ_ZERO
}

fn is_constraint_unnecessary(zone: &Zone, index_i: u32, index_j: u32) -> bool {
    let max_i = zone.get_constraint(index_i, 0);
    let min_j = zone.get_constraint(0, index_j);

    // let max_j = zone.get_constraint(index_j, 0);
    // let min_i = zone.get_constraint(0, index_i);

    // i-j <= c
    let c = zone.get_constraint(index_i, index_j);

    if zone.is_constraint_infinity(index_i, 0) {
        return true;
    }

    // max(i)-min(j) <? c
    // --> max(i) <? c + min(j)
    let c_plus_min_j = constraint_sum(c.0, c.1, min_j.0, min_j.1);

    if c_plus_min_j == max_i {
        return true;
    }
    false
}

fn constraint_sum(c1_strict: bool, c1: i32, c2_strict: bool, c2: i32) -> (bool, i32) {
    let strict = c1_strict || c2_strict;
    let c = c1 + c2;
    (strict, c)
}

fn build_guard_from_zone_helper(guards: &mut Vec<BoolExpression>) -> BoolExpression {
    let num_guards = guards.len();

    if let Some(guard) = guards.pop() {
        if num_guards == 1 {
            guard
        } else {
            BoolExpression::AndOp(
                Box::new(guard),
                Box::new(build_guard_from_zone_helper(guards)),
            )
        }
    } else {
        BoolExpression::Bool(true)
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
            QueryExpression::Parentheses(system) => format!("({})", system.pretty_string()),
            QueryExpression::VarName(name) => name.clone(),

            _ => panic!("Rule not implemented yet"),
        }
    }
}
