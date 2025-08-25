pub mod jupiter;
pub mod system;
pub mod compute_budget;
pub mod associated_token_account;

use crate::core::InstructionVisualizer;

/// Get all available visualizers
pub fn get_all_visualizers() -> Vec<Box<dyn InstructionVisualizer>> {
    vec![
        Box::new(jupiter::JupiterVisualizer),
        Box::new(system::SystemVisualizer),
        Box::new(compute_budget::ComputeBudgetVisualizer),
        Box::new(associated_token_account::AssociatedTokenAccountVisualizer),
    ]
}
