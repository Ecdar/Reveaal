#[macro_use]
pub mod common;
pub mod composition;
pub mod conjunction;
pub mod quotient;
pub mod transition_system;

pub use composition::Composition;
pub use conjunction::Conjunction;
pub use conjunction::PrunedComponent;
pub use quotient::Quotient;
pub use transition_system::{
    CompositionType, LocationTuple, TransitionSystem, TransitionSystemPtr,
};
