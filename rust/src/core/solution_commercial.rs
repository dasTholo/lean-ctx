use serde::{Deserialize, Serialize};

const ENTERPRISE_LICENSE_REQUIRED: &str = "requires enterprise license";

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct AdaptiveConfig {
    pub enabled: bool,
    pub learning_rate: f64,
    pub min_observations: u32,
}

impl Default for AdaptiveConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            learning_rate: 0.1,
            min_observations: 20,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct TeamPolicyConfig {
    pub enabled: bool,
    pub min_intensity: String,
    pub require_decision_logging: bool,
}

impl Default for TeamPolicyConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            min_intensity: "balanced".to_owned(),
            require_decision_logging: false,
        }
    }
}

#[derive(Default, Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct SolutionCommercialConfig {
    pub adaptive: AdaptiveConfig,
    pub team_policy: TeamPolicyConfig,
    pub fingerprints_enabled: bool,
    pub cross_project_patterns: bool,
}

/// Lists capabilities exposed by this OSS build and whether they are active.
///
/// The names include the tier so callers can render an actionable availability
/// status without inferring entitlement from the boolean alone.
pub fn commercial_features_available() -> Vec<(String, bool)> {
    vec![
        ("adaptive (available in basic)".to_owned(), true),
        ("fingerprint (available in basic)".to_owned(), true),
        ("team_policy (available in basic)".to_owned(), true),
        (
            "cross_project_patterns (requires enterprise license)".to_owned(),
            false,
        ),
        (
            "verified_attribution (requires enterprise license)".to_owned(),
            false,
        ),
        (
            "solution_audit_trail (requires enterprise license)".to_owned(),
            false,
        ),
    ]
}

#[derive(Serialize, Clone, Debug)]
pub struct AdaptiveRecommendation {
    pub suggested_intensity: String,
    pub confidence: f64,
    pub reason: String,
    pub observation_count: u32,
}

pub fn recommend_intensity(
    config: &AdaptiveConfig,
    decisions: &crate::core::solution_tracker::SolutionSnapshot,
) -> Option<AdaptiveRecommendation> {
    if !config.enabled || decisions.decisions_total < u64::from(config.min_observations) {
        return None;
    }

    let decision_total = decisions.decisions_total as f64;
    let stdlib_ratio = decisions
        .decisions_by_kind
        .get("stdlib")
        .copied()
        .unwrap_or_default() as f64
        / decision_total;
    let yagni_ratio = decisions
        .decisions_by_kind
        .get("yagni")
        .copied()
        .unwrap_or_default() as f64
        / decision_total;
    let efficiency_score = stdlib_ratio + yagni_ratio;
    let (suggested_intensity, confidence, reason) = if efficiency_score > 0.6 {
        (
            "aggressive",
            0.85,
            "Strong standard-library and YAGNI decision pattern",
        )
    } else if efficiency_score > 0.3 {
        (
            "balanced",
            0.70,
            "Moderate solution-efficiency decision pattern",
        )
    } else {
        (
            "minimal",
            0.60,
            "Limited solution-efficiency decision pattern",
        )
    };

    Some(AdaptiveRecommendation {
        suggested_intensity: suggested_intensity.to_owned(),
        confidence,
        reason: reason.to_owned(),
        observation_count: decisions.decisions_total.min(u32::MAX as u64) as u32,
    })
}

#[derive(Serialize, Clone, Debug)]
pub struct SolutionFingerprint {
    pub task_pattern: String,
    pub predicted_rung: String,
    pub confidence: f64,
}

pub fn predict_rung(task_description: &str) -> SolutionFingerprint {
    let description = task_description.to_ascii_lowercase();
    let (task_pattern, predicted_rung, confidence) = if description.contains("refactor") {
        ("refactor", "reuse", 0.85)
    } else if ["sort", "parse", "format"]
        .iter()
        .any(|keyword| description.contains(keyword))
    {
        ("stdlib", "stdlib", 0.80)
    } else if ["css", "html", "sql"]
        .iter()
        .any(|keyword| description.contains(keyword))
    {
        ("native", "native", 0.80)
    } else if ["config", "flag"]
        .iter()
        .any(|keyword| description.contains(keyword))
    {
        ("yagni", "yagni", 0.75)
    } else if description.contains("add dependency") {
        ("dependency", "dep-check", 0.80)
    } else {
        ("general", "minimum", 0.60)
    };

    SolutionFingerprint {
        task_pattern: task_pattern.to_owned(),
        predicted_rung: predicted_rung.to_owned(),
        confidence,
    }
}

