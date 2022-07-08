use crate::DBMLib::dbm::Federation;
use crate::ModelObjects::component;
use crate::ModelObjects::representations::BoolExpression;
use std::collections::HashMap;
use std::convert::TryFrom;
use crate::component::Declarations;

pub fn apply_constraint(
    constraint: &Option<BoolExpression>,
    decls: &Declarations,
    zone: &mut Federation,
) -> bool {
    return if let Some(guards) = constraint {
        apply_constraints_to_state(guards, decls, zone)
    } else {
        true
    };
}

pub fn apply_constraints_to_state(
    guard: &BoolExpression,
    decls: &Declarations,
    zone: &mut Federation,
) -> bool {
    apply_constraints_to_state_helper(guard, decls, zone)
}

pub fn apply_constraints_to_state_helper(
    guard: &BoolExpression,
    decls: &Declarations,
    zone: &mut Federation,
) -> bool {
    match guard {
        BoolExpression::AndOp(left, right) => {
            apply_constraints_to_state_helper(left, decls, zone)
                && apply_constraints_to_state_helper(right, decls, zone)
        }
        BoolExpression::OrOp(left, right) => {
            let mut clone = zone.clone();
            let res1 = apply_constraints_to_state_helper(left, decls, zone);
            let res2 = apply_constraints_to_state_helper(right, decls, &mut clone);
            *zone += clone;
            res1 || res2
        }
        BoolExpression::LessEQ(left, right) => {
            let (i, j, c) = get_indices(left, right, decls);
            // i-j<=c
            zone.constrain(i, j, c, false)
        }
        BoolExpression::GreatEQ(left, right) => {
            let (i, j, c) = get_indices(right, left, decls);
            // j-i <= -c -> c <= i-j
            zone.constrain(i, j, c, false)
        }
        BoolExpression::EQ(left, right) => {
            let (i, j, c) = get_indices(left, right, decls);
            // i-j <= c && j-i <= -c -> c <= i-j
            zone.constrain(i, j, c, false) && zone.constrain(j, i, -c, false)
        }
        BoolExpression::LessT(left, right) => {
            let (i, j, c) = get_indices(left, right, decls);
            // i-j < c
            zone.constrain(i, j, c, true)
        }
        BoolExpression::GreatT(left, right) => {
            let (i, j, c) = get_indices(right, left, decls);
            // j-i < -c -> c < i-j
            zone.constrain(i, j, c, true)
        }
        BoolExpression::Parentheses(expr) => apply_constraints_to_state_helper(expr, decls, zone),
        BoolExpression::Bool(val) => {
            if !*val {
                *zone = Federation::empty(zone.get_dimensions());
            }
            *val
        }
        _ => {
            panic!("Unexpected BoolExpression")
        }
    }
}

#[test]
fn test_get_indices_int_clock() {
    let decl = Declarations {
        clocks: HashMap::new(),
        ints: HashMap::new(),
    };

    let left = BoolExpression::Int(3);
    let right = BoolExpression::Clock(1);

    //Testing: left < right
    assert_eq!(get_indices(&left, &right, &decl), (0, 1, -3));
}

#[test]
fn test_get_indices_clock_int() {
    let decl = Declarations {
        clocks: HashMap::new(),
        ints: HashMap::new(),
    };

    let left = BoolExpression::Clock(1);
    let right = BoolExpression::Int(3);

    //Testing: left < right
    assert_eq!(get_indices(&left, &right, &decl), (1, 0, 3));
}

#[test]
fn test_get_indices_clock_clock() {
    let decl = Declarations {
        clocks: HashMap::new(),
        ints: HashMap::new(),
    };

    // i-j <= 0 -> i can at most be the value of j
    let left = BoolExpression::Clock(1);
    let right = BoolExpression::Clock(2);

    //Testing: left < right
    assert_eq!(get_indices(&left, &right, &decl), (1, 2, 0));
}

#[test]
fn test_get_indices_diff_int() {
    let decl = Declarations {
        clocks: HashMap::new(),
        ints: HashMap::new(),
    };
    // i-j < c -> c1-c2 < 3
    let left = BoolExpression::BDif(BoolExpression::Clock(1), BoolExpression::Clock(2));
    let right = BoolExpression::Int(3);

    //Testing: left < right
    assert_eq!(get_indices(&left, &right, &decl), (1, 2, 3));
}

