//! Open trust primitives for identity, classification, policy, and evidence.
//!
//! This module intentionally contains portable data types and deterministic
//! verification only. Central identity lifecycle, policy administration,
//! tenant orchestration, and residency enforcement belong to the enterprise
//! control plane.

pub mod classification;
pub mod evidence_chain;
pub mod identity;
pub mod policy;
pub mod tenant_isolation;

pub use classification::{
    ClassificationPolicy, ClassificationRule, ClassifiedData, ResidencyRequirement,
    SecurityClassification,
};
pub use evidence_chain::{ChainVerificationResult, EvidenceChain, EvidenceEntry, verify_chain};
pub use identity::{
    AgentId, HumanActorId, IdentityContext, IdentityScope, ProjectId, ServiceId, TenantId,
};
pub use policy::{
    AllowAll, ClassificationBased, DenyAll, PolicyContext, PolicyDecision, PolicyEvaluator,
    PolicyRule,
};
pub use tenant_isolation::{CrossTenantLeak, IsolationCheck, TenantBoundary, assert_same_tenant};
