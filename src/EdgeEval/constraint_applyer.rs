use crate::ModelObjects::representations::BoolExpression;
use crate::ModelObjects::component;
use super::super::DBMLib::lib;
use crate::ModelObjects::representations;


pub fn apply_constraints_to_state(guard : &BoolExpression, state : & component::State, zone : &mut [i32], dimensions : &u32) -> bool{
    if let BoolExpression::Bool(val) = apply_constraints_to_state_helper(guard, state, zone, dimensions, true).0 {
        val
    } else {
        panic!("unexpected value returned when attempting to apply constraints to zone")
    }
}

pub fn apply_constraints_to_state_helper(guard : &BoolExpression, state : & component::State, zone : &mut [i32], dimensions : &u32, should_apply : bool) -> (BoolExpression, bool){
    match guard {
        BoolExpression::AndOp(left, right) => {
            let (left, contains_clock_left) = apply_constraints_to_state_helper(&**left, state, zone, dimensions, true);
            if let BoolExpression::Bool(val) = left{
                if !val {
                    return (BoolExpression::Bool(false), false)
                }
            }
            let (right, contains_clock_right) = apply_constraints_to_state_helper(&**right, state, zone, dimensions, true);

            match left {
                BoolExpression::Bool(left_val) => {
                    match right {
                        BoolExpression::Bool(right_val) => {
                            return (BoolExpression::Bool(left_val && right_val), false)
                        },
                        _ => {
                            panic!("expected bool in apply guard && expression")
                        }
                    }
                },
                _ => {
                    panic!("expected bool in apply guard && expression")
                }
            }
        },
        BoolExpression::OrOp(left, right) => {
            let (mut left, contains_clock_left) = apply_constraints_to_state_helper(&**left, state, zone, dimensions, false);
            let (mut right, contains_clock_right) = apply_constraints_to_state_helper(&**right, state, zone, dimensions, false);

            if contains_clock_left && contains_clock_right {
                panic!("clock constrained on both sides of or operator, resulting in state that is not well defined")
            }

            if contains_clock_left {
                left = apply_constraints_to_state_helper(&left, state, zone, dimensions, true).0;
            } else if contains_clock_right {
                right = apply_constraints_to_state_helper(&right, state, zone, dimensions, true).0;
            }
            match left {
                BoolExpression::Bool(left_val) => {
                    match right {
                        BoolExpression::Bool(right_val) => {
                            return (BoolExpression::Bool(left_val || right_val), false)
                        },
                        _ => {
                            panic!("expected bool in apply guard || expression")
                        }
                    }
                },
                _ => {
                    panic!("expected bool in apply guard || expression")
                }
            }
        },
        BoolExpression::LessEQ(left, right) => {
            let (computed_left, contains_clock_left) = apply_constraints_to_state_helper(&**left, state, zone, dimensions, false);
            let (computed_right, contains_clock_right) = apply_constraints_to_state_helper(&**right, state, zone, dimensions, false);

            if !should_apply && (contains_clock_right || contains_clock_left) {
                return( BoolExpression::LessEQ(left.clone(), right.clone()), true)
            }
            match computed_left {
                BoolExpression::Clock(left_index) => {
                    match computed_right {
                        BoolExpression::Clock(right_index) => {
                            let result = lib::rs_dbm_add_LTE_constraint(zone, *dimensions, left_index, right_index, 0);

                            representations::print_DBM(zone, dimensions);
                            return (BoolExpression::Bool(result), false)
                        },
                        BoolExpression::Int(right_val) => {
                            //println!("Clock index: {:?} og bound: {:?}", left_index, right_val);
                            let result = lib::rs_dbm_add_LTE_constraint(zone, *dimensions, left_index, 0, right_val);
                            return (BoolExpression::Bool(result), false)
                        },
                        _ => {
                            panic!("invalid type in LEQ expression in guard")
                        }
                    }
                },
                BoolExpression::Int(left_val) => {
                    match computed_right {
                        BoolExpression::Clock(right_index) => {
                            let result = lib::rs_dbm_add_LTE_constraint(zone, *dimensions, 0, right_index, -1 * left_val);
                            return (BoolExpression::Bool(result), false)
                        },
                        BoolExpression::Int(right_val) => {
                            return (BoolExpression::Bool(left_val <= right_val), false)
                        },
                        _ => {
                            panic!("invalid type in LEQ expression in guard")
                        }
                    }
                },
                _ => {
                    panic!("invalid type in LEQ expression in guard")
                }
            }
        },
        BoolExpression::GreatEQ(left, right) => {
            let (computed_left, contains_clock_left) = apply_constraints_to_state_helper(&**left, state, zone, dimensions, false);
            let (computed_right, contains_clock_right) = apply_constraints_to_state_helper(&**right, state, zone, dimensions, false);

            if !should_apply && (contains_clock_right || contains_clock_left) {
                return( BoolExpression::GreatEQ(left.clone(), right.clone()), true)
            }
            match computed_left {
                BoolExpression::Clock(left_index) => {
                    //println!("CLOCK INDEX {:?}", left_index);
                    //println!("dimn: {:?}", dimensions);
                    match computed_right {
                        BoolExpression::Clock(right_index) => {

                            let result = lib::rs_dbm_add_LTE_constraint(zone, *dimensions, right_index, left_index, 0);

                            return (BoolExpression::Bool(result), false)
                        },
                        BoolExpression::Int(right_val) => {
                            let result = lib::rs_dbm_add_LTE_constraint(zone, *dimensions, 0, left_index, -1 * right_val);
                            return (BoolExpression::Bool(result), false)
                        },
                        _ => {
                            panic!("invalid type in LEQ expression in guard")
                        }
                    }
                },
                BoolExpression::Int(left_val) => {
                    match computed_right {
                        BoolExpression::Clock(right_index) => {
                            let result = lib::rs_dbm_add_LTE_constraint(zone, *dimensions, right_index, 0, left_val);
                            return (BoolExpression::Bool(result), false)
                        },
                        BoolExpression::Int(right_val) => {
                            return (BoolExpression::Bool(left_val >= right_val), false)
                        },
                        _ => {
                            panic!("invalid type in LEQ expression in guard")
                        }
                    }
                },
                _ => {
                    panic!("invalid type in LEQ expression in guard")
                }
            }
        },
        BoolExpression::EQ(left, right) => {
            let (computed_left, contains_clock_left) = apply_constraints_to_state_helper(&**left, state, zone, dimensions, false);
            let (computed_right, contains_clock_right) = apply_constraints_to_state_helper(&**right, state, zone, dimensions, false);

            if !should_apply && (contains_clock_right || contains_clock_left) {
                return( BoolExpression::GreatEQ(left.clone(), right.clone()), true)
            }
            match computed_left {
                BoolExpression::Clock(left_index) => {
                    //println!("CLOCK INDEX {:?}", left_index);
                    //println!("dimn: {:?}", dimensions);
                    match computed_right {
                        BoolExpression::Clock(right_index) => {
                            let result = lib::rs_dbm_add_EQ_constraint(zone, *dimensions, right_index, left_index);
                            return (BoolExpression::Bool(result), false)
                        },
                        BoolExpression::Int(right_val) => {
                            let result = lib::rs_dbm_add_EQ_const_constraint(zone, *dimensions, left_index, right_val);
                            return (BoolExpression::Bool(result), false)
                        },
                        _ => {
                            panic!("invalid type in EQ expression in guard")
                        }
                    }
                },
                BoolExpression::Int(left_val) => {
                    match computed_right {
                        BoolExpression::Clock(right_index) => {
                            let result = lib::rs_dbm_add_EQ_const_constraint(zone, *dimensions, right_index,left_val);
                            return (BoolExpression::Bool(result), false)
                        },
                        BoolExpression::Int(right_val) => {
                            return (BoolExpression::Bool(left_val == right_val), false)
                        },
                        _ => {
                            panic!("invalid type in EQ expression in guard")
                        }
                    }
                },
                _ => {
                    panic!("invalid type in EQ expression in guard")
                }
            }
        },
        BoolExpression::LessT(left, right) => {
            let (computed_left, contains_clock_left) = apply_constraints_to_state_helper(&**left, state, zone, dimensions, false);
            let (computed_right, contains_clock_right) = apply_constraints_to_state_helper(&**right, state, zone, dimensions, false);

            if !should_apply && (contains_clock_right || contains_clock_left) {
                return( BoolExpression::LessT(left.clone(), right.clone()), true)
            }

            match computed_left {
                BoolExpression::Clock(left_index) => {
                    match computed_right {
                        BoolExpression::Clock(right_index) => {
                            let result = lib::rs_dbm_add_LT_constraint(zone, *dimensions, left_index, right_index, 0);
                            return (BoolExpression::Bool(result), false)
                        },
                        BoolExpression::Int(right_val) => {
                            let result = lib::rs_dbm_add_LT_constraint(zone, *dimensions, left_index, 0, right_val);
                            return (BoolExpression::Bool(result), false)
                        },
                        _ => {
                            panic!("invalid type in LEQ expression in guard")
                        }
                    }
                },
                BoolExpression::Int(left_val) => {
                    match computed_right {
                        BoolExpression::Clock(right_index) => {
                            let result = lib::rs_dbm_add_LT_constraint(zone, *dimensions, 0, right_index, -1 * left_val);
                            return (BoolExpression::Bool(result), false)
                        },
                        BoolExpression::Int(right_val) => {
                            return (BoolExpression::Bool(left_val <= right_val), false)
                        },
                        _ => {
                            panic!("invalid type in LEQ expression in guard")
                        }
                    }
                },
                _ => {
                    panic!("invalid type in LEQ expression in guard")
                }
            }
        },
        BoolExpression::GreatT(left, right) => {
            let (computed_left, contains_clock_left) = apply_constraints_to_state_helper(&**left, state, zone, dimensions, false);
            let (computed_right, contains_clock_right) = apply_constraints_to_state_helper(&**right, state, zone, dimensions, false);

            if !should_apply && (contains_clock_right || contains_clock_left) {
                return( BoolExpression::GreatT(left.clone(), right.clone()), true)
            }
            match computed_left {
                BoolExpression::Clock(left_index) => {
                    match computed_right {
                        BoolExpression::Clock(right_index) => {
                            let result = lib::rs_dbm_add_LT_constraint(zone, *dimensions, right_index, left_index, 0);
                            return (BoolExpression::Bool(result), false)
                        },
                        BoolExpression::Int(right_val) => {
                            let result = lib::rs_dbm_add_LT_constraint(zone, *dimensions, 0, left_index, -1 * right_val);
                            return (BoolExpression::Bool(result), false)
                        },
                        _ => {
                            panic!("invalid type in LEQ expression in guard")
                        }
                    }
                },
                BoolExpression::Int(left_val) => {
                    match computed_right {
                        BoolExpression::Clock(right_index) => {
                            let result = lib::rs_dbm_add_LT_constraint(zone, *dimensions, right_index, 0, left_val);
                            return (BoolExpression::Bool(result), false)
                        },
                        BoolExpression::Int(right_val) => {
                            return (BoolExpression::Bool(left_val >= right_val), false)
                        },
                        _ => {
                            panic!("invalid type in LEQ expression in guard")
                        }
                    }
                },
                _ => {
                    panic!("invalid type in LEQ expression in guard")
                }
            }
        },
        BoolExpression::Parentheses(expr) => {
            return apply_constraints_to_state_helper(expr, state, zone, dimensions, should_apply)
        },
        BoolExpression::VarName(name) => {
            if let Some(clock_index) = state.get_declarations().get_clocks().get(name.as_str()) {
                return (BoolExpression::Clock(*clock_index), true)
            }
            if let Some(val) = state.get_declarations().get_ints().get(name.as_str()) {
                return (BoolExpression::Int(*val), false)
            }
            panic!("could not find variable: {:?} in declarations", name);
        },
        BoolExpression::Bool(val) => {
            return (BoolExpression::Bool(*val), false)
        },
        BoolExpression::Int(val) => {
            return (BoolExpression::Int(*val), false)
        },
        BoolExpression::Clock(index) => {
            return (BoolExpression::Clock(*index), false)
        }
    }

}
pub fn apply_constraints_to_state2(guard : &BoolExpression, full_state : &mut component::FullState, dimensions : &u32) -> BoolExpression{
    match guard {
        BoolExpression::AndOp(left, right) => {
            let left = apply_constraints_to_state2(&**left, full_state, dimensions);
            if let BoolExpression::Bool(val) = left{
                if !val {
                    return BoolExpression::Bool(false)
                }
            }
            let right = apply_constraints_to_state2(&**right, full_state, dimensions);

            match left {
                BoolExpression::Bool(left_val) => {
                    match right {
                        BoolExpression::Bool(right_val) => {
                            return BoolExpression::Bool(left_val && right_val)
                        },
                        _ => {
                            panic!("expected bool in apply guard && expression")
                        }
                    }
                },
                _ => {
                    panic!("expected bool in apply guard && expression")
                }
            }
        },
        BoolExpression::OrOp(left, right) => {
            let left = apply_constraints_to_state2(&**left, full_state, dimensions);
            let right = apply_constraints_to_state2(&**right, full_state, dimensions);

            match left {
                BoolExpression::Bool(left_val) => {
                    match right {
                        BoolExpression::Bool(right_val) => {
                            return BoolExpression::Bool(left_val || right_val)
                        },
                        _ => {
                            panic!("expected bool in apply guard || expression")
                        }
                    }
                },
                _ => {
                    panic!("expected bool in apply guard || expression")
                }
            }
        },
        BoolExpression::LessEQ(left, right) => {
            let computed_left = apply_constraints_to_state2(&**left, full_state, dimensions);
            let computed_right = apply_constraints_to_state2(&**right, full_state, dimensions);

            match computed_left {
                BoolExpression::Clock(left_index) => {
                    match computed_right {
                        BoolExpression::Clock(right_index) => {
                            let dim = *full_state.state.get_dimensions();
                            let result = lib::rs_dbm_add_LTE_constraint(full_state.get_zone(), *dimensions, left_index, right_index, 0);
                            return BoolExpression::Bool(result)
                        },
                        BoolExpression::Int(right_val) => {
                            let dim = *full_state.state.get_dimensions();
                            let result = lib::rs_dbm_add_LTE_constraint(full_state.get_zone(), *dimensions, left_index, 0, right_val);
                            return BoolExpression::Bool(result)
                        },
                        _ => {
                            panic!("invalid type in LEQ expression in guard")
                        }
                    }
                },
                BoolExpression::Int(left_val) => {
                    match computed_right {
                        BoolExpression::Clock(right_index) => {

                            let dim = *full_state.state.get_dimensions();
                            let result = lib::rs_dbm_add_LTE_constraint(full_state.get_zone(), *dimensions, 0, right_index, -1 * left_val);
                            return BoolExpression::Bool(result)
                        },
                        BoolExpression::Int(right_val) => {
                            return BoolExpression::Bool(left_val <= right_val)
                        },
                        _ => {
                            panic!("invalid type in LEQ expression in guard")
                        }
                    }
                },
                _ => {
                    panic!("invalid type in LEQ expression in guard")
                }
            }
        },
        BoolExpression::GreatEQ(left, right) => {
            let computed_left = apply_constraints_to_state2(&**left, full_state, dimensions);
            let computed_right = apply_constraints_to_state2(&**right, full_state, dimensions);
            match computed_left {
                BoolExpression::Clock(left_index) => {
                    match computed_right {
                        BoolExpression::Clock(right_index) => {
                            let dim = *full_state.state.get_dimensions();
                            let result = lib::rs_dbm_add_LTE_constraint(full_state.get_zone(), *dimensions, right_index, left_index, 0);
                            return BoolExpression::Bool(result)
                        },
                        BoolExpression::Int(right_val) => {
                            let dim = *full_state.state.get_dimensions();
                            let result = lib::rs_dbm_add_LTE_constraint(full_state.get_zone(), *dimensions, 0, left_index, -1 * right_val);
                            return BoolExpression::Bool(result)
                        },
                        _ => {
                            panic!("invalid type in LEQ expression in guard")
                        }
                    }
                },
                BoolExpression::Int(left_val) => {
                    match computed_right {
                        BoolExpression::Clock(right_index) => {
                            let dim = *full_state.state.get_dimensions();
                            let result = lib::rs_dbm_add_LTE_constraint(full_state.get_zone(), *dimensions, right_index, 0, left_val);
                            return BoolExpression::Bool(result)
                        },
                        BoolExpression::Int(right_val) => {
                            return BoolExpression::Bool(left_val >= right_val)
                        },
                        _ => {
                            panic!("invalid type in LEQ expression in guard")
                        }
                    }
                },
                _ => {
                    panic!("invalid type in LEQ expression in guard")
                }
            }
        },
        BoolExpression::LessT(left, right) => {
            let computed_left = apply_constraints_to_state2(&**left, full_state, dimensions);
            let computed_right = apply_constraints_to_state2(&**right, full_state, dimensions);

            match computed_left {
                BoolExpression::Clock(left_index) => {
                    match computed_right {
                        BoolExpression::Clock(right_index) => {
                            let dim = *full_state.state.get_dimensions();
                            let result = lib::rs_dbm_add_LT_constraint(full_state.get_zone(), *dimensions, left_index, right_index, 0);
                            return BoolExpression::Bool(result)
                        },
                        BoolExpression::Int(right_val) => {
                            let dim = *full_state.state.get_dimensions();
                            let result = lib::rs_dbm_add_LT_constraint(full_state.get_zone(), *dimensions, left_index, 0, right_val);
                            return BoolExpression::Bool(result)
                        },
                        _ => {
                            panic!("invalid type in LEQ expression in guard")
                        }
                    }
                },
                BoolExpression::Int(left_val) => {
                    match computed_right {
                        BoolExpression::Clock(right_index) => {
                            // int <= clock
                            let dim = *full_state.state.get_dimensions();;
                            let result = lib::rs_dbm_add_LT_constraint(full_state.get_zone(), *dimensions, 0, right_index, -1 * left_val);
                            return BoolExpression::Bool(result)
                        },
                        BoolExpression::Int(right_val) => {
                            return BoolExpression::Bool(left_val <= right_val)
                        },
                        _ => {
                            panic!("invalid type in LEQ expression in guard")
                        }
                    }
                },
                _ => {
                    panic!("invalid type in LEQ expression in guard")
                }
            }
        },
        BoolExpression::GreatT(left, right) => {
            let computed_left = apply_constraints_to_state2(&**left, full_state, dimensions);
            let computed_right = apply_constraints_to_state2(&**right, full_state, dimensions);
            match computed_left {
                BoolExpression::Clock(left_index) => {
                    match computed_right {
                        BoolExpression::Clock(right_index) => {
                            let dim = *full_state.state.get_dimensions();
                            let result = lib::rs_dbm_add_LT_constraint(full_state.get_zone(), *dimensions, right_index, left_index, 0);
                            return BoolExpression::Bool(result)
                        },
                        BoolExpression::Int(right_val) => {
                            let dim = *full_state.state.get_dimensions();
                            let result = lib::rs_dbm_add_LT_constraint(full_state.get_zone(), *dimensions, 0, left_index, -1 * right_val);
                            return BoolExpression::Bool(result)
                        },
                        _ => {
                            panic!("invalid type in LEQ expression in guard")
                        }
                    }
                },
                BoolExpression::Int(left_val) => {
                    match computed_right {
                        BoolExpression::Clock(right_index) => {
                            let dim = *full_state.state.get_dimensions();
                            let result = lib::rs_dbm_add_LT_constraint(full_state.get_zone(), *dimensions, right_index, 0, left_val);
                            return BoolExpression::Bool(result)
                        },
                        BoolExpression::Int(right_val) => {
                            return BoolExpression::Bool(left_val >= right_val)
                        },
                        _ => {
                            panic!("invalid type in LEQ expression in guard")
                        }
                    }
                },
                _ => {
                    panic!("invalid type in LEQ expression in guard")
                }
            }
        },
        BoolExpression::Parentheses(expr) => {
            return apply_constraints_to_state2(guard, full_state, dimensions)
        },
        BoolExpression::VarName(name) => {
            if let Some(clock_index) = full_state.state.get_declarations().get_clocks().get(name.as_str()) {
                return BoolExpression::Clock(*clock_index)
            }
            if let Some(val) = full_state.state.get_declarations().get_ints().get(name.as_str()) {
                return BoolExpression::Int(*val)
            }
        },
        BoolExpression::Bool(val) => {
            return BoolExpression::Bool(*val)
        },
        BoolExpression::Int(val) => {
            return BoolExpression::Int(*val)
        },
        BoolExpression::Clock(index) => {
            return BoolExpression::Clock(*index)
        }
        _ => {}
    }

    panic!("not implemented")
}