#[test]
fn test_get_indices_int_diff() {
    let decl = Declarations {
        clocks: HashMap::new(),
        ints: HashMap::new(),
    };
    // i-j < c -> c1-c2 > 3 -> c2-c1 < -3
    let left = BoolExpression::Int(3);
    let right = BoolExpression::BDif(BoolExpression::Clock(1), BoolExpression::Clock(2));

    //Testing: left < right
    assert_eq!(get_indices(&left, &right, &decl), (2, 1, -3));
}

#[test]
#[should_panic]
fn test_get_indices_add_int() {
    let decl = Declarations {
        clocks: HashMap::new(),
        ints: HashMap::new(),
    };
    let left = BoolExpression::Addition(Box::new(BoolExpression::Clock(1)), Box::new(BoolExpression::Clock(2)));
    let right = BoolExpression::Int(4);

    //Testing: left < right
    get_indices(&left, &right, &decl);
}

#[test]
fn test_get_indices_clock_diff_clock() {
    let decl = Declarations {
        clocks: HashMap::new(),
        ints: HashMap::new(),
    };
    let left = BoolExpression::Clock(1);
    let right = BoolExpression::BDif(BoolExpression::Clock(2), BoolExpression::Int(3));
    assert_eq!(get_indices(&left, &right, &decl), (1, 2, -3));
}

#[test]
fn test_get_indices_clock_add_clock() {
    let decl = Declarations {
        clocks: HashMap::new(),
        ints: HashMap::new(),
    };

    let left = BoolExpression::Addition(Box::new(BoolExpression::Clock(1)), Box::new(BoolExpression::Int(2)));
    let right = BoolExpression::Addition(Box::new(BoolExpression::Clock(3)), Box::new(BoolExpression::Int(4)));
    assert_eq!(get_indices(&left, &right, &decl), (1, 3, 2));

    let left = BoolExpression::Addition(Box::new(BoolExpression::Int(1)), Box::new(BoolExpression::Clock(2)));
    let right = BoolExpression::Addition(Box::new(BoolExpression::Clock(3)), Box::new(BoolExpression::Int(4)));
    assert_eq!(get_indices(&left, &right, &decl), (2, 3, 3));

    let left = BoolExpression::Addition(Box::new(BoolExpression::Clock(1)), Box::new(BoolExpression::Int(2)));
    let right = BoolExpression::Addition(Box::new(BoolExpression::Int(3)), Box::new(BoolExpression::Clock(4)));
    assert_eq!(get_indices(&left, &right, &decl), (1, 4, 1));

    let left = BoolExpression::Addition(Box::new(BoolExpression::Int(1)), Box::new(BoolExpression::Clock(2)));
    let right = BoolExpression::Addition(Box::new(BoolExpression::Int(3)), Box::new(BoolExpression::Clock(4)));
    assert_eq!(get_indices(&left, &right, &decl), (2, 4, 2));


}

#[test]
fn test_get_indices_clock_int_diff() {
    let decl = Declarations {
        clocks: HashMap::new(),
        ints: HashMap::new(),
    };
    // i-j < c -> c1-c2 > 3 -> c2-c1 < -3
    let left = BoolExpression::BDif(BoolExpression::Clock(1), BoolExpression::Int(2));
    let right = BoolExpression::BDif(BoolExpression::Clock(3), BoolExpression::Int(4));
    //Testing: left < right
    assert_eq!(get_indices(&left, &right, &decl), (1, 3, -2));

    let left = BoolExpression::BDif(BoolExpression::Int(1), BoolExpression::Clock(2));
    let right = BoolExpression::BDif(BoolExpression::Int(3), BoolExpression::Clock(4));
    //Testing: left < right
    assert_eq!(get_indices(&left, &right, &decl), (4, 2, 2));
}

#[test]
fn test_get_indices_clock_add_int() {
    let decl = Declarations {
        clocks: HashMap::new(),
        ints: HashMap::new(),
    };
    let left = BoolExpression::Clock(1);
    let right = BoolExpression::Addition(Box::new(BoolExpression::Clock(3)), Box::new(BoolExpression::Int(4)));
    assert_eq!(get_indices(&left, &right, &decl), (1, 3, 4));

    let left = BoolExpression::Addition(Box::new(BoolExpression::Clock(3)), Box::new(BoolExpression::Int(4)));
    let right = BoolExpression::Clock(1);
    assert_eq!(get_indices(&left, &right, &decl), (3, 1, -4));
}

