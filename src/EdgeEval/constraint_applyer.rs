use crate::component::Declarations;
use crate::DBMLib::dbm::Federation;
use crate::ModelObjects::component;
use crate::ModelObjects::representations::{ArithExpression, BoolExpression, Clock};
use std::collections::HashMap;
use std::convert::TryFrom;

pub fn apply_constraint(
    constraint: &Option<BoolExpression>,
    decls: &Declarations,
    zone: &mut Federation,
) -> bool {
    return if let Some(guards) = constraint {
        apply_constraints_to_state(guards, decls, zone)
            .expect(format!("Failed to apply constraint {:?}", guards).as_str())
    } else {
        true
    };
}

pub fn apply_constraints_to_state(
    guard: &BoolExpression,
    decls: &Declarations,
    zone: &mut Federation,
) -> Result<bool, String> {
    apply_constraints_to_state_helper(guard, decls, zone)
}

pub fn apply_constraints_to_state_helper(
    guard: &BoolExpression,
    decls: &Declarations,
    zone: &mut Federation,
) -> Result<bool, String> {
    match guard {
        BoolExpression::AndOp(left, right) => {
            Ok(apply_constraints_to_state_helper(left, decls, zone)?
                && apply_constraints_to_state_helper(right, decls, zone)?)
        }
        BoolExpression::OrOp(left, right) => {
            let mut clone = zone.clone();
            let res1 = apply_constraints_to_state_helper(left, decls, zone)?;
            let res2 = apply_constraints_to_state_helper(right, decls, &mut clone)?;
            *zone += clone;
            Ok(res1 || res2)
        }
        BoolExpression::LessEQ(left, right) => {
            let (i, j, c) = get_indices(left, right, decls)?;
            // i-j<=c
            Ok(zone.constrain(i, j, c, false))
        }
        BoolExpression::GreatEQ(left, right) => {
            let (i, j, c) = get_indices(right, left, decls)?;
            // j-i <= -c -> c <= i-j
            Ok(zone.constrain(i, j, c, false))
        }
        BoolExpression::EQ(left, right) => {
            let (i, j, c) = get_indices(left, right, decls)?;
            // i-j <= c && j-i <= -c -> c <= i-j
            Ok(zone.constrain(i, j, c, false) && zone.constrain(j, i, -c, false))
        }
        BoolExpression::LessT(left, right) => {
            let (i, j, c) = get_indices(left, right, decls)?;
            // i-j < c
            Ok(zone.constrain(i, j, c, true))
        }
        BoolExpression::GreatT(left, right) => {
            let (i, j, c) = get_indices(right, left, decls)?;
            // j-i < -c -> c < i-j
            Ok(zone.constrain(i, j, c, true))
        }
        BoolExpression::Parentheses(expr) => apply_constraints_to_state_helper(expr, decls, zone),
        BoolExpression::Bool(val) => {
            if !*val {
                *zone = Federation::empty(zone.get_dimensions());
            }
            Ok(*val)
        }
        _ => Err(format!("Unexpected BoolExpression")),
    }
}

