use anchor_lang::prelude::*;

#[account]
pub struct UserDeposit {
    pub slot: u64,
    pub amount: u64,
    pub owner: Pubkey,
    pub latest_deposit_timestamp: Option<i64>,
}

impl UserDeposit {
    pub const LEN: usize = 8 + 8 + 8 + 32 + 9;
}