#[test]
#[should_panic]
fn test_get_indices_int_add() {
    let decl = Declarations {
        clocks: HashMap::new(),
        ints: HashMap::new(),
    };
    let left = BoolExpression::Int(3);
    let right = BoolExpression::Addition(Box::new(BoolExpression::Clock(1)), Box::new(BoolExpression::Clock(2)));

    //Testing: left < right
    get_indices(&left, &right, &decl);
}

#[test]
fn test_get_indices_mult_int() {
    let decl = Declarations {
        clocks: HashMap::new(),
        ints: HashMap::new(),
    };
    let left = BoolExpression::Multiplication(Box::new(BoolExpression::Clock(2)), Box::new(BoolExpression::Int(3)));
    let right = BoolExpression::Int(10);
    //Testing: left < right
    assert_eq!(get_indices(&left, &right, &decl), (6, 0, 10));

    let left = BoolExpression::Multiplication(Box::new(BoolExpression::Int(3)), Box::new(BoolExpression::Clock(2)));
    let right = BoolExpression::Int(10);
    //Testing: left < right
    assert_eq!(get_indices(&left, &right, &decl), (6, 0, 10));

}

#[test]
#[should_panic]
fn test_get_indices_many_clocks() {
    let decl = Declarations {
        clocks: HashMap::new(),
        ints: HashMap::new(),
    };
    let left = BoolExpression::BDif(BoolExpression::Clock(1), BoolExpression::Clock(2));
    let right = Box::new(BoolExpression::Clock(3));

    //Testing: left < right
    get_indices(&left, &right, &decl);
}

#[test]
fn test_get_indices_int_int() {
    let decl = Declarations {
        clocks: HashMap::new(),
        ints: HashMap::new(),
    };
    let left = BoolExpression::Int(1);
    let right = BoolExpression::Int(2);
    //Testing: left < right
    assert_eq!(get_indices(&left, &right, &decl), (1, 0, 2));

    let left = BoolExpression::Int(1);
    let right = BoolExpression::Addition(Box::new(BoolExpression::Int(2)), Box::new(BoolExpression::Int(3)));
    //Testing: left < right
    assert_eq!(get_indices(&left, &right, &decl), (1, 0, 5));

    let left = BoolExpression::Addition(Box::new(BoolExpression::Int(1)), Box::new(BoolExpression::Int(2)));
    let right = BoolExpression::Int(3);
    //Testing: left < right
    assert_eq!(get_indices(&left, &right, &decl), (3, 0, 3));
}


#[test]
#[ignore]
fn test_get_indices_big_expr() {
    let decl = Declarations {
        clocks: HashMap::new(),
        ints: HashMap::new(),
    };
    let mut left = BoolExpression::BDif(BoolExpression::Int(10),
                                BoolExpression::BDif(BoolExpression::Int(9),
                                BoolExpression::BDif(BoolExpression::Int(8),
                                BoolExpression::BDif(BoolExpression::Int(7),
                                BoolExpression::BDif(BoolExpression::Clock(6),
                                BoolExpression::BDif(BoolExpression::Int(5),
                                BoolExpression::BDif(BoolExpression::Int(4),
                                BoolExpression::BDif(BoolExpression::Int(3),
                                BoolExpression::Int(2)))))))));
    let right = BoolExpression::Int(2);
    //left.simplify();
    //assert_eq!(left, BoolExpression::Int(-34));
    //Testing: left < right
    assert_eq!(get_indices(&left, &right, &decl), (6, 0, 20));
}

#[test]
fn test_get_indices_mix_expr() {
    let decl = Declarations {
        clocks: HashMap::new(),
        ints: HashMap::new(),
    };
    let mut left = BoolExpression::BDif(BoolExpression::Multiplication(
        Box::new(BoolExpression::Clock(3)),
        Box::new(BoolExpression::Int(3)),
    ), BoolExpression::Int(10));
    let right = BoolExpression::Clock(10);
    //left.simplify();
    //assert_eq!(left, BoolExpression::Int(-34));
    //Testing: left < right
    assert_eq!(get_indices(&left, &right, &decl), (9, 10, 10));
}