/// Assumes that the constraint is of the form left <?= right
fn get_indices(
    left: &ArithExpression,
    right: &ArithExpression,
    d: &Declarations,
) -> Result<(u32, u32, i32), String> {
    let left = &(replace_vars(left, d).simplify())?;
    let right = &(replace_vars(right, d).simplify())?;
    let (clocks_left, clocks_right) = (left.clock_var_count(), right.clock_var_count());

    if clocks_left + clocks_right == 0 {
        return Err(String::from(
            "Expressions must use clocks to get indices, this should be unreachable",
        ));
    } else if clocks_left + clocks_right > 2 {
        return Err(String::from("Too many clocks"));
    }

    let (left_const, right_const) = (get_const(left, d), get_const(right, d));
    let constant = right_const - left_const;

    let result: Result<(u32, u32, i32), String> = match (clocks_left, clocks_right) {
        (1, 1) => {
            let (c1, c2) = (
                get_clock_val(left, d, 1, false)?.0,
                get_clock_val(right, d, 1, false)?.0,
            );
            combine_clocks(c1, c2, constant, true)
        }
        (2, 0) => {
            let (c1, c2) = get_clock_val(left, d, 2, false)?;
            combine_clocks(c1, c2.unwrap(), constant, false)
        }
        (0, 2) => {
            let (mut c1, c2) = get_clock_val(right, d, 2, false)?;
            let mut c2 = c2.unwrap();
            c1.invert();
            c2.invert();
            combine_clocks(c1, c2, constant, false)
        }
        (1, 0) => {
            let c = get_clock_val(left, d, 1, false)?.0;
            if c.negated {
                Ok((0, c.value, constant))
            } else {
                Ok((c.value, 0, constant))
            }
        }
        (0, 1) => {
            let c = get_clock_val(right, d, 1, false)?.0;
            if !c.negated {
                Ok((0, c.value, constant))
            } else {
                Ok((c.value, 0, constant))
            }
        }
        _ => unreachable!(),
    };
    result
}

fn replace_vars(expr: &ArithExpression, decls: &Declarations) -> ArithExpression {
    //let mut out = expr.clone();
    match expr {
        ArithExpression::Parentheses(inner) => replace_vars(&inner, decls),
        ArithExpression::Difference(l, r) => {
            ArithExpression::ADif(replace_vars(&l, decls), replace_vars(&r, decls))
        }
        ArithExpression::Addition(l, r) => {
            ArithExpression::AAdd(replace_vars(&l, decls), replace_vars(&r, decls))
        }
        ArithExpression::Multiplication(l, r) => {
            ArithExpression::AMul(replace_vars(&l, decls), replace_vars(&r, decls))
        }
        ArithExpression::Division(l, r) => {
            ArithExpression::ADiv(replace_vars(&l, decls), replace_vars(&r, decls))
        }
        ArithExpression::Modulo(l, r) => {
            ArithExpression::AMod(replace_vars(&l, decls), replace_vars(&r, decls))
        }
        ArithExpression::Clock(x) => ArithExpression::Clock(*x),
        ArithExpression::VarName(name) => {
            if let Some(x) = decls.get_clocks().get(name.as_str()).and_then(|o| Some(*o)) {
                ArithExpression::Clock(x)
            } else {
                ArithExpression::Int(
                    decls
                        .get_ints()
                        .get(name.as_str())
                        .and_then(|o| Some(*o))
                        .unwrap(),
                )
            }
        }
        ArithExpression::Int(i) => ArithExpression::Int(*i),
    }
}

fn get_const(expr: &ArithExpression, decls: &Declarations) -> i32 {
    match expr {
        ArithExpression::Int(x) => *x,
        ArithExpression::Clock(_) => 0,
        ArithExpression::VarName(name) => decls
            .get_ints()
            .get(name)
            .and_then(|o| Some(*o))
            .unwrap_or(0),
        ArithExpression::Parentheses(x) => get_const(x, decls),
        ArithExpression::Difference(l, r) => get_const(l, decls) - get_const(r, decls),
        ArithExpression::Addition(l, r) => get_const(l, decls) + get_const(r, decls),
        ArithExpression::Multiplication(l, r) => get_const(l, decls) * get_const(r, decls),
        ArithExpression::Division(l, r) => get_const(l, decls) / get_const(r, decls),
        ArithExpression::Modulo(l, r) => get_const(l, decls) % get_const(r, decls),
        _ => 0,
    }
}

fn combine_clocks(
    c1: Clock,
    c2: Clock,
    constant: i32,
    same_sign: bool,
) -> Result<(u32, u32, i32), String> {
    if (same_sign && c1.negated != c2.negated) || (!same_sign && c1.negated == c2.negated) {
        Err(String::from("Same sign"))
    } else {
        if c1.negated == false {
            Ok((c1.value, c2.value, constant))
        } else {
            Ok((c2.value, c1.value, constant))
        }
    }
}

