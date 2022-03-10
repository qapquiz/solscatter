use anchor_lang::prelude::*;
use crate::state::user_deposit::UserDeposit;
use crate::state::main_state::MainState;

#[derive(Accounts)]
pub struct DepositInitialize<'info> {
    #[account(
        init,
        payer = depositor,
        seeds = [(main_state.current_slot + 1).to_le_bytes().as_ref()],
        bump,
        space = UserDeposit::LEN,
    )]
    pub user_deposit: Account<'info, UserDeposit>,
    #[account(
        mut,
        seeds = [b"main_state"],
        bump,
    )]
    pub main_state: Account<'info, MainState>,
    #[account(mut)]
    pub depositor: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<DepositInitialize>) -> Result<()> {
    let user_deposit = &mut ctx.accounts.user_deposit;
    let main_state = &mut ctx.accounts.main_state;
    let depositor = &ctx.accounts.depositor;

    user_deposit.slot = main_state.current_slot + 1;
    user_deposit.amount = 0;
    user_deposit.owner = depositor.key().clone();
    user_deposit.latest_deposit_timestamp = None;

    main_state.current_slot = main_state.current_slot + 1;
    Ok(())
}
