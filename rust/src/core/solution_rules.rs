//! Solution-selection guidance for injected instructions.

const START_MARKER: &str = "<!-- lean-ctx-solution -->";
const END_MARKER: &str = "<!-- /lean-ctx-solution -->";

const MINIMAL_RULES: &str =
    "Prefer the standard library and platform-native capabilities before adding dependencies.";
const BALANCED_RULES: &str = "Solution ladder: skip → reuse → stdlib → native → dependency → one-line → minimum.\nSafety: choose the smallest safe solution; validate inputs and preserve existing behavior.";
const AGGRESSIVE_RULES: &str = "Challenge first: can this be deleted or skipped? Prefer deletion.\nSolution ladder: skip → reuse → stdlib → native → dependency → one-line → minimum.\nSafety: choose the smallest safe solution; validate inputs and preserve existing behavior.";

/// Build the solution-selection rules block for an instruction intensity.
pub fn solution_rules_block(intensity: &str) -> String {
    let rules = match intensity {
        "minimal" => MINIMAL_RULES,
        "aggressive" => AGGRESSIVE_RULES,
        _ => BALANCED_RULES,
    };

    format!("{START_MARKER}\n{rules}\n{END_MARKER}")
}

/// The policy supplied to every solution-focused subagent.
pub fn solution_subagent_instructions(_intensity: &str) -> &'static str {
    "Challenge the need, reuse existing code, prefer stdlib or native capabilities, and add dependencies only when justified."
}

/// Build optional compose hints from project configuration and dependencies.
pub fn solution_compose_hints(config_enabled: bool, project_deps: &[String]) -> String {
    if !config_enabled {
        return String::new();
    }

    let mut lines = vec![START_MARKER.to_string()];
    if !project_deps.is_empty() {
        lines.push(format!("installed deps: {}", project_deps.join(", ")));
    }
    lines.push(
        "Prefer the standard library and platform-native capabilities before adding dependencies."
            .to_string(),
    );
    lines.push(END_MARKER.to_string());
    lines.join("\n")
}

/// Solution rules are injected only when configuration and request both allow it.
pub fn should_inject(config_enabled: bool, inject_flag: bool) -> bool {
    config_enabled && inject_flag
}
