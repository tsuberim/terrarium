use serde::{Deserialize, Serialize};

/// Tracks lifetime destroy vs free-mint totals for the 2:1 budget rule.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct EnergyLedger {
    pub destroyed: i64,
    pub free_minted: i64,
}

impl EnergyLedger {
    pub const FREE_MINT_RATIO_NUM: i64 = 1;
    pub const FREE_MINT_RATIO_DEN: i64 = 2;

    pub fn free_budget(&self) -> i64 {
        (self.destroyed * Self::FREE_MINT_RATIO_NUM / Self::FREE_MINT_RATIO_DEN)
            .saturating_sub(self.free_minted)
    }

    pub fn try_mint_free(&mut self, amount: i64) -> i64 {
        let grant = amount.max(0).min(self.free_budget());
        if grant > 0 {
            self.free_minted += grant;
        }
        grant
    }

    pub fn record_destroy(&mut self, amount: i64) {
        if amount > 0 {
            self.destroyed += amount;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn free_budget_is_half_destroyed_minus_minted() {
        let mut ledger = EnergyLedger {
            destroyed: 1_000,
            free_minted: 0,
        };
        assert_eq!(ledger.free_budget(), 500);
        assert_eq!(ledger.try_mint_free(300), 300);
        assert_eq!(ledger.free_budget(), 200);
        assert_eq!(ledger.try_mint_free(500), 200);
        assert_eq!(ledger.free_minted, 500);
    }
}
