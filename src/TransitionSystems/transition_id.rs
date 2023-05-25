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
    /// Returns a vector of transitionIDs for all automata involved in the transition
    /// For example
    /// ```
    /// use crate::reveaal::TransitionSystems::TransitionID;
    /// let id = TransitionID::Conjunction(
    ///     Box::new(TransitionID::Simple("a".to_string())),
    ///     Box::new(TransitionID::Simple("b".to_string())));
    /// let leaves = id.get_leaves();
    /// assert_eq!(leaves, vec![vec![TransitionID::Simple("a".to_string())], vec![TransitionID::Simple("b".to_string())]])
    /// ```
    /// Leaves will be `{{a}, {b}}`, as `a` is from the first automaton and `b` is from the second automaton
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

    /// Takes a path of TransitionIDs, and splits them into seperate paths for each automaton
    /// For example
    /// ```
    /// use crate::reveaal::TransitionSystems::TransitionID;
    /// let path =
    ///    vec![
    ///          TransitionID::Conjunction(
    ///              Box::new(TransitionID::Simple("a".to_string())),
    ///              Box::new(TransitionID::Simple("b".to_string()))),
    ///         TransitionID::Conjunction(
    ///             Box::new(TransitionID::Simple("c".to_string())),
    ///             Box::new(TransitionID::Simple("d".to_string())))
    ///     ];
    ///  let automta_paths = TransitionID::split_into_automata_lists(&path);
    ///  assert_eq!(automta_paths, Ok(
    ///     vec![
    ///         vec![
    ///             vec![TransitionID::Simple("a".to_string())],
    ///             vec![TransitionID::Simple("c".to_string())]],
    ///         vec![
    ///             vec![TransitionID::Simple("b".to_string())],
    ///             vec![TransitionID::Simple("d".to_string())]]]));
    /// ```
    /// `automata_paths` will be `{{a, c}, {b, d}}`, representing the paths for the two automata
    pub fn split_into_automata_lists(
        path: &Vec<TransitionID>,
    ) -> Result<Vec<Vec<Vec<TransitionID>>>, String> {
        if path.is_empty() {
            return Ok(Vec::new());
        }
        let leaves = path[0].get_leaves();
        let amount = leaves.len();
        let mut paths: Vec<Vec<Vec<TransitionID>>> = vec![Vec::new(); leaves.len()];

        for transitionID in path {
            let leaves = transitionID.get_leaves();
            for (automaton_index, transition) in leaves.iter().enumerate() {
                if leaves.len() != amount {
                    return Err(format!("Could not split into automata because first transition has {} automata but {:?} has {} automata", amount, leaves, leaves.len()));
                }
                paths[automaton_index].push(
                    transition
                        .iter()
                        .cloned()
                        .filter(|id| !matches!(id, TransitionID::None))
                        .collect(),
                );
            }
        }
        Ok(paths)
    }
}

impl Display for TransitionID {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TransitionID::Conjunction(left, right) => {
                match *(*left) {
                    TransitionID::Conjunction(_, _) => write!(f, "{}", (*left))?,
                    TransitionID::Simple(_) => write!(f, "{}", (*left))?,
                    _ => write!(f, "({})", (*left))?,
                };
                write!(f, "&&")?;
                match *(*right) {
                    TransitionID::Conjunction(_, _) => write!(f, "{}", (*right))?,
                    TransitionID::Simple(_) => write!(f, "{}", (*right))?,
                    _ => write!(f, "({})", (*right))?,
                };
            }
            TransitionID::Composition(left, right) => {
                match *(*left) {
                    TransitionID::Composition(_, _) => write!(f, "{}", (*left))?,
                    TransitionID::Simple(_) => write!(f, "{}", (*left))?,
                    _ => write!(f, "({})", (*left))?,
                };
                write!(f, "||")?;
                match *(*right) {
                    TransitionID::Composition(_, _) => write!(f, "{}", (*right))?,
                    TransitionID::Simple(_) => write!(f, "{}", (*right))?,
                    _ => write!(f, "({})", (*right))?,
                };
            }
            TransitionID::Quotient(left, right) => {
                for l in left {
                    match *(l) {
                        TransitionID::Simple(_) => write!(f, "{}", (l))?,
                        _ => write!(f, "({})", (l))?,
                    };
                }
                write!(f, "\\\\")?;
                for r in right {
                    match *(r) {
                        TransitionID::Simple(_) => write!(f, "{}", (r))?,
                        _ => write!(f, "({})", (r))?,
                    };
                }
            }
            TransitionID::Simple(name) => write!(f, "{}", name)?,
            TransitionID::None => write!(f, "NoID")?,
        }
        Ok(())
    }
}
