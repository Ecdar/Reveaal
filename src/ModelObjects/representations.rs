use colored::Colorize;
use edbm::util::constraints::{ClockIndex, Conjunction, Constraint, Disjunction};

use serde::Deserialize;

use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::ops;
//use serde::de::Unexpected::Option;

/// This file contains the nested enums used to represent systems on each side of refinement as well as all guards, updates etc
/// note that the enum contains a box (pointer) to an object as they can only hold pointers to data on the heap

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub enum BoolExpression {
    AndOp(Box<BoolExpression>, Box<BoolExpression>),
    OrOp(Box<BoolExpression>, Box<BoolExpression>),
    LessEQ(Box<ArithExpression>, Box<ArithExpression>),
    GreatEQ(Box<ArithExpression>, Box<ArithExpression>),
    LessT(Box<ArithExpression>, Box<ArithExpression>),
    GreatT(Box<ArithExpression>, Box<ArithExpression>),
    EQ(Box<ArithExpression>, Box<ArithExpression>),
    Bool(bool),
    Arithmetic(Box<ArithExpression>),
}

impl BoolExpression {
    pub fn swap_clock_names(
        &self,
        from_vars: &HashMap<String, ClockIndex>,
        to_vars: &HashMap<ClockIndex, String>,
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
            BoolExpression::Bool(val) => BoolExpression::Bool(*val),
            BoolExpression::Arithmetic(x) => {
                BoolExpression::Arithmetic(Box::new(x.swap_clock_names(from_vars, to_vars)))
            }
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
            BoolExpression::Bool(boolean) => boolean.to_string(),
            BoolExpression::Arithmetic(x) => x.encode_expr(),
        }
    }

    pub fn from_disjunction(
        disjunction: &Disjunction,
        naming: &HashMap<String, ClockIndex>,
    ) -> Option<Self> {
        let naming = naming
            .iter()
            .map(|(name, index)| (*index, name.clone()))
            .collect();
        if disjunction.conjunctions.is_empty() {
            Some(BoolExpression::Bool(false))
        } else if disjunction.conjunctions.len() == 1 {
            BoolExpression::from_conjunction(&disjunction.conjunctions[0], &naming)
        } else {
            let mut result = None;

            for conjunction in &disjunction.conjunctions {
                // If any is None (true), the disjuntion is None (true) so we use ?
                let expr = BoolExpression::from_conjunction(conjunction, &naming)?;

                match result {
                    None => result = Some(expr),
                    Some(res) => result = Some(BoolExpression::OrOp(Box::new(res), Box::new(expr))),
                }
            }

            result
        }
    }

    pub fn from_conjunction(
        conjunction: &Conjunction,
        naming: &HashMap<ClockIndex, String>,
    ) -> Option<Self> {
        if conjunction.constraints.is_empty() {
            //BoolExpression::Bool(true)
            None
        } else if conjunction.constraints.len() == 1 {
            Some(BoolExpression::from_constraint(
                &conjunction.constraints[0],
                naming,
            ))
        } else {
            let mut result = None;

            for constraint in &conjunction.constraints {
                let expr = BoolExpression::from_constraint(constraint, naming);
                match result {
                    None => result = Some(expr),
                    Some(res) => {
                        result = Some(BoolExpression::AndOp(Box::new(res), Box::new(expr)))
                    }
                }
            }

            result
        }
    }

    pub fn from_constraint(constraint: &Constraint, naming: &HashMap<ClockIndex, String>) -> Self {
        let ineq = constraint.ineq();
        let is_strict = ineq.is_strict();
        let bound = ineq.bound();
        let i = constraint.i;
        let j = constraint.j;

        match (i, j) {
            (0, 0) => {
                unreachable!("Constraint with i == 0 and j == 0 is not allowed");
            }
            (0, j) => {
                // negated lower bound
                match is_strict {
                    true => {
                        BoolExpression::GreatT(var_from_naming(naming, j), arith_from_int(-bound))
                    }
                    false => {
                        BoolExpression::GreatEQ(var_from_naming(naming, j), arith_from_int(-bound))
                    }
                }
            }
            (i, 0) => {
                // upper bound
                match is_strict {
                    true => {
                        BoolExpression::LessT(var_from_naming(naming, i), arith_from_int(bound))
                    }
                    false => {
                        BoolExpression::LessEQ(var_from_naming(naming, i), arith_from_int(bound))
                    }
                }
            }
            (i, j) => {
                // difference
                if bound == 0 {
                    // i-j<=0 -> i <= 0+j
                    match is_strict {
                        true => BoolExpression::LessT(
                            var_from_naming(naming, i),
                            var_from_naming(naming, j),
                        ),
                        false => BoolExpression::LessEQ(
                            var_from_naming(naming, i),
                            var_from_naming(naming, j),
                        ),
                    }
                } else {
                    match is_strict {
                        true => BoolExpression::LessT(
                            var_diff_from_naming(naming, i, j),
                            arith_from_int(bound),
                        ),
                        false => BoolExpression::LessEQ(
                            var_diff_from_naming(naming, i, j),
                            arith_from_int(bound),
                        ),
                    }
                }
            }
        }
    }

    pub fn get_max_constant(&self, clock: ClockIndex, clock_name: &str) -> i32 {
        let mut new_constraint = 0;

        self.iterate_constraints(&mut |left, right| {
            //Start by matching left and right operands to get constant, this might fail if it does we skip constraint defaulting to 0
            let constant = ArithExpression::get_constant(left, right, clock, clock_name);

            if new_constraint < constant {
                new_constraint = constant;
            }
        });

        new_constraint
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
            BoolExpression::Bool(_) => {}
            BoolExpression::Arithmetic(x) => x.swap_var_name(from_name, to_name),
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

    pub fn iterate_constraints<F>(&self, function: &mut F)
    where
        F: FnMut(&ArithExpression, &ArithExpression),
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

            BoolExpression::LessEQ(l, r) => {
                **l = l.simplify().expect("Can't simplify");
                **r = r.simplify().expect("Can't simplify");
                if let ArithExpression::Int(x) = **l {
                    if let ArithExpression::Int(y) = **r {
                        value = Some(BoolExpression::Bool(x <= y))
                    }
                }
            }
            BoolExpression::GreatEQ(l, r) => {
                **l = l.simplify().expect("Can't simplify");
                **r = r.simplify().expect("Can't simplify");
                if let ArithExpression::Int(x) = **l {
                    if let ArithExpression::Int(y) = **r {
                        value = Some(BoolExpression::Bool(x >= y))
                    }
                }
            }
            BoolExpression::LessT(l, r) => {
                **l = l.simplify().expect("Can't simplify");
                **r = r.simplify().expect("Can't simplify");
                if let ArithExpression::Int(x) = **l {
                    if let ArithExpression::Int(y) = **r {
                        value = Some(BoolExpression::Bool(x < y))
                    }
                }
            }
            BoolExpression::GreatT(l, r) => {
                **l = l.simplify().expect("Can't simplify");
                **r = r.simplify().expect("Can't simplify");
                if let ArithExpression::Int(x) = **l {
                    if let ArithExpression::Int(y) = **r {
                        value = Some(BoolExpression::Bool(x > y))
                    }
                }
            }
            BoolExpression::EQ(l, r) => {
                **l = l.simplify().expect("Can't simplify");
                **r = r.simplify().expect("Can't simplify");
                if let ArithExpression::Int(x) = **l {
                    if let ArithExpression::Int(y) = **r {
                        value = Some(BoolExpression::Bool(x == y))
                    }
                }
            }
            BoolExpression::Arithmetic(x) => **x = x.simplify().expect("Can't simplify"),
            BoolExpression::Bool(_) => {}
        }

        if let Some(new_val) = value {
            *self = new_val;
            true
        } else {
            changed
        }
    }

    /// Finds the clock names used in the expression
    pub fn get_varnames(&self) -> Vec<&str> {
        match self {
            BoolExpression::AndOp(p1, p2) | BoolExpression::OrOp(p1, p2) => p1
                .get_varnames()
                .iter()
                .chain(p2.get_varnames().iter())
                .copied()
                .collect(),
            BoolExpression::LessEQ(a1, a2)
            | BoolExpression::GreatEQ(a1, a2)
            | BoolExpression::LessT(a1, a2)
            | BoolExpression::GreatT(a1, a2)
            | BoolExpression::EQ(a1, a2) => a1
                .get_varnames()
                .iter()
                .chain(a2.get_varnames().iter())
                .copied()
                .collect(),
            BoolExpression::Bool(_) => vec![],
            BoolExpression::Arithmetic(a) => a.get_varnames(),
        }
    }

    /// Replaces all occurrences of `ArithExpression::VarName(old)` with `new`

    /// # Arguments
    /// `old`: The `varname` to be replaced

    /// `new`: The new varname
    pub fn replace_varname(&mut self, old: &String, new: &String) {
        match self {
            BoolExpression::AndOp(e1, e2) | BoolExpression::OrOp(e1, e2) => {
                e1.replace_varname(old, new);
                e2.replace_varname(old, new);
            }
            BoolExpression::LessEQ(e1, e2)
            | BoolExpression::GreatEQ(e1, e2)
            | BoolExpression::LessT(e1, e2)
            | BoolExpression::GreatT(e1, e2)
            | BoolExpression::EQ(e1, e2) => {
                e1.replace_varname(old, new);
                e2.replace_varname(old, new);
            }
            BoolExpression::Bool(_) => (),
            BoolExpression::Arithmetic(a) => a.replace_varname(old, new),
        }
    }

    pub fn BLessEQ(left: ArithExpression, right: ArithExpression) -> BoolExpression {
        BoolExpression::LessEQ(Box::new(left), Box::new(right))
    }
    pub fn BLessT(left: ArithExpression, right: ArithExpression) -> BoolExpression {
        BoolExpression::LessT(Box::new(left), Box::new(right))
    }
    pub fn BGreatEQ(left: ArithExpression, right: ArithExpression) -> BoolExpression {
        BoolExpression::GreatEQ(Box::new(left), Box::new(right))
    }
    pub fn BGreatT(left: ArithExpression, right: ArithExpression) -> BoolExpression {
        BoolExpression::GreatT(Box::new(left), Box::new(right))
    }
    pub fn BEQ(left: ArithExpression, right: ArithExpression) -> BoolExpression {
        BoolExpression::EQ(Box::new(left), Box::new(right))
    }
    pub fn BPar(inner: BoolExpression) -> BoolExpression {
        inner
    }
}

