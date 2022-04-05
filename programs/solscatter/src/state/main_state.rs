use anchor_lang::prelude::*;
use crate::state::fee::Fee;

#[account]
pub struct MainState {
    pub current_slot: u64,
    pub current_round: u64,
    pub total_deposit: u64,
    pub vrf_account_pubkey: Pubkey,
    pub penalty_period: i64,
    pub penalty_fee: f64,
    pub default_fee: f64,
    pub penalty_early_withdraw_fee: Fee,
    pub platform_fee: Fee,
}

impl MainState {
    pub const LEN: usize = 8 + 8 + 8 + 8 + 32 + 8 + 8 + 8 + 2 + 2;
}
