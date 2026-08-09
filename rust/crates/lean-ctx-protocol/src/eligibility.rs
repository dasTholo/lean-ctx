//! Static eligibility-policy contracts for controlled routing.
//!
//! The policy is an inspectable schema. Production calibration, candidate
//! ranking, and policy authoring remain in `lean-ctx-enterprise` (Class D).

use crate::{RiskClass, common::ValidationError};
use serde::{Deserialize, Serialize};

/// Static conditions that a task must satisfy before a control plane may use
/// an auto-routing decision.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EligibilityPolicy {
    pub min_evaluated_tasks: u64,
    pub min_confidence: f64,
    pub max_risk_class: RiskClass,
    #[serde(default)]
    pub allowed_regions: Vec<String>,
    #[serde(default)]
    pub allowed_providers: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required_task_class: Option<String>,
    pub quality_floor: f64,
}

impl Default for EligibilityPolicy {
    fn default() -> Self {
        Self {
            min_evaluated_tasks: 200,
            min_confidence: 0.8,
            max_risk_class: RiskClass::Medium,
            allowed_regions: Vec::new(),
            allowed_providers: Vec::new(),
            required_task_class: None,
            quality_floor: 0.99,
        }
    }
}

impl EligibilityPolicy {
    /// Validate policy bounds and list entries.
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.min_evaluated_tasks == 0 {
            return Err(ValidationError::new(
                "min_evaluated_tasks must be greater than zero",
            ));
        }
        validate_unit_interval(self.min_confidence, "min_confidence")?;
        validate_unit_interval(self.quality_floor, "quality_floor")?;
        validate_allowlist(&self.allowed_regions, "allowed_regions")?;
        validate_allowlist(&self.allowed_providers, "allowed_providers")?;
        if let Some(task_class) = &self.required_task_class {
            require_non_empty(task_class, "required_task_class")?;
        }
        Ok(())
    }
}

/// Static observations supplied to the deterministic reference evaluator.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EligibilityContext {
    pub evaluated_tasks: u64,
    pub confidence: f64,
    pub risk_class: RiskClass,
    pub region: String,
    pub provider: String,
    pub task_class: String,
    pub quality: f64,
}

/// Alias emphasizing that the context is an evaluator input, not a decision.
pub type EligibilityInput = EligibilityContext;

impl EligibilityContext {
    /// Validate static observation bounds.
    pub fn validate(&self) -> Result<(), ValidationError> {
        validate_unit_interval(self.confidence, "confidence")?;
        validate_unit_interval(self.quality, "quality")?;
        require_non_empty(&self.region, "region")?;
        require_non_empty(&self.provider, "provider")?;
        require_non_empty(&self.task_class, "task_class")
    }
}

/// Result of applying only the policy's static conditions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EligibilityResult {
    pub eligible: bool,
    pub reasons: Vec<String>,
    pub required_sample_size: u64,
}

impl EligibilityResult {
    /// Construct an eligible result with the policy's required sample size.
    pub fn eligible(required_sample_size: u64) -> Self {
        Self {
            eligible: true,
            reasons: Vec::new(),
            required_sample_size,
        }
    }
}

fn validate_allowlist(values: &[String], field: &str) -> Result<(), ValidationError> {
    for value in values {
        require_non_empty(value, &format!("{field} entry"))?;
    }
    Ok(())
}

fn require_non_empty(value: &str, field: &str) -> Result<(), ValidationError> {
    if value.trim().is_empty() {
        Err(ValidationError::new(format!("{field} must not be empty")))
    } else {
        Ok(())
    }
}

fn validate_unit_interval(value: f64, field: &str) -> Result<(), ValidationError> {
    if value.is_finite() && (0.0..=1.0).contains(&value) {
        Ok(())
    } else {
        Err(ValidationError::new(format!(
            "{field} must be finite and between 0 and 1"
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn policy_and_result_round_trip() {
        let policy = EligibilityPolicy {
            min_evaluated_tasks: 200,
            min_confidence: 0.8,
            max_risk_class: RiskClass::Medium,
            allowed_regions: vec!["eu".to_owned()],
            allowed_providers: vec!["provider-a".to_owned()],
            required_task_class: Some("coding".to_owned()),
            quality_floor: 0.99,
        };
        let context = EligibilityContext {
            evaluated_tasks: 250,
            confidence: 0.9,
            risk_class: RiskClass::Low,
            region: "eu".to_owned(),
            provider: "provider-a".to_owned(),
            task_class: "coding".to_owned(),
            quality: 1.0,
        };
        let result = EligibilityResult::eligible(policy.min_evaluated_tasks);

        let policy_json = serde_json::to_string(&policy).expect("policy serializes");
        let context_json = serde_json::to_string(&context).expect("context serializes");
        let result_json = serde_json::to_string(&result).expect("result serializes");
        assert_eq!(
            serde_json::from_str::<EligibilityPolicy>(&policy_json).unwrap(),
            policy
        );
        assert_eq!(
            serde_json::from_str::<EligibilityContext>(&context_json).unwrap(),
            context
        );
        assert_eq!(
            serde_json::from_str::<EligibilityResult>(&result_json).unwrap(),
            result
        );
    }
}
