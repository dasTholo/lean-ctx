//! Thompson Sampling model router (OSS stub).
//!
//! Enterprise selects the optimal LLM per task class using multi-armed bandits.
//! OSS: no-op (pass-through to configured model).

/// Global model router accessor (OSS: no-op mutex).
pub fn global_model_router() -> &'static std::sync::Mutex<ModelRouter> {
    use std::sync::Mutex;
    static ROUTER: std::sync::LazyLock<Mutex<ModelRouter>> =
        std::sync::LazyLock::new(|| Mutex::new(ModelRouter));
    &ROUTER
}

/// Model router (OSS: no-op).
pub struct ModelRouter;

impl ModelRouter {
    /// Selects a model from candidates (OSS: returns first).
    pub fn select_model<'a>(&self, _task_class: &str, candidates: &'a [&str]) -> &'a str {
        candidates.first().copied().unwrap_or("default")
    }

    /// Records outcome for learning (OSS: no-op).
    pub fn record_outcome(&mut self, _model: &str, _task_class: &str, _accepted: bool, _cost: f64) {
    }
}
