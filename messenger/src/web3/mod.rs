//! Web3 integrations — real calls only when wallet / RPC is configured.
//!
//! * No wallet → [`Web3Error::NotConfigured`] (never mock balances or quotes).
//! * Configured + build **without** `--features web3` → clear RPC error how to enable.
//! * Configured + `--features web3` → JSON-RPC over HTTP (`eth_chainId`, `eth_getBalance`).
//!
//! Escrow fee matches `smart-contracts/P2PEscrow.sol` (`FEE_BPS = 300` = 3%).
//! Env: `ADMIN_WALLET`, `RPC_URL`

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

#[derive(Serialize)]
struct JsonRpcRequest<'a> {
    jsonrpc: &'a str,
    method: &'a str,
    params: serde_json::Value,
    id: u64,
}

#[derive(Deserialize)]
struct JsonRpcResponse {
    result: Option<serde_json::Value>,
    error: Option<serde_json::Value>,
}

pub struct Web3Client {
    pub admin_wallet: Option<String>,
    pub rpc_url: Option<String>,
}

impl Web3Client {
    pub fn new(admin_wallet: Option<String>) -> Self {
        let admin_wallet = admin_wallet.or_else(|| std::env::var("ADMIN_WALLET").ok());
        let rpc_url = std::env::var("RPC_URL").ok();
        Self { admin_wallet, rpc_url }
    }

    pub fn is_configured(&self) -> bool {
        let wallet_ok = self
            .admin_wallet
            .as_ref()
            .map(|w| w.starts_with("0x") && w.len() >= 42)
            .unwrap_or(false);
        let rpc_ok = self
            .rpc_url
            .as_ref()
            .map(|u| u.starts_with("http"))
            .unwrap_or(false);
        wallet_ok && rpc_ok
    }

    pub fn escrow_fee_bps() -> u32 {
        300
    }

    pub async fn rpc(&self, method: &str, params: serde_json::Value) -> Result<serde_json::Value, Web3Error> {
        let url = self
            .rpc_url
            .as_ref()
            .filter(|u| u.starts_with("http"))
            .ok_or(Web3Error::NotConfigured)?;

        #[cfg(feature = "web3")]
        {
            return self.rpc_http(url, method, params).await;
        }

        #[cfg(not(feature = "web3"))]
        {
            let _ = (url, method, params);
            Err(Web3Error::Rpc(
                "build with --features web3 (reqwest) and set RPC_URL+ADMIN_WALLET".into(),
            ))
        }
    }

    #[cfg(feature = "web3")]
    async fn rpc_http(
        &self,
        url: &str,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, Web3Error> {
        let body = JsonRpcRequest {
            jsonrpc: "2.0",
            method,
            params,
            id: 1,
        };
        let client = reqwest::Client::new();
        let res = client
            .post(url)
            .json(&body)
            .send()
            .await
            .map_err(|e| Web3Error::Rpc(e.to_string()))?;
        let status = res.status();
        let text = res.text().await.map_err(|e| Web3Error::Rpc(e.to_string()))?;
        if !status.is_success() {
            return Err(Web3Error::Rpc(format!("{status}: {text}")));
        }
        let parsed: JsonRpcResponse =
            serde_json::from_str(&text).map_err(|e| Web3Error::Rpc(e.to_string()))?;
        if let Some(err) = parsed.error {
            return Err(Web3Error::Rpc(err.to_string()));
        }
        parsed.result.ok_or_else(|| Web3Error::Rpc("empty result".into()))
    }

    pub async fn chain_id(&self) -> Result<String, Web3Error> {
        if !self.is_configured() {
            return Err(Web3Error::NotConfigured);
        }
        let v = self.rpc("eth_chainId", serde_json::json!([])).await?;
        Ok(v.as_str().unwrap_or(&v.to_string()).to_string())
    }

    pub async fn admin_balance_wei(&self) -> Result<String, Web3Error> {
        if !self.is_configured() {
            return Err(Web3Error::NotConfigured);
        }
        let wallet = self.admin_wallet.as_ref().ok_or(Web3Error::NotConfigured)?;
        let v = self
            .rpc("eth_getBalance", serde_json::json!([wallet, "latest"]))
            .await?;
        Ok(v.as_str().unwrap_or(&v.to_string()).to_string())
    }

    pub async fn quote_swap(
        &self,
        from: &str,
        to: &str,
        amount: &str,
    ) -> Result<SwapQuote, Web3Error> {
        if !self.is_configured() {
            return Err(Web3Error::NotConfigured);
        }
        let _ = (from, to, amount);
        Err(Web3Error::Contract(
            "swap aggregator not linked — use P2PEscrow on-chain or set ZEROX_API_KEY path later"
                .into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn no_wallet_errors() {
        let c = Web3Client {
            admin_wallet: None,
            rpc_url: None,
        };
        assert!(!c.is_configured());
        let err = c.quote_swap("ETH", "USDC", "1").await.unwrap_err();
        assert!(matches!(err, Web3Error::NotConfigured));
        assert_eq!(Web3Client::escrow_fee_bps(), 300);
    }

    #[tokio::test]
    async fn partial_config_not_enough() {
        let c = Web3Client {
            admin_wallet: Some("0x0000000000000000000000000000000000000001".into()),
            rpc_url: None,
        };
        assert!(!c.is_configured());
        assert!(matches!(
            c.chain_id().await.unwrap_err(),
            Web3Error::NotConfigured
        ));
    }
}
