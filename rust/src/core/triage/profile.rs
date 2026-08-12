#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TaskProfileLocal {
    pub task_class: String,
    pub intent: String,
    pub complexity: String,
    pub scope: TaskScopeLocal,
    pub context_need_milli: u16,
    pub reasoning_need_milli: u16,
    pub risk_signal_milli: u16,
    pub confidence_milli: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum TaskScopeLocal {
    #[default]
    SingleFile,
    MultiFile,
    CrossModule,
    CrossProject,
}

impl Default for TaskProfileLocal {
    fn default() -> Self {
        Self {
            task_class: String::new(),
            intent: String::new(),
            complexity: "low".into(),
            scope: TaskScopeLocal::default(),
            context_need_milli: 0,
            reasoning_need_milli: 0,
            risk_signal_milli: 0,
            confidence_milli: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_scope_is_single_file() {
        assert_eq!(TaskScopeLocal::default(), TaskScopeLocal::SingleFile);
    }
    #[test]
    fn default_complexity_is_low() {
        assert_eq!(TaskProfileLocal::default().complexity, "low");
    }
    #[test]
    fn default_confidence_is_zero() {
        assert_eq!(TaskProfileLocal::default().confidence_milli, 0);
    }
}
