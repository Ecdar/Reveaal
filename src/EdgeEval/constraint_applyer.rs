use crate::DBMLib::dbm::Federation;
use crate::ModelObjects::component;
use crate::ModelObjects::representations::BoolExpression;
use std::collections::HashMap;

pub fn apply_constraint(
    constraint: &Option<BoolExpression>,
    decls: &component::Declarations,
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
    decls: &component::Declarations,
    zone: &mut Federation,
) -> bool {
    apply_constraints_to_state_helper(guard, decls, zone)
}

pub fn apply_constraints_to_state_helper(
    guard: &BoolExpression,
    decls: &component::Declarations,
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
            let (i, j, c) = get_indices(left, right, decls, false);
            // i-j<=c
            zone.constrain(i, j, c, false)
        }
        BoolExpression::GreatEQ(left, right) => {
            let (i, j, c) = get_indices(left, right, decls, true);
            // j-i <= -c -> c <= i-j
            zone.constrain(i, j, c, false)
        }
        BoolExpression::EQ(left, right) => {
            let (i, j, c) = get_indices(left, right, decls, false);
            // i-j <= c && j-i <= -c -> c <= i-j
            zone.constrain(i, j, c, false) && zone.constrain(j, i, -c, false)
        }
        BoolExpression::LessT(left, right) => {
            let (i, j, c) = get_indices(left, right, decls, false);
            // i-j < c
            zone.constrain(i, j, c, true)
        }
        BoolExpression::GreatT(left, right) => {
            let (i, j, c) = get_indices(left, right, decls, true);
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
    let decl = component::Declarations {
        clocks: HashMap::new(),
        ints: HashMap::new(),
    };

    let left = BoolExpression::Int(3);
    let right = BoolExpression::Clock(1);

    //Testing: left < right
    assert_eq!(get_indices(&left, &right, &decl, false), (0, 1, -3));
    //Testing: right > left
    assert_eq!(get_indices(&right, &left, &decl, true), (0, 1, -3));
}

#[test]
fn test_get_indices_clock_int() {
    let decl = component::Declarations {
        clocks: HashMap::new(),
        ints: HashMap::new(),
    };

    let left = BoolExpression::Clock(1);
    let right = BoolExpression::Int(3);

    //Testing: left < right
    assert_eq!(get_indices(&left, &right, &decl, false), (1, 0, 3));
    //Testing: right > left
    assert_eq!(get_indices(&right, &left, &decl, true), (1, 0, 3));
}

#[test]
fn test_get_indices_clock_clock() {
    let decl = component::Declarations {
        clocks: HashMap::new(),
        ints: HashMap::new(),
    };

    // i-j <= 0 -> i can at most be the value of j
    let left = BoolExpression::Clock(1);
    let right = BoolExpression::Clock(2);

    //Testing: left < right
    assert_eq!(get_indices(&left, &right, &decl, false), (1, 2, 0));
    //Testing: right > left
    assert_eq!(get_indices(&right, &left, &decl, true), (1, 2, 0));
}

#[test]
fn test_get_indices_diff_int() {
    let decl = component::Declarations {
        clocks: HashMap::new(),
        ints: HashMap::new(),
    };
    // i-j < c -> c1-c2 < 3
    let left = BoolExpression::BDif(BoolExpression::Clock(1), BoolExpression::Clock(2));
    let right = BoolExpression::Int(3);

    //Testing: left < right
    assert_eq!(get_indices(&left, &right, &decl, false), (1, 2, 3));
    //Testing: right > left
    assert_eq!(get_indices(&right, &left, &decl, true), (1, 2, 3));
}

#[test]
fn test_get_indices_int_diff() {
    let decl = component::Declarations {
        clocks: HashMap::new(),
        ints: HashMap::new(),
    };
    // i-j < c -> c1-c2 > 3 -> c2-c1 < -3
    let left = BoolExpression::Int(3);
    let right = BoolExpression::BDif(BoolExpression::Clock(1), BoolExpression::Clock(2));

    //Testing: left < right
    assert_eq!(get_indices(&left, &right, &decl, false), (2, 1, -3));
    //Testing: right > left
    assert_eq!(get_indices(&right, &left, &decl, true), (2, 1, -3));
}

fn get_indices(
    left: &BoolExpression,
    right: &BoolExpression,
    d: &component::Declarations,
    swap: bool, //Swap if left > right as opposed to left < right
) -> (u32, u32, i32) {
    let (left, right) = if swap {
        // i-j ?> c -> j-i < c if swap==true
        (right, left)
    } else {
        // i-j <? c if swap==false
        (left, right)
    };

    let result = if let BoolExpression::Difference(i, j) = left {
        try_form_index(get_clock(i, d), get_clock(j, d), get_constant(right))
    } else if let Some(c) = get_constant(left) {
        if let BoolExpression::Difference(i, j) = right {
            try_form_index(get_clock(j, d), get_clock(i, d), Some(-c))
        } else {
            try_form_index(Some(0), get_clock(right, d), Some(-c))
        }
    } else if let Some(clock) = get_clock(left, d) {
        let i1 = try_form_index(Some(clock), Some(0), get_constant(right));
        let i2 = try_form_index(Some(clock), get_clock(right, d), Some(0));
        i1.or(i2)
    } else {
        None
    };

    result.unwrap()
}

fn try_form_index(i: Option<u32>, j: Option<u32>, c: Option<i32>) -> Option<(u32, u32, i32)> {
    if i.and(j).and(c).is_some() {
        let res = (i.unwrap(), j.unwrap(), c.unwrap());
        if res.0 == 0 && res.1 == 0 {
            return None;
        }

        Some(res)
    } else {
        None
    }
}

fn get_clock(expr: &BoolExpression, decls: &component::Declarations) -> Option<u32> {
    match expr {
        BoolExpression::Clock(id) => Some(*id),
        BoolExpression::VarName(name) => decls.get_clocks().get(name).and_then(|o| Some(*o)),
        _ => None,
    }
}

fn get_constant(expr: &BoolExpression, //, decls: &component::Declarations
) -> Option<i32> {
    match expr {
        BoolExpression::Int(i) => Some(*i),
        //TODO: when integer variables/constants are introduced
        //BoolExpression::VarName(name) => decls.get_ints().get(name).and_then(|o| Some(*o)),
        _ => None,
    }
}

pub fn apply_constraints_to_state2(
    guard: &BoolExpression,
    state: &mut component::State,
    comp_index: usize,
) -> bool {
    apply_constraints_to_state(
        guard,
        &state.get_declarations(comp_index).clone(),
        &mut state.zone,
    )
}
