use anchor_lang::prelude::*;
use crate::state::{main_state::MainState, user_deposit::UserDeposit};

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(
        mut,
        has_one = owner,
    )]
    pub user_deposit: Account<'info, UserDeposit>,
    #[account(
        mut,
        seeds = [b"main_state"],
        bump,
    )]
    pub main_state: Account<'info, MainState>,
    pub owner: Signer<'info>,
    pub clock: Sysvar<'info, Clock>,
}

pub fn handler(ctx: Context<Deposit>, amount: u64) -> Result<()> {
    let user_deposit = &mut ctx.accounts.user_deposit;
    user_deposit.amount = user_deposit.amount + amount;
    user_deposit.latest_deposit_timestamp = Some(ctx.accounts.clock.unix_timestamp);

    let main_state = &mut ctx.accounts.main_state;
    main_state.total_deposit = main_state.total_deposit + amount;

    Ok(())
}
