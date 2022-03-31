use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Token};
use crate::seed::*;
use crate::state::user_deposit::UserDeposit;
use crate::state::main_state::MainState;
use crate::state::UserDepositReference;

#[derive(Accounts)]
#[instruction(_slot_number: u64)]
pub struct DepositInitialize<'info> {
    #[account(
        init,
        payer = depositor,
        seeds = [_slot_number.to_le_bytes().as_ref()],
        bump,
        space = UserDeposit::LEN,
        constraint = main_state.current_slot + 1 == _slot_number,
    )]
    pub user_deposit: Account<'info, UserDeposit>,
    #[account(
        init,
        payer = depositor,
        seeds = [depositor.key().as_ref()],
        bump,
        space = UserDepositReference::LEN,
    )]
    pub user_deposit_reference: Account<'info, UserDepositReference>,
    #[account(
        mut,
        seeds = [MAIN_STATE_SEED.as_bytes()],
        bump,
    )]
    pub main_state: Account<'info, MainState>,
    #[account(mut)]
    pub depositor: Signer<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<DepositInitialize>, _slot_number: u64) -> Result<()> {
    let user_deposit = &mut ctx.accounts.user_deposit;
    let user_deposit_reference = &mut ctx.accounts.user_deposit_reference;
    let main_state = &mut ctx.accounts.main_state;
    let depositor = &ctx.accounts.depositor;

    user_deposit.slot = main_state.current_slot + 1;
    user_deposit.amount = 0;
    user_deposit.owner = depositor.key().clone();
    user_deposit.latest_deposit_timestamp = None;
    user_deposit_reference.slot = main_state.current_slot + 1;

    main_state.current_slot = main_state.current_slot + 1;
    Ok(())
}
