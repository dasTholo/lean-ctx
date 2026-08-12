//! Translation between parsed references and registered context providers.
use crate::core::providers::registry::ProviderRegistry;
use crate::core::providers::{ProviderParams, ProviderResult};

use super::reference_resolver::{ReferenceType, ResolvedReference};

/// Registry-backed resolver used by the knowledge router.
#[derive(Clone)]
pub struct ProviderBridge<'a> {
    registry: &'a ProviderRegistry,
}

impl std::fmt::Debug for ProviderBridge<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ProviderBridge")
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProviderResolution {
    pub reference: String,
    pub provider_id: String,
    pub resolved: bool,
    pub estimated_tokens: u64,
}

impl<'a> ProviderBridge<'a> {
    pub fn new(registry: &'a ProviderRegistry) -> Self {
        Self { registry }
    }

    pub fn available_provider_ids(&self) -> Vec<String> {
        self.registry.available_provider_ids()
    }

    pub fn resolve_from_providers(
        &self,
        references: &[ResolvedReference],
        available_providers: &[&str],
    ) -> Vec<ProviderResolution> {
        let registered = self.available_provider_ids();
        references
            .iter()
            .filter_map(|reference| {
                let provider_id = match reference.ref_type {
                    ReferenceType::JiraIssue => {
                        provider(available_providers, &registered, "jira", "issue_tracker")
                    }
                    ReferenceType::GitHubPR | ReferenceType::GitHubIssue => {
                        provider(available_providers, &registered, "github", "vcs")
                    }
                    ReferenceType::FilePath => Some("local_files"),
                    ReferenceType::Function | ReferenceType::Url => None,
                };
                let provider_id = provider_id?;
                let resolved = {
                    provider_id == "local_files"
                        || available_providers.contains(&provider_id)
                        || registered
                            .iter()
                            .any(|registered_id| registered_id == provider_id)
                };
                Some(ProviderResolution {
                    reference: reference.identifier.clone(),
                    provider_id: provider_id.to_string(),
                    resolved,
                    estimated_tokens: tokens(reference),
                })
            })
            .collect()
    }

    /// Fetch the provider record represented by a resolved reference.
    pub fn fetch_reference(
        &self,
        reference: &ResolvedReference,
        resolution: &ProviderResolution,
    ) -> Result<ProviderResult, String> {
        if !resolution.resolved {
            return Err(format!(
                "Provider '{}' is not available for '{}'",
                resolution.provider_id, resolution.reference
            ));
        }

        let action = action_for(reference.ref_type, &resolution.provider_id)?;
        let provider = self
            .registry
            .get(&resolution.provider_id)
            .ok_or_else(|| format!("Provider '{}' not registered", resolution.provider_id))?;
        if !provider.is_available() {
            return Err(format!(
                "Provider '{}' is not available",
                resolution.provider_id
            ));
        }
        if !provider.supported_actions().contains(&action) {
            return Err(format!(
                "Provider '{}' does not support action '{action}'",
                resolution.provider_id
            ));
        }

        provider.execute(
            action,
            &ProviderParams {
                id: Some(reference.identifier.trim_start_matches('#').into()),
                query: Some(reference.identifier.clone()),
                limit: Some(1),
                ..ProviderParams::default()
            },
        )
    }
}

fn provider<'a>(
    available: &[&'a str],
    registered: &[String],
    id: &'a str,
    kind: &'a str,
) -> Option<&'a str> {
    if available.contains(&id) || registered.iter().any(|value| value == id) {
        Some(id)
    } else if available.contains(&kind) {
        Some(kind)
    } else {
        Some(id)
    }
}

fn action_for(reference_type: ReferenceType, provider_id: &str) -> Result<&'static str, String> {
    match (reference_type, provider_id) {
        (ReferenceType::GitHubPR, "github") => Ok("pull_requests"),
        (ReferenceType::JiraIssue, "jira") | (ReferenceType::GitHubIssue, "github") => Ok("issues"),
        _ => Err(format!(
            "Provider '{provider_id}' cannot fetch {reference_type:?} references"
        )),
    }
}

fn tokens(reference: &ResolvedReference) -> u64 {
    match reference.ref_type {
        ReferenceType::GitHubPR => 1_000,
        ReferenceType::JiraIssue | ReferenceType::GitHubIssue => 500,
        ReferenceType::FilePath => 128 + (reference.identifier.len() as u64).saturating_mul(8),
        ReferenceType::Function | ReferenceType::Url => 0,
    }
}
