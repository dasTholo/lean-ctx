//! Heuristics for recognizing solution decisions from edit context.

use std::collections::BTreeSet;

/// Detect a newly added standard-library import.
pub fn detect_stdlib_choice(old_content: &str, new_content: &str) -> Option<String> {
    let old_imports = stdlib_imports(old_content);

    stdlib_imports(new_content)
        .into_iter()
        .find(|import| !old_imports.contains(import))
        .map(|import| format!("Selected standard library import `{import}`"))
}

/// Detect an explicit lean-ctx debt marker.
pub fn detect_debt_marker(new_content: &str) -> Option<String> {
    new_content
        .lines()
        .map(str::trim_start)
        .find_map(|line| line.strip_prefix("// lean-ctx:"))
        .map(str::trim)
        .map(|marker| {
            if marker.is_empty() {
                "Added lean-ctx debt marker".to_string()
            } else {
                format!("Added lean-ctx debt marker: {marker}")
            }
        })
}

/// Detect dependencies present in the old manifest but absent from the new one.
pub fn detect_dep_removal(old_manifest: &str, new_manifest: &str) -> Option<String> {
    let old_dependencies = dependency_names(old_manifest);
    let new_dependencies = dependency_names(new_manifest);
    let removed: Vec<_> = old_dependencies
        .difference(&new_dependencies)
        .cloned()
        .collect();

    match removed.as_slice() {
        [] => None,
        [dependency] => Some(format!("Removed dependency: {dependency}")),
        dependencies => Some(format!("Removed dependencies: {}", dependencies.join(", "))),
    }
}

/// Detect a substantial implementation-size reduction.
///
/// A reduction counts when it removes at least five lines and cuts the size by
/// at least half. This ignores ordinary small edits while surfacing one-liner
/// style simplifications.
pub fn detect_one_liner(old_lines: usize, new_lines: usize) -> Option<String> {
    let removed_lines = old_lines.saturating_sub(new_lines);
    let reduced_by_half = new_lines <= old_lines / 2;

    (old_lines > new_lines && removed_lines >= 5 && reduced_by_half)
        .then(|| format!("Simplified implementation from {old_lines} to {new_lines} lines"))
}

fn stdlib_imports(content: &str) -> BTreeSet<String> {
    content
        .lines()
        .map(strip_line_comment)
        .map(str::trim)
        .filter(|line| line.starts_with("use std::") || line.starts_with("pub use std::"))
        .map(str::to_string)
        .collect()
}

fn dependency_names(manifest: &str) -> BTreeSet<String> {
    let mut dependencies = BTreeSet::new();
    let mut in_dependency_section = false;

    for line in manifest.lines().map(strip_line_comment).map(str::trim) {
        if let Some(section) = line
            .strip_prefix('[')
            .and_then(|line| line.strip_suffix(']'))
        {
            in_dependency_section = is_dependency_section(section);
            continue;
        }

        if !in_dependency_section || line.is_empty() {
            continue;
        }

        if let Some((name, _)) = line.split_once('=') {
            let name = name.trim().trim_matches(['\'', '"']);
            if !name.is_empty() {
                dependencies.insert(name.to_string());
            }
        }
    }

    dependencies
}

fn is_dependency_section(section: &str) -> bool {
    let section = section.trim();
    ["dependencies", "dev-dependencies", "build-dependencies"]
        .iter()
        .any(|dependency_section| {
            section == *dependency_section || section.ends_with(&format!(".{dependency_section}"))
        })
}

fn strip_line_comment(line: &str) -> &str {
    line.split_once('#').map_or(line, |(content, _)| content)
}

#[cfg(test)]
mod tests {
    use super::{detect_debt_marker, detect_dep_removal, detect_one_liner, detect_stdlib_choice};

    #[test]
    fn detects_new_stdlib_import() {
        assert_eq!(
            detect_stdlib_choice(
                "use crate::core::Config;\n",
                "use crate::core::Config;\nuse std::collections::HashMap;\n",
            ),
            Some("Selected standard library import `use std::collections::HashMap;`".to_string())
        );
    }

    #[test]
    fn ignores_preexisting_stdlib_import() {
        let content = "use std::collections::HashMap;\n";
        assert_eq!(detect_stdlib_choice(content, content), None);
    }

    #[test]
    fn detects_debt_marker() {
        assert_eq!(
            detect_debt_marker("  // lean-ctx: remove after migration\n"),
            Some("Added lean-ctx debt marker: remove after migration".to_string())
        );
    }

    #[test]
    fn ignores_content_without_debt_marker() {
        assert_eq!(detect_debt_marker("// TODO: revisit\n"), None);
    }

    #[test]
    fn detects_removed_dependencies_in_sorted_order() {
        let old_manifest = "[dependencies]\nzeta = \"1\"\nalpha = \"1\"\nkeep = \"1\"\n";
        let new_manifest = "[dependencies]\nkeep = \"2\"\n";

        assert_eq!(
            detect_dep_removal(old_manifest, new_manifest),
            Some("Removed dependencies: alpha, zeta".to_string())
        );
    }

    #[test]
    fn ignores_dependency_version_changes() {
        assert_eq!(
            detect_dep_removal(
                "[dev-dependencies]\npretty_assertions = \"1\"\n",
                "[dev-dependencies]\npretty_assertions = \"2\"\n",
            ),
            None
        );
    }

    #[test]
    fn detects_substantial_line_reduction() {
        assert_eq!(
            detect_one_liner(12, 4),
            Some("Simplified implementation from 12 to 4 lines".to_string())
        );
    }

    #[test]
    fn ignores_small_or_insufficient_reductions() {
        assert_eq!(detect_one_liner(12, 8), None);
        assert_eq!(detect_one_liner(4, 1), None);
    }
}
