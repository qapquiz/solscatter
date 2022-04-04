use anchor_lang::prelude::*;
use crate::{state::{
    main_state::MainState,
    user_deposit::UserDeposit,
    drawing_result::{DrawingResult, DrawingState},
}, seed::{MAIN_STATE_SEED, DRAWING_RESULT_SEED}};

#[derive(Accounts)]
#[instruction(_processing_slot: u64)]
pub struct Drawing<'info> {
    #[account(
        mut,
        seeds = [MAIN_STATE_SEED.as_bytes()],
        bump
    )]
    pub main_state: Account<'info, MainState>,
    #[account(
        mut,
        seeds = [
            DRAWING_RESULT_SEED.as_bytes(),
            main_state.current_round.to_le_bytes().as_ref(),
        ],
        bump,
        constraint = drawing_result.state == DrawingState::Processing,
    )]
    pub drawing_result: Account<'info, DrawingResult>,
    #[account(
        seeds = [_processing_slot.to_le_bytes().as_ref()],
        bump,
        constraint = drawing_result.last_processed_slot.checked_add(1).unwrap() == _processing_slot
    )]
    pub user_deposit: Account<'info, UserDeposit>,
    pub clock: Sysvar<'info, Clock>,
}

pub fn handler(ctx: Context<Drawing>, _processing_slot: u64) -> Result<()> {
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
                winner_count = winner_count.checked_add(1).unwrap();
            },
            None => {
                if random_number < user_deposit.amount {
                    drawing_result.winners[index] = Some(user_deposit.owner);
                    winner_count = winner_count.checked_add(1).unwrap();
                }

                drawing_result.random_numbers[index] = drawing_result.random_numbers[index].checked_sub(user_deposit.amount).unwrap();
            },
        }

        index += 1;
    }

    if winner_count == drawing_result.number_of_rewards {
        // all winners are found
        drawing_result.finished_timestamp = Some(ctx.accounts.clock.unix_timestamp);
        drawing_result.state = DrawingState::Finished;

        main_state.current_round = main_state.current_round.checked_add(1).unwrap();
        return Ok(());
    } 
    
    drawing_result.last_processed_slot = drawing_result.last_processed_slot.checked_add(1).unwrap();
    return Ok(());
}
