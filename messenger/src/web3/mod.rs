use serde::{Deserialize, Serialize};

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
}

impl Web3Client {
    pub fn new(admin_wallet: Option<String>) -> Self {
        Self { admin_wallet }
    }

    pub async fn quote_swap(&self, from: &str, to: &str, amount: &str) -> anyhow::Result<SwapQuote> {
        // Wire 0x/ABCEX when API keys present in env
        Ok(SwapQuote {
            from_token: from.into(),
            to_token: to.into(),
            amount_in: amount.into(),
            amount_out: amount.into(),
            fee_bps: 50,
        })
    }
}
