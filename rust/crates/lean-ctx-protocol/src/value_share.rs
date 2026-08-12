//! Savings attribution and value-share settlement contracts.

use crate::MoneyV1;
use serde::{Deserialize, Serialize};

/// Attributed, evidence-backed savings for one customer billing period.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SavingsAttribution {
    pub customer_id: String,
    pub period: String,
    pub baseline_cost: MoneyV1,
    pub treatment_cost: MoneyV1,
    pub proven_savings: MoneyV1,
    pub share_percentage: u16,
}

/// Lifecycle state of a value-share settlement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SettlementStatus {
    NoSettlement,
    Pending,
    Settled,
    Disputed,
}

/// Result of calculating a value-share settlement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SettlementRecord {
    pub attribution: SavingsAttribution,
    pub invoice_amount: MoneyV1,
    pub settlement_status: SettlementStatus,
}

/// Extension point for Enterprise billing implementations such as `VerifiedValueShare`.
pub trait ValueShareContract {
    /// Calculate the settlement owed for a savings attribution.
    fn calculate_share(&self, attribution: SavingsAttribution) -> SettlementRecord;
}

/// OSS value-share calculator that never produces a billable settlement.
#[derive(Debug, Default, Clone, Copy)]
pub struct LocalValueShare;

impl ValueShareContract for LocalValueShare {
    fn calculate_share(&self, attribution: SavingsAttribution) -> SettlementRecord {
        let invoice_amount = MoneyV1 {
            currency: attribution.proven_savings.currency.clone(),
            coefficient: 0,
            scale: attribution.proven_savings.scale,
        };
        SettlementRecord {
            attribution,
            invoice_amount,
            settlement_status: SettlementStatus::NoSettlement,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn attribution() -> SavingsAttribution {
        SavingsAttribution {
            customer_id: "customer-1".to_owned(),
            period: "2026-08".to_owned(),
            baseline_cost: MoneyV1 {
                currency: "USD".to_owned(),
                coefficient: 10_000,
                scale: 4,
            },
            treatment_cost: MoneyV1 {
                currency: "USD".to_owned(),
                coefficient: 5_000,
                scale: 4,
            },
            proven_savings: MoneyV1 {
                currency: "USD".to_owned(),
                coefficient: 5_000,
                scale: 4,
            },
            share_percentage: 200,
        }
    }

    #[test]
    fn local_value_share_returns_zero_settlement() {
        let settlement = LocalValueShare.calculate_share(attribution());
        assert_eq!(settlement.invoice_amount.coefficient, 0);
        assert_eq!(settlement.settlement_status, SettlementStatus::NoSettlement);
    }

    #[test]
    fn contract_is_object_safe() {
        let value_share: &dyn ValueShareContract = &LocalValueShare;
        assert_eq!(
            value_share
                .calculate_share(attribution())
                .invoice_amount
                .coefficient,
            0
        );
    }
}
