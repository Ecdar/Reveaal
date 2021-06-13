use crate::DBMLib::dbm::Zone;
use crate::ModelObjects::component;
use crate::ModelObjects::representations::BoolExpression;

pub fn apply_constraints_to_state(
    guard: &BoolExpression,
    state: &component::State,
    zone: &mut Zone,
) -> bool {
    if let BoolExpression::Bool(val) = apply_constraints_to_state_helper(guard, state, zone, true).0
    {
        val
    } else {
        panic!("unexpected value returned when attempting to apply constraints to zone")
    }
}

pub fn apply_constraints_to_state_helper(
    guard: &BoolExpression,
    state: &component::State,
    zone: &mut Zone,
    should_apply: bool,
) -> (BoolExpression, bool) {
    match guard {
        BoolExpression::AndOp(left, right) => {
            let (left, _contains_clock_left) =
                apply_constraints_to_state_helper(&**left, state, zone, true);
            if let BoolExpression::Bool(val) = left {
                if !val {
                    return (BoolExpression::Bool(false), false);
                }
            }
            let (right, _contains_clock_right) =
                apply_constraints_to_state_helper(&**right, state, zone, true);

            match left {
                BoolExpression::Bool(left_val) => match right {
                    BoolExpression::Bool(right_val) => {
                        return (BoolExpression::Bool(left_val && right_val), false)
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
                apply_constraints_to_state_helper(&**left, state, zone, false);
            let (mut right, contains_clock_right) =
                apply_constraints_to_state_helper(&**right, state, zone, false);

            if contains_clock_left && contains_clock_right {
                panic!("clock constrained on both sides of or operator, resulting in state that is not well defined")
            }

            if contains_clock_left {
                left = apply_constraints_to_state_helper(&left, state, zone, true).0;
            } else if contains_clock_right {
                right = apply_constraints_to_state_helper(&right, state, zone, true).0;
            }
            match left {
                BoolExpression::Bool(left_val) => match right {
                    BoolExpression::Bool(right_val) => {
                        return (BoolExpression::Bool(left_val || right_val), false)
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
                apply_constraints_to_state_helper(&**left, state, zone, false);
            let (computed_right, contains_clock_right) =
                apply_constraints_to_state_helper(&**right, state, zone, false);

            if !should_apply && (contains_clock_right || contains_clock_left) {
                return (BoolExpression::LessEQ(left.clone(), right.clone()), true);
            }
            match computed_left {
                BoolExpression::Clock(left_index) => {
                    match computed_right {
                        BoolExpression::Clock(right_index) => {
                            let result = zone.add_lte_constraint(left_index, right_index, 0);

                            println!("DBM: {}", zone);
                            return (BoolExpression::Bool(result), false);
                        }
                        BoolExpression::Int(right_val) => {
                            //println!("Clock index: {:?} og bound: {:?}", left_index, right_val);
                            let result = zone.add_lte_constraint(left_index, 0, right_val);
                            return (BoolExpression::Bool(result), false);
                        }
                        _ => {
                            panic!("invalid type in LEQ expression in guard")
                        }
                    }
                }
                BoolExpression::Int(left_val) => match computed_right {
                    BoolExpression::Clock(right_index) => {
                        let result = zone.add_lte_constraint(0, right_index, -1 * left_val);
                        return (BoolExpression::Bool(result), false);
                    }
                    BoolExpression::Int(right_val) => {
                        return (BoolExpression::Bool(left_val <= right_val), false)
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
                apply_constraints_to_state_helper(&**left, state, zone, false);
            let (computed_right, contains_clock_right) =
                apply_constraints_to_state_helper(&**right, state, zone, false);

            if !should_apply && (contains_clock_right || contains_clock_left) {
                return (BoolExpression::GreatEQ(left.clone(), right.clone()), true);
            }
            match computed_left {
                BoolExpression::Clock(left_index) => {
                    //println!("CLOCK INDEX {:?}", left_index);
                    //println!("dimn: {:?}", dimensions);
                    match computed_right {
                        BoolExpression::Clock(right_index) => {
                            let result = zone.add_lte_constraint(right_index, left_index, 0);

                            return (BoolExpression::Bool(result), false);
                        }
                        BoolExpression::Int(right_val) => {
                            let result = zone.add_lte_constraint(0, left_index, -1 * right_val);
                            return (BoolExpression::Bool(result), false);
                        }
                        _ => {
                            panic!("invalid type in LEQ expression in guard")
                        }
                    }
                }
                BoolExpression::Int(left_val) => match computed_right {
                    BoolExpression::Clock(right_index) => {
                        let result = zone.add_lte_constraint(right_index, 0, left_val);
                        return (BoolExpression::Bool(result), false);
                    }
                    BoolExpression::Int(right_val) => {
                        return (BoolExpression::Bool(left_val >= right_val), false)
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
                apply_constraints_to_state_helper(&**left, state, zone, false);
            let (computed_right, contains_clock_right) =
                apply_constraints_to_state_helper(&**right, state, zone, false);

            if !should_apply && (contains_clock_right || contains_clock_left) {
                return (BoolExpression::GreatEQ(left.clone(), right.clone()), true);
            }
            match computed_left {
                BoolExpression::Clock(left_index) => {
                    //println!("CLOCK INDEX {:?}", left_index);
                    //println!("dimn: {:?}", dimensions);
                    match computed_right {
                        BoolExpression::Clock(right_index) => {
                            let result = zone.add_eq_constraint(right_index, left_index);
                            return (BoolExpression::Bool(result), false);
                        }
                        BoolExpression::Int(right_val) => {
                            let result = zone.add_eq_const_constraint(left_index, right_val);
                            return (BoolExpression::Bool(result), false);
                        }
                        _ => {
                            panic!("invalid type in EQ expression in guard")
                        }
                    }
                }
                BoolExpression::Int(left_val) => match computed_right {
                    BoolExpression::Clock(right_index) => {
                        let result = zone.add_eq_const_constraint(right_index, left_val);
                        return (BoolExpression::Bool(result), false);
                    }
                    BoolExpression::Int(right_val) => {
                        return (BoolExpression::Bool(left_val == right_val), false)
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
                apply_constraints_to_state_helper(&**left, state, zone, false);
            let (computed_right, contains_clock_right) =
                apply_constraints_to_state_helper(&**right, state, zone, false);

            if !should_apply && (contains_clock_right || contains_clock_left) {
                return (BoolExpression::LessT(left.clone(), right.clone()), true);
            }

            match computed_left {
                BoolExpression::Clock(left_index) => match computed_right {
                    BoolExpression::Clock(right_index) => {
                        let result = zone.add_lt_constraint(left_index, right_index, 0);
                        return (BoolExpression::Bool(result), false);
                    }
                    BoolExpression::Int(right_val) => {
                        let result = zone.add_lt_constraint(left_index, 0, right_val);
                        return (BoolExpression::Bool(result), false);
                    }
                    _ => {
                        panic!("invalid type in LEQ expression in guard")
                    }
                },
                BoolExpression::Int(left_val) => match computed_right {
                    BoolExpression::Clock(right_index) => {
                        let result = zone.add_lt_constraint(0, right_index, -1 * left_val);
                        return (BoolExpression::Bool(result), false);
                    }
                    BoolExpression::Int(right_val) => {
                        return (BoolExpression::Bool(left_val <= right_val), false)
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
                apply_constraints_to_state_helper(&**left, state, zone, false);
            let (computed_right, contains_clock_right) =
                apply_constraints_to_state_helper(&**right, state, zone, false);

            if !should_apply && (contains_clock_right || contains_clock_left) {
                return (BoolExpression::GreatT(left.clone(), right.clone()), true);
            }
            match computed_left {
                BoolExpression::Clock(left_index) => match computed_right {
                    BoolExpression::Clock(right_index) => {
                        let result = zone.add_lt_constraint(right_index, left_index, 0);
                        return (BoolExpression::Bool(result), false);
                    }
                    BoolExpression::Int(right_val) => {
                        let result = zone.add_lt_constraint(0, left_index, -1 * right_val);
                        return (BoolExpression::Bool(result), false);
                    }
                    _ => {
                        panic!("invalid type in LEQ expression in guard")
                    }
                },
                BoolExpression::Int(left_val) => match computed_right {
                    BoolExpression::Clock(right_index) => {
                        let result = zone.add_lt_constraint(right_index, 0, left_val);
                        return (BoolExpression::Bool(result), false);
                    }
                    BoolExpression::Int(right_val) => {
                        return (BoolExpression::Bool(left_val >= right_val), false)
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
            return apply_constraints_to_state_helper(expr, state, zone, should_apply)
        }
        BoolExpression::VarName(name) => {
            if let Some(clock_index) = state.get_declarations().get_clocks().get(name.as_str()) {
                return (BoolExpression::Clock(*clock_index), true);
            } else if let Some(val) = state.get_declarations().get_ints().get(name.as_str()) {
                return (BoolExpression::Int(*val), false);
            } else {
                panic!("No clock or variable named {:?} was found", name)
            }
        }
        BoolExpression::Bool(val) => return (BoolExpression::Bool(*val), false),
        BoolExpression::Int(val) => return (BoolExpression::Int(*val), false),
        BoolExpression::Clock(index) => return (BoolExpression::Clock(*index), false),
    }
}
pub fn apply_constraints_to_state2(
    guard: &BoolExpression,
    full_state: &mut component::FullState,
) -> BoolExpression {
    match guard {
        BoolExpression::AndOp(left, right) => {
            let left = apply_constraints_to_state2(&**left, full_state);
            if let BoolExpression::Bool(val) = left {
                if !val {
                    return BoolExpression::Bool(false);
                }
            }
            let right = apply_constraints_to_state2(&**right, full_state);

            match left {
                BoolExpression::Bool(left_val) => match right {
                    BoolExpression::Bool(right_val) => {
                        return BoolExpression::Bool(left_val && right_val)
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
            let left = apply_constraints_to_state2(&**left, full_state);
            let right = apply_constraints_to_state2(&**right, full_state);

            match left {
                BoolExpression::Bool(left_val) => match right {
                    BoolExpression::Bool(right_val) => {
                        return BoolExpression::Bool(left_val || right_val)
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
            let computed_left = apply_constraints_to_state2(&**left, full_state);
            let computed_right = apply_constraints_to_state2(&**right, full_state);

            match computed_left {
                BoolExpression::Clock(left_index) => match computed_right {
                    BoolExpression::Clock(right_index) => {
                        let result = full_state
                            .zone
                            .add_lte_constraint(left_index, right_index, 0);
                        return BoolExpression::Bool(result);
                    }
                    BoolExpression::Int(right_val) => {
                        let result = full_state.zone.add_lte_constraint(left_index, 0, right_val);
                        return BoolExpression::Bool(result);
                    }
                    _ => {
                        panic!("invalid type in LEQ expression in guard")
                    }
                },
                BoolExpression::Int(left_val) => match computed_right {
                    BoolExpression::Clock(right_index) => {
                        let result =
                            full_state
                                .zone
                                .add_lte_constraint(0, right_index, -1 * left_val);
                        return BoolExpression::Bool(result);
                    }
                    BoolExpression::Int(right_val) => {
                        return BoolExpression::Bool(left_val <= right_val)
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
            let computed_left = apply_constraints_to_state2(&**left, full_state);
            let computed_right = apply_constraints_to_state2(&**right, full_state);
            match computed_left {
                BoolExpression::Clock(left_index) => match computed_right {
                    BoolExpression::Clock(right_index) => {
                        let result = full_state
                            .zone
                            .add_lte_constraint(right_index, left_index, 0);
                        return BoolExpression::Bool(result);
                    }
                    BoolExpression::Int(right_val) => {
                        let result =
                            full_state
                                .zone
                                .add_lte_constraint(0, left_index, -1 * right_val);
                        return BoolExpression::Bool(result);
                    }
                    _ => {
                        panic!("invalid type in LEQ expression in guard")
                    }
                },
                BoolExpression::Int(left_val) => match computed_right {
                    BoolExpression::Clock(right_index) => {
                        let result = full_state.zone.add_lte_constraint(right_index, 0, left_val);
                        return BoolExpression::Bool(result);
                    }
                    BoolExpression::Int(right_val) => {
                        return BoolExpression::Bool(left_val >= right_val)
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
        BoolExpression::LessT(left, right) => {
            let computed_left = apply_constraints_to_state2(&**left, full_state);
            let computed_right = apply_constraints_to_state2(&**right, full_state);

            match computed_left {
                BoolExpression::Clock(left_index) => match computed_right {
                    BoolExpression::Clock(right_index) => {
                        let result = full_state
                            .zone
                            .add_lt_constraint(left_index, right_index, 0);
                        return BoolExpression::Bool(result);
                    }
                    BoolExpression::Int(right_val) => {
                        let result = full_state.zone.add_lt_constraint(left_index, 0, right_val);
                        return BoolExpression::Bool(result);
                    }
                    _ => {
                        panic!("invalid type in LEQ expression in guard")
                    }
                },
                BoolExpression::Int(left_val) => match computed_right {
                    BoolExpression::Clock(right_index) => {
                        let result =
                            full_state
                                .zone
                                .add_lt_constraint(0, right_index, -1 * left_val);
                        return BoolExpression::Bool(result);
                    }
                    BoolExpression::Int(right_val) => {
                        return BoolExpression::Bool(left_val <= right_val)
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
            let computed_left = apply_constraints_to_state2(&**left, full_state);
            let computed_right = apply_constraints_to_state2(&**right, full_state);
            match computed_left {
                BoolExpression::Clock(left_index) => match computed_right {
                    BoolExpression::Clock(right_index) => {
                        let result = full_state
                            .zone
                            .add_lt_constraint(right_index, left_index, 0);
                        return BoolExpression::Bool(result);
                    }
                    BoolExpression::Int(right_val) => {
                        let result =
                            full_state
                                .zone
                                .add_lt_constraint(0, left_index, -1 * right_val);
                        return BoolExpression::Bool(result);
                    }
                    _ => {
                        panic!("invalid type in LEQ expression in guard")
                    }
                },
                BoolExpression::Int(left_val) => match computed_right {
                    BoolExpression::Clock(right_index) => {
                        let result = full_state.zone.add_lt_constraint(right_index, 0, left_val);
                        return BoolExpression::Bool(result);
                    }
                    BoolExpression::Int(right_val) => {
                        return BoolExpression::Bool(left_val >= right_val)
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
        BoolExpression::Parentheses(_expr) => {
            return apply_constraints_to_state2(guard, full_state)
        }
        BoolExpression::VarName(name) => {
            if let Some(clock_index) = full_state
                .state
                .get_declarations()
                .get_clocks()
                .get(name.as_str())
            {
                return BoolExpression::Clock(*clock_index);
            } else if let Some(val) = full_state
                .state
                .get_declarations()
                .get_ints()
                .get(name.as_str())
            {
                return BoolExpression::Int(*val);
            } else {
                panic!("no variable or clock named {:?}", name)
            }
        }
        BoolExpression::Bool(val) => return BoolExpression::Bool(*val),
        BoolExpression::Int(val) => return BoolExpression::Int(*val),
        BoolExpression::Clock(index) => return BoolExpression::Clock(*index),
        //_ => {}
        BoolExpression::EQ(left, right) => {
            let computed_left = apply_constraints_to_state2(&**left, full_state);
            let computed_right = apply_constraints_to_state2(&**right, full_state);

            match computed_left {
                BoolExpression::Clock(left_index) => {
                    match computed_right {
                        BoolExpression::Clock(right_index) => {
                            let result = full_state.zone.add_eq_constraint(right_index, left_index);
                            return BoolExpression::Bool(result);
                        }
                        BoolExpression::Int(right_val) => {
                            let result = full_state
                                .zone
                                .add_eq_const_constraint(left_index, right_val);
                            return BoolExpression::Bool(result);
                        }
                        _ => {
                            panic!("invalid type in EQ expression in guard")
                        }
                    }
                }
                BoolExpression::Int(left_val) => match computed_right {
                    BoolExpression::Clock(right_index) => {
                        let result = full_state
                            .zone
                            .add_eq_const_constraint(right_index, left_val);
                        return BoolExpression::Bool(result);
                    }
                    BoolExpression::Int(right_val) => {
                        return BoolExpression::Bool(left_val == right_val)
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
    }
}
