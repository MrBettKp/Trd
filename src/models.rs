use serde::Serialize;
use solana_sdk::pubkey::Pubkey;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize)]
pub struct USDCTransfer {
    pub signature: String,
    pub timestamp: DateTime<Utc>,
    pub from: String,
    pub to: String,
    pub amount: f64,
    pub direction: TransferDirection,
}

#[derive(Debug, Serialize)]
pub enum TransferDirection {
    Incoming,
    Outgoing,
}

#[derive(Debug)]
pub struct Config {
    pub rpc_url: String,
    pub wallet_address: Pubkey,
    pub usdc_mint_address: Pubkey,
    pub days_to_index: i64,
  }
