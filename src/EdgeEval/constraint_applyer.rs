use crate::DBMLib::dbm::{Federation, Zone};
use crate::ModelObjects::component;
use crate::ModelObjects::representations::BoolExpression;

pub fn apply_constraints_to_state(
    guard: &BoolExpression,
    state: &mut component::State,
    comp_index: usize,
) -> bool {
    apply_constraints_to_federation(
        guard,
        state.decorated_locations.get_decl(comp_index),
        &mut state.federation,
    )
}

pub fn apply_constraints_to_federation(
    guard: &BoolExpression,
    decls: &component::Declarations,
    fed: &mut Federation,
) -> bool {
    if let BoolExpression::Bool(val) = apply_constraints_to_fed_helper(guard, decls, fed, true).0 {
        val
    } else {
        panic!("unexpected value returned when attempting to apply constraints to federation")
    }
}

pub fn apply_constraints_to_zone(
    guard: &BoolExpression,
    decls: &component::Declarations,
    zone: &mut Zone,
) -> bool {
    if let BoolExpression::Bool(val) = apply_constraints_to_state_helper(guard, decls, zone, true).0
    {
        val
    } else {
        panic!("unexpected value returned when attempting to apply constraints to zone")
    }
}

pub fn apply_constraints_to_state_helper(
    guard: &BoolExpression,
    decls: &component::Declarations,
    zone: &mut Zone,
    should_apply: bool,
) -> (BoolExpression, bool) {
    match guard {
        BoolExpression::AndOp(left, right) => {
            let (left, _contains_clock_left) =
                apply_constraints_to_state_helper(&**left, decls, zone, true);
            if let BoolExpression::Bool(val) = left {
                if !val {
                    return (BoolExpression::Bool(false), false);
                }
            }
            let (right, _contains_clock_right) =
                apply_constraints_to_state_helper(&**right, decls, zone, true);

            match left {
                BoolExpression::Bool(left_val) => match right {
                    BoolExpression::Bool(right_val) => {
                        (BoolExpression::Bool(left_val && right_val), false)
                    }
                    _ => {
                        panic!("expected bool in apply guard && expression")
                    }
                },
                _ => {
                    panic!("expected bool in apply guard && expression")
                }
            }
        }
        BoolExpression::OrOp(left, right) => {
            let (mut left, contains_clock_left) =
                apply_constraints_to_state_helper(&**left, decls, zone, false);
            let (mut right, contains_clock_right) =
                apply_constraints_to_state_helper(&**right, decls, zone, false);

            if contains_clock_left && contains_clock_right {
                panic!("clock constrained on both sides of or operator, resulting in state that is not well defined")
            }

            if contains_clock_left {
                left = apply_constraints_to_state_helper(&left, decls, zone, true).0;
            } else if contains_clock_right {
                right = apply_constraints_to_state_helper(&right, decls, zone, true).0;
            }
            match left {
                BoolExpression::Bool(left_val) => match right {
                    BoolExpression::Bool(right_val) => {
                        (BoolExpression::Bool(left_val || right_val), false)
                    }
                    _ => {
                        panic!("expected bool in apply guard || expression")
                    }
                },
                _ => {
                    panic!("expected bool in apply guard || expression")
                }
            }
        }
        BoolExpression::LessEQ(left, right) => {
            let (computed_left, contains_clock_left) =
                apply_constraints_to_state_helper(&**left, decls, zone, false);
            let (computed_right, contains_clock_right) =
                apply_constraints_to_state_helper(&**right, decls, zone, false);

            if !should_apply && (contains_clock_right || contains_clock_left) {
                return (BoolExpression::LessEQ(left.clone(), right.clone()), true);
            }
            match computed_left {
                BoolExpression::Clock(left_index) => match computed_right {
                    BoolExpression::Clock(right_index) => {
                        let result = zone.add_lte_constraint(left_index, right_index, 0);

                        println!("DBM: {}", zone);
                        (BoolExpression::Bool(result), false)
                    }
                    BoolExpression::Int(right_val) => {
                        let result = zone.add_lte_constraint(left_index, 0, right_val);
                        (BoolExpression::Bool(result), false)
                    }
                    _ => {
                        panic!("invalid type in LEQ expression in guard")
                    }
                },
                BoolExpression::Int(left_val) => match computed_right {
                    BoolExpression::Clock(right_index) => {
                        let result = zone.add_lte_constraint(0, right_index, -left_val);
                        (BoolExpression::Bool(result), false)
                    }
                    BoolExpression::Int(right_val) => {
                        (BoolExpression::Bool(left_val <= right_val), false)
                    }
                    _ => {
                        panic!("invalid type in LEQ expression in guard")
                    }
                },
                _ => {
                    panic!("invalid type in LEQ expression in guard")
                }
            }
        }
        BoolExpression::GreatEQ(left, right) => {
            let (computed_left, contains_clock_left) =
                apply_constraints_to_state_helper(&**left, decls, zone, false);
            let (computed_right, contains_clock_right) =
                apply_constraints_to_state_helper(&**right, decls, zone, false);

            if !should_apply && (contains_clock_right || contains_clock_left) {
                return (BoolExpression::GreatEQ(left.clone(), right.clone()), true);
            }
            match computed_left {
                BoolExpression::Clock(left_index) => match computed_right {
                    BoolExpression::Clock(right_index) => {
                        let result = zone.add_lte_constraint(right_index, left_index, 0);

                        (BoolExpression::Bool(result), false)
                    }
                    BoolExpression::Int(right_val) => {
                        let result = zone.add_lte_constraint(0, left_index, -right_val);
                        (BoolExpression::Bool(result), false)
                    }
                    _ => {
                        panic!("invalid type in LEQ expression in guard")
                    }
                },
                BoolExpression::Int(left_val) => match computed_right {
                    BoolExpression::Clock(right_index) => {
                        let result = zone.add_lte_constraint(right_index, 0, left_val);
                        (BoolExpression::Bool(result), false)
                    }
                    BoolExpression::Int(right_val) => {
                        (BoolExpression::Bool(left_val >= right_val), false)
                    }
                    _ => {
                        panic!("invalid type in LEQ expression in guard")
                    }
                },
                _ => {
                    panic!("invalid type in LEQ expression in guard")
                }
            }
        }
        BoolExpression::EQ(left, right) => {
            let (computed_left, contains_clock_left) =
                apply_constraints_to_state_helper(&**left, decls, zone, false);
            let (computed_right, contains_clock_right) =
                apply_constraints_to_state_helper(&**right, decls, zone, false);

            if !should_apply && (contains_clock_right || contains_clock_left) {
                return (BoolExpression::GreatEQ(left.clone(), right.clone()), true);
            }
            match computed_left {
                BoolExpression::Clock(left_index) => match computed_right {
                    BoolExpression::Clock(right_index) => {
                        let result = zone.add_eq_constraint(right_index, left_index);
                        (BoolExpression::Bool(result), false)
                    }
                    BoolExpression::Int(right_val) => {
                        let result = zone.add_eq_const_constraint(left_index, right_val);
                        (BoolExpression::Bool(result), false)
                    }
                    _ => {
                        panic!("invalid type in EQ expression in guard")
                    }
                },
                BoolExpression::Int(left_val) => match computed_right {
                    BoolExpression::Clock(right_index) => {
                        let result = zone.add_eq_const_constraint(right_index, left_val);
                        (BoolExpression::Bool(result), false)
                    }
                    BoolExpression::Int(right_val) => {
                        (BoolExpression::Bool(left_val == right_val), false)
                    }
                    _ => {
                        panic!("invalid type in EQ expression in guard")
                    }
                },
                _ => {
                    panic!("invalid type in EQ expression in guard")
                }
            }
        }
        BoolExpression::LessT(left, right) => {
            let (computed_left, contains_clock_left) =
                apply_constraints_to_state_helper(&**left, decls, zone, false);
            let (computed_right, contains_clock_right) =
                apply_constraints_to_state_helper(&**right, decls, zone, false);

            if !should_apply && (contains_clock_right || contains_clock_left) {
                return (BoolExpression::LessT(left.clone(), right.clone()), true);
            }

            match computed_left {
                BoolExpression::Clock(left_index) => match computed_right {
                    BoolExpression::Clock(right_index) => {
                        let result = zone.add_lt_constraint(left_index, right_index, 0);
                        (BoolExpression::Bool(result), false)
                    }
                    BoolExpression::Int(right_val) => {
                        let result = zone.add_lt_constraint(left_index, 0, right_val);
                        (BoolExpression::Bool(result), false)
                    }
                    _ => {
                        panic!("invalid type in LEQ expression in guard")
                    }
                },
                BoolExpression::Int(left_val) => match computed_right {
                    BoolExpression::Clock(right_index) => {
                        let result = zone.add_lt_constraint(0, right_index, -left_val);
                        (BoolExpression::Bool(result), false)
                    }
                    BoolExpression::Int(right_val) => {
                        (BoolExpression::Bool(left_val <= right_val), false)
                    }
                    _ => {
                        panic!("invalid type in LEQ expression in guard")
                    }
                },
                _ => {
                    panic!("invalid type in LEQ expression in guard")
                }
            }
        }
        BoolExpression::GreatT(left, right) => {
            let (computed_left, contains_clock_left) =
                apply_constraints_to_state_helper(&**left, decls, zone, false);
            let (computed_right, contains_clock_right) =
                apply_constraints_to_state_helper(&**right, decls, zone, false);

            if !should_apply && (contains_clock_right || contains_clock_left) {
                return (BoolExpression::GreatT(left.clone(), right.clone()), true);
            }
            match computed_left {
                BoolExpression::Clock(left_index) => match computed_right {
                    BoolExpression::Clock(right_index) => {
                        let result = zone.add_lt_constraint(right_index, left_index, 0);
                        (BoolExpression::Bool(result), false)
                    }
                    BoolExpression::Int(right_val) => {
                        let result = zone.add_lt_constraint(0, left_index, -right_val);
                        (BoolExpression::Bool(result), false)
                    }
                    _ => {
                        panic!("invalid type in LEQ expression in guard")
                    }
                },
                BoolExpression::Int(left_val) => match computed_right {
                    BoolExpression::Clock(right_index) => {
                        let result = zone.add_lt_constraint(right_index, 0, left_val);
                        (BoolExpression::Bool(result), false)
                    }
                    BoolExpression::Int(right_val) => {
                        (BoolExpression::Bool(left_val >= right_val), false)
                    }
                    _ => {
                        panic!("invalid type in LEQ expression in guard")
                    }
                },
                _ => {
                    panic!("invalid type in LEQ expression in guard")
                }
            }
        }
        BoolExpression::Parentheses(expr) => {
            apply_constraints_to_state_helper(expr, decls, zone, should_apply)
        }
        BoolExpression::VarName(name) => {
            if let Some(clock_index) = decls.get_clocks().get(name.as_str()) {
                (BoolExpression::Clock(*clock_index), true)
            } else if let Some(val) = decls.get_ints().get(name.as_str()) {
                (BoolExpression::Int(*val), false)
            } else {
                panic!("No clock or variable named {:?} was found", name)
            }
        }
        BoolExpression::Bool(val) => (BoolExpression::Bool(*val), false),
        BoolExpression::Int(val) => (BoolExpression::Int(*val), false),
        BoolExpression::Clock(index) => (BoolExpression::Clock(*index), false),
    }
}

