use crate::DBMLib::dbm::Zone;
use crate::ModelObjects::component;
use crate::ModelObjects::representations::BoolExpression;
use simple_error::bail;
use std::error::Error;

pub fn apply_constraint(
    constraint: &Option<BoolExpression>,
    decls: &component::Declarations,
    zone: &mut Zone,
) -> Result<bool, Box<dyn Error>> {
    if let Some(guards) = constraint {
        apply_constraints_to_state(guards, decls, zone)
    } else {
        Ok(true)
    }
}

pub fn apply_constraints_to_state(
    guard: &BoolExpression,
    decls: &component::Declarations,
    zone: &mut Zone,
) -> Result<bool, Box<dyn Error>> {
    if let BoolExpression::Bool(val) =
        apply_constraints_to_state_helper(guard, decls, zone, true)?.0
    {
        Ok(val)
    } else {
        bail!("unexpected value returned when attempting to apply constraints to zone");
    }
}

pub fn apply_constraints_to_state_declarations(
    guard: &BoolExpression,
    decls: &component::Declarations,
    zone: &mut Zone,
) -> Result<bool, Box<dyn Error>> {
    if let BoolExpression::Bool(val) =
        apply_constraints_to_state_helper(guard, decls, zone, true)?.0
    {
        Ok(val)
    } else {
        bail!("unexpected value returned when attempting to apply constraints to zone")
    }
}

