//! Versioned registry for common capability adapters.

use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};

use lean_ctx_protocol::CapabilityManifestV1;

use super::super::invocation::CapabilityAdapter;
use crate::core::ocla::{OclaError, OclaResult};

/// Stable lookup key for an adapter manifest.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct AdapterKey {
    pub capability_id: String,
    pub version: String,
}

impl AdapterKey {
    #[must_use]
    pub fn new(capability_id: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            capability_id: capability_id.into(),
            version: version.into(),
        }
    }

    #[must_use]
    pub fn as_string(&self) -> String {
        format!("{}@{}", self.capability_id, self.version)
    }
}

/// Result of checking one adapter's health.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdapterHealth {
    pub key: AdapterKey,
    pub healthy: bool,
}

/// Thread-safe registry keyed by capability ID and manifest version.
pub struct AdapterRegistry {
    adapters: RwLock<BTreeMap<AdapterKey, Arc<dyn CapabilityAdapter>>>,
}

impl AdapterRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self {
            adapters: RwLock::new(BTreeMap::new()),
        }
    }

    /// Register one adapter, rejecting invalid or duplicate manifests.
    pub fn register<A>(&self, adapter: A) -> OclaResult<()>
    where
        A: CapabilityAdapter + 'static,
    {
        let manifest = adapter.manifest();
        manifest.validate().map_err(|error| {
            OclaError::InvalidRequest(format!("invalid capability manifest: {error}"))
        })?;
        let key = AdapterKey::new(manifest.capability_id.as_str(), manifest.version.clone());
        let mut adapters = self
            .adapters
            .write()
            .map_err(|_| OclaError::InvalidRequest("adapter registry lock poisoned".into()))?;
        if adapters.contains_key(&key) {
            return Err(OclaError::InvalidRequest(format!(
                "adapter already registered: {}",
                key.as_string()
            )));
        }
        adapters.insert(key, Arc::new(adapter));
        Ok(())
    }

    /// Register an already shared adapter object.
    pub fn register_arc(&self, adapter: Arc<dyn CapabilityAdapter>) -> OclaResult<()> {
        let manifest = adapter.manifest();
        manifest.validate().map_err(|error| {
            OclaError::InvalidRequest(format!("invalid capability manifest: {error}"))
        })?;
        let key = AdapterKey::new(manifest.capability_id.as_str(), manifest.version.clone());
        let mut adapters = self
            .adapters
            .write()
            .map_err(|_| OclaError::InvalidRequest("adapter registry lock poisoned".into()))?;
        if adapters.contains_key(&key) {
            return Err(OclaError::InvalidRequest(format!(
                "adapter already registered: {}",
                key.as_string()
            )));
        }
        adapters.insert(key, adapter);
        Ok(())
    }

    /// Look up an adapter by its capability ID and exact version.
    pub fn lookup(&self, capability_id: &str, version: &str) -> Option<Arc<dyn CapabilityAdapter>> {
        let key = AdapterKey::new(capability_id, version);
        self.adapters
            .read()
            .ok()
            .and_then(|adapters| adapters.get(&key).cloned())
    }

    /// Alias matching the existing provider registry vocabulary.
    pub fn get(&self, capability_id: &str, version: &str) -> Option<Arc<dyn CapabilityAdapter>> {
        self.lookup(capability_id, version)
    }

    /// Health-check every registered adapter in deterministic key order.
    pub fn health_check_all(&self) -> OclaResult<Vec<AdapterHealth>> {
        let adapters = self
            .adapters
            .read()
            .map_err(|_| OclaError::InvalidRequest("adapter registry lock poisoned".into()))?;
        adapters
            .iter()
            .map(|(key, adapter)| {
                Ok(AdapterHealth {
                    key: key.clone(),
                    healthy: adapter.health_check()?,
                })
            })
            .collect()
    }

    /// Return all manifests, sorted by `(capability_id, version)`.
    pub fn list_available_adapters(&self) -> Vec<CapabilityManifestV1> {
        self.adapters
            .read()
            .map(|adapters| {
                adapters
                    .values()
                    .map(|adapter| adapter.manifest().clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Short alias for discovery callers.
    pub fn list_available(&self) -> Vec<CapabilityManifestV1> {
        self.list_available_adapters()
    }

    /// List exact registry keys without exposing adapter implementations.
    pub fn keys(&self) -> Vec<AdapterKey> {
        self.adapters
            .read()
            .map(|adapters| adapters.keys().cloned().collect())
            .unwrap_or_default()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.adapters.read().map_or(0, |adapters| adapters.len())
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for AdapterRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::ocla::adapters::PassthroughAdapter;
    use std::collections::BTreeSet;

    #[test]
    fn duplicate_versions_are_rejected() {
        let registry = AdapterRegistry::new();
        registry
            .register(PassthroughAdapter::new())
            .expect("first registration");
        assert!(registry.register(PassthroughAdapter::new()).is_err());
    }

    #[test]
    fn empty_registry_is_safe_to_inspect() {
        let registry = AdapterRegistry::new();
        assert!(registry.is_empty());
        assert!(registry.keys().is_empty());
        assert!(
            registry
                .health_check_all()
                .expect("health checks")
                .is_empty()
        );
    }

    #[test]
    fn key_order_is_deterministic() {
        let keys = BTreeSet::from([
            AdapterKey::new("capability://b", "1.0.0"),
            AdapterKey::new("capability://a", "1.0.0"),
        ]);
        assert_eq!(
            keys.into_iter().next().expect("first key").capability_id,
            "capability://a"
        );
    }
}