fn get_const(expr: &BoolExpression, decls: &Declarations) -> i32 {
    match expr {
        BoolExpression::Int(x) => *x,
        BoolExpression::Clock(_) => 0,
        BoolExpression::VarName(name) => decls.get_ints().get(name).and_then(|o| Some(*o)).unwrap_or(0),
        BoolExpression::Parentheses(x) => get_const(x, decls),
        BoolExpression::Difference(l, r) => get_const(l, decls) - get_const(r, decls),
        BoolExpression::Addition(l, r) => get_const(l, decls) + get_const(r, decls),
        BoolExpression::Multiplication(l, r) => get_const(l, decls) * get_const(r, decls),
        BoolExpression::Division(l, r) => get_const(l, decls) / get_const(r, decls),
        BoolExpression::Modulo(l, r) => get_const(l, decls) % get_const(r, decls),
        _ => 0,
    }
}

struct Clock { value: u32, negated: bool }

fn get_clock_val(expression: &BoolExpression, decls: &Declarations, found: Option<(&BoolExpression, Option<&String>)>) -> Clock {
    let mut parents: Vec<&BoolExpression> = vec![];

    let mut cur_expr: Option<&BoolExpression> = Some(expression);
    let mut value: Option<u32> = None;
    let mut neg: bool = false;
    let mut go_left: bool = true;
    while let Some(e) = cur_expr {
        match e {
            BoolExpression::Clock(x) => {
                value = if let Some(y) = found {
                    if y.0 == e { None } else { Some(*x) }
                } else { Some(*x) };
                cur_expr = parents.pop();
                go_left = false;
            }
            BoolExpression::Int(_) => { cur_expr = parents.pop(); go_left = false; },
            BoolExpression::VarName(name) => {
                value = if let Some(y) = found {
                    if y.1.unwrap_or(&String::new()) == name { None } else { decls.get_clocks().get(name).and_then(|o| Some(*o)) }
                } else { decls.get_clocks().get(name).and_then(|o| Some(*o)) };
                cur_expr = parents.pop();
            },
            BoolExpression::Difference(l, r) => {
                if let Some(_) = value {
                    break;
                }
                if clock_count(l, decls) > 0 && go_left {
                    parents.push(e);
                    cur_expr = Some(l);
                } else if clock_count(r, decls) > 0 {
                    parents.push(e);
                    cur_expr = Some(r);
                    neg = true;
                    go_left = true;
                }
            },
            BoolExpression::Addition(l, r) => {
                if let Some(_) = value {
                    break;
                }
                if clock_count(l, decls) > 0 && go_left {
                    parents.push(e);
                    cur_expr = Some(l);
                } else if clock_count(r, decls) > 0 {
                    parents.push(e);
                    cur_expr = Some(r);
                    neg = false;
                    go_left = true;
                }
            },
            BoolExpression::Multiplication(l, r) => {
                if value.is_some() {
                    value = Some(value.unwrap() * if clock_count(l, decls) >= 1 {
                        get_const(r, decls) as u32
                    } else {
                        get_const(l, decls) as u32
                    });
                    break;
                }
                if clock_count(l, decls) > 0 && go_left {
                    parents.push(e);
                    cur_expr = Some(l);
                } else if clock_count(r, decls) > 0 {
                    parents.push(e);
                    cur_expr = Some(r);
                    neg = false;
                    go_left = true;
                }
            },
            BoolExpression::Division(l, r) => {
                if value.is_some() {
                    value = Some(value.unwrap() / if clock_count(l, decls) >= 1 {
                        get_const(r, decls) as u32
                    } else {
                        get_const(l, decls) as u32
                    });
                    break;
                }
                if clock_count(l, decls) > 0 && go_left {
                    parents.push(e);
                    cur_expr = Some(l);
                } else if clock_count(r, decls) > 0 {
                    parents.push(e);
                    cur_expr = Some(r);
                    neg = false;
                    go_left = true;
                }
            },
            BoolExpression::Modulo(l, r) => {
                if value.is_some() {
                    value = Some(value.unwrap() % if clock_count(l, decls) >= 1 {
                        get_const(r, decls) as u32
                    } else {
                        get_const(l, decls) as u32
                    });
                    break;
                }
                if clock_count(l, decls) > 0 && go_left {
                    parents.push(e);
                    cur_expr = Some(l);
                } else if clock_count(r, decls) > 0 {
                    parents.push(e);
                    cur_expr = Some(r);
                    neg = false;
                    go_left = true;
                }
            },
            _ => panic!("lol"),
        };
    }

    Clock { value: value.unwrap_or_else(|| panic!("gamer")), negated: neg }
}

