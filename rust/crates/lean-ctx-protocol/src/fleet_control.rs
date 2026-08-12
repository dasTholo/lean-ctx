//! Multi-agent authorization and budget-control contracts.

use crate::{AgentId, MoneyV1};
use serde::{Deserialize, Serialize};

/// Policy constraining one agent's model and provider access.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentPolicy {
    pub agent_id: AgentId,
    pub budget_limit: MoneyV1,
    pub allowed_models: Vec<String>,
    pub allowed_providers: Vec<String>,
    pub priority: u8,
}

/// Current authorization and spend state for an agent fleet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FleetState {
    pub active_agents: Vec<AgentId>,
    pub total_spend: MoneyV1,
    pub budget_remaining: MoneyV1,
}

/// Extension point for Enterprise fleet controllers such as `BudgetedFleetControl`.
pub trait FleetControlContract {
    /// Authorize an agent's estimated cost before it is incurred.
    fn authorize(&self, agent_id: &AgentId, cost_estimate: &MoneyV1) -> bool;
}

/// OSS single-user fleet controller that imposes no authorization limits.
#[derive(Debug, Default, Clone, Copy)]
pub struct LocalFleetControl;

impl FleetControlContract for LocalFleetControl {
    fn authorize(&self, _agent_id: &AgentId, _cost_estimate: &MoneyV1) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cost() -> MoneyV1 {
        MoneyV1 {
            currency: "USD".to_owned(),
            coefficient: 1,
            scale: 4,
        }
    }

    #[test]
    fn local_fleet_control_always_authorizes() {
        let agent_id = AgentId::try_from("agent-1".to_owned()).expect("identifier should be valid");
        assert!(LocalFleetControl.authorize(&agent_id, &cost()));
    }

    #[test]
    fn contract_is_object_safe() {
        let fleet_control: &dyn FleetControlContract = &LocalFleetControl;
        let agent_id = AgentId::try_from("agent-1".to_owned()).expect("identifier should be valid");
        assert!(fleet_control.authorize(&agent_id, &cost()));
    }
}