fn get_clock_val(
    expression: &ArithExpression,
    decls: &Declarations,
    count: i32,
    negated: bool,
) -> Result<(Clock, Option<Clock>), String> {
    let mut nxt_expr: Option<&ArithExpression> = None;
    let mut nxt_negated = false;
    let val = match expression {
        ArithExpression::Parentheses(inner) => get_clock_val(inner, decls, count, negated)?.0,
        ArithExpression::Difference(l, r) => {
            if let ArithExpression::Clock(x) = **l {
                nxt_expr = Some(&**r);
                nxt_negated = true;
                Clock::pos(x)
            } else if let ArithExpression::Clock(x) = **r {
                nxt_expr = Some(&**l);
                Clock::neg(x)
            } else {
                return Err(String::from("No Clocks"));
            }
        }
        ArithExpression::Addition(l, r) => {
            if let ArithExpression::Clock(x) = **l {
                nxt_expr = Some(&**r);
                Clock::pos(x)
            } else if let ArithExpression::Clock(x) = **r {
                nxt_expr = Some(&**l);
                Clock::pos(x)
            } else {
                return Err(String::from("No Clocks"));
            }
        }
        ArithExpression::Multiplication(_, _)
        | ArithExpression::Division(_, _)
        | ArithExpression::Modulo(_, _) => {
            return Err(format!("Multiplication with clock is illegal"));
        }
        ArithExpression::Clock(x) => Clock::new(*x, negated),
        _ => return Err(String::from("No Clocks")),
    };

    if count > 1 {
        Ok((
            val,
            Some(get_clock_val(nxt_expr.unwrap(), decls, count - 1, nxt_negated)?.0),
        ))
    } else {
        Ok((val, None))
    }
}

mod test {
    use super::get_indices;
    use crate::component::Declarations;
    use crate::ModelObjects::representations::ArithExpression;
    use std::collections::HashMap;

    #[test]
    fn test_get_indices_int_clock() {
        let decl = Declarations {
            clocks: HashMap::new(),
            ints: HashMap::new(),
        };

        let left = ArithExpression::Int(3);
        let right = ArithExpression::Clock(1);

        //Testing: left < right
        assert_eq!(get_indices(&left, &right, &decl).ok(), Some((0, 1, -3)));
    }

    #[test]
    fn test_get_indices_clock_int() {
        let decl = Declarations {
            clocks: HashMap::new(),
            ints: HashMap::new(),
        };

        let left = ArithExpression::Clock(1);
        let right = ArithExpression::Int(3);

        //Testing: left < right
        assert_eq!(get_indices(&left, &right, &decl).ok(), Some((1, 0, 3)));
    }

    #[test]
    fn test_get_indices_clock_clock() {
        let decl = Declarations {
            clocks: HashMap::new(),
            ints: HashMap::new(),
        };

        // i-j <= 0 -> i can at most be the value of j
        let left = ArithExpression::Clock(1);
        let right = ArithExpression::Clock(2);

        //Testing: left < right
        assert_eq!(get_indices(&left, &right, &decl).ok(), Some((1, 2, 0)));
    }

    #[test]
    fn test_get_indices_diff_int() {
        let decl = Declarations {
            clocks: HashMap::new(),
            ints: HashMap::new(),
        };
        // i-j < c -> c1-c2 < 3
        let left = ArithExpression::ADif(ArithExpression::Clock(1), ArithExpression::Clock(2));
        let right = ArithExpression::Int(3);
        //Testing: left < right
        assert_eq!(get_indices(&left, &right, &decl).ok(), Some((1, 2, 3)));

        let left = ArithExpression::ADif(ArithExpression::Clock(1), ArithExpression::Int(2));
        let right = ArithExpression::Int(3);
        //Testing: left < right
        assert_eq!(get_indices(&left, &right, &decl).ok(), Some((1, 0, 5)));

        let left = ArithExpression::ADif(ArithExpression::Int(1), ArithExpression::Clock(2));
        let right = ArithExpression::Int(3);
        //Testing: left < right
        assert_eq!(get_indices(&left, &right, &decl), Ok((0, 2, 2)));
    }

