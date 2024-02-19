use crate::model_objects::expressions::SystemExpression;
use crate::model_objects::Component;
use crate::system::query_failures::{SyntaxResult, SystemRecipeFailure};
use crate::transition_systems::{
    CompiledComponent, Composition, Conjunction, Quotient, TransitionSystemPtr,
};
use crate::ComponentLoader;
use edbm::util::constraints::ClockIndex;
use log::debug;

#[derive(Clone, Debug)]
pub enum SystemRecipe {
    Composition(Box<SystemRecipe>, Box<SystemRecipe>),
    Conjunction(Box<SystemRecipe>, Box<SystemRecipe>),
    Quotient(Box<SystemRecipe>, Box<SystemRecipe>, ClockIndex),
    Component(Box<Component>),
}

impl SystemRecipe {
    pub fn compile(self, dim: ClockIndex) -> Result<TransitionSystemPtr, Box<SystemRecipeFailure>> {
        let mut component_index = 0;
        self._compile(dim + 1, &mut component_index)
    }

    pub fn compile_with_index(
        self,
        dim: ClockIndex,
        component_index: &mut u32,
    ) -> Result<TransitionSystemPtr, Box<SystemRecipeFailure>> {
        self._compile(dim + 1, component_index)
    }

    fn _compile(
        self,
        dim: ClockIndex,
        component_index: &mut u32,
    ) -> Result<TransitionSystemPtr, Box<SystemRecipeFailure>> {
        match self {
            SystemRecipe::Composition(left, right) => Composition::new_ts(
                left._compile(dim, component_index)?,
                right._compile(dim, component_index)?,
                dim,
            ),
            SystemRecipe::Conjunction(left, right) => Conjunction::new_ts(
                left._compile(dim, component_index)?,
                right._compile(dim, component_index)?,
                dim,
            ),
            SystemRecipe::Quotient(left, right, clock_index) => Quotient::new_ts(
                left._compile(dim, component_index)?,
                right._compile(dim, component_index)?,
                clock_index,
                dim,
            ),
            SystemRecipe::Component(comp) => {
                CompiledComponent::compile(*comp, dim, component_index)
                    .map(|comp| comp as TransitionSystemPtr)
            }
        }
    }

    /// Gets the number of `Components`s in the `SystemRecipe`
    pub fn get_component_count(&self) -> usize {
        match self {
            SystemRecipe::Composition(left, right)
            | SystemRecipe::Conjunction(left, right)
            | SystemRecipe::Quotient(left, right, _) => {
                left.get_component_count() + right.get_component_count()
            }
            SystemRecipe::Component(_) => 1,
        }
    }

    pub fn get_components(&self) -> Vec<&Component> {
        match self {
            SystemRecipe::Composition(left, right)
            | SystemRecipe::Conjunction(left, right)
            | SystemRecipe::Quotient(left, right, _) => {
                let mut o = left.get_components();
                o.extend(right.get_components());
                o
            }
            SystemRecipe::Component(c) => vec![c],
        }
    }
}

