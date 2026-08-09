//! Public rollout cohort contracts and deterministic assignment reference.
//!
//! Assignment only maps a task identifier to configured cohort metadata. It
//! does not check eligibility, bypass a policy, or select a model/provider.
//! Production rollout intelligence remains in `lean-ctx-enterprise` (Class D).

use crate::common::ValidationError;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Versioned rollout configuration made of ordered percentage cohorts.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RolloutConfig {
    #[serde(default)]
    pub cohorts: Vec<RolloutCohort>,
}

impl RolloutConfig {
    /// Validate names, percentages, and ordered-cohort bounds.
    pub fn validate(&self) -> Result<(), ValidationError> {
        let mut total = 0_u16;
        for (index, cohort) in self.cohorts.iter().enumerate() {
            if cohort.name.trim().is_empty() {
                return Err(ValidationError::new(format!(
                    "cohorts[{index}].name must not be empty"
                )));
            }
            if cohort.percentage == 0 {
                return Err(ValidationError::new(format!(
                    "cohorts[{index}].percentage must be greater than zero"
                )));
            }
            if self
                .cohorts
                .iter()
                .take(index)
                .any(|previous| previous.name == cohort.name)
            {
                return Err(ValidationError::new(format!(
                    "duplicate rollout cohort name: {}",
                    cohort.name
                )));
            }
            total = total.saturating_add(u16::from(cohort.percentage));
            if total > 100 {
                return Err(ValidationError::new(
                    "rollout cohort percentages must total at most 100",
                ));
            }
        }
        Ok(())
    }

    /// Deterministically assign a task to the ordered cohort ranges.
    pub fn assign(&self, task_id: &str) -> RolloutAssignment {
        let bucket = bucket_for(task_id);
        let mut upper_bound = 0_u8;
        let mut selected = None;
        for cohort in &self.cohorts {
            upper_bound = upper_bound.saturating_add(cohort.percentage);
            if bucket < upper_bound {
                selected = Some(cohort);
                break;
            }
        }

        RolloutAssignment {
            task_id: task_id.to_owned(),
            bucket,
            cohort: selected.map(|cohort| cohort.name.clone()),
            eligibility_override: selected.is_some_and(|cohort| cohort.eligibility_override),
            kill_switch: selected.is_some_and(|cohort| cohort.kill_switch),
        }
    }
}

/// One ordered cohort in a rollout configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RolloutCohort {
    pub name: String,
    pub percentage: u8,
    /// Metadata for the external eligibility policy; never applied here.
    pub eligibility_override: bool,
    /// Metadata indicating that the external rollout owner has disabled work.
    pub kill_switch: bool,
}

/// Stable assignment result derived from a task identifier and cohort config.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RolloutAssignment {
    pub task_id: String,
    /// SHA-256-derived bucket in the inclusive range 0..=99.
    pub bucket: u8,
    pub cohort: Option<String>,
    pub eligibility_override: bool,
    pub kill_switch: bool,
}

impl RolloutAssignment {
    /// Return whether the task fell outside all configured cohort ranges.
    pub fn is_unassigned(&self) -> bool {
        self.cohort.is_none()
    }
}

/// Standalone form for callers that do not need the method syntax.
pub fn assign_rollout(task_id: &str, config: &RolloutConfig) -> RolloutAssignment {
    config.assign(task_id)
}

fn bucket_for(task_id: &str) -> u8 {
    let digest = Sha256::digest(task_id.as_bytes());
    let value = u64::from_be_bytes(digest[..8].try_into().expect("SHA-256 has eight bytes"));
    (value % 100) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> RolloutConfig {
        RolloutConfig {
            cohorts: vec![
                RolloutCohort {
                    name: "internal-10".to_owned(),
                    percentage: 10,
                    eligibility_override: false,
                    kill_switch: false,
                },
                RolloutCohort {
                    name: "internal-50".to_owned(),
                    percentage: 50,
                    eligibility_override: true,
                    kill_switch: true,
                },
            ],
        }
    }

    #[test]
    fn rollout_types_round_trip_and_validate() {
        let config = config();
        let assignment = config.assign("task-1");
        let config_json = serde_json::to_string(&config).expect("config serializes");
        let assignment_json = serde_json::to_string(&assignment).expect("assignment serializes");
        assert_eq!(
            serde_json::from_str::<RolloutConfig>(&config_json).unwrap(),
            config
        );
        assert_eq!(
            serde_json::from_str::<RolloutAssignment>(&assignment_json).unwrap(),
            assignment
        );
        config.validate().expect("valid rollout");
    }

    #[test]
    fn assignment_is_sha_based_deterministic_and_metadata_only() {
        let config = config();
        let first = config.assign("task-1");
        let second = config.assign("task-1");
        assert_eq!(first, second);
        assert!(first.bucket < 100);
        assert_eq!(assign_rollout("task-1", &config), first);
        assert_eq!(first.task_id, "task-1");
    }

    #[test]
    fn invalid_rollout_percentages_are_rejected() {
        let invalid = RolloutConfig {
            cohorts: vec![
                RolloutCohort {
                    name: "a".to_owned(),
                    percentage: 60,
                    eligibility_override: false,
                    kill_switch: false,
                },
                RolloutCohort {
                    name: "b".to_owned(),
                    percentage: 60,
                    eligibility_override: false,
                    kill_switch: false,
                },
            ],
        };
        assert!(invalid.validate().is_err());
    }
}
