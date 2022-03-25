use anchor_lang::prelude::*;

use crate::error::SolscatterError;

#[account]
pub struct DrawingResult {
    pub round: u64,
    pub state: DrawingState,
    pub reward_amount: u64,
    pub number_of_rewards: u8,
    pub winners: Vec<Option<Winner>>,
    pub random_numbers: Vec<u64>,
    pub total_deposit: u64,
    pub last_processed_slot: u64,
    pub finished_timestamp: Option<i64>,
}

impl DrawingResult {
    pub fn space(number_of_rewards: u8) -> Result<usize> {
        if number_of_rewards <= 0 {
            return Err(error!(SolscatterError::NumberOfRewardsMustMoreThanZero));
        }
        if number_of_rewards > 10 {
            return Err(error!(SolscatterError::NumberOfRewardsMustLessOrEqualTen));
        }
        return Ok(8 + // discriminator
            8 + // round
            DrawingState::LEN + // state
            8 + // reward_amount
            1 + // number_of_rewards
            4 * ((1 + Winner::LEN) * number_of_rewards as usize) + // winners
            4 * (8 * number_of_rewards as usize) + // random_numbers
            8 + // total_deposit
            8 + // last_processed_slot
            9 // finished_timestamp
        );
    }
}

#[derive(Debug, PartialEq, Eq, Clone, AnchorSerialize, AnchorDeserialize)]
pub enum DrawingState {
    Processing,
    Finished,
}

impl DrawingState {
    pub const LEN: usize = 1;
}

#[derive(Debug, PartialEq, Eq, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct Winner {
    pub pubkey: Pubkey,
    pub can_claim: bool,
}

impl Winner {
    pub const LEN: usize = 32 + 1;
}
