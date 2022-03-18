use anchor_lang::prelude::*;
use crate::state::{drawing_result::{DrawingResult, DrawingState}, main_state::MainState};
use crate::error::SolscatterError;

#[derive(Accounts)]
#[instruction(number_of_rewards: u8, random_numbers: Vec<u64>)]
pub struct StartDrawingPhase<'info> {
    #[account(
        init,
        payer = signer,
        space = DrawingResult::space(number_of_rewards)?,
        seeds = [
            b"drawing_result", 
            main_state.current_round.to_le_bytes().as_ref(),
        ],
        bump,
    )]
    pub drawing_result: Account<'info, DrawingResult>,
    #[account(
        seeds = [b"main_state"],
        bump
    )]
    pub main_state: Account<'info, MainState>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<StartDrawingPhase>, number_of_rewards: u8, random_numbers: Vec<u64>) -> Result<()> {
    if number_of_rewards <= 0 {
        return Err(error!(SolscatterError::NumberOfRewardsMustMoreThanZero));
    }

    if random_numbers.len() != number_of_rewards as usize {
        return Err(error!(SolscatterError::NumberOfRandomNumbersNotMatchWithNumberOfRewards));
    }

    let main_state = &ctx.accounts.main_state;
    let drawing_result = &mut ctx.accounts.drawing_result;
    drawing_result.round = main_state.current_round;
    drawing_result.state = DrawingState::Processing;
    drawing_result.number_of_rewards = number_of_rewards;
    drawing_result.winners = vec!(); 
    drawing_result.random_numbers = random_numbers;
    drawing_result.total_deposit = main_state.total_deposit;
    drawing_result.last_processed_slot = 0;
    drawing_result.finished_timestamp = None;

    for _ in 0..number_of_rewards {
        drawing_result.winners.push(None);
    }

    Ok(())
}