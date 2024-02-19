use crate::transition_systems::variant_eq;
use std::fmt::{Display, Formatter};

/// TransitionID is used to represent which edges a given transition consists of.
/// Works similarly to LocationID.
/// Note that Transitions may have a None id, if it is not created from an edge.
#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub enum TransitionID {
    Conjunction(Box<TransitionID>, Box<TransitionID>),
    Composition(Box<TransitionID>, Box<TransitionID>),
    Quotient(Vec<TransitionID>, Vec<TransitionID>),
    Simple(String),
    None,
}

impl TransitionID {
    /// Returns a vector of transitionIDs for all components involved in the transition
    /// For example
    /// ```
    /// use crate::reveaal::transition_systems::TransitionID;
    /// let id = TransitionID::Conjunction(
    ///     Box::new(TransitionID::Simple("a".to_string())),
    ///     Box::new(TransitionID::Simple("b".to_string())));
    /// let leaves = id.get_leaves();
    /// assert_eq!(leaves, vec![vec![TransitionID::Simple("a".to_string())], vec![TransitionID::Simple("b".to_string())]])
    /// ```
    /// Leaves will be {{a}, {b}}, as it is from the first component and b is from the second component
    pub fn get_leaves(&self) -> Vec<Vec<TransitionID>> {
        let mut result = Vec::new();
        self.get_leaves_helper(&mut result, 0);
        result
    }

    fn get_leaves_helper(
        &self,
        current_leaves: &mut Vec<Vec<TransitionID>>,
        index: usize,
    ) -> usize {
        match self {
            TransitionID::Conjunction(l, r) | TransitionID::Composition(l, r) => {
                let index_left = l.get_leaves_helper(current_leaves, index);
                r.get_leaves_helper(current_leaves, index_left + 1) // return index right
            }
            TransitionID::Quotient(l, r) => {
                let mut index_left = index;
                for t in l {
                    index_left = t.get_leaves_helper(current_leaves, index);
                }
                let mut index_right = index_left;
                for s in r {
                    index_right = s.get_leaves_helper(current_leaves, index_left + 1);
                }
                index_right
            }
            TransitionID::Simple(_) | TransitionID::None => {
                if current_leaves.len() <= index {
                    current_leaves.push(Vec::new());
                }
                current_leaves[index].push(self.clone());
                index
            }
        }
    }

    /// Takes a path of TransitionIDs, and splits them into seperate paths for each component
    /// For example
    /// ```
    /// use crate::reveaal::transition_systems::TransitionID;
    /// let path =
    ///    vec![
    ///          TransitionID::Conjunction(
    ///              Box::new(TransitionID::Simple("a".to_string())),
    ///              Box::new(TransitionID::Simple("b".to_string()))),
    ///         TransitionID::Conjunction(
    ///             Box::new(TransitionID::Simple("c".to_string())),
    ///             Box::new(TransitionID::Simple("d".to_string())))
    ///     ];
    ///  let component_paths = TransitionID::split_into_component_lists(&path);
    ///  assert_eq!(component_paths, Ok(
    ///     vec![
    ///         vec![
    ///             vec![TransitionID::Simple("a".to_string())],
    ///             vec![TransitionID::Simple("c".to_string())]],
    ///         vec![
    ///             vec![TransitionID::Simple("b".to_string())],
    ///             vec![TransitionID::Simple("d".to_string())]]]));
    /// ```
    /// component_paths will be {{a, c}, {b, d}}, representing the paths for the two components
    pub fn split_into_component_lists(
        path: &Vec<TransitionID>,
    ) -> Result<Vec<Vec<Vec<TransitionID>>>, String> {
        if path.is_empty() {
            return Ok(Vec::new());
        }
        let leaves = path[0].get_leaves();
        let amount = leaves.len();
        let mut paths: Vec<Vec<Vec<TransitionID>>> = vec![Vec::new(); leaves.len()];

        for transition_id in path {
            let leaves = transition_id.get_leaves();
            for (component_index, transition) in leaves.iter().enumerate() {
                if leaves.len() != amount {
                    return Err(format!("Could not split into components because first transition has {} components but {:?} has {} components", amount, leaves, leaves.len()));
                }
                paths[component_index].push(
                    transition
                        .iter()
                        .filter(|id| !matches!(id, TransitionID::None))
                        .cloned()
                        .collect(),
                );
            }
        }
        Ok(paths)
    }
}

