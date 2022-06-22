//use core::panicking::panic;
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
    let decl = component::Declarations {
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
    let decl = component::Declarations {
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
    let decl = component::Declarations {
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
    let decl = component::Declarations {
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
    let decl = component::Declarations {
        clocks: HashMap::new(),
        ints: HashMap::new(),
    };
    // i-j < c -> c1-c2 > 3 -> c2-c1 < -3
    let left = BoolExpression::Int(3);
    let right = BoolExpression::BDif(BoolExpression::Clock(1), BoolExpression::Clock(2));

    //Testing: left < right
    assert_eq!(get_indices(&left, &right, &decl), (2, 1, -3));
}

#[derive(PartialEq)]
enum ClockConstOption {
    Clock(u32),
    Const(i32),
    None,
}

impl ClockConstOption {
    fn unwrap(&self) -> i32 {
        match self {
            ClockConstOption::Clock(c) => *c as i32,
            ClockConstOption::Const(a) => *a,
            ClockConstOption::None => panic!("Used unwrap on None"),
        }
    }
}

/// Assumes that the constraint is of the form left <?= right
fn get_indices(
    left: &BoolExpression,
    right: &BoolExpression,
    d: &component::Declarations,
) -> (u32, u32, i32) {
    let result = if let BoolExpression::Difference(i, j) = left {
        try_form_index(get_clock(i, d), get_clock(j, d), get_constant(right))
    } else if let ClockConstOption::Const(c) = get_constant(left) {
        if let BoolExpression::Difference(i, j) = right {
            let i1 = try_form_index(get_clock(j, d), get_clock(i, d), ClockConstOption::Const(-c));
            let i2 = try_form_index(get_constant(j), get_clock(i, d), ClockConstOption::Const(-c));
            let i3 = try_form_index(get_clock(j, d), get_constant(i), ClockConstOption::Const(-c));
            let i4 = try_form_index(get_constant(j), get_constant(i), ClockConstOption::Const(-c));
            i1.or(i2).or(i3).or(i4)
        } else {
            try_form_index(ClockConstOption::Const(0), get_clock(right, d), ClockConstOption::Const(-c))
        }
    } else if let ClockConstOption::Clock(clock) = get_clock(left, d) {
        let i1 = try_form_index(ClockConstOption::Clock(clock), ClockConstOption::Const(0), get_constant(right));
        let i2 = try_form_index(ClockConstOption::Clock(clock), get_clock(right, d), ClockConstOption::Const(0));
        let i3 = try_form_index(ClockConstOption::Clock(clock), ClockConstOption::Const(0), get_clock(right, d));
        let i4 = try_form_index(ClockConstOption::Clock(clock), get_constant(right), ClockConstOption::Const(0));
        i1.or(i2).or(i3).or(i4)
    } else {
        None
    };

    //TODO: Should probably not convert values
    match result {
        Some(res) => {
            if res.0 == ClockConstOption::None ||
                res.1 == ClockConstOption::None ||
                res.2 == ClockConstOption::None {
                panic!("TEMP");
            }

            (res.0.unwrap() as u32, res.1.unwrap() as u32, res.2.unwrap())
        },
        None => panic!("LOL"),
    }
    /*
    result.unwrap_or_else(|| {
        panic!(
            "Failed to get index from left: {:?} right: {:?} decls: {:?}",
            left, right, d
        )
    }) */
}

fn try_form_index(i: ClockConstOption, j: ClockConstOption, c: ClockConstOption) -> Option<(ClockConstOption, ClockConstOption, ClockConstOption)> {
    if i == ClockConstOption::None || j == ClockConstOption::None || c == ClockConstOption::None {
        None
    }
    else { // if let (Some(i), Some(j), Some(c)) = (i, j, c) {
        let res = (i, j, c);
        if res.0.unwrap() == 0 && res.1.unwrap() == 0 {
            return None;
        }

        Some(res)
    }
}

fn try_form_index_old(i: Option<u32>, j: Option<u32>, c: Option<i32>) -> Option<(u32, u32, i32)> {
    if let (Some(i), Some(j), Some(c)) = (i, j, c) {
        let res = (i, j, c);
        if res.0 == 0 && res.1 == 0 {
            return None;
        }

        Some(res)
    } else {
        None
    }
}

fn get_clock(expr: &BoolExpression, decls: &component::Declarations) -> ClockConstOption {
    match expr {
        BoolExpression::Clock(id) => ClockConstOption::Clock(*id),
        BoolExpression::VarName(name) => ClockConstOption::Clock(decls.get_clocks()
            .get(name)
            .and_then(|o| Some(*o))
            .unwrap()),
        _ => ClockConstOption::None,
    }
}

fn get_constant(expr: &BoolExpression, //, decls: &component::Declarations
) -> ClockConstOption {
    match expr {
        BoolExpression::Int(i) => ClockConstOption::Const(*i),
        //TODO: when integer variables/constants are introduced
        //BoolExpression::VarName(name) => decls.get_ints().get(name).and_then(|o| Some(*o)),
        _ => ClockConstOption::None,
    }
}
