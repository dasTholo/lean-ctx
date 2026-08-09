//! Sandboxed external capability reference adapters.
//!
//! Reference adapters are deliberately isolated from the production adapter
//! registry.  They may collect observations and benchmark reports, but shadow
//! execution never supplies a production response or mutates one.

pub mod comparison_receipt;
pub mod kill_switch;
pub mod report;
pub mod rtk_shell;
pub mod shadow_runner;

pub use comparison_receipt::{
    ComparisonDecision, ComparisonReceipt, QUALITY_FLOOR_SCORE, QualityCheck, decide,
    evaluate_quality,
};
pub use kill_switch::KillSwitch;
pub use report::{
    AggregateTokenStats, CategorySummary, ComparisonReport, EXPECTED_FIXTURE_COUNT, FixtureInput,
    FixtureMetadata, ReportError, WorkloadComparison, generate_comparison_report,
    generate_from_fixtures, generate_report, load_fixtures, write_json, write_text,
};
pub use rtk_shell::{CapabilityFailure, RtkConfig, RtkHealthReport, RtkShellAdapter};
pub use shadow_runner::{
    QualityAssessment, ShadowComparisonReport, ShadowDecision, ShadowRunner,
    StructuralQualityAssessment,
};
