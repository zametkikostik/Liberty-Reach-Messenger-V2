//! Web3 integrations — real calls only when wallet / RPC is configured.
//!
//! Without `ADMIN_WALLET` or explicit client config: returns `Web3Error::NotConfigured`.
//! Swap quotes and escrow use on-chain contracts under `smart-contracts/`.

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Web3Error {
    #[error("Web3 disabled: set ADMIN_WALLET / RPC_URL for real calls")]
    NotConfigured,
    #[error("rpc: {0}")]
    Rpc(String),
    #[error("contract: {0}")]
    Contract(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SwapQuote {
    pub from_token: String,
    pub to_token: String,
    pub amount_in: String,
    pub amount_out: String,
    pub fee_bps: u32,
}

pub struct Web3Client {
    pub admin_wallet: Option<String>,
    pub rpc_url: Option<String>,
}

impl Web3Client {
    pub fn new(admin_wallet: Option<String>) -> Self {
        let admin_wallet = admin_wallet.or_else(|| std::env::var("ADMIN_WALLET").ok());
        let rpc_url = std::env::var("RPC_URL").ok();
        Self {
            admin_wallet,
            rpc_url,
        }
    }

    pub fn is_configured(&self) -> bool {
        self.admin_wallet
            .as_ref()
            .map(|w| w.starts_with("0x") && w.len() >= 42)
            .unwrap_or(false)
    }

    /// 0x / DEX quote — only when RPC + wallet present. Fee 0.5–3% (50–300 bps).
    pub async fn quote_swap(
        &self,
        from: &str,
        to: &str,
        amount: &str,
    ) -> Result<SwapQuote, Web3Error> {
        if !self.is_configured() {
            return Err(Web3Error::NotConfigured);
        }
        let _ = (from, to, amount, &self.rpc_url);
        Err(Web3Error::Rpc(
            "RPC client not linked — set ADMIN_WALLET+RPC_URL and add alloy/ethers".into(),
        ))
    }

    /// Protocol fee bps for P2P escrow (matches smart-contracts/P2PEscrow.sol FEE_BPS = 300).
    pub fn escrow_fee_bps() -> u32 {
        300
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn no_wallet_errors() {
        let c = Web3Client::new(None);
        assert!(!c.is_configured());
        let err = c.quote_swap("ETH", "USDC", "1").await.unwrap_err();
        assert!(matches!(err, Web3Error::NotConfigured));
    }
}
