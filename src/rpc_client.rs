use solana_client::rpc_client::RpcClient;
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey, signature::Signature};
use solana_transaction_status::{UiTransactionEncoding, EncodedTransaction, UiTransactionTokenBalance, option_serializer::OptionSerializer};
use chrono::{Utc, Duration};
use anyhow::{Result, Context};
use std::str::FromStr;

use crate::models::{USDCTransfer, TransferDirection, Config};

pub struct SolanaRpcClient {
    client: RpcClient,
}

impl SolanaRpcClient {
    pub fn new(rpc_url: String) -> Self {
        let client = RpcClient::new_with_commitment(
            rpc_url,
            CommitmentConfig::confirmed(),
        );
        SolanaRpcClient { client }
    }

    pub async fn get_usdc_transfers(
        &self,
        config: &Config,
    ) -> Result<Vec<USDCTransfer>> {
        let wallet_address = config.wallet_address;
        let usdc_mint_address = config.usdc_mint_address;
        let days_to_index = config.days_to_index;

        // Calculate time range (last 24 hours)
        let end_time = Utc::now();
        let start_time = end_time - Duration::days(days_to_index);

        // Create config for get_signatures_for_address
        let config = solana_client::rpc_config::GetConfirmedSignaturesForAddress2Config {
            before: None,
            until: None,
            limit: None,
            commitment: Some(self.client.commitment()),
            min_context_slot: None,
        };

        // Get all signatures for the wallet
        let signatures = self.client.get_signatures_for_address_with_config(
            &wallet_address,
            config,
        )?;

        let mut transfers = Vec::new();

        for signature in signatures {
            let signature_str = signature.signature;
            let timestamp = signature.block_time.map(|t| {
                chrono::DateTime::<Utc>::from_timestamp(t, 0).unwrap()
            });

            // Skip if outside our time range
            let timestamp = match timestamp {
                Some(t) if t >= start_time => t,
                _ => continue,
            };

            // Get the full transaction
            let sig = Signature::from_str(&signature_str)?;
            let tx = self.client.get_transaction(
                &sig,
                UiTransactionEncoding::Json,
            )?;

            let tx_meta = tx.transaction.meta.context("No transaction metadata")?;
            let transaction_data = match tx.transaction.transaction {
                EncodedTransaction::Json(tx_data) => tx_data,
                _ => continue,
            };

            let message = transaction_data.message;

            // Get token balances if they exist
            let pre_token_balances = match tx_meta.pre_token_balances {
                OptionSerializer::Some(balances) => balances,
                _ => vec![],
            };
            let post_token_balances = match tx_meta.post_token_balances {
                OptionSerializer::Some(balances) => balances,
                _ => vec![],
            };

            // Find USDC token accounts involved
            let usdc_accounts: Vec<_> = post_token_balances
                .iter()
                .filter(|b| b.mint == usdc_mint_address.to_string())
                .collect();

            for balance in usdc_accounts {
                let account = balance.owner.parse::<Pubkey>()?;
                let pre_balance = pre_token_balances
                    .iter()
                    .find(|b| b.owner == balance.owner && b.mint == balance.mint)
                    .map(|b| b.ui_token_amount.ui_amount)
                    .unwrap_or(Some(0.0))
                    .unwrap_or(0.0);

                let post_balance = balance.ui_token_amount.ui_amount.unwrap_or(0.0);
                let amount = (post_balance - pre_balance).abs();

                if amount > 0.0 {
                    let direction = if account == wallet_address {
                        TransferDirection::Incoming
                    } else {
                        TransferDirection::Outgoing
                    };

                    let from = if direction == TransferDirection::Incoming {
                        pre_token_balances
                            .iter()
                            .find(|b| b.mint == usdc_mint_address.to_string())
                            .map(|b| b.owner.to_string())
                            .unwrap_or_else(|| "unknown".to_string())
                    } else {
                        wallet_address.to_string()
                    };

                    let to = if direction == TransferDirection::Incoming {
                        wallet_address.to_string()
                    } else {
                        post_token_balances
                            .iter()
                            .find(|b| b.mint == usdc_mint_address.to_string() && b.owner != from)
                            .map(|b| b.owner.to_string())
                            .unwrap_or_else(|| "unknown".to_string())
                    };

                    transfers.push(USDCTransfer {
                        signature: signature_str.clone(),
                        timestamp,
                        from,
                        to,
                        amount,
                        direction,
                    });
                }
            }
        }

        Ok(transfers)
    }
}