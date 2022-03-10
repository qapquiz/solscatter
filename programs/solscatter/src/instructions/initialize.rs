use anchor_lang::prelude::*;
use crate::state::main_state::MainState;

#[derive(Accounts)]
pub struct Initialize<'info>  {
    #[account(
        init,
        payer = signer,
        space = MainState::LEN,
        seeds = [b"main_state"],
        bump,
    )]
    pub main_state: Account<'info, MainState>,
    /// CHECK: This is switchboard pubkey will check with address =
    pub switchboard: AccountInfo<'info>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Initialize>) -> Result<()> {
    let main_state = &mut ctx.accounts.main_state;
    main_state.current_slot = 0;
    main_state.current_round = 1;
    main_state.total_deposit = 0;
    main_state.switchboard_pubkey = ctx.accounts.switchboard.key();

    Ok(())
}