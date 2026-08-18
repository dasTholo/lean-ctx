//! Dynamic reasoning budget (OSS stub).
//!
//! Enterprise adjusts thinking.budget_tokens / reasoning_effort based on task
//! complexity. OSS: no-op pass-through.

use crate::core::config::ReasoningBudgetConfig;

/// Applies reasoning budget adjustments to the request body.
/// OSS: no-op (returns without modification).
pub fn apply_reasoning_budget_with_config(
    _body: &mut serde_json::Value,
    _task_class: &str,
    _complexity: &str,
    _config: &ReasoningBudgetConfig,
) {
}
