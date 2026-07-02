// Offramp Swap Plugin Interface
use serde::{Deserialize, Serialize};
use std::error::Error;

pub mod bitnob;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quote {
    pub quote_id: String,
    pub amount_sats: u64,
    pub fiat_amount: f64,
    pub fiat_currency: String,
    pub fee_sats: u64,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayoutDestination {
    pub bank_code: String,
    pub account_number: String,
    pub account_name: String,
    pub phone_number: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapResult {
    pub swap_id: String,
    pub status: SwapStatus,
    pub amount_sats: u64,
    pub fiat_amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SwapStatus {
    Pending,
    Succeeded,
    Failed(String),
}

pub trait SwapProvider: Send + Sync {
    fn name(&self) -> &str;
    fn get_quote(&self, amount_sats: u64, currency: &str) -> Result<Quote, Box<dyn Error>>;
    fn initiate_payout(
        &self,
        quote_id: &str,
        destination: PayoutDestination,
    ) -> Result<SwapResult, Box<dyn Error>>;
    fn get_status(&self, swap_id: &str) -> Result<SwapStatus, Box<dyn Error>>;
}

pub fn initialize_offramp() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_offramp_init() {
        assert!(initialize_offramp());
    }
}
