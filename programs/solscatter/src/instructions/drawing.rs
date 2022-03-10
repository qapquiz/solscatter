use anchor_lang::prelude::*;
use crate::state::{
    main_state::MainState,
    user_deposit::UserDeposit,
    drawing_result::{DrawingResult, DrawingState},
};

#[derive(Accounts)]
pub struct Drawing<'info> {
    #[account(
        mut,
        seeds = [b"main_state"],
        bump
    )]
    pub main_state: Account<'info, MainState>,
    #[account(
        mut,
        seeds = [
            b"drawing_result",
            main_state.current_round.to_le_bytes().as_ref(),
        ],
        bump,
        constraint = drawing_result.state == DrawingState::Processing,
    )]
    pub drawing_result: Account<'info, DrawingResult>,
    #[account(
        seeds = [(drawing_result.last_processed_slot + 1).to_le_bytes().as_ref()],
        bump,
    )]
    pub user_deposit: Account<'info, UserDeposit>,
    pub clock: Sysvar<'info, Clock>,
}

pub fn handler(ctx: Context<Drawing>) -> Result<()> {
    let drawing_result = &mut ctx.accounts.drawing_result;
    let random_numbers = drawing_result.random_numbers.clone();
    let winners = drawing_result.winners.clone();

    let main_state = &mut ctx.accounts.main_state;
    let user_deposit = &ctx.accounts.user_deposit;

    let mut index: usize = 0;
    let mut winner_count: u8 = 0;
    for random_number in random_numbers.into_iter() {
        match winners[index] {
            Some(_) => {
                winner_count += 1;
            },
            None => {
                if random_number < user_deposit.amount {
                    drawing_result.winners[index] = Some(user_deposit.owner);
                    winner_count += 1;
                }

                drawing_result.random_numbers[index] -= user_deposit.amount;
            },
        }

        index += 1;
    }

    if winner_count == drawing_result.number_of_rewards {
        // all winners are found
        drawing_result.finished_timestamp = Some(ctx.accounts.clock.unix_timestamp);
        drawing_result.state = DrawingState::Finished;

        main_state.current_round = main_state.current_round + 1;
        return Ok(());
    } 
    
    drawing_result.last_processed_slot = drawing_result.last_processed_slot + 1;
    return Ok(());
}
