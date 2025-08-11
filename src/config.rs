use std::str::FromStr;
use solana_sdk::pubkey::Pubkey;
use dotenv::dotenv;
use std::env;
use anyhow::{Result, anyhow};

use crate::models::Config;

pub fn load_config() -> Result<Config> {
    dotenv().ok();

    let rpc_url = env::var("RPC_URL")
        .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string());
    
    let wallet_address = env::var("WALLET_ADDRESS")
        .map_err(|_| anyhow!("WALLET_ADDRESS must be set"))?;
    
    let wallet_address = Pubkey::from_str(&wallet_address)?;

    // USDC mint address on Solana mainnet
    let usdc_mint_address = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")?;

    let days_to_index = env::var("DAYS_TO_INDEX")
        .map(|s| s.parse().unwrap_or(1))
        .unwrap_or(1);

    Ok(Config {
        rpc_url,
        wallet_address,
        usdc_mint_address,
        days_to_index,
    })
}
