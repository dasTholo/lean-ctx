//! Non-blocking bridge between MCP tool execution and the decision loop.

use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{Arc, Mutex, OnceLock},
    time::Instant,
};

use crate::core::{
    decision_loop::protocol_profile,
    task_spine::TaskSpine,
    triage::{TaskAnalysisInput, TriageEngine},
    value_gate::{
        ExecutionCost, OutcomeSignal, TaskOutcome, ValueGate, ValueGateStore,
        cost_tracker::calculate_cost,
    },
};

#[derive(Debug)]
/// Bridges MCP tool lifecycle events into decision-loop accounting.
pub struct DecisionLoopRuntime {
    triage: TriageEngine,
    value_gate_store: Arc<Mutex<ValueGateStore>>,
}

#[derive(Debug)]
/// Tracks a tool invocation while its decision-loop work is in progress.
pub struct TaskContext {
    pub task_id: String,
    pub profile_intent: String,
    pub profile_complexity: String,
    pub start_time: Instant,
}

impl DecisionLoopRuntime {
    pub fn get_or_init() -> &'static Self {
        static RUNTIME: OnceLock<DecisionLoopRuntime> = OnceLock::new();
        RUNTIME.get_or_init(|| Self {
            triage: TriageEngine::default(),
            value_gate_store: Arc::new(Mutex::new(ValueGateStore::default())),
        })
    }

    pub fn on_tool_start(
        &self,
        tool_name: &str,
        query: &str,
        session_id: &str,
        agent_id: &str,
    ) -> TaskContext {
        catch_unwind(AssertUnwindSafe(|| {
            self.on_tool_start_inner(tool_name, query, session_id, agent_id)
        }))
        .unwrap_or_else(|_| TaskContext {
            task_id: String::new(),
            profile_intent: String::new(),
            profile_complexity: String::new(),
            start_time: Instant::now(),
        })
    }

    fn on_tool_start_inner(
        &self,
        tool_name: &str,
        query: &str,
        session_id: &str,
        agent_id: &str,
    ) -> TaskContext {
        let profile = self
            .triage
            .analyze(&TaskAnalysisInput {
                query: format!("{tool_name}: {query}"),
                ..Default::default()
            })
            .map(|hypothesis| hypothesis.profile)
            .unwrap_or_default();
        let mut envelope = TaskSpine::create_envelope(query, session_id, agent_id);
        TaskSpine::enrich_from_triage(&mut envelope, &protocol_profile(&profile));
        TaskContext {
            task_id: envelope.task_id.as_str().to_owned(),
            profile_intent: profile.intent,
            profile_complexity: profile.complexity,
            start_time: Instant::now(),
        }
    }

    pub fn on_tool_end(
        &self,
        ctx: &TaskContext,
        input_tokens: u64,
        output_tokens: u64,
        model: &str,
        success: bool,
    ) {
        let _ = catch_unwind(AssertUnwindSafe(|| {
            self.on_tool_end_inner(ctx, input_tokens, output_tokens, model, success);
        }));
    }

    fn on_tool_end_inner(
        &self,
        ctx: &TaskContext,
        input_tokens: u64,
        output_tokens: u64,
        model: &str,
        success: bool,
    ) {
        let cost = ExecutionCost {
            input_tokens,
            output_tokens,
            cache_read_tokens: 0,
            model: model.to_owned(),
            provider: "mcp".to_owned(),
            estimated_cost_micros: calculate_cost(input_tokens, output_tokens, 0, model),
        };
        let outcome = TaskOutcome {
            task_id: ctx.task_id.clone(),
            completed: true,
            signals: vec![if success {
                OutcomeSignal::BuildSucceeded
            } else {
                OutcomeSignal::CompileError
            }],
        };
        let assessment = ValueGate::evaluate_task(&ctx.task_id, &cost, &outcome);
        self.value_gate_store
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .record(&assessment);
    }

    #[cfg(test)]
    pub(crate) fn with_triage(triage: TriageEngine) -> Self {
        Self {
            triage,
            value_gate_store: Arc::new(Mutex::new(ValueGateStore::default())),
        }
    }

    #[cfg(test)]
    pub(crate) fn latest_assessment_accepted(&self) -> Option<bool> {
        self.value_gate_store
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .recent(1)
            .first()
            .map(|assessment| assessment.outcome_accepted)
    }

    #[cfg(test)]
    pub(crate) fn assessment_for(
        &self,
        task_id: &str,
    ) -> Option<crate::core::value_gate::ValueAssessment> {
        self.value_gate_store
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .recent(100)
            .into_iter()
            .find(|assessment| assessment.task_id == task_id)
    }
}
