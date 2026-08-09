//! Small policy-evaluation primitives.

use serde::{Deserialize, Serialize};

use super::classification::{ClassifiedData, SecurityClassification};
use super::identity::IdentityContext;

/// Result of a local policy evaluation.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum PolicyDecision {
    /// The requested action may proceed.
    Allow,
    /// The requested action must not proceed.
    Deny,
    /// The action may proceed only with an audit/evidence record.
    Audit,
}

/// A portable rule descriptor. Enterprise systems may attach richer rule
/// languages while preserving these interoperable fields.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PolicyRule {
    /// Condition such as `classification>=confidential`, `always`, or `*`.
    pub condition: String,
    /// Exact action or prefix wildcard to which the rule applies.
    pub action: String,
    /// Whether a matching action must produce evidence.
    pub evidence_required: bool,
}

impl PolicyRule {
    /// Creates a rule descriptor.
    #[must_use]
    pub fn new(
        condition: impl Into<String>,
        action: impl Into<String>,
        evidence_required: bool,
    ) -> Self {
        Self {
            condition: condition.into(),
            action: action.into(),
            evidence_required,
        }
    }
}

/// Context projection consumed by policy evaluators.
pub trait PolicyContext {
    /// Returns a classification when the context carries one.
    fn classification(&self) -> Option<SecurityClassification>;

    /// Returns an identity when the context carries one.
    fn identity(&self) -> Option<&IdentityContext>;
}

impl PolicyContext for IdentityContext {
    fn classification(&self) -> Option<SecurityClassification> {
        None
    }

    fn identity(&self) -> Option<&IdentityContext> {
        Some(self)
    }
}

impl PolicyContext for SecurityClassification {
    fn classification(&self) -> Option<SecurityClassification> {
        Some(*self)
    }

    fn identity(&self) -> Option<&IdentityContext> {
        None
    }
}

impl<T> PolicyContext for ClassifiedData<T> {
    fn classification(&self) -> Option<SecurityClassification> {
        Some(self.classification)
    }

    fn identity(&self) -> Option<&IdentityContext> {
        None
    }
}

/// Interface implemented by local or enterprise policy evaluators.
pub trait PolicyEvaluator {
    /// Evaluates `action` against the supplied portable context.
    fn evaluate(&self, context: &dyn PolicyContext, action: &str) -> PolicyDecision;
}

/// Reference evaluator that allows every action.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AllowAll;

impl PolicyEvaluator for AllowAll {
    fn evaluate(&self, _context: &dyn PolicyContext, _action: &str) -> PolicyDecision {
        PolicyDecision::Allow
    }
}

/// Reference evaluator that denies every action.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DenyAll;

impl PolicyEvaluator for DenyAll {
    fn evaluate(&self, _context: &dyn PolicyContext, _action: &str) -> PolicyDecision {
        PolicyDecision::Deny
    }
}

/// Deterministic policy evaluator that applies a maximum classification and
/// optional evidence rules.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ClassificationBased {
    /// Highest classification allowed without a deny decision.
    pub maximum_allowed: SecurityClassification,
    /// Additional action/evidence rules.
    #[serde(default)]
    pub rules: Vec<PolicyRule>,
}

impl ClassificationBased {
    /// Creates a classification gate with the supplied maximum level.
    #[must_use]
    pub fn new(maximum_allowed: SecurityClassification) -> Self {
        Self {
            maximum_allowed,
            rules: Vec::new(),
        }
    }

    /// Creates a gate with an initial rule set.
    #[must_use]
    pub fn from_rules(maximum_allowed: SecurityClassification, rules: Vec<PolicyRule>) -> Self {
        Self {
            maximum_allowed,
            rules,
        }
    }

    /// Adds a rule in builder style.
    #[must_use]
    pub fn with_rule(mut self, rule: PolicyRule) -> Self {
        self.rules.push(rule);
        self
    }

    /// Adds a rule to the evaluator.
    pub fn add_rule(&mut self, rule: PolicyRule) {
        self.rules.push(rule);
    }
}

impl PolicyEvaluator for ClassificationBased {
    fn evaluate(&self, context: &dyn PolicyContext, action: &str) -> PolicyDecision {
        let Some(classification) = context.classification() else {
            // A classification gate cannot prove safety for an unclassified
            // context, so it requests an auditable decision.
            return PolicyDecision::Audit;
        };

        if classification > self.maximum_allowed {
            return PolicyDecision::Deny;
        }

        for rule in &self.rules {
            if action_matches(&rule.action, action)
                && condition_matches(&rule.condition, classification)
                && rule.evidence_required
            {
                return PolicyDecision::Audit;
            }
        }
        PolicyDecision::Allow
    }
}

fn action_matches(pattern: &str, action: &str) -> bool {
    let pattern = pattern.trim();
    pattern == "*"
        || pattern.eq_ignore_ascii_case(action.trim())
        || pattern
            .strip_suffix('*')
            .is_some_and(|prefix| action.trim().starts_with(prefix))
}

fn condition_matches(condition: &str, classification: SecurityClassification) -> bool {
    let condition = condition.trim();
    if condition.is_empty() || condition == "*" || condition.eq_ignore_ascii_case("always") {
        return true;
    }

    let (operator, value) = if let Some(value) = condition.strip_prefix("classification>=") {
        (">=", value)
    } else if let Some(value) = condition.strip_prefix("classification<=") {
        ("<=", value)
    } else if let Some(value) = condition.strip_prefix("classification=") {
        ("=", value)
    } else if let Some(value) = condition.strip_prefix("classification:") {
        ("=", value)
    } else {
        ("=", condition)
    };

    let Some(required) = SecurityClassification::parse(value) else {
        return false;
    };
    match operator {
        ">=" => classification >= required,
        "<=" => classification <= required,
        "=" => classification == required,
        _ => false,
    }
}
