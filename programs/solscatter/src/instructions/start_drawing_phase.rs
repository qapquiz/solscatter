use anchor_lang::prelude::*;
use crate::state::{drawing_result::{DrawingResult, DrawingState}, main_state::MainState};

#[derive(Accounts)]
pub struct StartDrawingPhase<'info> {
    #[account(
        init,
        payer = signer,
        space = DrawingResult::LEN,
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

pub fn handler(ctx: Context<StartDrawingPhase>, random_number: u64) -> Result<()> {
    let main_state = &ctx.accounts.main_state;
    let drawing_result = &mut ctx.accounts.drawing_result;
    drawing_result.round = main_state.current_round;
    drawing_result.state = DrawingState::Processing;
    drawing_result.winner = None;
    drawing_result.random_number = random_number;
    drawing_result.total_deposit = main_state.total_deposit;
    drawing_result.last_processed_slot = 0;
    drawing_result.finished_timestamp = None;

    Ok(())
}