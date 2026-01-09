use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct DepositIntent {
    pub instruction: String,
    pub amount: u64,
}

pub fn build_deposit_intent(amount: u64) -> DepositIntent {
    DepositIntent {
        instruction: "deposit".to_string(),
        amount,
    }
}
