use super::{ValueAssessment, cpao};
use crate::core::savings_ledger::roi_report;
use serde::Serialize;

#[derive(Debug, Clone, Default, Serialize)]
pub struct ValueReport {
    pub total_tasks: usize,
    pub accepted_rate: f64,
    pub cpao_micros: Option<u64>,
    pub etpao_tokens: Option<u64>,
    pub total_cost_micros: u64,
    pub savings_usd: f64,
    pub tasks: Vec<ValueAssessment>,
}

pub fn build(tasks: Vec<ValueAssessment>) -> ValueReport {
    let total_cost_micros = tasks.iter().map(|task| task.cost_micros).sum();
    let total_tokens = tasks.iter().map(|task| task.total_tokens).sum();
    let costs = tasks
        .iter()
        .map(|task| task.cost_micros)
        .collect::<Vec<_>>();
    let accepted = tasks
        .iter()
        .map(|task| task.outcome_accepted)
        .collect::<Vec<_>>();
    let accepted_count = accepted.iter().filter(|&&ok| ok).count() as u64;
    ValueReport {
        total_tasks: tasks.len(),
        accepted_rate: if tasks.is_empty() {
            0.0
        } else {
            accepted_count as f64 / tasks.len() as f64
        },
        cpao_micros: cpao::cost_per_accepted_outcome(&costs, &accepted),
        etpao_tokens: cpao::etpao(total_tokens, accepted_count),
        total_cost_micros,
        savings_usd: roi_report(crate::core::agent_identity::current_agent_id()).saved_usd,
        tasks,
    }
}

pub fn table(report: &ValueReport) -> String {
    let mut output = format!(
        "Value Report\nTasks: {}  Accepted: {:.1}%  CPAO: {}  ETPAO: {}  Cost: ${:.4}  Savings: ${:.2}\n\n{:<12} {:>10} {:<9} {:<18}\n{}\n",
        report.total_tasks,
        report.accepted_rate * 100.0,
        money(report.cpao_micros),
        tokens(report.etpao_tokens),
        report.total_cost_micros as f64 / 1_000_000.0,
        report.savings_usd,
        "Task",
        "Cost",
        "Outcome",
        "Model",
        "-".repeat(54)
    );
    for task in &report.tasks {
        output.push_str(&format!(
            "{:<12} {:>10} {:<9} {:<18}\n",
            short(&task.task_id),
            money(Some(task.cost_micros)),
            outcome(task),
            task.model
        ));
    }
    output
}

pub fn markdown(report: &ValueReport) -> String {
    let mut output = format!(
        "# Value Report\n\n- **Total tasks:** {}\n- **Accepted rate:** {:.1}%\n- **CPAO:** {}\n- **ETPAO:** {}\n- **Total cost:** ${:.4}\n- **Savings:** ${:.2}\n\n| Task | Cost | Outcome | Model |\n|---|---:|---|---|\n",
        report.total_tasks,
        report.accepted_rate * 100.0,
        money(report.cpao_micros),
        tokens(report.etpao_tokens),
        report.total_cost_micros as f64 / 1_000_000.0,
        report.savings_usd
    );
    for task in &report.tasks {
        output.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            short(&task.task_id),
            money(Some(task.cost_micros)),
            outcome(task),
            task.model
        ));
    }
    output
}

pub fn json(report: &ValueReport) -> String {
    serde_json::to_string_pretty(report).unwrap_or_else(|_| "{}".to_string())
}
fn outcome(task: &ValueAssessment) -> &str {
    if task.outcome_accepted {
        "accepted"
    } else {
        "rejected"
    }
}
fn short(id: &str) -> &str {
    id.get(..12).unwrap_or(id)
}
fn money(value: Option<u64>) -> String {
    value.map_or_else(
        || "n/a".to_string(),
        |value| format!("${:.4}", value as f64 / 1_000_000.0),
    )
}
fn tokens(value: Option<u64>) -> String {
    value.map_or_else(|| "n/a".to_string(), |value| value.to_string())
}