pub fn validate_team_policy(
    policy: &TeamPolicyConfig,
    current_intensity: &str,
) -> Result<(), String> {
    if !policy.enabled {
        return Ok(());
    }

    if intensity_rank(current_intensity) < intensity_rank(&policy.min_intensity) {
        return Err(format!(
            "Current intensity '{current_intensity}' is below the team minimum '{}'.",
            policy.min_intensity
        ));
    }

    Ok(())
}

// Commercial feature gate: implementations belong exclusively to the private
// enterprise repository. OSS exposes only stable request boundaries and never
// aggregates, uploads, or retains commercial feature data.

/// OSS boundary for `POST /cross-project-patterns:analyze`.
pub fn analyze_cross_project_patterns(_organization_id: &str) -> Result<(), String> {
    Err(ENTERPRISE_LICENSE_REQUIRED.to_owned())
}

/// OSS boundary for `POST /attribution/envelopes` verification.
pub fn verify_attribution(_event_id: &str) -> Result<(), String> {
    Err(ENTERPRISE_LICENSE_REQUIRED.to_owned())
}

/// OSS boundary for `POST /solution-audit/events`.
pub fn append_solution_audit_event(_project_id: &str) -> Result<(), String> {
    Err(ENTERPRISE_LICENSE_REQUIRED.to_owned())
}

fn intensity_rank(intensity: &str) -> u8 {
    match intensity {
        "minimal" => 1,
        "balanced" => 2,
        "aggressive" => 3,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adaptive_needs_min_observations() {
        let config = AdaptiveConfig {
            enabled: true,
            min_observations: 20,
            ..AdaptiveConfig::default()
        };
        let decisions = crate::core::solution_tracker::SolutionSnapshot {
            decisions_total: 19,
            decisions_by_kind: Default::default(),
            loc_added: 0,
            loc_removed: 0,
            loc_net_saved: 0,
            output_tokens_baseline: 0,
            output_tokens_actual: 0,
            output_reduction_pct: 0,
        };

        assert!(recommend_intensity(&config, &decisions).is_none());
    }

    #[test]
    fn fingerprint_detects_refactor() {
        let fingerprint = predict_rung("Refactor the request pipeline");

        assert_eq!(fingerprint.predicted_rung, "reuse");
    }

    #[test]
    fn team_policy_enforces_minimum() {
        let policy = TeamPolicyConfig {
            enabled: true,
            min_intensity: "balanced".to_owned(),
            ..TeamPolicyConfig::default()
        };

        assert!(validate_team_policy(&policy, "minimal").is_err());
    }

    #[test]
    fn disabled_policy_allows_everything() {
        assert!(validate_team_policy(&TeamPolicyConfig::default(), "off").is_ok());
    }

    #[test]
    fn feature_availability_marks_the_oss_and_enterprise_boundaries() {
        assert_eq!(
            commercial_features_available(),
            vec![
                ("adaptive (available in basic)".to_owned(), true),
                ("fingerprint (available in basic)".to_owned(), true),
                ("team_policy (available in basic)".to_owned(), true),
                (
                    "cross_project_patterns (requires enterprise license)".to_owned(),
                    false,
                ),
                (
                    "verified_attribution (requires enterprise license)".to_owned(),
                    false,
                ),
                (
                    "solution_audit_trail (requires enterprise license)".to_owned(),
                    false,
                ),
            ]
        );
    }

    #[test]
    fn enterprise_stubs_do_not_expose_commercial_logic() {
        for result in [
            analyze_cross_project_patterns("org-123"),
            verify_attribution("event-123"),
            append_solution_audit_event("project-123"),
        ] {
            assert_eq!(result, Err(ENTERPRISE_LICENSE_REQUIRED.to_owned()));
        }
    }
}
