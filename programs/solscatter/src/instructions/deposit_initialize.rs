use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{TokenAccount, Mint, Token};
use crate::seed::*;
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
        seeds = [MAIN_STATE_SEED],
        bump,
    )]
    pub main_state: Account<'info, MainState>,
    #[account(mut)]
    pub depositor: Signer<'info>,

    pub yi_underlying_mint: Box<Account<'info, Mint>>,
    #[account(
        init_if_needed,
        payer = depositor,
        associated_token::mint = yi_underlying_mint,
        associated_token::authority = depositor,
    )]
    pub sol_ust_token_account: Account<'info, TokenAccount>,
    pub rent: Sysvar<'info, Rent>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
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