impl Display for TransitionID {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut handle = |left: &TransitionID,
                          right: &TransitionID,
                          eq: &TransitionID,
                          sep: &str|
         -> std::fmt::Result {
            match (
                variant_eq(left, eq) || variant_eq(left, &TransitionID::Simple("".to_string())),
                variant_eq(right, eq) || variant_eq(right, &TransitionID::Simple("".to_string())),
            ) {
                (true, true) => write!(f, "{} {} {}", left, sep, right)?,
                (true, false) => write!(f, "{} {} ({})", left, sep, right)?,
                (false, true) => write!(f, "({}) {} {}", left, sep, right)?,
                (false, false) => write!(f, "({}) {} ({})", left, sep, right)?,
            }
            Ok(())
        };
        match self {
            TransitionID::Conjunction(left, right) => handle(
                left.as_ref(),
                right.as_ref(),
                &TransitionID::Conjunction(left.clone(), right.clone()),
                "&&",
            )?,
            TransitionID::Composition(left, right) => handle(
                left.as_ref(),
                right.as_ref(),
                &TransitionID::Composition(left.clone(), right.clone()),
                "||",
            )?,

            TransitionID::Quotient(left, right) => {
                for l in left {
                    match *(l) {
                        TransitionID::Simple(_) => write!(f, "{}", l)?,
                        _ => write!(f, "({})", l)?,
                    };
                }
                write!(f, "\\\\")?;
                for r in right {
                    match *(r) {
                        TransitionID::Simple(_) => write!(f, "{}", r)?,
                        _ => write!(f, "({})", r)?,
                    };
                }
            }
            TransitionID::Simple(name) => write!(f, "{}", name)?,
            TransitionID::None => write!(f, "NoID")?,
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::model_objects::expressions::SystemExpression;
    use crate::parse_queries::tests::create_system_recipe_and_machine;
    use crate::system::save_component::{combine_components, PruningStrategy};
    use crate::transition_systems::TransitionID;
    use std::collections::HashSet;
    use std::iter::FromIterator;
    use std::rc::Rc;
    use test_case::test_case;

    const FOLDER_PATH: &str = "samples/json/EcdarUniversity";

    #[test_case(SystemExpression::Component("Machine".to_string(), None), vec![
    "E0".to_string(),
    "E1".to_string(),
    "E2".to_string(),
    "E3".to_string(),
    "E4".to_string()]; "Simple save component transition id test")]
    #[test_case(
    SystemExpression::Conjunction(
    Box::new(SystemExpression::Component("HalfAdm1".to_string(), None)),
    Box::new(SystemExpression::Component("HalfAdm2".to_string(), None))),
    vec![
    "E0".to_string(),
    "E1".to_string(),
    "E2".to_string(),
    "E3".to_string(),
    "E4".to_string(),
    "E5".to_string(),
    "E6".to_string(),
    "E7".to_string(),
    "E8".to_string(),
    "E9".to_string(),
    "E10".to_string(),
    "E11".to_string()
    ]; "Conjunction save HalfAdm1 and HalfAdm2")]
    fn transition_save_id_checker(
        machine_expression: SystemExpression,
        transition_ids: Vec<String>,
    ) {
        let mock_model = Box::new(machine_expression);
        let mut expected_ids: HashSet<&String> = HashSet::from_iter(transition_ids.iter());
        let (_, system) = create_system_recipe_and_machine(*mock_model, FOLDER_PATH);

        let mut comp = combine_components(&system, PruningStrategy::NoPruning);

        comp.remake_edge_ids();

        for edge in comp.edges {
            if expected_ids.contains(&edge.id) {
                expected_ids.remove(&edge.id);
            } else {
                panic!("Found unexpected ID in component: {}", &edge.id)
            }
        }
        assert_eq!(expected_ids.len(), 0);
    }

    #[test_case(SystemExpression::Component("Machine".to_string(), None), vec![
    TransitionID::Simple("E25".to_string()),
    TransitionID::Simple("E26".to_string()),
    TransitionID::Simple("E27".to_string()),
    TransitionID::Simple("E28".to_string()),
    TransitionID::Simple("E29".to_string())]; "Simple transition id test")]
    #[test_case(
    SystemExpression::Conjunction(
    Box::new(SystemExpression::Component("HalfAdm1".to_string(), None)),
    Box::new(SystemExpression::Component("HalfAdm2".to_string(), None))),
    vec![
    TransitionID::Conjunction(
    Box::new(TransitionID::Simple("E43".to_string())),
    Box::new(TransitionID::Simple("E31".to_string()))
    ),
    TransitionID::Conjunction(
    Box::new(TransitionID::Simple("E37".to_string())),
    Box::new(TransitionID::Simple("E34".to_string()))
    ),
    TransitionID::Conjunction(
    Box::new(TransitionID::Simple("E42".to_string())),
    Box::new(TransitionID::Simple("E33".to_string()))
    ),
    TransitionID::Conjunction(
    Box::new(TransitionID::Simple("E37".to_string())),
    Box::new(TransitionID::Simple("E35".to_string()))
    ),
    TransitionID::Conjunction(
    Box::new(TransitionID::Simple("E42".to_string())),
    Box::new(TransitionID::Simple("E30".to_string()))
    ),
    TransitionID::Conjunction(
    Box::new(TransitionID::Simple("E39".to_string())),
    Box::new(TransitionID::Simple("E31".to_string()))
    ),
    TransitionID::Conjunction(
    Box::new(TransitionID::Simple("E38".to_string())),
    Box::new(TransitionID::Simple("E32".to_string()))
    ),
    TransitionID::Conjunction(
    Box::new(TransitionID::Simple("E41".to_string())),
    Box::new(TransitionID::Simple("E34".to_string()))
    ),
    TransitionID::Conjunction(
    Box::new(TransitionID::Simple("E40".to_string())),
    Box::new(TransitionID::Simple("E33".to_string()))
    ),
    TransitionID::Conjunction(
    Box::new(TransitionID::Simple("E40".to_string())),
    Box::new(TransitionID::Simple("E30".to_string()))
    ),
    TransitionID::Conjunction(
    Box::new(TransitionID::Simple("E38".to_string())),
    Box::new(TransitionID::Simple("E36".to_string()))
    ),
    TransitionID::Conjunction(
    Box::new(TransitionID::Simple("E41".to_string())),
    Box::new(TransitionID::Simple("E35".to_string()))
    )
    ]; "Conjunction HalfAdm1 and HalfAdm2")]
    fn transition_id_checker(
        machine_expression: SystemExpression,
        transition_ids: Vec<TransitionID>,
    ) {
        let mock_model = Box::new(machine_expression);
        let mut expected_ids: HashSet<&TransitionID> = HashSet::from_iter(transition_ids.iter());
        let (_, system) = create_system_recipe_and_machine(*mock_model, FOLDER_PATH);
        for loc in system.get_all_locations() {
            for ac in system.get_actions() {
                for tran in system.next_transitions(Rc::clone(&loc), &ac) {
                    if expected_ids.contains(&tran.id) {
                        expected_ids.remove(&tran.id);
                    } else {
                        panic!("Found unexpected ID in transition system: {}", &tran.id)
                    }
                }
            }
        }
        assert_eq!(expected_ids.len(), 0);
    }
}
