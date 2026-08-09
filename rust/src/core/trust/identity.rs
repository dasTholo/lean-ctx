//! Stable identity value types shared by local and enterprise deployments.

use serde::{Deserialize, Serialize};
use std::fmt;

macro_rules! identity_id {
    ($name:ident, $documentation:literal) => {
        #[doc = $documentation]
        #[derive(
            Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize,
        )]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            /// Creates an identifier from its opaque string representation.
            #[must_use]
            pub fn new(value: impl Into<String>) -> Self {
                Self(value.into())
            }

            /// Returns the opaque identifier without allocating.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }

            /// Returns whether the identifier is empty.
            #[must_use]
            pub fn is_empty(&self) -> bool {
                self.0.is_empty()
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self::new(value)
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self::new(value)
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                self.as_str()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(self.as_str())
            }
        }
    };
}

identity_id!(
    TenantId,
    "Opaque identifier for the tenant that owns an execution or data object."
);
identity_id!(
    ProjectId,
    "Opaque identifier for a project within a tenant."
);
identity_id!(
    AgentId,
    "Opaque identifier for an agent execution identity."
);
identity_id!(
    HumanActorId,
    "Opaque identifier for the human actor responsible for an action."
);
identity_id!(
    ServiceId,
    "Opaque identifier for a service acting on behalf of a tenant."
);

/// Scope at which an identity or reference is valid.
#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum IdentityScope {
    /// Valid only inside one local runtime or process.
    #[default]
    Local,
    /// Valid for one tenant and its authorized projects.
    Tenant,
    /// Explicitly spans more than one tenant and requires enterprise policy.
    CrossTenant,
}

/// Identity attributes carried with a request, evidence entry, or policy
/// evaluation.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IdentityContext {
    /// Owning tenant. This is mandatory even for local contexts.
    pub tenant: TenantId,
    /// Optional project within the tenant.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project: Option<ProjectId>,
    /// Optional agent execution identity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent: Option<AgentId>,
    /// Optional human actor who initiated or approved the action.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub human_actor: Option<HumanActorId>,
    /// Optional service identity used for a delegated operation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service: Option<ServiceId>,
    /// Optional session or trace-local identity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session: Option<String>,
}

impl IdentityContext {
    /// Creates a context with only its required tenant identity.
    #[must_use]
    pub fn new(tenant: impl Into<TenantId>) -> Self {
        Self {
            tenant: tenant.into(),
            project: None,
            agent: None,
            human_actor: None,
            service: None,
            session: None,
        }
    }

    /// Sets the project scope.
    #[must_use]
    pub fn with_project(mut self, project: impl Into<ProjectId>) -> Self {
        self.project = Some(project.into());
        self
    }

    /// Sets the agent identity.
    #[must_use]
    pub fn with_agent(mut self, agent: impl Into<AgentId>) -> Self {
        self.agent = Some(agent.into());
        self
    }

    /// Sets the human actor identity.
    #[must_use]
    pub fn with_human_actor(mut self, human_actor: impl Into<HumanActorId>) -> Self {
        self.human_actor = Some(human_actor.into());
        self
    }

    /// Sets the delegated service identity.
    #[must_use]
    pub fn with_service(mut self, service: impl Into<ServiceId>) -> Self {
        self.service = Some(service.into());
        self
    }

    /// Sets the session identity.
    #[must_use]
    pub fn with_session(mut self, session: impl Into<String>) -> Self {
        self.session = Some(session.into());
        self
    }

    /// Returns the tenant without exposing mutable identity state.
    #[must_use]
    pub fn tenant_id(&self) -> &TenantId {
        &self.tenant
    }

    /// Returns the most specific declared scope for this context.
    #[must_use]
    pub fn scope(&self) -> IdentityScope {
        if self.tenant.is_empty() {
            IdentityScope::Local
        } else {
            IdentityScope::Tenant
        }
    }
}