    #[test]
    fn test_get_indices_int_diff() {
        let decl = Declarations {
            clocks: HashMap::new(),
            ints: HashMap::new(),
        };
        // i-j < c -> c1-c2 > 3 -> c2-c1 < -3
        let left = ArithExpression::Int(3);
        let right = ArithExpression::ADif(ArithExpression::Clock(1), ArithExpression::Clock(2));

        //Testing: left < right
        assert_eq!(get_indices(&left, &right, &decl), Ok((2, 1, -3)));
    }

    #[test]
    fn test_get_indices_add_int() {
        let decl = Declarations {
            clocks: HashMap::new(),
            ints: HashMap::new(),
        };
        let left = ArithExpression::Addition(
            Box::new(ArithExpression::Clock(1)),
            Box::new(ArithExpression::Clock(2)),
        );
        let right = ArithExpression::Int(4);

        //Testing: left < right
        assert_eq!(get_indices(&left, &right, &decl).ok(), None);
    }

    #[test]
    fn test_get_indices_clock_diff_clock() {
        let decl = Declarations {
            clocks: HashMap::new(),
            ints: HashMap::new(),
        };
        let left = ArithExpression::Clock(1);
        let right = ArithExpression::ADif(ArithExpression::Clock(2), ArithExpression::Int(3));
        assert_eq!(get_indices(&left, &right, &decl), Ok((1, 2, -3)));

        let left = ArithExpression::ADif(ArithExpression::Clock(2), ArithExpression::Int(3));
        let right = ArithExpression::Clock(1);
        assert_eq!(get_indices(&left, &right, &decl), Ok((2, 1, 3)));

        let left = ArithExpression::ADif(ArithExpression::Int(2), ArithExpression::Clock(3));
        let right = ArithExpression::Clock(1);
        assert_eq!(get_indices(&left, &right, &decl).ok(), None);
    }

    #[test]
    fn test_get_indices_clock_add_clock() {
        let decl = Declarations {
            clocks: HashMap::new(),
            ints: HashMap::new(),
        };

        let left = ArithExpression::Addition(
            Box::new(ArithExpression::Clock(1)),
            Box::new(ArithExpression::Int(2)),
        );
        let right = ArithExpression::Addition(
            Box::new(ArithExpression::Clock(3)),
            Box::new(ArithExpression::Int(4)),
        );
        assert_eq!(get_indices(&left, &right, &decl), Ok((1, 3, 2)));

        let left = ArithExpression::Addition(
            Box::new(ArithExpression::Int(1)),
            Box::new(ArithExpression::Clock(2)),
        );
        let right = ArithExpression::Addition(
            Box::new(ArithExpression::Clock(3)),
            Box::new(ArithExpression::Int(4)),
        );
        assert_eq!(get_indices(&left, &right, &decl), Ok((2, 3, 3)));

        let left = ArithExpression::Addition(
            Box::new(ArithExpression::Clock(1)),
            Box::new(ArithExpression::Int(2)),
        );
        let right = ArithExpression::Addition(
            Box::new(ArithExpression::Int(3)),
            Box::new(ArithExpression::Clock(4)),
        );
        assert_eq!(get_indices(&left, &right, &decl), Ok((1, 4, 1)));

        let left = ArithExpression::Addition(
            Box::new(ArithExpression::Int(1)),
            Box::new(ArithExpression::Clock(2)),
        );
        let right = ArithExpression::Addition(
            Box::new(ArithExpression::Int(3)),
            Box::new(ArithExpression::Clock(4)),
        );
        assert_eq!(get_indices(&left, &right, &decl), Ok((2, 4, 2)));
    }