fn find_clock<'a>(expression: &'a BoolExpression, decls: &'a Declarations) -> Option<(&'a BoolExpression, Option<&'a String>)> {
    let mut parents: Vec<&BoolExpression> = vec![];
    let mut cur_expr: Option<&BoolExpression> = Some(expression);
    let mut go_right: bool = false;
    let mut out: Option<(&BoolExpression, Option<&String>)> = None;
    while let Some(e) = cur_expr {
        match e {
            BoolExpression::Clock(_) => {
                if out.is_none() {
                    out = Some((e, None));
                } else {
                    out.unwrap().0 = e;
                }
                break;
            },
            BoolExpression::Int(_) => (),
            BoolExpression::VarName(name) => {
                decls.get_clocks().get(name).and_then(|o| Some(*o));
                if decls.get_clocks().contains_key(name) {
                    if out.is_none() {
                        out = Some((e, Some(name)));
                    } else {
                        out.unwrap().1 = Some(name);
                    }
                    break;
                }
            },
            BoolExpression::Difference(l, r) => {
                if clock_count(l, decls) > 0 && !go_right {
                    parents.push(e);
                    cur_expr = Some(l);
                } else if clock_count(r, decls) > 0 && go_right {
                    parents.push(e);
                    cur_expr = Some(r);
                    go_right = false;
                } else {
                    cur_expr = parents.pop();
                }
            },
            BoolExpression::Addition(l, r) => {
                if clock_count(l, decls) > 0 && !go_right {
                    parents.push(e);
                    cur_expr = Some(l);
                } else if clock_count(r, decls) > 0 && go_right {
                    parents.push(e);
                    cur_expr = Some(r);
                    go_right = false;
                } else {
                    cur_expr = parents.pop();
                }
            },
            _ => panic!("lol"),
        };
    };
    out
}

/// Assumes that the constraint is of the form left <?= right
fn get_indices(
    left: &BoolExpression,
    right: &BoolExpression,
    d: &Declarations,
) -> (u32, u32, i32) {
    let clocks_left = clock_count(left, d);
    let clocks_right = clock_count(right, d);

    let constant = get_const(right, d) - get_const(left, d);

    let result: Result<(u32, u32, i32), String> = if clocks_left + clocks_right > 2 {
        Err(String::from("temp - too many clocks"))
    } else if clocks_left + clocks_right == 2 {
        if clocks_left == 1 {
            let l = get_clock_val(left, d, None);
            let r = get_clock_val(right, d, None);
            if l.negated != r.negated {
                Err(String::from("Same sign"))
            } else if l.negated == true {
                Ok((r.value, l.value, constant))
            } else {
                Ok((l.value, r.value, constant))
            }
        } else if clocks_left == 2 {
            let v1 = get_clock_val(left, d, None);
            let v2 = get_clock_val(left, d, find_clock(left, d));
            if v1.negated == v2.negated {
                Err(String::from("Same sign"))
            } else if v1.negated == true {
                Ok((v2.value, v1.value, constant))
            } else {
                Ok((v1.value, v2.value, constant))
            }
        } else {
            let v1 = get_clock_val(right, d, None);
            let v2 = get_clock_val(right, d, find_clock(right, d));
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
            let v = get_clock_val(left, d, None);
            if v.negated {
                Ok((0, v.value, constant))
            } else {
                Ok((v.value, 0, constant))
            }
        } else if clocks_right == 1 {
            let v = get_clock_val(right, d, None);
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

    if let Some(x) = result.as_ref().ok() {
        *x
    } else {
        panic!("Failed due to error: \"{}\"", result.err().unwrap())
    }
}

fn clock_count(expr: &BoolExpression, decls: &Declarations) -> i32 {
    match expr {
        BoolExpression::Clock(_) => 1,
        BoolExpression::VarName(name) =>
            if let Some(_) = decls.get_clocks().get(name) {
                1
            } else {
                0
            },
        BoolExpression::Difference(l, r) =>
            clock_count(l, decls) + clock_count(r, decls),
        BoolExpression::Addition(l, r) =>
            clock_count(l, decls) + clock_count(r, decls),
        BoolExpression::Multiplication(l, r) =>
            clock_count(l, decls) + clock_count(r, decls),
        BoolExpression::Division(l, r) =>
            clock_count(l, decls) + clock_count(r, decls),
        BoolExpression::Modulo(l, r) =>
            clock_count(l, decls) + clock_count(r, decls),
        _ => 0,
    }
}