pub fn apply_constraints_to_fed_helper(
    guard: &BoolExpression,
    decls: &component::Declarations,
    fed: &mut Federation,
    should_apply: bool,
) -> (BoolExpression, bool) {
    match guard {
        BoolExpression::AndOp(left, right) => {
            let (left, _contains_clock_left) =
                apply_constraints_to_fed_helper(&**left, decls, fed, true);
            if let BoolExpression::Bool(val) = left {
                if !val {
                    return (BoolExpression::Bool(false), false);
                }
            }
            let (right, _contains_clock_right) =
                apply_constraints_to_fed_helper(&**right, decls, fed, true);

            match left {
                BoolExpression::Bool(left_val) => match right {
                    BoolExpression::Bool(right_val) => {
                        (BoolExpression::Bool(left_val && right_val), false)
                    }
                    _ => {
                        panic!("expected bool in apply guard && expression")
                    }
                },
                _ => {
                    panic!("expected bool in apply guard && expression")
                }
            }
        }
        BoolExpression::OrOp(left, right) => {
            let (mut left, contains_clock_left) =
                apply_constraints_to_fed_helper(&**left, decls, fed, false);
            let (mut right, contains_clock_right) =
                apply_constraints_to_fed_helper(&**right, decls, fed, false);

            if contains_clock_left && contains_clock_right {
                panic!("clock constrained on both sides of or operator, resulting in state that is not well defined")
            }

            if contains_clock_left {
                left = apply_constraints_to_fed_helper(&left, decls, fed, true).0;
            } else if contains_clock_right {
                right = apply_constraints_to_fed_helper(&right, decls, fed, true).0;
            }
            match left {
                BoolExpression::Bool(left_val) => match right {
                    BoolExpression::Bool(right_val) => {
                        (BoolExpression::Bool(left_val || right_val), false)
                    }
                    _ => {
                        panic!("expected bool in apply guard || expression")
                    }
                },
                _ => {
                    panic!("expected bool in apply guard || expression")
                }
            }
        }
        BoolExpression::LessEQ(left, right) => {
            let (computed_left, contains_clock_left) =
                apply_constraints_to_fed_helper(&**left, decls, fed, false);
            let (computed_right, contains_clock_right) =
                apply_constraints_to_fed_helper(&**right, decls, fed, false);

            if !should_apply && (contains_clock_right || contains_clock_left) {
                return (BoolExpression::LessEQ(left.clone(), right.clone()), true);
            }
            match computed_left {
                BoolExpression::Clock(left_index) => match computed_right {
                    BoolExpression::Clock(right_index) => {
                        let result = fed.add_lte_constraint(left_index, right_index, 0);

                        (BoolExpression::Bool(result), false)
                    }
                    BoolExpression::Int(right_val) => {
                        let result = fed.add_lte_constraint(left_index, 0, right_val);
                        (BoolExpression::Bool(result), false)
                    }
                    _ => {
                        panic!("invalid type in LEQ expression in guard")
                    }
                },
                BoolExpression::Int(left_val) => match computed_right {
                    BoolExpression::Clock(right_index) => {
                        let result = fed.add_lte_constraint(0, right_index, -left_val);
                        (BoolExpression::Bool(result), false)
                    }
                    BoolExpression::Int(right_val) => {
                        (BoolExpression::Bool(left_val <= right_val), false)
                    }
                    _ => {
                        panic!("invalid type in LEQ expression in guard")
                    }
                },
                _ => {
                    panic!("invalid type in LEQ expression in guard")
                }
            }
        }
        BoolExpression::GreatEQ(left, right) => {
            let (computed_left, contains_clock_left) =
                apply_constraints_to_fed_helper(&**left, decls, fed, false);
            let (computed_right, contains_clock_right) =
                apply_constraints_to_fed_helper(&**right, decls, fed, false);

            if !should_apply && (contains_clock_right || contains_clock_left) {
                return (BoolExpression::GreatEQ(left.clone(), right.clone()), true);
            }
            match computed_left {
                BoolExpression::Clock(left_index) => match computed_right {
                    BoolExpression::Clock(right_index) => {
                        let result = fed.add_lte_constraint(right_index, left_index, 0);

                        (BoolExpression::Bool(result), false)
                    }
                    BoolExpression::Int(right_val) => {
                        let result = fed.add_lte_constraint(0, left_index, -right_val);
                        (BoolExpression::Bool(result), false)
                    }
                    _ => {
                        panic!("invalid type in LEQ expression in guard")
                    }
                },
                BoolExpression::Int(left_val) => match computed_right {
                    BoolExpression::Clock(right_index) => {
                        let result = fed.add_lte_constraint(right_index, 0, left_val);
                        (BoolExpression::Bool(result), false)
                    }
                    BoolExpression::Int(right_val) => {
                        (BoolExpression::Bool(left_val >= right_val), false)
                    }
                    _ => {
                        panic!("invalid type in LEQ expression in guard")
                    }
                },
                _ => {
                    panic!("invalid type in LEQ expression in guard")
                }
            }
        }
        BoolExpression::EQ(left, right) => {
            let (computed_left, contains_clock_left) =
                apply_constraints_to_fed_helper(&**left, decls, fed, false);
            let (computed_right, contains_clock_right) =
                apply_constraints_to_fed_helper(&**right, decls, fed, false);

            if !should_apply && (contains_clock_right || contains_clock_left) {
                return (BoolExpression::GreatEQ(left.clone(), right.clone()), true);
            }
            match computed_left {
                BoolExpression::Clock(left_index) => match computed_right {
                    BoolExpression::Clock(right_index) => {
                        let result = fed.add_eq_constraint(right_index, left_index);
                        (BoolExpression::Bool(result), false)
                    }
                    BoolExpression::Int(right_val) => {
                        let result = fed.add_eq_const_constraint(left_index, right_val);
                        (BoolExpression::Bool(result), false)
                    }
                    _ => {
                        panic!("invalid type in EQ expression in guard")
                    }
                },
                BoolExpression::Int(left_val) => match computed_right {
                    BoolExpression::Clock(right_index) => {
                        let result = fed.add_eq_const_constraint(right_index, left_val);
                        (BoolExpression::Bool(result), false)
                    }
                    BoolExpression::Int(right_val) => {
                        (BoolExpression::Bool(left_val == right_val), false)
                    }
                    _ => {
                        panic!("invalid type in EQ expression in guard")
                    }
                },
                _ => {
                    panic!("invalid type in EQ expression in guard")
                }
            }
        }
        BoolExpression::LessT(left, right) => {
            let (computed_left, contains_clock_left) =
                apply_constraints_to_fed_helper(&**left, decls, fed, false);
            let (computed_right, contains_clock_right) =
                apply_constraints_to_fed_helper(&**right, decls, fed, false);

            if !should_apply && (contains_clock_right || contains_clock_left) {
                return (BoolExpression::LessT(left.clone(), right.clone()), true);
            }

            match computed_left {
                BoolExpression::Clock(left_index) => match computed_right {
                    BoolExpression::Clock(right_index) => {
                        let result = fed.add_lt_constraint(left_index, right_index, 0);
                        (BoolExpression::Bool(result), false)
                    }
                    BoolExpression::Int(right_val) => {
                        let result = fed.add_lt_constraint(left_index, 0, right_val);
                        (BoolExpression::Bool(result), false)
                    }
                    _ => {
                        panic!("invalid type in LEQ expression in guard")
                    }
                },
                BoolExpression::Int(left_val) => match computed_right {
                    BoolExpression::Clock(right_index) => {
                        let result = fed.add_lt_constraint(0, right_index, -left_val);
                        (BoolExpression::Bool(result), false)
                    }
                    BoolExpression::Int(right_val) => {
                        (BoolExpression::Bool(left_val <= right_val), false)
                    }
                    _ => {
                        panic!("invalid type in LEQ expression in guard")
                    }
                },
                _ => {
                    panic!("invalid type in LEQ expression in guard")
                }
            }
        }
        BoolExpression::GreatT(left, right) => {
            let (computed_left, contains_clock_left) =
                apply_constraints_to_fed_helper(&**left, decls, fed, false);
            let (computed_right, contains_clock_right) =
                apply_constraints_to_fed_helper(&**right, decls, fed, false);

            if !should_apply && (contains_clock_right || contains_clock_left) {
                return (BoolExpression::GreatT(left.clone(), right.clone()), true);
            }
            match computed_left {
                BoolExpression::Clock(left_index) => match computed_right {
                    BoolExpression::Clock(right_index) => {
                        let result = fed.add_lt_constraint(right_index, left_index, 0);
                        (BoolExpression::Bool(result), false)
                    }
                    BoolExpression::Int(right_val) => {
                        let result = fed.add_lt_constraint(0, left_index, -right_val);
                        (BoolExpression::Bool(result), false)
                    }
                    _ => {
                        panic!("invalid type in LEQ expression in guard")
                    }
                },
                BoolExpression::Int(left_val) => match computed_right {
                    BoolExpression::Clock(right_index) => {
                        let result = fed.add_lt_constraint(right_index, 0, left_val);
                        (BoolExpression::Bool(result), false)
                    }
                    BoolExpression::Int(right_val) => {
                        (BoolExpression::Bool(left_val >= right_val), false)
                    }
                    _ => {
                        panic!("invalid type in LEQ expression in guard")
                    }
                },
                _ => {
                    panic!("invalid type in LEQ expression in guard")
                }
            }
        }
        BoolExpression::Parentheses(expr) => {
            apply_constraints_to_fed_helper(expr, decls, fed, should_apply)
        }
        BoolExpression::VarName(name) => {
            if let Some(clock_index) = decls.get_clocks().get(name.as_str()) {
                (BoolExpression::Clock(*clock_index), true)
            } else if let Some(val) = decls.get_ints().get(name.as_str()) {
                (BoolExpression::Int(*val), false)
            } else {
                panic!("No clock or variable named {:?} was found", name)
            }
        }
        BoolExpression::Bool(val) => (BoolExpression::Bool(*val), false),
        BoolExpression::Int(val) => (BoolExpression::Int(*val), false),
        BoolExpression::Clock(index) => (BoolExpression::Clock(*index), false),
    }
}
