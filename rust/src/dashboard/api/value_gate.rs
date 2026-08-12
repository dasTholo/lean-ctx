//! Read-only ValueGate assessment summary for the cockpit.

use serde_json::json;

pub(crate) fn handle(path: &str, method: &str) -> Option<(&'static str, &'static str, String)> {
    (path == "/api/value-gate/summary").then(|| {
        if !method.eq_ignore_ascii_case("GET") {
            return ("405 Method Not Allowed", "application/json", r#"{"error":"method not allowed"}"#.to_string());
        }
        let store = crate::core::value_gate::store();
        let aggregate = store.aggregate();
        let recent_assessments: Vec<_> = store.recent(10).into_iter().map(|a| json!({
            "task_id": a.task_id, "cost_micros": a.cost_micros,
            "outcome_accepted": a.outcome_accepted, "cpao_micros": a.cpao_micros,
            "evidence": a.evidence, "timestamp": a.timestamp,
        })).collect();
        let body = serde_json::to_string(&json!({
            "recent_assessments": recent_assessments,
            "aggregate": { "total": aggregate.total, "accepted": aggregate.accepted,
                "rejected": aggregate.rejected, "avg_cpao": aggregate.avg_cpao,
                "total_cost": aggregate.total_cost },
        })).unwrap_or_else(|_| r#"{"recent_assessments":[],"aggregate":{"total":0,"accepted":0,"rejected":0,"avg_cpao":null,"total_cost":0}}"#.to_string());
        ("200 OK", "application/json", body)
    })
}
