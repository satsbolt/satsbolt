// Bitnob Offramp Swap Provider Implementation
use crate::{PayoutDestination, Quote, SwapProvider, SwapResult, SwapStatus};
use chrono::{Duration, Utc};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::error::Error;

// --- Bitnob API Requests and Responses ---

#[derive(Serialize)]
struct BitnobQuoteRequest<'a> {
    amount: u64,
    source_asset: &'a str,
    target_asset: &'a str,
}

#[derive(Deserialize)]
struct BitnobQuoteResponse {
    status: String,
    data: Option<BitnobQuoteData>,
    message: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BitnobQuoteData {
    id: String,
    target_amount: f64,
    fee: f64,
}

#[derive(Serialize)]
struct BitnobPayoutRequest<'a> {
    #[serde(rename = "quoteId")]
    quote_id: &'a str,
    #[serde(rename = "bankCode")]
    bank_code: &'a str,
    #[serde(rename = "accountNumber")]
    account_number: &'a str,
    #[serde(rename = "accountName")]
    account_name: &'a str,
}

#[derive(Deserialize)]
struct BitnobPayoutResponse {
    status: String,
    data: Option<BitnobPayoutData>,
    message: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BitnobPayoutData {
    id: String,
    status: String,
    amount: f64,
    target_amount: f64,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct BitnobStatusResponse {
    status: String,
    data: Option<BitnobStatusData>,
    message: Option<String>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct BitnobStatusData {
    id: String,
    status: String,
}

// --- Bitnob Provider ---

pub struct BitnobProvider {
    client: Client,
    api_key: String,
    base_url: String,
}

impl BitnobProvider {
    pub fn new(api_key: String, base_url: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url,
        }
    }
}

impl SwapProvider for BitnobProvider {
    fn name(&self) -> &str {
        "Bitnob"
    }

    fn get_quote(&self, amount_sats: u64, currency: &str) -> Result<Quote, Box<dyn Error>> {
        let url = format!("{}/payouts/quotes", self.base_url);
        let payload = BitnobQuoteRequest {
            amount: amount_sats,
            source_asset: "SATS",
            target_asset: currency,
        };

        let response: BitnobQuoteResponse = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()?
            .json()?;

        if response.status != "success" {
            let err_msg = response
                .message
                .unwrap_or_else(|| "Failed to get quote".to_string());
            return Err(err_msg.into());
        }

        let data = response
            .data
            .ok_or("Bitnob returned empty data for quote")?;

        Ok(Quote {
            quote_id: data.id,
            amount_sats,
            fiat_amount: data.target_amount,
            fiat_currency: currency.to_string(),
            fee_sats: data.fee as u64,
            expires_at: Utc::now() + Duration::minutes(15),
        })
    }

    fn initiate_payout(
        &self,
        quote_id: &str,
        destination: PayoutDestination,
    ) -> Result<SwapResult, Box<dyn Error>> {
        let url = format!("{}/payouts/initiate", self.base_url);
        let payload = BitnobPayoutRequest {
            quote_id,
            bank_code: &destination.bank_code,
            account_number: &destination.account_number,
            account_name: &destination.account_name,
        };

        let response: BitnobPayoutResponse = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()?
            .json()?;

        if response.status != "success" {
            let err_msg = response
                .message
                .unwrap_or_else(|| "Failed to initiate payout".to_string());
            return Err(err_msg.into());
        }

        let data = response
            .data
            .ok_or("Bitnob returned empty data for payout initiation")?;

        let status = match data.status.as_str() {
            "success" | "completed" => SwapStatus::Succeeded,
            "failed" => SwapStatus::Failed("Bitnob transaction failed".to_string()),
            _ => SwapStatus::Pending,
        };

        Ok(SwapResult {
            swap_id: data.id,
            status,
            amount_sats: data.amount as u64,
            fiat_amount: data.target_amount,
        })
    }

    fn get_status(&self, swap_id: &str) -> Result<SwapStatus, Box<dyn Error>> {
        let url = format!("{}/payouts/transactions/{}", self.base_url, swap_id);

        let response: BitnobStatusResponse = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()?
            .json()?;

        if response.status != "success" {
            let err_msg = response
                .message
                .unwrap_or_else(|| "Failed to fetch status".to_string());
            return Err(err_msg.into());
        }

        let data = response.data.ok_or("Bitnob status data empty")?;
        match data.status.as_str() {
            "success" | "completed" => Ok(SwapStatus::Succeeded),
            "failed" => Ok(SwapStatus::Failed("Bitnob transaction failed".to_string())),
            _ => Ok(SwapStatus::Pending),
        }
    }
}

// --- Mock Sandbox Fallback Provider for testing ---

pub struct MockSwapProvider;

impl SwapProvider for MockSwapProvider {
    fn name(&self) -> &str {
        "Mock Sandbox Provider"
    }

    fn get_quote(&self, amount_sats: u64, currency: &str) -> Result<Quote, Box<dyn Error>> {
        // Assume exchange rate: 1 Sat = 0.0006 target currency
        let fiat_amount = (amount_sats as f64) * 0.0006;
        let fee_sats = 100; // Static 100 sats fee

        Ok(Quote {
            quote_id: format!("mock_quote_{}", uuid::Uuid::new_v4()),
            amount_sats,
            fiat_amount,
            fiat_currency: currency.to_string(),
            fee_sats,
            expires_at: Utc::now() + Duration::minutes(15),
        })
    }

    fn initiate_payout(
        &self,
        _quote_id: &str,
        destination: PayoutDestination,
    ) -> Result<SwapResult, Box<dyn Error>> {
        // For testing, mock a successful transaction creation
        let amount_sats = 10000;
        let fiat_amount = (amount_sats as f64) * 0.0006;

        // Auto-fail if destination is special name "fail" for testing
        if destination.account_name.to_lowercase() == "fail" {
            return Ok(SwapResult {
                swap_id: format!("mock_swap_failed_{}", uuid::Uuid::new_v4()),
                status: SwapStatus::Failed("Simulated destination failure".to_string()),
                amount_sats,
                fiat_amount,
            });
        }

        Ok(SwapResult {
            swap_id: format!("mock_swap_{}", uuid::Uuid::new_v4()),
            status: SwapStatus::Pending, // Starts as pending, becomes succeeded on status check
            amount_sats,
            fiat_amount,
        })
    }

    fn get_status(&self, swap_id: &str) -> Result<SwapStatus, Box<dyn Error>> {
        if swap_id.contains("failed") {
            Ok(SwapStatus::Failed("Mock payout failed".to_string()))
        } else {
            Ok(SwapStatus::Succeeded)
        }
    }
}

// --- Factory Constructor ---

pub fn get_provider(api_key: Option<String>, base_url: Option<String>) -> Box<dyn SwapProvider> {
    if let (Some(key), Some(url)) = (api_key, base_url) {
        if !key.trim().is_empty() && !url.trim().is_empty() {
            return Box::new(BitnobProvider::new(key, url));
        }
    }
    Box::new(MockSwapProvider)
}