    #[test]
    fn test_get_indices_clock_int_diff() {
        let decl = Declarations {
            clocks: HashMap::new(),
            ints: HashMap::new(),
        };
        // i-j < c -> c1-c2 > 3 -> c2-c1 < -3
        let left = ArithExpression::ADif(ArithExpression::Clock(1), ArithExpression::Int(2));
        let right = ArithExpression::ADif(ArithExpression::Clock(3), ArithExpression::Int(4));
        //Testing: left < right
        assert_eq!(get_indices(&left, &right, &decl), Ok((1, 3, -2)));

        let left = ArithExpression::ADif(ArithExpression::Int(1), ArithExpression::Clock(2));
        let right = ArithExpression::ADif(ArithExpression::Int(3), ArithExpression::Clock(4));
        //Testing: left < right
        assert_eq!(get_indices(&left, &right, &decl), Ok((4, 2, 2)));
    }

    #[test]
    fn test_get_indices_clock_add_int() {
        let decl = Declarations {
            clocks: HashMap::new(),
            ints: HashMap::new(),
        };
        let left = ArithExpression::Clock(1);
        let right = ArithExpression::Addition(
            Box::new(ArithExpression::Clock(3)),
            Box::new(ArithExpression::Int(4)),
        );
        assert_eq!(get_indices(&left, &right, &decl), Ok((1, 3, 4)));

        let left = ArithExpression::Addition(
            Box::new(ArithExpression::Clock(3)),
            Box::new(ArithExpression::Int(4)),
        );
        let right = ArithExpression::Clock(1);
        assert_eq!(get_indices(&left, &right, &decl), Ok((3, 1, -4)));
    }

    #[test]
    fn test_get_indices_int_add() {
        let decl = Declarations {
            clocks: HashMap::new(),
            ints: HashMap::new(),
        };
        let left = ArithExpression::Int(3);
        let right = ArithExpression::Addition(
            Box::new(ArithExpression::Clock(1)),
            Box::new(ArithExpression::Clock(2)),
        );

        //Testing: left < right
        assert_eq!(get_indices(&left, &right, &decl).ok(), None);
    }

    #[test]
    fn test_get_indices_high_operators() {
        let decl = Declarations {
            clocks: HashMap::new(),
            ints: HashMap::new(),
        };
        let left = ArithExpression::Multiplication(
            Box::new(ArithExpression::Clock(2)),
            Box::new(ArithExpression::Int(3)),
        );
        let right = ArithExpression::Int(10);
        //Testing: left < right
        assert_eq!(get_indices(&left, &right, &decl).ok(), None);

        let left = ArithExpression::Multiplication(
            Box::new(ArithExpression::Int(3)),
            Box::new(ArithExpression::Clock(2)),
        );
        let right = ArithExpression::Int(10);
        //Testing: left < right
        assert_eq!(get_indices(&left, &right, &decl).ok(), None);

        let left = ArithExpression::Division(
            Box::new(ArithExpression::Clock(2)),
            Box::new(ArithExpression::Int(3)),
        );
        let right = ArithExpression::Int(10);
        //Testing: left < right
        assert_eq!(get_indices(&left, &right, &decl).ok(), None);

        let left = ArithExpression::Modulo(
            Box::new(ArithExpression::Clock(2)),
            Box::new(ArithExpression::Int(3)),
        );
        let right = ArithExpression::Int(10);
        //Testing: left < right
        assert_eq!(get_indices(&left, &right, &decl).ok(), None);

        let left = ArithExpression::Multiplication(
            Box::new(ArithExpression::Int(4)),
            Box::new(ArithExpression::Int(3)),
        );
        let right = ArithExpression::Clock(10);
        //Testing: left < right
        assert_eq!(get_indices(&left, &right, &decl), Ok((0, 10, -12)));

        let left = ArithExpression::Division(
            Box::new(ArithExpression::Int(4)),
            Box::new(ArithExpression::Int(2)),
        );
        let right = ArithExpression::Clock(10);
        //Testing: left < right
        assert_eq!(get_indices(&left, &right, &decl), Ok((0, 10, -2)));

        let left = ArithExpression::Modulo(
            Box::new(ArithExpression::Int(4)),
            Box::new(ArithExpression::Int(3)),
        );
        let right = ArithExpression::Clock(10);
        //Testing: left < right
        assert_eq!(get_indices(&left, &right, &decl), Ok((0, 10, -1)));
    }

