use anchor_lang::prelude::*;

#[account]
pub struct DrawingResult {
    pub round: u64,
    pub state: DrawingState,
    pub number_of_rewards: u8,
    pub winners: Vec<Option<Pubkey>>,
    pub random_numbers: Vec<u64>,
    pub total_deposit: u64,
    pub last_processed_slot: u64,
    pub finished_timestamp: Option<i64>,
}

impl DrawingResult {
    pub fn space(number_of_rewards: u8) -> usize {
        8 + // discriminator
        8 + // round
        1 + // state
        1 + // number_of_rewards
        4 * (33 * number_of_rewards as usize) + // winners
        4 * (8 * number_of_rewards as usize) + // random_numbers
        8 + // total_deposit
        8 + // last_processed_slot
        9 // finished_timestamp
    }
}

#[derive(Debug, PartialEq, Eq, Clone, AnchorSerialize, AnchorDeserialize)]
pub enum DrawingState {
    Processing,
    Finished,
}