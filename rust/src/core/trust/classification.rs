//! Data classification and residency metadata.

use serde::{Deserialize, Serialize};

use super::identity::TenantId;
use super::tenant_isolation::TenantBoundary;

/// Ordered sensitivity levels. Higher levels must not flow to lower-trust
/// destinations without an explicit policy decision.
#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[repr(u8)]
pub enum SecurityClassification {
    /// Intended for unrestricted disclosure.
    #[default]
    Public = 0,
    /// Non-public operational information.
    Internal = 1,
    /// Information requiring an authorized recipient.
    Confidential = 2,
    /// Information requiring explicit need-to-know controls.
    Restricted = 3,
    /// Highest sensitivity; enterprise controls must explicitly authorize use.
    TopSecret = 4,
}

impl SecurityClassification {
    /// Returns the stable numeric ordering used by policy evaluators.
    #[must_use]
    pub const fn rank(self) -> u8 {
        self as u8
    }

    /// Returns whether data at this level may flow to `destination`.
    #[must_use]
    pub const fn may_flow_to(self, destination: Self) -> bool {
        self.rank() <= destination.rank()
    }

    /// Parses the stable case-insensitive name used in policy conditions.
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "public" => Some(Self::Public),
            "internal" => Some(Self::Internal),
            "confidential" => Some(Self::Confidential),
            "restricted" => Some(Self::Restricted),
            "topsecret" | "top_secret" | "top-secret" => Some(Self::TopSecret),
            _ => None,
        }
    }
}

/// Region constraints attached to classified data.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct ResidencyRequirement {
    /// Regions in which the data may be stored or processed. Empty means
    /// unrestricted unless `data_sovereign` is true.
    #[serde(default)]
    pub allowed_regions: Vec<String>,
    /// Regions that are forbidden even when they appear in the allow list.
    #[serde(default)]
    pub prohibited_regions: Vec<String>,
    /// Requires a sovereign deployment and therefore a non-empty allow list.
    #[serde(default)]
    pub data_sovereign: bool,
}

impl ResidencyRequirement {
    /// Creates an unconstrained residency requirement.
    #[must_use]
    pub fn unrestricted() -> Self {
        Self::default()
    }

    /// Creates a requirement allowing only the supplied regions.
    #[must_use]
    pub fn allowed_in<I, S>(regions: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            allowed_regions: regions.into_iter().map(Into::into).collect(),
            ..Self::default()
        }
    }

    /// Creates a sovereign requirement for the supplied regions.
    #[must_use]
    pub fn sovereign<I, S>(regions: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            allowed_regions: regions.into_iter().map(Into::into).collect(),
            data_sovereign: true,
            ..Self::default()
        }
    }

    /// Adds one allowed region.
    #[must_use]
    pub fn with_allowed_region(mut self, region: impl Into<String>) -> Self {
        self.allowed_regions.push(region.into());
        self
    }

    /// Adds one prohibited region.
    #[must_use]
    pub fn with_prohibited_region(mut self, region: impl Into<String>) -> Self {
        self.prohibited_regions.push(region.into());
        self
    }

    /// Marks the requirement as sovereign.
    #[must_use]
    pub const fn with_sovereignty(mut self, sovereign: bool) -> Self {
        self.data_sovereign = sovereign;
        self
    }

    /// Checks whether processing in `region` is permitted.
    #[must_use]
    pub fn allows_region(&self, region: &str) -> bool {
        let region = region.trim();
        if region.is_empty() || (self.data_sovereign && self.allowed_regions.is_empty()) {
            return false;
        }

        let prohibited = self
            .prohibited_regions
            .iter()
            .any(|candidate| region_matches(candidate, region));
        if prohibited {
            return false;
        }

        self.allowed_regions.is_empty()
            || self
                .allowed_regions
                .iter()
                .any(|candidate| region_matches(candidate, region))
    }

    /// Returns a validation error for an internally contradictory requirement.
    pub fn validate(&self) -> Result<(), String> {
        if self.data_sovereign && self.allowed_regions.is_empty() {
            return Err("sovereign residency requires at least one allowed region".to_owned());
        }
        Ok(())
    }
}

