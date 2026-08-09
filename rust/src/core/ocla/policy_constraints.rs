//! Hard, public policy constraints applied before scheduler selection.
//!
//! These constraints are deliberately limited to policy facts. They do not
//! contain customer data, rates, performance observations, or learned weights.

use lean_ctx_protocol::DataClassification;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::scheduler_service::ExecutionCandidate;

/// Hard policy limits for a public candidate set.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyConstraints {
    pub allowed_regions: Option<Vec<String>>,
    pub allowed_providers: Option<Vec<String>>,
    pub allowed_classifications: Option<Vec<DataClassification>>,
    pub max_cost_micros: Option<u64>,
    pub min_quality_milli: Option<u32>,
    pub max_latency_ms: Option<u64>,
    pub require_local_execution: bool,
    pub require_reversible: bool,
}

/// Reason a candidate failed a hard policy check.
#[derive(Clone, Debug, Eq, PartialEq, Error, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PolicyViolation {
    #[error("provider is not allowed: {provider}")]
    ProviderNotAllowed { provider: String },
    #[error("candidate has no public region metadata")]
    RegionMetadataMissing,
    #[error("region is not allowed: {region}")]
    RegionNotAllowed { region: String },
    #[error("candidate has no public classification metadata")]
    ClassificationMetadataMissing,
    #[error("classification is not allowed: {classification:?}")]
    ClassificationNotAllowed { classification: DataClassification },
    #[error("candidate has no public cost estimate")]
    CostMetadataMissing,
    #[error("expected cost exceeds policy maximum: {actual} > {maximum}")]
    CostExceeded { actual: u64, maximum: u64 },
    #[error("candidate has no public quality estimate")]
    QualityMetadataMissing,
    #[error("expected quality is below policy minimum: {actual} < {minimum}")]
    QualityBelowMinimum { actual: u32, minimum: u32 },
    #[error("candidate has no public latency estimate")]
    LatencyMetadataMissing,
    #[error("expected latency exceeds policy maximum: {actual} > {maximum}")]
    LatencyExceeded { actual: u64, maximum: u64 },
    #[error("candidate is not marked as local execution")]
    LocalExecutionRequired,
    #[error("candidate is not marked as reversible")]
    ReversibleExecutionRequired,
}

impl PolicyConstraints {
    /// Check all hard constraints for one candidate.
    ///
    /// Region and execution-property hints use an explicit, public reference
    /// convention: providers may be written as `provider@region`, and a plan's
    /// policy reference may contain `classification:<name>`,
    /// `execution:local`, or `reversible:true`. If a policy requires such a
    /// fact but the candidate does not publish it, the candidate is rejected.
    pub fn permits(&self, candidate: &ExecutionCandidate) -> Result<(), PolicyViolation> {
        if let Some(allowed) = &self.allowed_providers
            && !allowed
                .iter()
                .any(|provider| provider == &candidate.provider)
        {
            return Err(PolicyViolation::ProviderNotAllowed {
                provider: candidate.provider.clone(),
            });
        }

        if let Some(allowed) = &self.allowed_regions {
            let regions = candidate_regions(candidate);
            if regions.is_empty() {
                return Err(PolicyViolation::RegionMetadataMissing);
            }
            if !allowed
                .iter()
                .any(|allowed_region| regions.iter().any(|region| region == allowed_region))
            {
                return Err(PolicyViolation::RegionNotAllowed {
                    region: regions.join(","),
                });
            }
        }

        if let Some(allowed) = &self.allowed_classifications {
            let classification = candidate_classification(candidate)
                .ok_or(PolicyViolation::ClassificationMetadataMissing)?;
            if !allowed
                .iter()
                .any(|allowed_classification| allowed_classification == &classification)
            {
                return Err(PolicyViolation::ClassificationNotAllowed { classification });
            }
        }

        if let Some(maximum) = self.max_cost_micros {
            let cost = candidate
                .expected_cost_micros
                .ok_or(PolicyViolation::CostMetadataMissing)?;
            if cost > maximum {
                return Err(PolicyViolation::CostExceeded {
                    actual: cost,
                    maximum,
                });
            }
        }

        if let Some(minimum) = self.min_quality_milli {
            let quality = candidate
                .expected_quality_milli
                .ok_or(PolicyViolation::QualityMetadataMissing)?;
            if quality < minimum {
                return Err(PolicyViolation::QualityBelowMinimum {
                    actual: quality,
                    minimum,
                });
            }
        }

        if let Some(maximum) = self.max_latency_ms {
            let latency = candidate
                .expected_latency_ms
                .ok_or(PolicyViolation::LatencyMetadataMissing)?;
            if latency > maximum {
                return Err(PolicyViolation::LatencyExceeded {
                    actual: latency,
                    maximum,
                });
            }
        }

        if self.require_local_execution && !candidate_is_local(candidate) {
            return Err(PolicyViolation::LocalExecutionRequired);
        }
        if self.require_reversible && !candidate_is_reversible(candidate) {
            return Err(PolicyViolation::ReversibleExecutionRequired);
        }
        Ok(())
    }
}

fn candidate_regions(candidate: &ExecutionCandidate) -> Vec<String> {
    let provider_region = candidate
        .provider
        .split_once('@')
        .map(|(_, region)| region.to_owned())
        .filter(|region| !region.is_empty());
    let reference_regions = policy_reference_tokens(candidate).find_map(|token| {
        token
            .strip_prefix("regions:")
            .map(|regions| regions.split(',').filter(|region| !region.is_empty()))
    });
    let mut regions = provider_region.into_iter().collect::<Vec<_>>();
    if let Some(reference_regions) = reference_regions {
        regions.extend(reference_regions.map(str::to_owned));
    }
    regions.sort();
    regions.dedup();
    regions
}

fn policy_reference_tokens(candidate: &ExecutionCandidate) -> impl Iterator<Item = &str> {
    candidate
        .plan
        .policy_decision_ref
        .as_deref()
        .into_iter()
        .flat_map(|reference| reference.split([';', '|']))
}

fn candidate_classification(candidate: &ExecutionCandidate) -> Option<DataClassification> {
    let value = policy_reference_tokens(candidate)
        .find_map(|token| token.strip_prefix("classification:"))
        .or_else(|| {
            policy_reference_tokens(candidate)
                .find_map(|token| token.strip_prefix("classifications:"))
                .and_then(|values| values.split(',').next())
        })?;
    match value {
        "public" => Some(DataClassification::Public),
        "internal" => Some(DataClassification::Internal),
        "confidential" => Some(DataClassification::Confidential),
        "restricted" => Some(DataClassification::Restricted),
        _ => None,
    }
}

fn candidate_is_local(candidate: &ExecutionCandidate) -> bool {
    candidate.provider == "local"
        || candidate.provider.starts_with("local:")
        || policy_reference_tokens(candidate).any(|token| token == "execution:local")
}

fn candidate_is_reversible(candidate: &ExecutionCandidate) -> bool {
    policy_reference_tokens(candidate).any(|token| token == "reversible:true")
        || candidate
            .plan
            .fallback_refs
            .iter()
            .any(|reference| reference == "reversible")
}
