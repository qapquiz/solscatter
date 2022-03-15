use anchor_lang::prelude::*;

#[account]
pub struct MainState {
    pub current_slot: u64,
    pub current_round: u64,
    pub total_deposit: u64,
    pub vrf_account_pubkey: Pubkey,
}

impl MainState {
    pub const LEN: usize = 8 + 8 + 8 + 8 + 32;
}