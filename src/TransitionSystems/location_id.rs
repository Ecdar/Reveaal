use std::fmt::{Display, Formatter};

use crate::{ModelObjects::representations::QueryExpression, System::specifics::SpecialLocation};

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub enum LocationID {
    Conjunction(Box<LocationID>, Box<LocationID>),
    Composition(Box<LocationID>, Box<LocationID>),
    Quotient(Box<LocationID>, Box<LocationID>),
    /// Represents the potentially complete identifier of a location
    Simple(String),
    Special(SpecialLocation),
    /// Used for representing a partial state and it is generated when a location's name is set as `_`
    AnyLocation,
}

impl LocationID {
    /// A debug method to construct a location ID from a string.
    /// e.g. "A" -> Simple("A"), "A && B" -> LocationID::Conjunction(Simple("A"), Simple("B")), etc.
    pub fn from_string(string: &str) -> Self {
        // A bit of a hack but we use the parser get the a query expression from which we can
        // determine to composition types needed to construct the location ID
        // TODO: This is a bit of a hack, but it works for now.
        let query = crate::DataReader::parse_queries::parse_to_expression_tree(&format!(
            "consistency: {}",
            string
        ))
        .unwrap()
        .remove(0);

        match query {
            QueryExpression::Consistency(x) => (*x).into(),
            _ => unreachable!(),
        }
    }

    /// Does an inorder walk of the [`LocationID`] tree mapping it to a list of [`LocationID::Simple`].
    pub fn inorder_vec_tranform(&self) -> Vec<Self> {
        match self {
            LocationID::Composition(left, right)
            | LocationID::Quotient(left, right)
            | LocationID::Conjunction(left, right) => {
                let mut left = left.inorder_vec_tranform();
                let mut right = right.inorder_vec_tranform();
                left.append(&mut right);
                left
            }
            LocationID::Simple(_) | LocationID::AnyLocation | LocationID::Special(_) => {
                vec![self.clone()]
            }
        }
    }

    /// It check whether the [`LocationID`] is a partial location by search through [`LocationID`] structure and see if there is any [`LocationID::AnyLocation`]
    pub fn is_partial_location(&self) -> bool {
        // TODO: Remove this function and implement it on a new PartialLocationID type
        match self {
            LocationID::Composition(left, right)
            | LocationID::Conjunction(left, right)
            | LocationID::Quotient(left, right) => {
                left.is_partial_location() || right.is_partial_location()
            }
            LocationID::Simple { .. } | LocationID::Special(_) => false,
            LocationID::AnyLocation => true,
        }
    }

    pub(super) fn get_unique_string(&self) -> String {
        match self {
            LocationID::Composition(a, b) => {
                format!("({}||{})", a.get_unique_string(), b.get_unique_string())
            }
            LocationID::Conjunction(a, b) => {
                format!("({}&&{})", a.get_unique_string(), b.get_unique_string())
            }
            LocationID::Quotient(a, b) => {
                format!("({}\\{})", a.get_unique_string(), b.get_unique_string())
            }
            LocationID::AnyLocation => "_".to_string(),
            LocationID::Simple(location_id) => location_id.clone(),
            LocationID::Special(location_id) => location_id.to_string(),
        }
    }
}

impl From<QueryExpression> for LocationID {
    fn from(item: QueryExpression) -> Self {
        match item {
            QueryExpression::Conjunction(left, right) => {
                LocationID::Conjunction(Box::new((*left).into()), Box::new((*right).into()))
            }
            QueryExpression::Composition(left, right) => {
                LocationID::Composition(Box::new((*left).into()), Box::new((*right).into()))
            }
            QueryExpression::Quotient(left, right) => {
                LocationID::Quotient(Box::new((*left).into()), Box::new((*right).into()))
            }
            QueryExpression::Parentheses(inner) => (*inner).into(),
            QueryExpression::VarName(name) => LocationID::Simple(name),
            _ => panic!(
                "Cannot convert queryexpression with {:?} to LocationID",
                item
            ),
        }
    }
}

impl Display for LocationID {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            LocationID::Conjunction(left, right) => {
                match **left {
                    LocationID::Conjunction(_, _) => write!(f, "{}", (*left))?,
                    LocationID::Simple(_) => write!(f, "{}", (*left))?,
                    _ => write!(f, "({})", (*left))?,
                };
                write!(f, "&&")?;
                match **right {
                    LocationID::Conjunction(_, _) => write!(f, "{}", (*right))?,
                    LocationID::Simple(_) => write!(f, "{}", (*right))?,
                    _ => write!(f, "({})", (*right))?,
                };
            }
            LocationID::Composition(left, right) => {
                match **left {
                    LocationID::Composition(_, _) => write!(f, "{}", (*left))?,
                    LocationID::Simple(_) => write!(f, "{}", (*left))?,
                    _ => write!(f, "({})", (*left))?,
                };
                write!(f, "||")?;
                match **right {
                    LocationID::Composition(_, _) => write!(f, "{}", (*right))?,
                    LocationID::Simple(_) => write!(f, "{}", (*right))?,
                    _ => write!(f, "({})", (*right))?,
                };
            }
            LocationID::Quotient(left, right) => {
                match **left {
                    LocationID::Simple(_) => write!(f, "{}", (*left))?,
                    _ => write!(f, "({})", (*left))?,
                };
                write!(f, "\\\\")?;
                match **right {
                    LocationID::Simple(_) => write!(f, "{}", (*right))?,
                    _ => write!(f, "({})", (*right))?,
                };
            }
            LocationID::Simple(location_id) => {
                write!(f, "{}", location_id)?;
            }
            LocationID::AnyLocation => write!(f, "_")?,
            LocationID::Special(location_id) => write!(f, "{}", location_id)?,
        }
        Ok(())
    }
}
