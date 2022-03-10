use anchor_lang::prelude::*;

#[account]
pub struct DrawingResult {
    pub round: u64,
    pub state: DrawingState,
    pub winner: Option<Pubkey>,
    pub random_number: u64,
    pub total_deposit: u64,
    pub last_processed_slot: u64,
    pub finished_timestamp: Option<i64>,
}

impl DrawingResult {
    pub const LEN: usize = 8 + 8 + 1 + 33 + 8 + 8 + 8 + 9;
}

#[derive(Debug, PartialEq, Eq, Clone, AnchorSerialize, AnchorDeserialize)]
pub enum DrawingState {
    Processing,
    Finished,
}