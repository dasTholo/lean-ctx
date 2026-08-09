//! Public, payload-free receipt for comparing two capability arms.

use serde::Serialize;

use super::super::invocation::CapabilityObservationV1;

/// Minimum external-arm quality score accepted by the comparison report.
pub const QUALITY_FLOOR_SCORE: u64 = 90;

const EXTERNAL_SAVES_TOKENS: &str =
    "external arm uses fewer tokens while meeting the quality floor";
const NATIVE_SAVES_TOKENS: &str = "native arm uses fewer tokens while meeting the quality floor";
const QUALITY_FLOOR_REASON: &str = "external arm did not meet the quality floor";
const INFORMATION_LOSS_REASON: &str = "native arm preserves materially more structural information";
const TIED_REASON: &str = "both arms have equivalent token cost and quality";

/// A complete comparison observation without embedding shell output.
#[derive(Clone, Debug, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ComparisonReceipt {
    pub task_id: String,
    pub native_observation: CapabilityObservationV1,
    pub external_observation: Option<CapabilityObservationV1>,
    pub decision: ComparisonDecision,
    pub native_tokens: u64,
    pub external_tokens: Option<u64>,
    pub native_latency_ms: u64,
    pub external_latency_ms: Option<u64>,
    pub quality_check: QualityCheck,
}

impl ComparisonReceipt {
    /// Build a receipt while copying token and latency totals from observations.
    #[must_use]
    pub fn new(
        task_id: impl Into<String>,
        native_observation: CapabilityObservationV1,
        external_observation: Option<CapabilityObservationV1>,
        decision: ComparisonDecision,
        quality_check: QualityCheck,
    ) -> Self {
        let native_tokens = native_observation.output_tokens;
        let native_latency_ms = native_observation.latency_ms;
        let external_tokens = external_observation
            .as_ref()
            .map(|observation| observation.output_tokens);
        let external_latency_ms = external_observation
            .as_ref()
            .map(|observation| observation.latency_ms);

        Self {
            task_id: task_id.into(),
            native_observation,
            external_observation,
            decision,
            native_tokens,
            external_tokens,
            native_latency_ms,
            external_latency_ms,
            quality_check,
        }
    }

    /// Return the signed token savings of the external arm.
    ///
    /// A positive value is a saving; a negative value is an external overhead.
    #[must_use]
    pub fn token_delta(&self) -> Option<i64> {
        self.external_tokens
            .map(|external| self.native_tokens as i64 - external as i64)
    }
}

/// Decision made after comparing token cost and quality.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComparisonDecision {
    ExternalPreferred { reason: &'static str },
    NativePreferred { reason: &'static str },
    Inconclusive { reason: &'static str },
    ExternalUnavailable,
}

impl ComparisonDecision {
    /// Stable text label used by reports and dashboards.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::ExternalPreferred { .. } => "external_preferred",
            Self::NativePreferred { .. } => "native_preferred",
            Self::Inconclusive { .. } => "inconclusive",
            Self::ExternalUnavailable => "external_unavailable",
        }
    }

    /// Whether this decision counts as an external-arm win.
    #[must_use]
    pub const fn is_external_preferred(self) -> bool {
        matches!(self, Self::ExternalPreferred { .. })
    }

    /// Whether this decision counts as a native-arm win.
    #[must_use]
    pub const fn is_native_preferred(self) -> bool {
        matches!(self, Self::NativePreferred { .. })
    }
}

/// Quality result for the comparison's observable structure.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum QualityCheck {
    StructurallyEquivalent,
    QualityFloorFailed,
    InformationLoss { severity: String },
}

impl QualityCheck {
    /// Whether the external arm met the minimum quality floor.
    #[must_use]
    pub const fn meets_floor(&self) -> bool {
        !matches!(self, Self::QualityFloorFailed)
    }
}

/// Compare quality scores carried in the two payload-free observations.
///
/// Adapters may attach a `quality_score` metric from 0 through 100.  Missing
/// external scores fail closed; a ten-point gap is tolerated as equivalent,
/// while larger gaps are recorded as information loss.
#[must_use]
pub fn evaluate_quality(
    native_observation: &CapabilityObservationV1,
    external_observation: Option<&CapabilityObservationV1>,
) -> QualityCheck {
    let Some(external_observation) = external_observation else {
        return QualityCheck::QualityFloorFailed;
    };

    if !native_observation.success || !external_observation.success {
        return QualityCheck::QualityFloorFailed;
    }

    let native_score = native_observation
        .metrics
        .get("quality_score")
        .copied()
        .unwrap_or(100);
    let external_score = external_observation
        .metrics
        .get("quality_score")
        .copied()
        .unwrap_or(0);

    if external_score < QUALITY_FLOOR_SCORE {
        QualityCheck::QualityFloorFailed
    } else {
        let gap = native_score.saturating_sub(external_score);
        if gap > 10 {
            QualityCheck::InformationLoss {
                severity: if gap >= 25 { "high" } else { "moderate" }.to_string(),
            }
        } else {
            QualityCheck::StructurallyEquivalent
        }
    }
}

/// Select a comparison decision from token totals and the quality result.
#[must_use]
pub fn decide(
    native_tokens: u64,
    external_tokens: Option<u64>,
    quality_check: &QualityCheck,
) -> ComparisonDecision {
    let Some(external_tokens) = external_tokens else {
        return ComparisonDecision::ExternalUnavailable;
    };

    match quality_check {
        QualityCheck::QualityFloorFailed => ComparisonDecision::NativePreferred {
            reason: QUALITY_FLOOR_REASON,
        },
        QualityCheck::InformationLoss { .. } => ComparisonDecision::NativePreferred {
            reason: INFORMATION_LOSS_REASON,
        },
        QualityCheck::StructurallyEquivalent => match external_tokens.cmp(&native_tokens) {
            std::cmp::Ordering::Less => ComparisonDecision::ExternalPreferred {
                reason: EXTERNAL_SAVES_TOKENS,
            },
            std::cmp::Ordering::Greater => ComparisonDecision::NativePreferred {
                reason: NATIVE_SAVES_TOKENS,
            },
            std::cmp::Ordering::Equal => ComparisonDecision::Inconclusive {
                reason: TIED_REASON,
            },
        },
    }
}
