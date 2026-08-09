//! Public `model=auto` wire contracts.
//!
//! These types describe the boundary between the OSS Runtime and a routing
//! control plane. The production decision intelligence that produces a
//! decision remains in `lean-ctx-enterprise` (Class D); this module contains
//! only transportable data and structural validation.

use crate::common::ValidationError;
use serde::{Deserialize, Serialize};

/// Requested routing behavior at the public client or gateway boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutingMode {
    /// Use the explicitly requested model without auto-routing.
    Explicit,
    /// Permit an external control plane to provide an auto decision.
    Auto,
    /// Produce or transport a decision for observation without changing use.
    Shadow,
}

/// Runtime configuration for the public `model=auto` surface.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AutoRoutingConfig {
    /// Whether auto-routing is accepted at the public boundary.
    pub enabled: bool,
    /// Model used when an auto decision is unavailable or rejected.
    pub fallback_model: String,
    /// Provider used with [`Self::fallback_model`].
    pub fallback_provider: String,
    /// Task classes for which the boundary may accept auto-routing.
    #[serde(default)]
    pub eligible_task_classes: Vec<String>,
}

impl Default for AutoRoutingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            fallback_model: "baseline".to_owned(),
            fallback_provider: "baseline".to_owned(),
            eligible_task_classes: Vec::new(),
        }
    }
}

impl AutoRoutingConfig {
    /// Validate structural configuration invariants.
    pub fn validate(&self) -> Result<(), ValidationError> {
        require_non_empty(&self.fallback_model, "fallback_model")?;
        require_non_empty(&self.fallback_provider, "fallback_provider")?;
        for task_class in &self.eligible_task_classes {
            require_non_empty(task_class, "eligible_task_classes entry")?;
        }
        Ok(())
    }

    /// Return whether a task class is present in the static allowlist.
    pub fn allows_task_class(&self, task_class: &str) -> bool {
        self.enabled
            && self
                .eligible_task_classes
                .iter()
                .any(|candidate| candidate == task_class)
    }
}

/// Evidence thresholds that a shadow scheduler must satisfy before auto-routing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SchedulerGate {
    pub min_evaluated_tasks: u64,
    pub max_outcome_degradation_pct: f64,
    pub min_cost_improvement_pct: f64,
    pub require_holdout_evidence: bool,
}

/// Observed shadow-scheduler performance used to evaluate [`SchedulerGate`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AutoRoutingEvidence {
    pub evaluated_tasks: u64,
    pub outcome_degradation_pct: f64,
    pub cost_improvement_pct: f64,
    pub has_holdout_evidence: bool,
}

/// Request envelope sent to an auto-routing boundary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AutoRoutingRequest {
    pub task_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requested_model: Option<String>,
    pub routing_mode: RoutingMode,
}

impl AutoRoutingRequest {
    /// Validate request identity and explicit-mode requirements.
    pub fn validate(&self) -> Result<(), ValidationError> {
        require_non_empty(&self.task_id, "task_id")?;
        if let Some(model) = &self.requested_model {
            require_non_empty(model, "requested_model")?;
        }
        if self.routing_mode == RoutingMode::Explicit && self.requested_model.is_none() {
            return Err(ValidationError::new(
                "explicit routing requires requested_model",
            ));
        }
        Ok(())
    }
}

/// Candidate selected by a control plane, with a portable fallback reference.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoutingDecision {
    pub selected_model: String,
    pub selected_provider: String,
    pub rationale_code: String,
    pub fallback_plan: String,
    /// Calibrated confidence supplied by the decision producer, in [0, 1].
    pub confidence: f64,
}

impl RoutingDecision {
    /// Validate fields without interpreting or recomputing the decision.
    pub fn validate(&self) -> Result<(), ValidationError> {
        require_non_empty(&self.selected_model, "selected_model")?;
        require_non_empty(&self.selected_provider, "selected_provider")?;
        require_non_empty(&self.rationale_code, "rationale_code")?;
        require_non_empty(&self.fallback_plan, "fallback_plan")?;
        validate_unit_interval(self.confidence, "confidence")
    }
}

/// Receipt fields that preserve the requested and selected model lineage.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoutingReceipt {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requested_model: Option<String>,
    pub baseline_model: String,
    pub selected_model: String,
    pub decision_ref: String,
    pub method_version: String,
}

impl RoutingReceipt {
    /// Validate receipt lineage fields.
    pub fn validate(&self) -> Result<(), ValidationError> {
        if let Some(model) = &self.requested_model {
            require_non_empty(model, "requested_model")?;
        }
        require_non_empty(&self.baseline_model, "baseline_model")?;
        require_non_empty(&self.selected_model, "selected_model")?;
        require_non_empty(&self.decision_ref, "decision_ref")?;
        require_non_empty(&self.method_version, "method_version")
    }
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
    fn wire_types_round_trip() {
        let config = AutoRoutingConfig {
            enabled: true,
            fallback_model: "baseline-model".to_owned(),
            fallback_provider: "baseline-provider".to_owned(),
            eligible_task_classes: vec!["coding".to_owned()],
        };
        let request = AutoRoutingRequest {
            task_id: "task-1".to_owned(),
            requested_model: Some("requested-model".to_owned()),
            routing_mode: RoutingMode::Auto,
        };
        let decision = RoutingDecision {
            selected_model: "selected-model".to_owned(),
            selected_provider: "selected-provider".to_owned(),
            rationale_code: "reference".to_owned(),
            fallback_plan: "baseline-plan".to_owned(),
            confidence: 0.8,
        };
        let receipt = RoutingReceipt {
            requested_model: request.requested_model.clone(),
            baseline_model: config.fallback_model.clone(),
            selected_model: decision.selected_model.clone(),
            decision_ref: "decision-1".to_owned(),
            method_version: "v1".to_owned(),
        };

        let config_json = serde_json::to_string(&config).expect("config serializes");
        let request_json = serde_json::to_string(&request).expect("request serializes");
        let decision_json = serde_json::to_string(&decision).expect("decision serializes");
        let receipt_json = serde_json::to_string(&receipt).expect("receipt serializes");
        assert_eq!(
            serde_json::from_str::<AutoRoutingConfig>(&config_json).unwrap(),
            config
        );
        assert_eq!(
            serde_json::from_str::<AutoRoutingRequest>(&request_json).unwrap(),
            request
        );
        assert_eq!(
            serde_json::from_str::<RoutingDecision>(&decision_json).unwrap(),
            decision
        );
        assert_eq!(
            serde_json::from_str::<RoutingReceipt>(&receipt_json).unwrap(),
            receipt
        );
        config.validate().expect("valid config");
        request.validate().expect("valid request");
        decision.validate().expect("valid decision");
        receipt.validate().expect("valid receipt");
    }

    #[test]
    fn structural_validation_rejects_invalid_values() {
        let request = AutoRoutingRequest {
            task_id: "task-1".to_owned(),
            requested_model: None,
            routing_mode: RoutingMode::Explicit,
        };
        assert!(request.validate().is_err());

        let decision = RoutingDecision {
            selected_model: "model".to_owned(),
            selected_provider: "provider".to_owned(),
            rationale_code: "reason".to_owned(),
            fallback_plan: "baseline".to_owned(),
            confidence: 1.1,
        };
        assert!(decision.validate().is_err());
    }
}
