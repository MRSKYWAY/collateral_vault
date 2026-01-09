use serde_json::json;
use crate::models::IntentResponse;

const PROGRAM: &str = "collateral_vault";

// fn base_intent(instruction: &str, params: serde_json::Value) -> IntentResponse {
//     IntentResponse {
//     program: PROGRAM,
//     instruction,
//     params,
//     note: "Client must build and sign the Anchor instruction",
// }
// }

const NOTE: &str = "Client must build and sign the Anchor instruction";

pub fn deposit_intent(amount: u64) -> IntentResponse {
    IntentResponse {
        program: PROGRAM,
        instruction: "deposit",
        params: json!({ "amount": amount }),
        note: NOTE,
    }
}

pub fn withdraw_intent(amount: u64) -> IntentResponse {
    IntentResponse {
        program: PROGRAM,
        instruction: "withdraw",
        params: json!({ "amount": amount }),
        note: NOTE,
    }
}

pub fn lock_intent(amount: u64) -> IntentResponse {
    IntentResponse {
        program: PROGRAM,
        instruction: "lock_collateral",
        params: json!({ "amount": amount }),
        note: NOTE,
    }
}

pub fn unlock_intent(amount: u64) -> IntentResponse {
    IntentResponse {
        program: PROGRAM,
        instruction: "unlock_collateral",
        params: json!({ "amount": amount }),
        note: NOTE,
    }
}

pub fn transfer_intent(from: &str, to: &str, amount: u64) -> IntentResponse {
    IntentResponse {
        program: PROGRAM,
        instruction: "transfer_collateral",
        params: json!({
            "from": from,
            "to": to,
            "amount": amount
        }),
        note: NOTE,
    }
}

