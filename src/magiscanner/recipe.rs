use crate::magiscanner::dish::Dish;
use crate::magiscanner::operation::{ArgValue, Operation, OperationError};
use std::collections::HashMap;

/// One step in a recipe: an operation plus its configured arguments.
pub struct RecipeStep {
    pub operation: Box<dyn Operation>,
    pub args: HashMap<String, ArgValue>,
    pub disabled: bool,
}

/// A recipe is an ordered list of operations that execute sequentially.
/// Mirrors CyberChef's Recipe: takes a Dish, runs each step, passes
/// output of step N as input to step N+1.
pub struct Recipe {
    pub steps: Vec<RecipeStep>,
}

#[derive(Debug, thiserror::Error)]
pub enum RecipeError {
    #[error("step {step} ({name}): {source}")]
    StepFailed {
        step: usize,
        name: String,
        source: OperationError,
    },
}

impl Recipe {
    pub fn new() -> Self {
        Self { steps: Vec::new() }
    }

    pub fn add_step(&mut self, step: RecipeStep) {
        self.steps.push(step);
    }

    /// Execute the full recipe, returning the final output.
    /// Steps that fail are logged and skipped — the previous data carries forward.
    pub fn execute(&self, dish: Dish) -> Result<Dish, RecipeError> {
        let mut data = dish.into_bytes();
        for (i, step) in self.steps.iter().enumerate() {
            if step.disabled {
                continue;
            }
            let name = step.operation.meta().name.to_string();
            match step.operation.run(&data, &step.args) {
                Ok(output) => data = output,
                Err(e) => {
                    tracing::debug!(step = i, operation = name, error = %e, "recipe step failed, skipping");
                }
            }
        }
        Ok(Dish::new(data))
    }

    pub fn step_count(&self) -> usize {
        self.steps.len()
    }

    pub fn active_step_count(&self) -> usize {
        self.steps.iter().filter(|s| !s.disabled).count()
    }
}

impl Default for Recipe {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_recipe() {
        let recipe = Recipe::new();
        let dish = Dish::from_str("hello");
        let result = recipe.execute(dish).unwrap();
        assert_eq!(result.as_str().unwrap(), "hello");
    }
}
