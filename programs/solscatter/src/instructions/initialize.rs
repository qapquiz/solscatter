use crate::{
    error::SolscatterError,
    state::{main_state::MainState, VrfClient},
};
use anchor_lang::prelude::*;
use switchboard_v2::VrfAccountData;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = signer,
        space = MainState::LEN,
        seeds = [b"main_state"],
        bump,
    )]
    pub main_state: Account<'info, MainState>,
    #[account(
        init,
        payer = signer,
        seeds = [
            b"vrf_client",
            switchboard_vrf.key().as_ref(),
            signer.to_account_info().key().as_ref(),
        ],
        bump,
    )]
    pub vrf_client: AccountLoader<'info, VrfClient>,
    /// CHECK: This is switchboard pubkey will check with address =
    pub switchboard_vrf: AccountInfo<'info>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> Initialize<'info> {
    pub fn validate(&self, ctx: &Context<Self>) -> Result<()> {
        let vrf_account_info = &ctx.accounts.switchboard_vrf;
        VrfAccountData::new(vrf_account_info)
            .map_err(|_| SolscatterError::InvalidSwitchboardVrfAccount)?;
        Ok(())
    }
}

pub fn handler(ctx: Context<Initialize>) -> Result<()> {
    let main_state = &mut ctx.accounts.main_state;
    main_state.current_slot = 0;
    main_state.current_round = 1;
    main_state.total_deposit = 0;
    main_state.switchboard_pubkey = ctx.accounts.switchboard_vrf.key();

    Ok(())
}