pub fn get_system_recipe(
    side: &SystemExpression,
    component_loader: &mut dyn ComponentLoader,
    clock_index: &mut ClockIndex,
    quotient_index: &mut Option<ClockIndex>,
) -> Result<Box<SystemRecipe>, SyntaxResult> {
    match side {
        SystemExpression::Composition(left, right) => Ok(Box::new(SystemRecipe::Composition(
            get_system_recipe(left, component_loader, clock_index, quotient_index)?,
            get_system_recipe(right, component_loader, clock_index, quotient_index)?,
        ))),
        SystemExpression::Conjunction(left, right) => Ok(Box::new(SystemRecipe::Conjunction(
            get_system_recipe(left, component_loader, clock_index, quotient_index)?,
            get_system_recipe(right, component_loader, clock_index, quotient_index)?,
        ))),
        SystemExpression::Quotient(left, right) => {
            let left = get_system_recipe(left, component_loader, clock_index, quotient_index)?;
            let right = get_system_recipe(right, component_loader, clock_index, quotient_index)?;

            let q_index = match quotient_index {
                Some(q_i) => *q_i,
                None => {
                    *clock_index += 1;
                    debug!("Quotient clock index: {}", *clock_index);

                    quotient_index.replace(*clock_index);
                    quotient_index.unwrap()
                }
            };

            Ok(Box::new(SystemRecipe::Quotient(left, right, q_index)))
        }
        SystemExpression::Component(name, id) => {
            let mut component = component_loader.get_component(name)?.clone();
            component.set_clock_indices(clock_index);
            component.special_id = id.clone();
            // Logic for locations
            debug!("{} Clocks: {:?}", name, component.declarations.clocks);

            Ok(Box::new(SystemRecipe::Component(Box::new(component))))
        }
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use crate::extract_system_rep::ExecutableQueryError;
    use crate::system::query_failures::{ActionFailure, SystemRecipeFailure};
    use crate::system::query_failures::{ConsistencyFailure, DeterminismFailure};
    use crate::test_helpers::json_run_query;

    const COMPILED_COMP_PATH: &str = "samples/json/SystemRecipe/CompiledComponent";
    const COMPOSITION_PATH: &str = "samples/json/SystemRecipe/Composition";
    const CONJUNCTION_PATH: &str = "samples/json/SystemRecipe/Conjunction";
    const QUOTIENT_PATH: &str = "samples/json/SystemRecipe/Quotient";

    #[test]
    fn compiled_component1_fails_correctly() {
        let actual =
            json_run_query(COMPILED_COMP_PATH, "consistency: CompiledComponent1").unwrap_err();
        assert!(matches!(
            actual,
            ExecutableQueryError::SystemRecipeFailure(SystemRecipeFailure::Action(
                ActionFailure::NotDisjoint(_, _),
                _
            ))
        ));
    }

    #[test]
    fn compiled_component1_fails_with_correct_actions() {
        let expected_actions: HashSet<_> = HashSet::from(["Input".to_string()]);
        if let Some(ExecutableQueryError::SystemRecipeFailure(SystemRecipeFailure::Action(
            ActionFailure::NotDisjoint(left, right),
            _,
        ))) = json_run_query(COMPILED_COMP_PATH, "consistency: CompiledComponent1").err()
        {
            assert_eq!(
                left.actions
                    .intersection(&right.actions)
                    .cloned()
                    .collect::<HashSet<_>>(),
                expected_actions
            );
        } else {
            panic!("Models in samples/action have been changed, REVERT!");
        }
    }

    #[test]
    fn compiled_component2_fails_correctly() {
        let actual =
            json_run_query(COMPILED_COMP_PATH, "consistency: CompiledComponent2").unwrap_err();
        assert!(matches!(
            actual,
            ExecutableQueryError::SystemRecipeFailure(SystemRecipeFailure::Action(
                ActionFailure::NotDisjoint(_, _),
                _
            ))
        ));
    }

    #[test]
    fn compiled_component2_fails_with_correct_actions() {
        let expected_actions: HashSet<_> = HashSet::from(["Input1".to_string()]);
        if let Some(ExecutableQueryError::SystemRecipeFailure(SystemRecipeFailure::Action(
            ActionFailure::NotDisjoint(left, right),
            _,
        ))) = json_run_query(COMPILED_COMP_PATH, "consistency: CompiledComponent2").err()
        {
            assert_eq!(
                left.actions
                    .intersection(&right.actions)
                    .cloned()
                    .collect::<HashSet<_>>(),
                expected_actions
            );
        } else {
            panic!("Models in samples/action have been changed, REVERT!");
        }
    }

    #[test]
    fn compiled_component3_fails_correctly() {
        let actual =
            json_run_query(COMPILED_COMP_PATH, "consistency: CompiledComponent3").unwrap_err();
        assert!(matches!(
            actual,
            ExecutableQueryError::SystemRecipeFailure(SystemRecipeFailure::Action(
                ActionFailure::NotDisjoint(_, _),
                _
            ))
        ));
    }

    #[test]
    fn compiled_component3_fails_with_correct_actions() {
        let expected_actions: HashSet<_> =
            HashSet::from(["Input1".to_string(), "Input2".to_string()]);
        if let Some(ExecutableQueryError::SystemRecipeFailure(SystemRecipeFailure::Action(
            ActionFailure::NotDisjoint(left, right),
            _,
        ))) = json_run_query(COMPILED_COMP_PATH, "consistency: CompiledComponent3").err()
        {
            assert_eq!(
                left.actions
                    .intersection(&right.actions)
                    .cloned()
                    .collect::<HashSet<_>>(),
                expected_actions
            );
        } else {
            panic!("Models in samples/action have been changed, REVERT!");
        }
    }

    #[test]
    fn compostion1_fails_correctly() {
        let actual = json_run_query(
            COMPOSITION_PATH,
            "consistency: LeftComposition1 || RightComposition1",
        )
        .unwrap_err();
        assert!(matches!(
            actual,
            ExecutableQueryError::SystemRecipeFailure(SystemRecipeFailure::Action(
                ActionFailure::NotDisjoint(_, _),
                _
            ))
        ));
    }

    #[test]
    fn composition1_fails_with_correct_actions() {
        let expected_actions = HashSet::from(["Output1".to_string()]);
        if let Some(ExecutableQueryError::SystemRecipeFailure(SystemRecipeFailure::Action(
            ActionFailure::NotDisjoint(left, right),
            _,
        ))) = json_run_query(
            COMPOSITION_PATH,
            "consistency: LeftComposition1 || RightComposition1",
        )
        .err()
        {
            assert_eq!(
                left.actions
                    .intersection(&right.actions)
                    .cloned()
                    .collect::<HashSet<_>>(),
                expected_actions
            );
        } else {
            panic!("Models in samples/action have been changed, REVERT!");
        }
    }

    #[test]
    fn compostion2_fails_correctly() {
        let actual = json_run_query(
            COMPOSITION_PATH,
            "consistency: LeftComposition2 || RightComposition2",
        )
        .unwrap_err();
        assert!(matches!(
            actual,
            ExecutableQueryError::SystemRecipeFailure(SystemRecipeFailure::Action(
                ActionFailure::NotDisjoint(_, _),
                _
            ))
        ));
    }

    #[test]
    fn composition2_fails_with_correct_actions() {
        let expected_actions = HashSet::from(["Output1".to_string(), "Output2".to_string()]);
        if let Some(ExecutableQueryError::SystemRecipeFailure(SystemRecipeFailure::Action(
            ActionFailure::NotDisjoint(left, right),
            _,
        ))) = json_run_query(
            COMPOSITION_PATH,
            "consistency: LeftComposition2 || RightComposition2",
        )
        .err()
        {
            assert_eq!(
                left.actions
                    .intersection(&right.actions)
                    .cloned()
                    .collect::<HashSet<_>>(),
                expected_actions
            );
        } else {
            panic!("Models in samples/action have been changed, REVERT!");
        }
    }

    #[test]
    fn compostion3_fails_correctly() {
        let actual = json_run_query(
            COMPOSITION_PATH,
            "consistency: LeftComposition3 || RightComposition3",
        )
        .unwrap_err();
        assert!(matches!(
            actual,
            ExecutableQueryError::SystemRecipeFailure(SystemRecipeFailure::Action(
                ActionFailure::NotDisjoint(_, _),
                _
            ))
        ));
    }

    #[test]
    fn composition3_fails_with_correct_actions() {
        let expected_actions = HashSet::from(["Output2".to_string()]);
        if let Some(ExecutableQueryError::SystemRecipeFailure(SystemRecipeFailure::Action(
            ActionFailure::NotDisjoint(left, right),
            _,
        ))) = json_run_query(
            COMPOSITION_PATH,
            "consistency: LeftComposition3 || RightComposition3",
        )
        .err()
        {
            assert_eq!(
                left.actions
                    .intersection(&right.actions)
                    .cloned()
                    .collect::<HashSet<_>>(),
                expected_actions
            );
        } else {
            panic!("Models in samples/action have been changed, REVERT!");
        }
    }

    #[test]
    fn conjunction1_fails_correctly() {
        let actual = json_run_query(
            CONJUNCTION_PATH,
            "consistency: LeftConjunction1 && RightConjunction1",
        )
        .unwrap_err();
        assert!(matches!(
            actual,
            ExecutableQueryError::SystemRecipeFailure(SystemRecipeFailure::Action(
                ActionFailure::NotDisjoint(_, _),
                _
            ))
        ));
    }

    #[test]
    fn conjunction1_fails_with_correct_actions() {
        let expected_actions = HashSet::from(["Input1".to_string()]); // Assuming inputs are checked first
        if let Some(ExecutableQueryError::SystemRecipeFailure(SystemRecipeFailure::Action(
            ActionFailure::NotDisjoint(left, right),
            _,
        ))) = json_run_query(
            CONJUNCTION_PATH,
            "consistency: LeftConjunction1 && RightConjunction1",
        )
        .err()
        {
            assert_eq!(
                left.actions
                    .intersection(&right.actions)
                    .cloned()
                    .collect::<HashSet<_>>(),
                expected_actions
            );
        } else {
            panic!("Models in samples/action have been changed, REVERT!");
        }
    }

    #[test]
    fn conjunction2_fails_correctly() {
        let actual = json_run_query(
            CONJUNCTION_PATH,
            "consistency: LeftConjunction2 && RightConjunction2",
        )
        .unwrap_err();
        assert!(matches!(
            actual,
            ExecutableQueryError::SystemRecipeFailure(SystemRecipeFailure::Action(
                ActionFailure::NotDisjoint(_, _),
                _
            ))
        ));
    }

    #[test]
    fn conjunction2_fails_with_correct_actions() {
        let expected_actions = HashSet::from(["Input1".to_string()]);
        if let Some(ExecutableQueryError::SystemRecipeFailure(SystemRecipeFailure::Action(
            ActionFailure::NotDisjoint(left, right),
            _,
        ))) = json_run_query(
            CONJUNCTION_PATH,
            "consistency: LeftConjunction2 && RightConjunction2",
        )
        .err()
        {
            assert_eq!(
                left.actions
                    .intersection(&right.actions)
                    .cloned()
                    .collect::<HashSet<_>>(),
                expected_actions
            );
        } else {
            panic!("Models in samples/action have been changed, REVERT!");
        }
    }

    #[test]
    fn conjunction3_fails_correctly() {
        let actual = json_run_query(
            CONJUNCTION_PATH,
            "consistency: LeftConjunction3 && RightConjunction3",
        )
        .unwrap_err();
        assert!(matches!(
            actual,
            ExecutableQueryError::SystemRecipeFailure(SystemRecipeFailure::Action(
                ActionFailure::NotDisjoint(_, _),
                _
            ))
        ));
    }

    #[test]
    fn conjunction3_fails_with_correct_actions() {
        let expected_actions = HashSet::from(["Output1".to_string()]);
        if let Some(ExecutableQueryError::SystemRecipeFailure(SystemRecipeFailure::Action(
            ActionFailure::NotDisjoint(left, right),
            _,
        ))) = json_run_query(
            CONJUNCTION_PATH,
            "consistency: LeftConjunction3 && RightConjunction3",
        )
        .err()
        {
            assert_eq!(
                left.actions
                    .intersection(&right.actions)
                    .cloned()
                    .collect::<HashSet<_>>(),
                expected_actions
            );
        } else {
            panic!("Models in samples/action have been changed, REVERT!");
        }
    }

    #[test]
    fn quotient1_fails_correctly() {
        let actual = json_run_query(
            QUOTIENT_PATH,
            "consistency: LeftQuotient1 // RightQuotient1",
        )
        .unwrap_err();
        assert!(matches!(
            actual,
            ExecutableQueryError::SystemRecipeFailure(SystemRecipeFailure::Action(
                ActionFailure::NotDisjoint(_, _),
                _
            ))
        ));
    }

    #[test]
    fn quotient1_fails_with_correct_actions() {
        let expected_actions = HashSet::from(["Input1".to_string()]);
        if let Some(ExecutableQueryError::SystemRecipeFailure(SystemRecipeFailure::Action(
            ActionFailure::NotDisjoint(left, right),
            _,
        ))) = json_run_query(
            QUOTIENT_PATH,
            "consistency: LeftQuotient1 // RightQuotient1",
        )
        .err()
        {
            assert_eq!(
                left.actions
                    .intersection(&right.actions)
                    .cloned()
                    .collect::<HashSet<_>>(),
                expected_actions
            );
        } else {
            panic!("Models in samples/action have been changed, REVERT!");
        }
    }

    #[test]
    fn left_quotient_fails_correctly() {
        let actual = json_run_query(
            QUOTIENT_PATH,
            "consistency: NotDeterministicQuotientComp // DeterministicQuotientComp",
        )
        .unwrap_err();
        println!("{:?}", actual);
        assert!(matches!(
            actual,
            ExecutableQueryError::SystemRecipeFailure(SystemRecipeFailure::Inconsistent(
                ConsistencyFailure::NotDeterministic(DeterminismFailure { .. }),
                _
            ))
        ));
    }

    #[test]
    fn right_quotient_fails_correctly() {
        let actual = json_run_query(
            QUOTIENT_PATH,
            "consistency: DeterministicQuotientComp // NotDeterministicQuotientComp",
        )
        .unwrap_err();
        println!("{:?}", actual);
        assert!(matches!(
            actual,
            ExecutableQueryError::SystemRecipeFailure(SystemRecipeFailure::Inconsistent(
                ConsistencyFailure::NotDeterministic(DeterminismFailure { .. }),
                _
            ))
        ));
    }
}
