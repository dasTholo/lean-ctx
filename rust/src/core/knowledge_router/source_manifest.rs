#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SourceManifestEntry {
    pub source_id: String,
    pub display_name: String,
    pub kinds: Vec<String>,
    pub capabilities: SourceCapabilities,
    pub freshness_typical_ms: u64,
    pub cost_class: CostClass,
}
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SourceCapabilities {
    pub search: bool,
    pub exact_get: bool,
    pub delta_sync: bool,
    pub live_query: bool,
    pub graph_edges: bool,
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CostClass {
    #[default]
    Negligible,
    Low,
    Medium,
    High,
}
impl SourceManifestEntry {
    pub fn is_valid(&self) -> bool {
        !self.source_id.is_empty() && !self.display_name.is_empty() && !self.kinds.is_empty()
    }
}
pub fn builtin_manifests() -> Vec<SourceManifestEntry> {
    vec![
        entry(
            "local_files",
            "LocalFiles",
            vec!["file"],
            false,
            CostClass::Negligible,
        ),
        entry("jira", "Jira", vec!["issue"], true, CostClass::Low),
        entry(
            "github",
            "GitHub",
            vec!["issue", "pull_request"],
            true,
            CostClass::Low,
        ),
    ]
}
fn entry(
    source_id: &str,
    display_name: &str,
    kinds: Vec<&str>,
    live_query: bool,
    cost_class: CostClass,
) -> SourceManifestEntry {
    SourceManifestEntry {
        source_id: source_id.into(),
        display_name: display_name.into(),
        kinds: kinds.into_iter().map(str::to_owned).collect(),
        capabilities: SourceCapabilities {
            search: true,
            exact_get: true,
            delta_sync: true,
            live_query,
            graph_edges: true,
        },
        freshness_typical_ms: 60_000,
        cost_class,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn builtin_has_three_entries() {
        assert_eq!(builtin_manifests().len(), 3);
    }
    #[test]
    fn builtin_entries_are_valid() {
        assert!(
            builtin_manifests()
                .iter()
                .all(SourceManifestEntry::is_valid)
        );
    }
}
