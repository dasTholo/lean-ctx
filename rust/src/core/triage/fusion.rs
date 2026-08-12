use super::{ProfileHypothesis, profile::TaskProfileLocal};

#[derive(Debug, Clone, Copy, Default)]
pub struct FusionEngine;

#[derive(Debug, Clone, Copy, Default)]
pub struct RuntimeSignals {
    pub files_count: usize,
    pub modules_affected: usize,
    pub has_compiler_errors: bool,
    pub context_cached_ratio: f32,
}

impl FusionEngine {
    pub fn fuse(hypotheses: &[ProfileHypothesis], signals: &RuntimeSignals) -> TaskProfileLocal {
        let mut profile = hypotheses
            .iter()
            .max_by_key(|h| h.confidence_milli)
            .map_or_else(TaskProfileLocal::default, |h| h.profile.clone());
        if signals.files_count > 8 || signals.modules_affected > 3 {
            profile.complexity = "high".into();
        }
        profile
    }
}
pub fn fuse(hypotheses: &[ProfileHypothesis], signals: &RuntimeSignals) -> TaskProfileLocal {
    FusionEngine::fuse(hypotheses, signals)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::triage::TriageBackendLocal;
    fn h(c: u16, complexity: &str) -> ProfileHypothesis {
        ProfileHypothesis {
            profile: TaskProfileLocal {
                complexity: complexity.into(),
                confidence_milli: c,
                ..Default::default()
            },
            confidence_milli: c,
            backend: TriageBackendLocal::Rules,
        }
    }
    #[test]
    fn empty_uses_default() {
        assert_eq!(
            fuse(&[], &RuntimeSignals::default()),
            TaskProfileLocal::default()
        );
    }
    #[test]
    fn highest_confidence_wins() {
        assert_eq!(
            fuse(
                &[h(400, "low"), h(900, "medium")],
                &RuntimeSignals::default()
            )
            .complexity,
            "medium"
        );
    }
    #[test]
    fn files_or_modules_raise_complexity() {
        for signals in [
            RuntimeSignals {
                files_count: 9,
                ..Default::default()
            },
            RuntimeSignals {
                modules_affected: 4,
                ..Default::default()
            },
        ] {
            assert_eq!(fuse(&[h(900, "low")], &signals).complexity, "high");
        }
    }
}
