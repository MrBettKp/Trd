mod models;
mod config;
mod rpc_client;

use anyhow::Result;
use chrono::Utc;
use clap::Parser;
use std::fs::File;
use std::io::Write;

use crate::{
    config::load_config,
    models::{USDCTransfer, TransferDirection},
    rpc_client::SolanaRpcClient,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Output file path (JSON format)
    #[arg(short, long, default_value = "usdc_transfers.json")]
    output: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let config = load_config()?;

    println!("Indexing USDC transfers for wallet: {}", config.wallet_address);
    println!("Time range: last {} days", config.days_to_index);
    println!("Using RPC endpoint: {}", config.rpc_url);

    let rpc_client = SolanaRpcClient::new(config.rpc_url.clone());
    let transfers = rpc_client.get_usdc_transfers(&config).await?;

    println!("Found {} USDC transfers", transfers.len());

    // Write results to JSON file
    let mut file = File::create(&args.output)?;
    let json = serde_json::to_string_pretty(&transfers)?;
    file.write_all(json.as_bytes())?;

    println!("Results saved to {}", args.output);

    // Print summary to console
    print_summary(&transfers, &config.wallet_address.to_string());

    Ok(())
}

fn print_summary(transfers: &[USDCTransfer], wallet_address: &str) {
    println!("\nUSDC Transfer Summary:");
    println!("{:<25} | {:<10} | {:<8} | {:<45} | {:<45}", 
        "Timestamp", "Amount", "Direction", "From", "To");

    for transfer in transfers {
        let direction = match transfer.direction {
            TransferDirection::Incoming => "IN",
            TransferDirection::Outgoing => "OUT",
        };

        let from = if transfer.from.len() > 12 {
            format!("{}...{}", &transfer.from[0..4], &transfer.from[transfer.from.len()-4..])
        } else {
            transfer.from.clone()
        };

        let to = if transfer.to.len() > 12 {
            format!("{}...{}", &transfer.to[0..4], &transfer.to[transfer.to.len()-4..])
        } else {
            transfer.to.clone()
        };

        println!("{:<25} | {:<10.6} | {:<8} | {:<45} | {:<45}", 
            transfer.timestamp.format("%Y-%m-%d %H:%M:%S").to_string(),
            transfer.amount,
            direction,
            from,
            to);
    }

    let total_in: f64 = transfers.iter()
        .filter(|t| matches!(t.direction, TransferDirection::Incoming))
        .map(|t| t.amount)
        .sum();

    let total_out: f64 = transfers.iter()
        .filter(|t| matches!(t.direction, TransferDirection::Outgoing))
        .map(|t| t.amount)
        .sum();

    println!("\nTotal IN: {:.6} USDC", total_in);
    println!("Total OUT: {:.6} USDC", total_out);
    println!("Net change: {:.6} USDC", total_in - total_out);
}
