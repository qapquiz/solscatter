use anchor_lang::prelude::*;

#[account]
pub struct MainState {
    pub current_slot: u64,
    pub current_round: u64,
    pub total_deposit: u64,
    pub vrf_account_pubkey: Pubkey,
    pub penalty_period: i64,
    pub penalty_fee: f64,
    pub default_fee: f64,
}

impl MainState {
    pub const LEN: usize = 8 + 8 + 8 + 8 + 32 + 9 + 8 + 8;
}