/// A value carrying its sensitivity and residency requirements together.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ClassifiedData<T> {
    /// The protected value.
    pub data: T,
    /// Sensitivity classification for the value.
    pub classification: SecurityClassification,
    /// Storage and processing constraints for the value.
    pub residency: ResidencyRequirement,
}

impl<T> ClassifiedData<T> {
    /// Wraps `data` with classification and residency metadata.
    #[must_use]
    pub fn new(
        data: T,
        classification: SecurityClassification,
        residency: ResidencyRequirement,
    ) -> Self {
        Self {
            data,
            classification,
            residency,
        }
    }

    /// Borrows the wrapped value.
    #[must_use]
    pub const fn as_ref(&self) -> &T {
        &self.data
    }

    /// Maps the wrapped value while preserving the security metadata.
    #[must_use]
    pub fn map<U>(self, map: impl FnOnce(T) -> U) -> ClassifiedData<U> {
        ClassifiedData {
            data: map(self.data),
            classification: self.classification,
            residency: self.residency,
        }
    }

    /// Unwraps the protected value.
    #[must_use]
    pub fn into_inner(self) -> T {
        self.data
    }

    /// Checks whether the value may be processed in `region`.
    #[must_use]
    pub fn is_allowed_in(&self, region: &str) -> bool {
        self.residency.allows_region(region)
    }
}

impl<T: TenantBoundary> TenantBoundary for ClassifiedData<T> {
    fn tenant_id(&self) -> &TenantId {
        self.data.tenant_id()
    }
}

/// One content-type-to-classification rule.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ClassificationRule {
    /// Exact content type or a prefix pattern ending in `*`.
    pub content_type: String,
    /// Classification assigned when the rule matches.
    pub classification: SecurityClassification,
}

/// Deterministic reference policy for automatic classification by content type.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ClassificationPolicy {
    /// Classification used when no rule matches.
    pub default: SecurityClassification,
    /// Rules evaluated as a set; the highest matching classification wins.
    #[serde(default)]
    pub rules: Vec<ClassificationRule>,
}

impl Default for ClassificationPolicy {
    fn default() -> Self {
        Self {
            default: SecurityClassification::Public,
            rules: Vec::new(),
        }
    }
}

impl ClassificationPolicy {
    /// Creates a policy with the supplied fallback classification.
    #[must_use]
    pub fn new(default: SecurityClassification) -> Self {
        Self {
            default,
            rules: Vec::new(),
        }
    }

    /// Adds a content-type rule in builder style.
    #[must_use]
    pub fn with_rule(
        mut self,
        content_type: impl Into<String>,
        classification: SecurityClassification,
    ) -> Self {
        self.add_rule(content_type, classification);
        self
    }

    /// Adds a content-type rule.
    pub fn add_rule(
        &mut self,
        content_type: impl Into<String>,
        classification: SecurityClassification,
    ) {
        self.rules.push(ClassificationRule {
            content_type: content_type.into(),
            classification,
        });
    }

    /// Classifies a content type using the most restrictive matching rule.
    #[must_use]
    pub fn classify(&self, content_type: &str) -> SecurityClassification {
        self.rules
            .iter()
            .filter(|rule| content_type_matches(&rule.content_type, content_type))
            .map(|rule| rule.classification)
            .max()
            .unwrap_or(self.default)
    }

    /// Alias emphasizing that this is an automatic classification decision.
    #[must_use]
    pub fn auto_classify(&self, content_type: &str) -> SecurityClassification {
        self.classify(content_type)
    }
}

fn region_matches(pattern: &str, value: &str) -> bool {
    let pattern = pattern.trim();
    pattern == "*" || pattern.eq_ignore_ascii_case(value)
}

fn content_type_matches(pattern: &str, value: &str) -> bool {
    let pattern = pattern.trim();
    if pattern == "*" || pattern.eq_ignore_ascii_case(value.trim()) {
        return true;
    }
    pattern
        .strip_suffix('*')
        .is_some_and(|prefix| value.trim().starts_with(prefix.trim_end_matches('/')))
}