fn var_from_naming(
    naming: &HashMap<ClockIndex, String>,
    index: ClockIndex,
) -> Box<ArithExpression> {
    Box::new(ArithExpression::VarName(
        naming.get(&index).unwrap().to_string(),
    ))
}

fn var_diff_from_naming(
    naming: &HashMap<ClockIndex, String>,
    i: ClockIndex,
    j: ClockIndex,
) -> Box<ArithExpression> {
    Box::new(ArithExpression::Difference(
        var_from_naming(naming, i),
        var_from_naming(naming, j),
    ))
}

fn arith_from_int(value: i32) -> Box<ArithExpression> {
    Box::new(ArithExpression::Int(value))
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

                let l = match **left {
                    BoolExpression::OrOp(_, _) => format!("({})", left),
                    _ => format!("{}", left),
                };
                let r = match **right {
                    BoolExpression::OrOp(_, _) => format!("({})", right),
                    _ => format!("{}", right),
                };
                write!(f, "{} && {}", l, r)?;
            }
            BoolExpression::OrOp(left, right) => {
                let l = match **left {
                    BoolExpression::AndOp(_, _) => format!("({})", left),
                    _ => format!("{}", left),
                };
                let r = match **right {
                    BoolExpression::AndOp(_, _) => format!("({})", right),
                    _ => format!("{}", right),
                };
                write!(f, "{} || {}", l, r)?;
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
            BoolExpression::Bool(val) => {
                if *val {
                    write!(f, "{}", val.to_string().green())?;
                } else {
                    write!(f, "{}", val.to_string().red())?;
                }
            }
            BoolExpression::Arithmetic(x) => {
                write!(f, "{}", x.encode_expr())?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub enum ArithExpression {
    Parentheses(Box<ArithExpression>),
    Difference(Box<ArithExpression>, Box<ArithExpression>),
    Addition(Box<ArithExpression>, Box<ArithExpression>),
    Multiplication(Box<ArithExpression>, Box<ArithExpression>),
    Division(Box<ArithExpression>, Box<ArithExpression>),
    Modulo(Box<ArithExpression>, Box<ArithExpression>),
    Clock(ClockIndex),
    VarName(String),
    Int(i32),
}

impl ArithExpression {
    pub fn swap_clock_names(
        &self,
        from_vars: &HashMap<String, ClockIndex>,
        to_vars: &HashMap<ClockIndex, String>,
    ) -> ArithExpression {
        match self {
            ArithExpression::Difference(left, right) => ArithExpression::Difference(
                Box::new(left.swap_clock_names(from_vars, to_vars)),
                Box::new(right.swap_clock_names(from_vars, to_vars)),
            ),
            ArithExpression::Addition(left, right) => ArithExpression::Addition(
                Box::new(left.swap_clock_names(from_vars, to_vars)),
                Box::new(right.swap_clock_names(from_vars, to_vars)),
            ),
            ArithExpression::Multiplication(left, right) => ArithExpression::Multiplication(
                Box::new(left.swap_clock_names(from_vars, to_vars)),
                Box::new(right.swap_clock_names(from_vars, to_vars)),
            ),
            ArithExpression::Division(left, right) => ArithExpression::Division(
                Box::new(left.swap_clock_names(from_vars, to_vars)),
                Box::new(right.swap_clock_names(from_vars, to_vars)),
            ),
            ArithExpression::Modulo(left, right) => ArithExpression::Modulo(
                Box::new(left.swap_clock_names(from_vars, to_vars)),
                Box::new(right.swap_clock_names(from_vars, to_vars)),
            ),
            ArithExpression::Clock(_) => panic!("Did not expect clock index in boolexpression, cannot swap clock names in misformed bexpr"),
            ArithExpression::VarName(name) => {
                let index = from_vars.get(name).unwrap();
                let new_name = to_vars[index].clone();
                ArithExpression::VarName(new_name)
            },
            ArithExpression::Int(val) => ArithExpression::Int(*val),
            ArithExpression::Parentheses(inner) => inner.swap_clock_names(from_vars, to_vars),
        }
    }

    pub fn encode_expr(&self) -> String {
        match self {
            ArithExpression::Difference(left, right) => {
                [left.encode_expr(), String::from("-"), right.encode_expr()].concat()
            }
            ArithExpression::Addition(left, right) => {
                [left.encode_expr(), String::from("+"), right.encode_expr()].concat()
            }
            ArithExpression::Multiplication(left, right) => {
                [left.encode_expr(), String::from("*"), right.encode_expr()].concat()
            }
            ArithExpression::Division(left, right) => {
                [left.encode_expr(), String::from("/"), right.encode_expr()].concat()
            }
            ArithExpression::Modulo(left, right) => {
                [left.encode_expr(), String::from("%"), right.encode_expr()].concat()
            }
            ArithExpression::Clock(_) => [String::from("??")].concat(),
            ArithExpression::VarName(var) => var.clone(),
            ArithExpression::Int(num) => num.to_string(),
            ArithExpression::Parentheses(inner) => format!("({})", inner.encode_expr()),
        }
    }

    pub fn get_max_constant(&self, clock: ClockIndex, clock_name: &str) -> i32 {
        let mut new_constraint = 0;

        self.iterate_constraints(&mut |left, right| {
            //Start by matching left and right operands to get constant, this might fail if it does we skip constraint defaulting to 0
            let constant = ArithExpression::get_constant(left, right, clock, clock_name);

            if new_constraint < constant {
                new_constraint = constant;
            }
        });

        new_constraint // * 2 + 1 // This should not actually be a dbm_raw, as it is converted from bound to raw in the c code
    }

    pub fn swap_var_name(&mut self, from_name: &str, to_name: &str) {
        match self {
            ArithExpression::Difference(left, right) => {
                left.swap_var_name(from_name, to_name);
                right.swap_var_name(from_name, to_name);
            }
            ArithExpression::Addition(left, right) => {
                left.swap_var_name(from_name, to_name);
                right.swap_var_name(from_name, to_name);
            }
            ArithExpression::Multiplication(left, right) => {
                left.swap_var_name(from_name, to_name);
                right.swap_var_name(from_name, to_name);
            }
            ArithExpression::Division(left, right) => {
                left.swap_var_name(from_name, to_name);
                right.swap_var_name(from_name, to_name);
            }
            ArithExpression::Modulo(left, right) => {
                left.swap_var_name(from_name, to_name);
                right.swap_var_name(from_name, to_name);
            }
            ArithExpression::Clock(_) => {
                //Assuming ids are correctly offset we dont have to do anything here
            }
            ArithExpression::VarName(name) => {
                if *name == from_name {
                    *name = to_name.to_string();
                }
            }
            ArithExpression::Int(_) => {}
            ArithExpression::Parentheses(inner) => inner.swap_var_name(from_name, to_name),
        }
    }

    pub fn get_constant(left: &Self, right: &Self, clock: ClockIndex, clock_name: &str) -> i32 {
        match left {
            ArithExpression::Clock(clock_id) => {
                if *clock_id == clock {
                    if let ArithExpression::Int(constant) = right {
                        return *constant;
                    }
                }
            }
            ArithExpression::VarName(name) => {
                if name.eq(clock_name) {
                    if let ArithExpression::Int(constant) = right {
                        return *constant;
                    }
                }
            }
            ArithExpression::Int(constant) => match right {
                ArithExpression::Clock(clock_id) => {
                    if *clock_id == clock {
                        return *constant;
                    }
                }
                ArithExpression::VarName(name) => {
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
        F: FnMut(&ArithExpression, &ArithExpression),
    {
        match self {
            ArithExpression::Parentheses(inner) => inner.iterate_constraints(function),
            ArithExpression::Difference(left, right) => function(left, right),
            ArithExpression::Addition(left, right) => function(left, right),
            ArithExpression::Multiplication(left, right) => function(left, right),
            ArithExpression::Division(left, right) => function(left, right),
            ArithExpression::Modulo(left, right) => function(left, right),
            ArithExpression::Clock(_) => {}
            ArithExpression::VarName(_) => {}
            ArithExpression::Int(_) => {}
        }
    }

    pub fn simplify(&self) -> Result<ArithExpression, String> {
        let mut out = self.clone();
        let mut diffs: Vec<(ArithExpression, Operation)> = vec![];
        let mut op = Operation::None;
        while let Some(x) = out.move_clock_and_vars(op)? {
            op = x.1.clone();
            diffs.push(x);
        }
        while let Some((val, op)) = diffs.pop() {
            match op {
                Operation::Dif(right) => {
                    out = match right {
                        true => ArithExpression::ADif(out, val),
                        false => ArithExpression::ADif(val, out),
                    }
                }
                Operation::Add(right) => {
                    out = match right {
                        true => ArithExpression::AAdd(out, val),
                        false => ArithExpression::AAdd(val, out),
                    }
                }
                Operation::Mul(right) => {
                    out = match right {
                        true => ArithExpression::AMul(out, val),
                        false => ArithExpression::AMul(val, out),
                    }
                }
                Operation::Div(right) => {
                    out = match right {
                        true => ArithExpression::ADiv(out, val),
                        false => ArithExpression::ADiv(val, out),
                    }
                }
                Operation::Mod(right) => {
                    out = match right {
                        true => ArithExpression::AMod(out, val),
                        false => ArithExpression::AMod(val, out),
                    }
                }
                Operation::None => out = val,
            }
        }
        while out.simplify_helper() {}
        Ok(out)
    }

    fn move_clock_and_vars(
        &mut self,
        prev_op: Operation,
    ) -> Result<Option<(ArithExpression, Operation)>, String> {
        let mut switch: Option<ArithExpression> = None;
        let out = match self {
            ArithExpression::Parentheses(inner) => inner.move_clock_and_vars(prev_op)?,
            ArithExpression::Clock(x) => {
                switch = Some(ArithExpression::Int(0));
                Some((ArithExpression::Clock(*x), prev_op))
            }
            ArithExpression::VarName(string) => {
                switch = Some(ArithExpression::Int(0));
                Some((ArithExpression::VarName(string.clone()), prev_op))
            }
            ArithExpression::Int(_) => None,
            ArithExpression::Difference(l, r) => {
                if l.clock_var_count() > 0 {
                    switch = ArithExpression::clone_expr(l, r, None)?;
                    l.move_clock_and_vars(Operation::Dif(false))?
                } else if r.clock_var_count() > 0 {
                    switch = ArithExpression::clone_expr(r, l, None)?;
                    r.move_clock_and_vars(Operation::Dif(true))?
                } else {
                    None
                }
            }
            ArithExpression::Addition(l, r) => {
                if l.clock_var_count() > 0 {
                    switch = ArithExpression::clone_expr(l, r, None)?;
                    l.move_clock_and_vars(Operation::Add(false))?
                } else if r.clock_var_count() > 0 {
                    switch = ArithExpression::clone_expr(r, l, None)?;
                    r.move_clock_and_vars(Operation::Add(true))?
                } else {
                    None
                }
            }
            ArithExpression::Multiplication(l, r) => {
                if l.clock_var_count() > 0 {
                    switch = ArithExpression::clone_expr(
                        l,
                        r,
                        Some("Can't parse multiplication with clocks"),
                    )?;
                    l.move_clock_and_vars(Operation::Mul(false))?
                } else if r.clock_var_count() > 0 {
                    switch = ArithExpression::clone_expr(
                        r,
                        l,
                        Some("Can't parse multiplication with clocks"),
                    )?;
                    r.move_clock_and_vars(Operation::Mul(true))?
                } else {
                    None
                }
            }
            ArithExpression::Division(l, r) => {
                if l.clock_var_count() > 0 {
                    switch = ArithExpression::clone_expr(
                        l,
                        r,
                        Some("Can't parse division with clocks"),
                    )?;
                    l.move_clock_and_vars(Operation::Div(false))?
                } else if r.clock_var_count() > 0 {
                    switch = ArithExpression::clone_expr(
                        r,
                        l,
                        Some("Can't parse division with clocks"),
                    )?;
                    r.move_clock_and_vars(Operation::Div(true))?
                } else {
                    None
                }
            }
            ArithExpression::Modulo(l, r) => {
                if l.clock_var_count() > 0 {
                    switch =
                        ArithExpression::clone_expr(l, r, Some("Can't parse modulo with clocks"))?;
                    l.move_clock_and_vars(Operation::Mod(false))?
                } else if r.clock_var_count() > 0 {
                    switch =
                        ArithExpression::clone_expr(r, l, Some("Can't parse modulo with clocks"))?;
                    r.move_clock_and_vars(Operation::Mod(true))?
                } else {
                    None
                }
            }
        };

        if let Some(x) = switch {
            *self = x;
        }
        Ok(out)
    }

    fn clone_expr(
        checker: &ArithExpression,
        cloner: &ArithExpression,
        err_msg: Option<&str>,
    ) -> Result<Option<ArithExpression>, String> {
        if let ArithExpression::Clock(_) = *checker {
            if let Some(e) = err_msg {
                Err(e.to_string())
            } else {
                Ok(Some(cloner.clone()))
            }
        } else if let ArithExpression::VarName(_) = *checker {
            Ok(Some(cloner.clone()))
        } else {
            Ok(None)
        }
    }

    fn simplify_helper(&mut self) -> bool {
        let mut changed = false;
        let mut value: Option<ArithExpression> = None;
        match self {
            ArithExpression::Parentheses(inner) => {
                value = Some((**inner).clone());
            }
            ArithExpression::Difference(l, r) => {
                changed = l.simplify_helper() | r.simplify_helper();
                if let (ArithExpression::Int(x), ArithExpression::Int(y)) = (l.as_ref(), r.as_ref())
                {
                    value = Some(ArithExpression::Int(x - y));
                }
            }
            ArithExpression::Addition(l, r) => {
                changed = l.simplify_helper() | r.simplify_helper();
                if let (ArithExpression::Int(x), ArithExpression::Int(y)) = (l.as_ref(), r.as_ref())
                {
                    value = Some(ArithExpression::Int(x + y));
                }
            }
            ArithExpression::Multiplication(l, r) => {
                changed = l.simplify_helper() | r.simplify_helper();
                if let (ArithExpression::Int(x), ArithExpression::Int(y)) = (l.as_ref(), r.as_ref())
                {
                    value = Some(ArithExpression::Int(x * y));
                }
            }
            ArithExpression::Division(l, r) => {
                changed = l.simplify_helper() | r.simplify_helper();
                if let (ArithExpression::Int(x), ArithExpression::Int(y)) = (l.as_ref(), r.as_ref())
                {
                    value = Some(ArithExpression::Int(x / y));
                }
            }
            ArithExpression::Modulo(l, r) => {
                changed = l.simplify_helper() | r.simplify_helper();
                if let (ArithExpression::Int(x), ArithExpression::Int(y)) = (l.as_ref(), r.as_ref())
                {
                    value = Some(ArithExpression::Int(x % y));
                }
            }
            ArithExpression::Clock(_) => {}
            ArithExpression::VarName(_) => {}
            ArithExpression::Int(_) => {}
        }

        if let Some(new_val) = value {
            *self = new_val;
            true
        } else {
            changed
        }
    }

    /// Finds the clock names used in the expression
    pub fn get_varnames(&self) -> Vec<&str> {
        match self {
            ArithExpression::Parentheses(p) => p.get_varnames(),
            ArithExpression::Difference(a1, a2)
            | ArithExpression::Addition(a1, a2)
            | ArithExpression::Multiplication(a1, a2)
            | ArithExpression::Division(a1, a2)
            | ArithExpression::Modulo(a1, a2) => a1
                .get_varnames()
                .iter()
                .chain(a2.get_varnames().iter())
                .copied()
                .collect(),
            ArithExpression::Clock(_) | ArithExpression::Int(_) => vec![],
            ArithExpression::VarName(name) => vec![name.as_str()],
        }
    }

    /// Replaces all occurrences of `ArithExpression::VarName(old)` with `new`

    /// # Arguments
    /// `old`: The `varname` to be replaced

    /// `new`: The new varname
    pub fn replace_varname(&mut self, old: &String, new: &String) {
        match self {
            ArithExpression::Parentheses(p) => p.replace_varname(old, new),
            ArithExpression::Difference(a1, a2)
            | ArithExpression::Addition(a1, a2)
            | ArithExpression::Multiplication(a1, a2)
            | ArithExpression::Division(a1, a2)
            | ArithExpression::Modulo(a1, a2) => {
                a1.replace_varname(old, new);
                a2.replace_varname(old, new);
            }
            ArithExpression::Clock(_) | ArithExpression::Int(_) => (),
            ArithExpression::VarName(name) => {
                if *name == *old {
                    *name = new.to_string();
                }
            }
        }
    }

    pub fn clock_var_count(&self) -> u32 {
        match self {
            ArithExpression::Clock(_) => 1,
            ArithExpression::VarName(_) => 1,
            ArithExpression::Parentheses(inner) => inner.clock_var_count(),
            ArithExpression::Difference(l, r)
            | ArithExpression::Addition(l, r)
            | ArithExpression::Multiplication(l, r)
            | ArithExpression::Division(l, r)
            | ArithExpression::Modulo(l, r) => l.clock_var_count() + r.clock_var_count(),
            _ => 0,
        }
    }

    pub fn APar(inner: ArithExpression) -> ArithExpression {
        inner
    }

    pub fn ADif(left: ArithExpression, right: ArithExpression) -> ArithExpression {
        if let ArithExpression::Int(0) = right {
            return left;
        }

        if let ArithExpression::Int(i) = left {
            if let ArithExpression::Int(j) = right {
                return ArithExpression::Int(i - j);
            }
        }

        ArithExpression::Difference(Box::new(left), Box::new(right))
    }

    pub fn AAdd(left: ArithExpression, right: ArithExpression) -> ArithExpression {
        if let ArithExpression::Int(0) = right {
            return left;
        } else if let ArithExpression::Int(0) = left {
            return right;
        }

        if let ArithExpression::Int(i) = left {
            if let ArithExpression::Int(j) = right {
                return ArithExpression::Int(i + j);
            }
        }

        ArithExpression::Addition(Box::new(left), Box::new(right))
    }

    pub fn AMul(left: ArithExpression, right: ArithExpression) -> ArithExpression {
        if right == ArithExpression::Int(0) || left == ArithExpression::Int(0) {
            return ArithExpression::Int(0);
        }

        if let ArithExpression::Int(i) = left {
            if let ArithExpression::Int(j) = right {
                return ArithExpression::Int(i * j);
            }
        }

        ArithExpression::Multiplication(Box::new(left), Box::new(right))
    }

    pub fn ADiv(left: ArithExpression, right: ArithExpression) -> ArithExpression {
        if right == ArithExpression::Int(0) || left == ArithExpression::Int(0) {
            return ArithExpression::Int(0);
        }

        if let ArithExpression::Int(i) = left {
            if let ArithExpression::Int(j) = right {
                return ArithExpression::Int(i / j);
            }
        }

        ArithExpression::Division(Box::new(left), Box::new(right))
    }

    pub fn AMod(left: ArithExpression, right: ArithExpression) -> ArithExpression {
        if let ArithExpression::Int(i) = left {
            if let ArithExpression::Int(j) = right {
                return ArithExpression::Int(i % j);
            }
        }

        ArithExpression::Modulo(Box::new(left), Box::new(right))
    }
}

impl Display for ArithExpression {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ArithExpression::Parentheses(expr) => {
                let l_par = "(".to_string().yellow();
                let r_par = ")".to_string().yellow();
                write!(f, "{}{}{}", l_par, expr, r_par)?;
            }
            ArithExpression::Clock(id) => {
                write!(f, "{}", format!("c:{}", id).magenta())?;
            }
            ArithExpression::VarName(name) => {
                write!(f, "{}", name.to_string().blue())?;
            }
            ArithExpression::Int(num) => {
                write!(f, "{}", num)?;
            }
            ArithExpression::Difference(left, right) => {
                write!(f, "{}-{}", left, right)?;
            }
            ArithExpression::Addition(left, right) => {
                write!(f, "{}+{}", left, right)?;
            }
            ArithExpression::Multiplication(left, right) => {
                write!(f, "{}*{}", left, right)?;
            }
            ArithExpression::Division(left, right) => {
                write!(f, "{}/{}", left, right)?;
            }
            ArithExpression::Modulo(left, right) => {
                write!(f, "{}%{}", left, right)?;
            }
        }
        Ok(())
    }
}

/// Variants represent whether the clock was on the rhs of an expression or not (true == right)
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
enum Operation {
    Dif(bool),
    Add(bool),
    Mul(bool),
    Div(bool),
    Mod(bool),
    None,
}

pub struct Clock {
    pub value: ClockIndex,
    pub negated: bool,
}

impl Clock {
    pub fn new(v: ClockIndex, n: bool) -> Clock {
        Clock {
            value: v,
            negated: n,
        }
    }

    pub fn neg(v: ClockIndex) -> Clock {
        Clock {
            value: v,
            negated: true,
        }
    }

    pub fn pos(v: ClockIndex) -> Clock {
        Clock {
            value: v,
            negated: false,
        }
    }

    pub fn invert(&mut self) {
        self.negated = !self.negated;
    }
}

fn get_op(exp: &BoolExpression) -> Option<String> {
    match exp {
        BoolExpression::EQ(_, _) => Some("=".to_string()),
        BoolExpression::LessEQ(_, _) => Some("≤".to_string()),
        BoolExpression::LessT(_, _) => Some("<".to_string()),
        _ => None,
    }
}

#[derive(Debug, Clone)]
pub enum QueryExpression {
    Refinement(SystemExpression, SystemExpression),
    Consistency(SystemExpression),
    Reachability {
        system: SystemExpression,
        from: Option<StateExpression>,
        to: StateExpression,
    },
    Implementation(SystemExpression),
    Determinism(SystemExpression),
    Specification(SystemExpression),
    GetComponent(SaveExpression),
    Prune(SaveExpression),
    BisimMinim(SaveExpression),
}

#[derive(Debug, Clone)]
pub struct SaveExpression {
    pub system: SystemExpression,
    pub name: Option<String>,
}

#[derive(Debug, Clone)]
pub enum StateExpression {
    LEQ(OperandExpression, OperandExpression),
    GEQ(OperandExpression, OperandExpression),
    EQ(OperandExpression, OperandExpression),
    LT(OperandExpression, OperandExpression),
    GT(OperandExpression, OperandExpression),
    AND(Vec<StateExpression>),
    OR(Vec<StateExpression>),
    Location(ComponentVariable),
    NOT(Box<StateExpression>),
    Bool(bool),
}

#[derive(Debug, Clone)]
pub enum OperandExpression {
    Number(i32),
    Clock(ComponentVariable),
    Difference(Box<OperandExpression>, Box<OperandExpression>),
    Sum(Box<OperandExpression>, Box<OperandExpression>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentVariable {
    /// Fx. `"A[Temp].x"` -> `"A"`
    pub component: String,
    /// Fx. `"A[Temp].x"` -> `Some("Temp")` or `"A.x"` -> `None`
    pub special_id: Option<String>,
    /// Fx. `"A[Temp].x"` -> `"x"`
    pub variable: String,
}

#[derive(Debug, Clone)]
pub enum SystemExpression {
    /// Fx. `"A[Temp]"` -> `Component("A", Some("Temp"))`
    /// Fx. `"A"` -> `Component("A", None)`
    Component(String, Option<String>),
    Quotient(Box<SystemExpression>, Box<SystemExpression>),
    Composition(Box<SystemExpression>, Box<SystemExpression>),
    Conjunction(Box<SystemExpression>, Box<SystemExpression>),
}

impl SystemExpression {
    pub fn pretty_string(&self) -> String {
        match self {
            SystemExpression::Component(name, Some(id)) => format!("{}[{}]", name, id),
            SystemExpression::Component(name, None) => name.clone(),
            SystemExpression::Quotient(left, right) => {
                format!("({} \\\\ {})", left.pretty_string(), right.pretty_string())
            }
            SystemExpression::Composition(left, right) => {
                format!("({} || {})", left.pretty_string(), right.pretty_string())
            }
            SystemExpression::Conjunction(left, right) => {
                format!("({} && {})", left.pretty_string(), right.pretty_string())
            }
        }
    }
}

impl SaveExpression {
    pub fn pretty_string(&self) -> String {
        match &self.name {
            Some(name) => format!("{} save-as {}", name, self.system.pretty_string()),
            None => self.system.pretty_string(),
        }
    }
}

impl OperandExpression {
    pub fn to_arith_expression(
        &self,
        comps: &Vec<&crate::component::Component>,
    ) -> Result<ArithExpression, String> {
        match self {
            OperandExpression::Number(n) => Ok(ArithExpression::Int(*n)),
            OperandExpression::Clock(ComponentVariable {
                component,
                special_id,
                variable,
            }) => {
                let comp = comps
                    .iter()
                    .find(|c| c.name == *component && c.special_id == *special_id)
                    .ok_or_else(|| format!("Component '{}' not found", component))?;
                let clock_id = comp
                    .declarations
                    .get_clock_index_by_name(variable)
                    .ok_or_else(|| {
                        format!(
                            "Clock '{}' not found for component '{}'",
                            variable, component
                        )
                    })?;

                Ok(ArithExpression::Clock(*clock_id))
            }
            OperandExpression::Difference(left, right) => Ok(ArithExpression::Difference(
                Box::new(left.to_arith_expression(comps)?),
                Box::new(right.to_arith_expression(comps)?),
            )),
            OperandExpression::Sum(left, right) => Ok(ArithExpression::Addition(
                Box::new(left.to_arith_expression(comps)?),
                Box::new(right.to_arith_expression(comps)?),
            )),
        }
    }
    pub fn pretty_string(&self) -> String {
        match self {
            OperandExpression::Number(n) => format!("{}", n),
            OperandExpression::Clock(ComponentVariable {
                component,
                special_id: None,
                variable,
            }) => format!("{}.{}", component, variable),
            OperandExpression::Clock(ComponentVariable {
                component,
                special_id: Some(id),
                variable,
            }) => format!("{}[{}].{}", component, id, variable),
            OperandExpression::Difference(left, right) => {
                format!("{} - {}", left.pretty_string(), right.pretty_string())
            }
            OperandExpression::Sum(left, right) => {
                format!("{} + {}", left.pretty_string(), right.pretty_string())
            }
        }
    }
}

impl StateExpression {
    pub fn to_bool_expression(
        &self,
        comps: &Vec<&crate::component::Component>,
    ) -> Result<BoolExpression, String> {
        match self {
            StateExpression::LEQ(left, right) => Ok(BoolExpression::LessEQ(
                Box::new(left.to_arith_expression(comps)?),
                Box::new(right.to_arith_expression(comps)?),
            )),
            StateExpression::GEQ(left, right) => Ok(BoolExpression::GreatEQ(
                Box::new(left.to_arith_expression(comps)?),
                Box::new(right.to_arith_expression(comps)?),
            )),
            StateExpression::EQ(left, right) => Ok(BoolExpression::EQ(
                Box::new(left.to_arith_expression(comps)?),
                Box::new(right.to_arith_expression(comps)?),
            )),
            StateExpression::LT(left, right) => Ok(BoolExpression::LessT(
                Box::new(left.to_arith_expression(comps)?),
                Box::new(right.to_arith_expression(comps)?),
            )),
            StateExpression::GT(left, right) => Ok(BoolExpression::GreatT(
                Box::new(left.to_arith_expression(comps)?),
                Box::new(right.to_arith_expression(comps)?),
            )),
            StateExpression::AND(exprs) => {
                let mut exprs = exprs
                    .iter()
                    .map(|e| e.to_bool_expression(comps))
                    .collect::<Result<Vec<BoolExpression>, _>>()?
                    .into_iter();
                let first = exprs
                    .next()
                    .expect("AND expression must have at least one operand");
                Ok(exprs.fold(first, |acc, e| {
                    BoolExpression::AndOp(Box::new(acc), Box::new(e))
                }))
            }
            StateExpression::OR(exprs) => {
                let mut exprs = exprs
                    .iter()
                    .map(|e| e.to_bool_expression(comps))
                    .collect::<Result<Vec<BoolExpression>, _>>()?
                    .into_iter();
                let first = exprs
                    .next()
                    .expect("OR expression must have at least one operand");
                Ok(exprs.fold(first, |acc, e| {
                    BoolExpression::OrOp(Box::new(acc), Box::new(e))
                }))
            }
            StateExpression::NOT(_expr) => {
                unimplemented!("NOT expressions are not supported yet")
            }
            StateExpression::Location(_) => {
                // Locations here should just be ignored
                Ok(BoolExpression::Bool(true))
            }
            StateExpression::Bool(b) => Ok(BoolExpression::Bool(*b)),
        }
    }
    pub fn pretty_string(&self) -> String {
        match self {
            StateExpression::LEQ(left, right) => {
                format!("{} <= {}", left.pretty_string(), right.pretty_string())
            }
            StateExpression::GEQ(left, right) => {
                format!("{} >= {}", left.pretty_string(), right.pretty_string())
            }
            StateExpression::EQ(left, right) => {
                format!("{} == {}", left.pretty_string(), right.pretty_string())
            }
            StateExpression::LT(left, right) => {
                format!("{} < {}", left.pretty_string(), right.pretty_string())
            }
            StateExpression::GT(left, right) => {
                format!("{} > {}", left.pretty_string(), right.pretty_string())
            }
            StateExpression::AND(exprs) => {
                let mut s = "(".to_string();
                for (i, expr) in exprs.iter().enumerate() {
                    if i > 0 {
                        s.push_str(" && ");
                    }
                    s.push_str(&expr.pretty_string());
                }
                s.push(')');
                s
            }
            StateExpression::OR(exprs) => {
                let mut s = "(".to_string();
                for (i, expr) in exprs.iter().enumerate() {
                    if i > 0 {
                        s.push_str(" || ");
                    }
                    s.push_str(&expr.pretty_string());
                }
                s.push(')');
                s
            }
            StateExpression::Location(ComponentVariable {
                component,
                special_id: None,
                variable,
            }) => format!("{}.{}", component, variable),
            StateExpression::Location(ComponentVariable {
                component,
                special_id: Some(id),
                variable,
            }) => format!("{}[{}].{}", component, id, variable),
            StateExpression::NOT(expr) => format!("!({})", expr.pretty_string()),
            StateExpression::Bool(b) => format!("{}", b),
        }
    }
}

impl QueryExpression {
    pub fn pretty_string(&self) -> String {
        match self {
            QueryExpression::Refinement(left, right) => format!(
                "refinement: {} <= {}",
                left.pretty_string(),
                right.pretty_string()
            ),
            QueryExpression::Reachability { system, from, to } => {
                format!(
                    "reachability: {} @ {} -> {}",
                    system.pretty_string(),
                    match from {
                        Some(expr) => expr.pretty_string(),
                        None => "init".to_string(),
                    },
                    to.pretty_string()
                )
            }
            QueryExpression::Consistency(system) => {
                format!("consistency: {}", system.pretty_string())
            }
            QueryExpression::GetComponent(comp) => {
                format!("get-component: {}", comp.pretty_string())
            }
            QueryExpression::Prune(comp) => {
                format!("prune: {}", comp.pretty_string())
            }
            QueryExpression::BisimMinim(comp) => {
                format!("bisim-minim: {}", comp.pretty_string())
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
        }
    }
}