    #[test]
    fn test_get_indices_many_clocks() {
        let decl = Declarations {
            clocks: HashMap::new(),
            ints: HashMap::new(),
        };
        let left = ArithExpression::ADif(ArithExpression::Clock(1), ArithExpression::Clock(2));
        let right = Box::new(ArithExpression::Clock(3));

        //Testing: left < right
        assert_eq!(get_indices(&left, &right, &decl).ok(), None);
    }

    #[test]
    fn test_get_indices_int_int() {
        let decl = Declarations {
            clocks: HashMap::new(),
            ints: HashMap::new(),
        };
        let left = ArithExpression::Int(1);
        let right = ArithExpression::Int(2);
        //Testing: left < right
        assert_eq!(
            get_indices(&left, &right, &decl),
            Err(
                "Expressions must use clocks to get indices, this should be unreachable"
                    .to_string()
            )
        );

        let left = ArithExpression::Addition(
            Box::new(ArithExpression::Int(1)),
            Box::new(ArithExpression::Int(2)),
        );
        let right = ArithExpression::Int(3);
        //Testing: left < right
        assert_eq!(
            get_indices(&left, &right, &decl),
            Err(
                "Expressions must use clocks to get indices, this should be unreachable"
                    .to_string()
            )
        );
    }

    #[test]
    fn test_get_indices_big_expr() {
        let decl = Declarations {
            clocks: HashMap::new(),
            ints: HashMap::new(),
        };
        let mut left = ArithExpression::ADif(
            // = 4
            ArithExpression::Int(10),
            ArithExpression::ADif(
                ArithExpression::Int(9),
                ArithExpression::ADif(
                    ArithExpression::Int(8),
                    ArithExpression::ADif(
                        ArithExpression::Int(7),
                        ArithExpression::ADif(
                            ArithExpression::Clock(6),
                            ArithExpression::ADif(
                                ArithExpression::Int(5),
                                ArithExpression::ADif(
                                    ArithExpression::Int(4),
                                    ArithExpression::ADif(
                                        ArithExpression::Int(3),
                                        ArithExpression::Int(2),
                                    ),
                                ),
                            ),
                        ),
                    ),
                ),
            ),
        );
        let right = ArithExpression::Int(2);
        //Testing: left < right
        assert_eq!(get_indices(&left, &right, &decl), Ok((6, 0, 6)));
    }

    #[test]
    fn test_get_indices_mix_expr() {
        let decl = Declarations {
            clocks: HashMap::new(),
            ints: HashMap::new(),
        };
        let mut left = ArithExpression::ADif(
            ArithExpression::Multiplication(
                Box::new(ArithExpression::Clock(3)),
                Box::new(ArithExpression::Int(3)),
            ),
            ArithExpression::Int(10),
        );
        let right = ArithExpression::Clock(10);
        //Testing: left < right
        assert_eq!(get_indices(&left, &right, &decl).ok(), None);

        let mut left = ArithExpression::ADif(
            ArithExpression::Multiplication(
                Box::new(ArithExpression::Int(3)),
                Box::new(ArithExpression::Int(3)),
            ),
            ArithExpression::Clock(10),
        );
        let right = ArithExpression::ADif(ArithExpression::Int(10), ArithExpression::Clock(10));
        //Testing: left < right
        assert_eq!(get_indices(&left, &right, &decl), Ok((10, 10, 1)));
    }
}
