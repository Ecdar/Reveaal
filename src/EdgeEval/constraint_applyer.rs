use crate::component::Declarations;
use crate::DBMLib::dbm::Federation;
use crate::ModelObjects::component;
use crate::ModelObjects::representations::{ArithExpression, BoolExpression};
use std::collections::HashMap;
use std::convert::TryFrom;

pub fn apply_constraint(
    constraint: &Option<BoolExpression>,
    decls: &Declarations,
    zone: &mut Federation,
) -> bool {
    return if let Some(guards) = constraint {
        match apply_constraints_to_state(guards, decls, zone) {
            Ok(x) => x,
            Err(e) => panic!("Error due to: {}", e), //TODO: Fix/Remove panic
        }
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
    assert_eq!(get_indices(&left, &right, &decl).ok(), Some((0, 2, 2)));
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
    assert_eq!(get_indices(&left, &right, &decl).ok(), Some((2, 1, -3)));
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
    assert_eq!(get_indices(&left, &right, &decl).ok(), Some((1, 2, -3)));

    let left = ArithExpression::ADif(ArithExpression::Clock(2), ArithExpression::Int(3));
    let right = ArithExpression::Clock(1);
    assert_eq!(get_indices(&left, &right, &decl).ok(), Some((2, 1, 3)));

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
    assert_eq!(get_indices(&left, &right, &decl).ok(), Some((1, 3, 2)));

    let left = ArithExpression::Addition(
        Box::new(ArithExpression::Int(1)),
        Box::new(ArithExpression::Clock(2)),
    );
    let right = ArithExpression::Addition(
        Box::new(ArithExpression::Clock(3)),
        Box::new(ArithExpression::Int(4)),
    );
    assert_eq!(get_indices(&left, &right, &decl).ok(), Some((2, 3, 3)));

    let left = ArithExpression::Addition(
        Box::new(ArithExpression::Clock(1)),
        Box::new(ArithExpression::Int(2)),
    );
    let right = ArithExpression::Addition(
        Box::new(ArithExpression::Int(3)),
        Box::new(ArithExpression::Clock(4)),
    );
    assert_eq!(get_indices(&left, &right, &decl).ok(), Some((1, 4, 1)));

    let left = ArithExpression::Addition(
        Box::new(ArithExpression::Int(1)),
        Box::new(ArithExpression::Clock(2)),
    );
    let right = ArithExpression::Addition(
        Box::new(ArithExpression::Int(3)),
        Box::new(ArithExpression::Clock(4)),
    );
    assert_eq!(get_indices(&left, &right, &decl).ok(), Some((2, 4, 2)));
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
    assert_eq!(get_indices(&left, &right, &decl).ok(), Some((1, 3, -2)));

    let left = ArithExpression::ADif(ArithExpression::Int(1), ArithExpression::Clock(2));
    let right = ArithExpression::ADif(ArithExpression::Int(3), ArithExpression::Clock(4));
    //Testing: left < right
    assert_eq!(get_indices(&left, &right, &decl).ok(), Some((4, 2, 2)));
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
    assert_eq!(get_indices(&left, &right, &decl).ok(), Some((1, 3, 4)));

    let left = ArithExpression::Addition(
        Box::new(ArithExpression::Clock(3)),
        Box::new(ArithExpression::Int(4)),
    );
    let right = ArithExpression::Clock(1);
    assert_eq!(get_indices(&left, &right, &decl).ok(), Some((3, 1, -4)));
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
    assert_eq!(get_indices(&left, &right, &decl).ok(), Some((0, 10, -12)));

    let left = ArithExpression::Division(
        Box::new(ArithExpression::Int(4)),
        Box::new(ArithExpression::Int(2)),
    );
    let right = ArithExpression::Clock(10);
    //Testing: left < right
    assert_eq!(get_indices(&left, &right, &decl).ok(), Some((0, 10, -2)));

    let left = ArithExpression::Modulo(
        Box::new(ArithExpression::Int(4)),
        Box::new(ArithExpression::Int(3)),
    );
    let right = ArithExpression::Clock(10);
    //Testing: left < right
    assert_eq!(get_indices(&left, &right, &decl).ok(), Some((0, 10, -1)));
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
    assert_eq!(get_indices(&left, &right, &decl).ok(), Some((1, 0, 2)));

    let left = ArithExpression::Int(1);
    let right = ArithExpression::Addition(
        Box::new(ArithExpression::Int(2)),
        Box::new(ArithExpression::Int(3)),
    );
    //Testing: left < right
    assert_eq!(get_indices(&left, &right, &decl).ok(), Some((1, 0, 5)));

    let left = ArithExpression::Addition(
        Box::new(ArithExpression::Int(1)),
        Box::new(ArithExpression::Int(2)),
    );
    let right = ArithExpression::Int(3);
    //Testing: left < right
    assert_eq!(get_indices(&left, &right, &decl).ok(), Some((3, 0, 3)));
}

#[test] //TODO: Shouldn't panic, will update to fix expression
#[should_panic]
fn test_get_indices_big_expr() {
    let decl = Declarations {
        clocks: HashMap::new(),
        ints: HashMap::new(),
    };
    let mut left = ArithExpression::ADif(
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
    assert_eq!(get_indices(&left, &right, &decl).ok(), None);
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
    assert_eq!(get_indices(&left, &right, &decl).ok(), Some((10, 10, 1)));
}

/// Assumes that the constraint is of the form left <?= right
fn get_indices(
    left: &ArithExpression,
    right: &ArithExpression,
    d: &Declarations,
) -> Result<(u32, u32, i32), String> {
    let clocks_left = clock_count(left, d);
    let clocks_right = clock_count(right, d);

    let constant = get_const(right, d) - get_const(left, d);

    let result: Result<(u32, u32, i32), String> = if clocks_left + clocks_right > 2 {
        Err(String::from("Too many clocks"))
    } else if clocks_left + clocks_right == 2 {
        if clocks_left == 1 {
            let (l, _) = get_clock_val(left, d)?;
            let (r, _) = get_clock_val(right, d)?;
            if l.negated != r.negated {
                Err(String::from("Same sign"))
            } else if l.negated == true {
                Ok((r.value, l.value, constant))
            } else {
                Ok((l.value, r.value, constant))
            }
        } else if clocks_left == 2 {
            let (v1, v2) = get_clock_val(left, d)?;
            let v2 = v2.unwrap();
            if v1.negated == v2.negated {
                Err(String::from("Same sign"))
            } else if v1.negated == true {
                Ok((v2.value, v1.value, constant))
            } else {
                Ok((v1.value, v2.value, constant))
            }
        } else {
            let (v1, v2) = get_clock_val(right, d)?;
            let v2 = v2.unwrap();
            if v1.negated == v2.negated {
                Err(String::from("Same sign"))
            } else if v1.negated == true {
                Ok((v1.value, v2.value, constant))
            } else {
                Ok((v2.value, v1.value, constant))
            }
        }
    } else {
        if clocks_left == 1 {
            let (v, _) = get_clock_val(left, d)?;
            if v.negated {
                Ok((0, v.value, constant))
            } else {
                Ok((v.value, 0, constant))
            }
        } else if clocks_right == 1 {
            let (v, _) = get_clock_val(right, d)?;
            if !v.negated {
                Ok((0, v.value, constant))
            } else {
                Ok((v.value, 0, constant))
            }
        } else {
            let lhs = get_const(left, d);
            if lhs.is_negative() {
                Ok((0, (-lhs) as u32, get_const(right, d)))
            } else {
                Ok((lhs as u32, 0, get_const(right, d)))
            }
        }
    };

    result
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

struct Clock {
    value: u32,
    negated: bool,
}

fn get_clock_val(
    expression: &ArithExpression,
    decls: &Declarations,
) -> Result<(Clock, Option<Clock>), String> {
    let mut parents: Vec<&ArithExpression> = vec![];

    let total_count = clock_count(expression, decls);
    let mut cur_expr: Option<&ArithExpression> = Some(expression);
    let mut new_val: Option<Clock> = None;
    let mut neg: bool = false;
    let mut go_right: bool = false;
    while let Some(e) = cur_expr {
        match e {
            ArithExpression::Clock(x) => {
                if let Some(y) = new_val {
                    return Ok((
                        y,
                        Some(Clock {
                            value: *x,
                            negated: neg,
                        }),
                    ));
                } else if total_count == 1 {
                    return Ok((
                        Clock {
                            value: *x,
                            negated: neg,
                        },
                        None,
                    ));
                } else {
                    new_val = Some(Clock {
                        value: *x,
                        negated: neg,
                    });
                    cur_expr = parents.pop();
                    go_right = true;
                }
            }
            ArithExpression::Int(_) => {
                cur_expr = parents.pop();
            }
            ArithExpression::VarName(name) => {
                if let Some(x) = decls.get_clocks().get(name).and_then(|o| Some(*o)) {
                    if let Some(y) = new_val {
                        return Ok((
                            y,
                            Some(Clock {
                                value: x,
                                negated: neg,
                            }),
                        ));
                    } else if total_count == 1 {
                        return Ok((
                            Clock {
                                value: x,
                                negated: neg,
                            },
                            None,
                        ));
                    } else {
                        new_val = Some(Clock {
                            value: x,
                            negated: neg,
                        });
                        cur_expr = parents.pop();
                        go_right = true;
                    }
                } else {
                    cur_expr = parents.pop();
                }
            }
            ArithExpression::Difference(l, r) => {
                let left = clock_count(l, decls);
                let right = clock_count(r, decls);
                if left == 2 {
                    parents.push(e);
                    cur_expr = Some(l);
                    neg = false;
                } else if right == 2 || (right == 1 && go_right) || left == 0 {
                    parents.push(e);
                    cur_expr = Some(r);
                    neg = true;
                } else {
                    parents.push(e);
                    cur_expr = Some(l);
                    neg = false;
                }
            }
            ArithExpression::Addition(l, r) => {
                let left = clock_count(l, decls);
                let right = clock_count(r, decls);
                if left == 2 {
                    parents.push(e);
                    cur_expr = Some(l);
                    neg = false;
                } else if right == 2 || (right == 1 && go_right) || left == 0 {
                    parents.push(e);
                    cur_expr = Some(r);
                    neg = false;
                } else {
                    parents.push(e);
                    cur_expr = Some(l);
                    neg = false;
                }
            }
            ArithExpression::Multiplication(l, r) => {
                return Err(format!("Multiplication with clock is illegal"));
                if new_val.is_some() {
                    return Err(format!("Multiplication with clock is illegal"));
                }
                let left = clock_count(l, decls);
                let right = clock_count(r, decls);
                if left == 2 {
                    parents.push(e);
                    cur_expr = Some(l);
                    neg = false;
                } else if right == 2 || (right == 1 && go_right) {
                    parents.push(e);
                    cur_expr = Some(r);
                    neg = false;
                } else {
                    parents.push(e);
                    cur_expr = Some(l);
                    neg = false;
                }
            }
            ArithExpression::Division(l, r) => {
                return Err(format!("Division with clock is illegal"));
                if new_val.is_some() {
                    return Err(format!("Division with clock is illegal"));
                }
                let left = clock_count(l, decls);
                let right = clock_count(r, decls);
                if left == 2 {
                    parents.push(e);
                    cur_expr = Some(l);
                    neg = false;
                } else if right == 2 || (right == 1 && go_right) {
                    parents.push(e);
                    cur_expr = Some(r);
                    neg = false;
                } else {
                    parents.push(e);
                    cur_expr = Some(l);
                    neg = false;
                }
            }
            ArithExpression::Modulo(l, r) => {
                return Err(format!("Modulo with clock is illegal"));
                if new_val.is_some() {
                    return Err(format!("Modulo with clock is illegal"));
                }
                let left = clock_count(l, decls);
                let right = clock_count(r, decls);
                if left == 2 {
                    parents.push(e);
                    cur_expr = Some(l);
                    neg = false;
                } else if right == 2 || (right == 1 && go_right) {
                    parents.push(e);
                    cur_expr = Some(r);
                    neg = false;
                } else {
                    parents.push(e);
                    cur_expr = Some(l);
                    neg = false;
                }
            }
            _ => panic!("lol"),
        };
    }
    Ok((new_val.unwrap(), None))
}

fn clock_count(expr: &ArithExpression, decls: &Declarations) -> i32 {
    match expr {
        ArithExpression::Clock(_) => 1,
        ArithExpression::VarName(name) => {
            if let Some(_) = decls.get_clocks().get(name) {
                1
            } else {
                0
            }
        }
        ArithExpression::Difference(l, r) => clock_count(l, decls) + clock_count(r, decls),
        ArithExpression::Addition(l, r) => clock_count(l, decls) + clock_count(r, decls),
        ArithExpression::Multiplication(l, r) => clock_count(l, decls) + clock_count(r, decls),
        ArithExpression::Division(l, r) => clock_count(l, decls) + clock_count(r, decls),
        ArithExpression::Modulo(l, r) => clock_count(l, decls) + clock_count(r, decls),
        _ => 0,
    }
}
