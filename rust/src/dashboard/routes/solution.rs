use serde::Serialize;

pub(super) fn handle(
    path: &str,
    _query_str: &str,
    _method: &str,
    _body: &str,
) -> Option<(&'static str, &'static str, String)> {
    match path {
        "/api/solution" => Some(solution_response()),
        _ => None,
    }
}

#[derive(Serialize)]
struct SolutionApiResponse {
    enabled: bool,
    intensity: String,
    output_savings: OutputSavings,
    loc: LocMetrics,
    decisions: DecisionMetrics,
}

#[derive(Serialize)]
struct OutputSavings {
    tokens_total: u64,
    tokens_optimized: u64,
    reduction_pct: u8,
}

#[derive(Serialize)]
struct LocMetrics {
    added: u64,
    removed: u64,
    net_reduced: i64,
}

#[derive(Serialize)]
struct DecisionMetrics {
    total: u64,
    stdlib: u64,
    native: u64,
    reuse: u64,
    yagni: u64,
    one_line: u64,
    debt_open: u64,
}

fn solution_response() -> (&'static str, &'static str, String) {
    let config = crate::core::config::Config::load();
    let snapshot = crate::core::solution_tracker::snapshot();
    let response = SolutionApiResponse {
        enabled: config.solution.enabled,
        intensity: config.solution.effective_intensity().label().to_string(),
        output_savings: OutputSavings {
            tokens_total: snapshot.output_tokens_baseline,
            tokens_optimized: snapshot.output_tokens_actual,
            reduction_pct: snapshot.output_reduction_pct,
        },
        loc: LocMetrics {
            added: snapshot.loc_added,
            removed: snapshot.loc_removed,
            net_reduced: snapshot.loc_net_saved,
        },
        decisions: DecisionMetrics {
            total: snapshot.decisions_total,
            stdlib: snapshot
                .decisions_by_kind
                .get("stdlib")
                .copied()
                .unwrap_or_default(),
            native: snapshot
                .decisions_by_kind
                .get("native")
                .copied()
                .unwrap_or_default(),
            reuse: snapshot
                .decisions_by_kind
                .get("reuse")
                .copied()
                .unwrap_or_default(),
            yagni: snapshot
                .decisions_by_kind
                .get("yagni")
                .copied()
                .unwrap_or_default(),
            one_line: snapshot
                .decisions_by_kind
                .get("oneline")
                .copied()
                .unwrap_or_default(),
            debt_open: snapshot
                .decisions_by_kind
                .get("debt")
                .copied()
                .unwrap_or_default(),
        },
    };

    (
        "200 OK",
        "application/json",
        serde_json::to_string(&response).unwrap_or_else(|_| "{}".to_string()),
    )
}
