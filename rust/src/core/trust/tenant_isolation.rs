//! Small, composable tenant-boundary checks.

use std::fmt;

use super::identity::{IdentityContext, TenantId};

/// Marker/access trait implemented by values that carry a tenant boundary.
pub trait TenantBoundary {
    /// Returns the tenant that owns the value.
    fn tenant_id(&self) -> &TenantId;
}

impl TenantBoundary for TenantId {
    fn tenant_id(&self) -> &TenantId {
        self
    }
}

impl TenantBoundary for IdentityContext {
    fn tenant_id(&self) -> &TenantId {
        &self.tenant
    }
}

/// Error returned when two tenant-scoped values would be joined across a
/// boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrossTenantLeak {
    /// Tenant selected by the left/owning value.
    pub expected: TenantId,
    /// Tenant carried by the right/referenced value.
    pub actual: TenantId,
}

impl CrossTenantLeak {
    /// Creates a cross-tenant error from the two observed tenants.
    #[must_use]
    pub fn new(expected: TenantId, actual: TenantId) -> Self {
        Self { expected, actual }
    }
}

impl fmt::Display for CrossTenantLeak {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "tenant boundary violation: expected {}, found {}",
            self.expected, self.actual
        )
    }
}

impl std::error::Error for CrossTenantLeak {}

/// Stateless isolation helper for pair and collection checks.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct IsolationCheck;

impl IsolationCheck {
    /// Checks that two tenant-scoped values belong to the same tenant.
    pub fn same_tenant<A: TenantBoundary + ?Sized, B: TenantBoundary + ?Sized>(
        left: &A,
        right: &B,
    ) -> Result<(), CrossTenantLeak> {
        assert_same_tenant(left, right)
    }

    /// Checks that every value in a collection belongs to its first tenant.
    pub fn all_same_tenant<T: TenantBoundary>(values: &[T]) -> Result<(), CrossTenantLeak> {
        let Some(first) = values.first() else {
            return Ok(());
        };
        for value in values.iter().skip(1) {
            assert_same_tenant(first, value)?;
        }
        Ok(())
    }

    /// Alias suitable for call sites that describe the operation as a check.
    pub fn verify<A: TenantBoundary + ?Sized, B: TenantBoundary + ?Sized>(
        left: &A,
        right: &B,
    ) -> Result<(), CrossTenantLeak> {
        Self::same_tenant(left, right)
    }
}

/// Verifies that two values share exactly the same tenant identifier.
pub fn assert_same_tenant<A: TenantBoundary + ?Sized, B: TenantBoundary + ?Sized>(
    left: &A,
    right: &B,
) -> Result<(), CrossTenantLeak> {
    if left.tenant_id() == right.tenant_id() {
        Ok(())
    } else {
        Err(CrossTenantLeak::new(
            left.tenant_id().clone(),
            right.tenant_id().clone(),
        ))
    }
}