pub fn apply_constraints_to_state_helper(
    guard: &BoolExpression,
    decls: &component::Declarations,
    zone: &mut Zone,
    should_apply: bool,
) -> Result<(BoolExpression, bool), Box<dyn Error>> {
    match guard {
        BoolExpression::AndOp(left, right) => {
            let (left, _contains_clock_left) =
                apply_constraints_to_state_helper(&**left, decls, zone, true)?;
            if let BoolExpression::Bool(val) = left {
                if !val {
                    return Ok((BoolExpression::Bool(false), false));
                }
            }
            let (right, _contains_clock_right) =
                apply_constraints_to_state_helper(&**right, decls, zone, true)?;

            match left {
                BoolExpression::Bool(left_val) => match right {
                    BoolExpression::Bool(right_val) => {
                        Ok((BoolExpression::Bool(left_val && right_val), false))
                    }
                    _ => {
                        bail!("expected bool in apply guard && expression")
                    }
                },
                _ => {
                    bail!("expected bool in apply guard && expression")
                }
            }
        }
        BoolExpression::OrOp(left, right) => {
            let (mut left, contains_clock_left) =
                apply_constraints_to_state_helper(&**left, decls, zone, false)?;
            let (mut right, contains_clock_right) =
                apply_constraints_to_state_helper(&**right, decls, zone, false)?;

            if contains_clock_left && contains_clock_right {
                bail!("clock constrained on both sides of or operator, resulting in state that is not well defined")
            }

            if contains_clock_left {
                left = apply_constraints_to_state_helper(&left, decls, zone, true)?.0;
            } else if contains_clock_right {
                right = apply_constraints_to_state_helper(&right, decls, zone, true)?.0;
            }
            match left {
                BoolExpression::Bool(left_val) => match right {
                    BoolExpression::Bool(right_val) => {
                        Ok((BoolExpression::Bool(left_val || right_val), false))
                    }
                    _ => {
                        bail!("expected bool in apply guard || expression")
                    }
                },
                _ => {
                    bail!("expected bool in apply guard || expression")
                }
            }
        }
        BoolExpression::LessEQ(left, right) => {
            let (computed_left, contains_clock_left) =
                apply_constraints_to_state_helper(&**left, decls, zone, false)?;
            let (computed_right, contains_clock_right) =
                apply_constraints_to_state_helper(&**right, decls, zone, false)?;

            if !should_apply && (contains_clock_right || contains_clock_left) {
                return Ok((BoolExpression::LessEQ(left.clone(), right.clone()), true));
            }
            match computed_left {
                BoolExpression::Clock(left_index) => match computed_right {
                    BoolExpression::Clock(right_index) => {
                        let result = zone.add_lte_constraint(left_index, right_index, 0);

                        println!("DBM: {}", zone);
                        Ok((BoolExpression::Bool(result), false))
                    }
                    BoolExpression::Int(right_val) => {
                        let result = zone.add_lte_constraint(left_index, 0, right_val);
                        Ok((BoolExpression::Bool(result), false))
                    }
                    _ => {
                        bail!("invalid type in LEQ expression in guard")
                    }
                },
                BoolExpression::Int(left_val) => match computed_right {
                    BoolExpression::Clock(right_index) => {
                        let result = zone.add_lte_constraint(0, right_index, -left_val);
                        Ok((BoolExpression::Bool(result), false))
                    }
                    BoolExpression::Int(right_val) => {
                        Ok((BoolExpression::Bool(left_val <= right_val), false))
                    }
                    _ => {
                        bail!("invalid type in LEQ expression in guard")
                    }
                },
                _ => {
                    bail!("invalid type in LEQ expression in guard")
                }
            }
        }
        BoolExpression::GreatEQ(left, right) => {
            let (computed_left, contains_clock_left) =
                apply_constraints_to_state_helper(&**left, decls, zone, false)?;
            let (computed_right, contains_clock_right) =
                apply_constraints_to_state_helper(&**right, decls, zone, false)?;

            if !should_apply && (contains_clock_right || contains_clock_left) {
                return Ok((BoolExpression::GreatEQ(left.clone(), right.clone()), true));
            }
            match computed_left {
                BoolExpression::Clock(left_index) => match computed_right {
                    BoolExpression::Clock(right_index) => {
                        let result = zone.add_lte_constraint(right_index, left_index, 0);

                        Ok((BoolExpression::Bool(result), false))
                    }
                    BoolExpression::Int(right_val) => {
                        let result = zone.add_lte_constraint(0, left_index, -right_val);
                        Ok((BoolExpression::Bool(result), false))
                    }
                    _ => {
                        bail!("invalid type in LEQ expression in guard")
                    }
                },
                BoolExpression::Int(left_val) => match computed_right {
                    BoolExpression::Clock(right_index) => {
                        let result = zone.add_lte_constraint(right_index, 0, left_val);
                        Ok((BoolExpression::Bool(result), false))
                    }
                    BoolExpression::Int(right_val) => {
                        Ok((BoolExpression::Bool(left_val >= right_val), false))
                    }
                    _ => {
                        bail!("invalid type in LEQ expression in guard")
                    }
                },
                _ => {
                    bail!("invalid type in LEQ expression in guard")
                }
            }
        }
        BoolExpression::EQ(left, right) => {
            let (computed_left, contains_clock_left) =
                apply_constraints_to_state_helper(&**left, decls, zone, false)?;
            let (computed_right, contains_clock_right) =
                apply_constraints_to_state_helper(&**right, decls, zone, false)?;

            if !should_apply && (contains_clock_right || contains_clock_left) {
                return Ok((BoolExpression::GreatEQ(left.clone(), right.clone()), true));
            }
            match computed_left {
                BoolExpression::Clock(left_index) => match computed_right {
                    BoolExpression::Clock(right_index) => {
                        let result = zone.add_eq_constraint(right_index, left_index);
                        Ok((BoolExpression::Bool(result), false))
                    }
                    BoolExpression::Int(right_val) => {
                        let result = zone.add_eq_const_constraint(left_index, right_val);
                        Ok((BoolExpression::Bool(result), false))
                    }
                    _ => {
                        bail!("invalid type in EQ expression in guard")
                    }
                },
                BoolExpression::Int(left_val) => match computed_right {
                    BoolExpression::Clock(right_index) => {
                        let result = zone.add_eq_const_constraint(right_index, left_val);
                        Ok((BoolExpression::Bool(result), false))
                    }
                    BoolExpression::Int(right_val) => {
                        Ok((BoolExpression::Bool(left_val == right_val), false))
                    }
                    _ => {
                        bail!("invalid type in EQ expression in guard")
                    }
                },
                _ => {
                    bail!("invalid type in EQ expression in guard")
                }
            }
        }
        BoolExpression::LessT(left, right) => {
            let (computed_left, contains_clock_left) =
                apply_constraints_to_state_helper(&**left, decls, zone, false)?;
            let (computed_right, contains_clock_right) =
                apply_constraints_to_state_helper(&**right, decls, zone, false)?;

            if !should_apply && (contains_clock_right || contains_clock_left) {
                return Ok((BoolExpression::LessT(left.clone(), right.clone()), true));
            }

            match computed_left {
                BoolExpression::Clock(left_index) => match computed_right {
                    BoolExpression::Clock(right_index) => {
                        let result = zone.add_lt_constraint(left_index, right_index, 0);
                        Ok((BoolExpression::Bool(result), false))
                    }
                    BoolExpression::Int(right_val) => {
                        let result = zone.add_lt_constraint(left_index, 0, right_val);
                        Ok((BoolExpression::Bool(result), false))
                    }
                    _ => {
                        bail!("invalid type in LEQ expression in guard")
                    }
                },
                BoolExpression::Int(left_val) => match computed_right {
                    BoolExpression::Clock(right_index) => {
                        let result = zone.add_lt_constraint(0, right_index, -left_val);
                        Ok((BoolExpression::Bool(result), false))
                    }
                    BoolExpression::Int(right_val) => {
                        Ok((BoolExpression::Bool(left_val <= right_val), false))
                    }
                    _ => {
                        bail!("invalid type in LEQ expression in guard")
                    }
                },
                _ => {
                    bail!("invalid type in LEQ expression in guard")
                }
            }
        }
        BoolExpression::GreatT(left, right) => {
            let (computed_left, contains_clock_left) =
                apply_constraints_to_state_helper(&**left, decls, zone, false)?;
            let (computed_right, contains_clock_right) =
                apply_constraints_to_state_helper(&**right, decls, zone, false)?;

            if !should_apply && (contains_clock_right || contains_clock_left) {
                return Ok((BoolExpression::GreatT(left.clone(), right.clone()), true));
            }
            match computed_left {
                BoolExpression::Clock(left_index) => match computed_right {
                    BoolExpression::Clock(right_index) => {
                        let result = zone.add_lt_constraint(right_index, left_index, 0);
                        Ok((BoolExpression::Bool(result), false))
                    }
                    BoolExpression::Int(right_val) => {
                        let result = zone.add_lt_constraint(0, left_index, -right_val);
                        Ok((BoolExpression::Bool(result), false))
                    }
                    _ => {
                        bail!("invalid type in LEQ expression in guard")
                    }
                },
                BoolExpression::Int(left_val) => match computed_right {
                    BoolExpression::Clock(right_index) => {
                        let result = zone.add_lt_constraint(right_index, 0, left_val);
                        Ok((BoolExpression::Bool(result), false))
                    }
                    BoolExpression::Int(right_val) => {
                        Ok((BoolExpression::Bool(left_val >= right_val), false))
                    }
                    _ => {
                        bail!("invalid type in LEQ expression in guard")
                    }
                },
                _ => {
                    bail!("invalid type in LEQ expression in guard")
                }
            }
        }
        BoolExpression::Parentheses(expr) => {
            apply_constraints_to_state_helper(expr, decls, zone, should_apply)
        }
        BoolExpression::VarName(name) => {
            if let Some(clock_index) = decls.get_clocks().get(name.as_str()) {
                Ok((BoolExpression::Clock(*clock_index), true))
            } else if let Some(val) = decls.get_ints().get(name.as_str()) {
                Ok((BoolExpression::Int(*val), false))
            } else {
                bail!("No clock or variable named {:?} was found", name)
            }
        }
        BoolExpression::Bool(val) => Ok((BoolExpression::Bool(*val), false)),
        BoolExpression::Int(val) => Ok((BoolExpression::Int(*val), false)),
        BoolExpression::Clock(index) => Ok((BoolExpression::Clock(*index), false)),
    }
}
pub fn apply_constraints_to_state2(
    guard: &BoolExpression,
    state: &mut component::State,
    comp_index: usize,
) -> Result<BoolExpression, Box<dyn Error>> {
    match guard {
        BoolExpression::AndOp(left, right) => {
            let left = apply_constraints_to_state2(&**left, state, comp_index)?;
            if let BoolExpression::Bool(val) = left {
                if !val {
                    return Ok(BoolExpression::Bool(false));
                }
            }
            let right = apply_constraints_to_state2(&**right, state, comp_index)?;

            match left {
                BoolExpression::Bool(left_val) => match right {
                    BoolExpression::Bool(right_val) => {
                        return Ok(BoolExpression::Bool(left_val && right_val))
                    }
                    _ => {
                        bail!("expected bool in apply guard && expression")
                    }
                },
                _ => {
                    bail!("expected bool in apply guard && expression")
                }
            }
        }
        BoolExpression::OrOp(left, right) => {
            let left = apply_constraints_to_state2(&**left, state, comp_index)?;
            let right = apply_constraints_to_state2(&**right, state, comp_index)?;

            match left {
                BoolExpression::Bool(left_val) => match right {
                    BoolExpression::Bool(right_val) => {
                        Ok(BoolExpression::Bool(left_val || right_val))
                    }
                    _ => {
                        bail!("expected bool in apply guard || expression")
                    }
                },
                _ => {
                    bail!("expected bool in apply guard || expression")
                }
            }
        }
        BoolExpression::LessEQ(left, right) => {
            let computed_left = apply_constraints_to_state2(&**left, state, comp_index)?;
            let computed_right = apply_constraints_to_state2(&**right, state, comp_index)?;

            match computed_left {
                BoolExpression::Clock(left_index) => match computed_right {
                    BoolExpression::Clock(right_index) => {
                        let result = state.zone.add_lte_constraint(left_index, right_index, 0);
                        Ok(BoolExpression::Bool(result))
                    }
                    BoolExpression::Int(right_val) => {
                        let result = state.zone.add_lte_constraint(left_index, 0, right_val);
                        Ok(BoolExpression::Bool(result))
                    }
                    _ => {
                        bail!("invalid type in LEQ expression in guard")
                    }
                },
                BoolExpression::Int(left_val) => match computed_right {
                    BoolExpression::Clock(right_index) => {
                        let result = state.zone.add_lte_constraint(0, right_index, -left_val);
                        Ok(BoolExpression::Bool(result))
                    }
                    BoolExpression::Int(right_val) => {
                        Ok(BoolExpression::Bool(left_val <= right_val))
                    }
                    _ => {
                        bail!("invalid type in LEQ expression in guard")
                    }
                },
                _ => {
                    bail!("invalid type in LEQ expression in guard")
                }
            }
        }
        BoolExpression::GreatEQ(left, right) => {
            let computed_left = apply_constraints_to_state2(&**left, state, comp_index)?;
            let computed_right = apply_constraints_to_state2(&**right, state, comp_index)?;
            match computed_left {
                BoolExpression::Clock(left_index) => match computed_right {
                    BoolExpression::Clock(right_index) => {
                        let result = state.zone.add_lte_constraint(right_index, left_index, 0);
                        Ok(BoolExpression::Bool(result))
                    }
                    BoolExpression::Int(right_val) => {
                        let result = state.zone.add_lte_constraint(0, left_index, -right_val);
                        Ok(BoolExpression::Bool(result))
                    }
                    _ => {
                        bail!("invalid type in LEQ expression in guard")
                    }
                },
                BoolExpression::Int(left_val) => match computed_right {
                    BoolExpression::Clock(right_index) => {
                        let result = state.zone.add_lte_constraint(right_index, 0, left_val);
                        Ok(BoolExpression::Bool(result))
                    }
                    BoolExpression::Int(right_val) => {
                        Ok(BoolExpression::Bool(left_val >= right_val))
                    }
                    _ => {
                        bail!("invalid type in LEQ expression in guard")
                    }
                },
                _ => {
                    bail!("invalid type in LEQ expression in guard")
                }
            }
        }
        BoolExpression::LessT(left, right) => {
            let computed_left = apply_constraints_to_state2(&**left, state, comp_index)?;
            let computed_right = apply_constraints_to_state2(&**right, state, comp_index)?;

            match computed_left {
                BoolExpression::Clock(left_index) => match computed_right {
                    BoolExpression::Clock(right_index) => {
                        let result = state.zone.add_lt_constraint(left_index, right_index, 0);
                        Ok(BoolExpression::Bool(result))
                    }
                    BoolExpression::Int(right_val) => {
                        let result = state.zone.add_lt_constraint(left_index, 0, right_val);
                        Ok(BoolExpression::Bool(result))
                    }
                    _ => {
                        bail!("invalid type in LEQ expression in guard")
                    }
                },
                BoolExpression::Int(left_val) => match computed_right {
                    BoolExpression::Clock(right_index) => {
                        let result = state.zone.add_lt_constraint(0, right_index, -left_val);
                        Ok(BoolExpression::Bool(result))
                    }
                    BoolExpression::Int(right_val) => {
                        Ok(BoolExpression::Bool(left_val <= right_val))
                    }
                    _ => {
                        bail!("invalid type in LEQ expression in guard")
                    }
                },
                _ => {
                    bail!("invalid type in LEQ expression in guard")
                }
            }
        }
        BoolExpression::GreatT(left, right) => {
            let computed_left = apply_constraints_to_state2(&**left, state, comp_index)?;
            let computed_right = apply_constraints_to_state2(&**right, state, comp_index)?;
            match computed_left {
                BoolExpression::Clock(left_index) => match computed_right {
                    BoolExpression::Clock(right_index) => {
                        let result = state.zone.add_lt_constraint(right_index, left_index, 0);
                        Ok(BoolExpression::Bool(result))
                    }
                    BoolExpression::Int(right_val) => {
                        let result = state.zone.add_lt_constraint(0, left_index, -right_val);
                        Ok(BoolExpression::Bool(result))
                    }
                    _ => {
                        bail!("invalid type in LEQ expression in guard")
                    }
                },
                BoolExpression::Int(left_val) => match computed_right {
                    BoolExpression::Clock(right_index) => {
                        let result = state.zone.add_lt_constraint(right_index, 0, left_val);
                        Ok(BoolExpression::Bool(result))
                    }
                    BoolExpression::Int(right_val) => {
                        Ok(BoolExpression::Bool(left_val >= right_val))
                    }
                    _ => {
                        bail!("invalid type in LEQ expression in guard")
                    }
                },
                _ => {
                    bail!("invalid type in LEQ expression in guard")
                }
            }
        }
        BoolExpression::Parentheses(expr) => apply_constraints_to_state2(expr, state, comp_index),
        BoolExpression::VarName(name) => {
            if let Some(clock_index) = state
                .get_declarations(comp_index)
                .get_clocks()
                .get(name.as_str())
            {
                Ok(BoolExpression::Clock(*clock_index))
            } else if let Some(val) = state
                .get_declarations(comp_index)
                .get_ints()
                .get(name.as_str())
            {
                Ok(BoolExpression::Int(*val))
            } else {
                bail!("no variable or clock named {:?}", name)
            }
        }
        BoolExpression::Bool(val) => Ok(BoolExpression::Bool(*val)),
        BoolExpression::Int(val) => Ok(BoolExpression::Int(*val)),
        BoolExpression::Clock(index) => Ok(BoolExpression::Clock(*index)),
        BoolExpression::EQ(left, right) => {
            let computed_left = apply_constraints_to_state2(&**left, state, comp_index)?;
            let computed_right = apply_constraints_to_state2(&**right, state, comp_index)?;

            match computed_left {
                BoolExpression::Clock(left_index) => match computed_right {
                    BoolExpression::Clock(right_index) => {
                        let result = state.zone.add_eq_constraint(right_index, left_index);
                        Ok(BoolExpression::Bool(result))
                    }
                    BoolExpression::Int(right_val) => {
                        let result = state.zone.add_eq_const_constraint(left_index, right_val);
                        Ok(BoolExpression::Bool(result))
                    }
                    _ => {
                        bail!("invalid type in EQ expression in guard")
                    }
                },
                BoolExpression::Int(left_val) => match computed_right {
                    BoolExpression::Clock(right_index) => {
                        let result = state.zone.add_eq_const_constraint(right_index, left_val);
                        Ok(BoolExpression::Bool(result))
                    }
                    BoolExpression::Int(right_val) => {
                        Ok(BoolExpression::Bool(left_val == right_val))
                    }
                    _ => {
                        bail!("invalid type in EQ expression in guard")
                    }
                },
                _ => {
                    bail!("invalid type in EQ expression in guard")
                }
            }
        }
    }
}
