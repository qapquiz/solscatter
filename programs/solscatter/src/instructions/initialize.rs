use crate::{
    STATE_SEED,
    error::SolscatterError,
    state::{main_state::MainState, VrfClientState},
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
            STATE_SEED,
            vrf_account_info.key().as_ref(),
            signer.to_account_info().key().as_ref(),
        ],
        bump,
    )]
    pub vrf_client_state: AccountLoader<'info, VrfClientState>,
    /// CHECK: This is our VrfAccountData
    pub vrf_account_info: AccountInfo<'info>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> Initialize<'info> {
    pub fn validate(&self, ctx: &Context<Self>) -> Result<()> {
        let vrf_account_info = &ctx.accounts.vrf_account_info;
        VrfAccountData::new(vrf_account_info)
            .map_err(|_| SolscatterError::InvalidSwitchboardVrfAccount)?;
        Ok(())
    }
}

pub fn handler(ctx: Context<Initialize>) -> Result<()> {
    let state = &mut ctx.accounts.vrf_client_state.load_init()?;
    state.max_result = u64::MAX;
    state.vrf = ctx.accounts.vrf_account_info.key().clone();
    state.authority = ctx.accounts.signer.to_account_info().key().clone();

    let main_state = &mut ctx.accounts.main_state;
    main_state.current_slot = 0;
    main_state.current_round = 1;
    main_state.total_deposit = 0;
    main_state.vrf_account_pubkey = ctx.accounts.vrf_account_info.key();

    Ok(())
}
