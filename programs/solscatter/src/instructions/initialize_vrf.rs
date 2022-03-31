use anchor_lang::prelude::*;
use switchboard_v2::VrfAccountData;

use crate::error::SolscatterError;
use crate::seed::STATE_SEED;
use crate::state::VrfClientState;

#[derive(Accounts)]
pub struct InitializeVrf<'info> {
    #[account(
        init,
        payer = signer,
        seeds = [
            STATE_SEED.as_bytes(),
            vrf_account_info.key().as_ref(),
            signer.to_account_info().key().as_ref(),
        ],
        bump,
        space = VrfClientState::LEN,
    )]
    pub vrf_client_state: AccountLoader<'info, VrfClientState>,
    /// CHECK: This is our VrfAccountData
    pub vrf_account_info: AccountInfo<'info>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> InitializeVrf<'info> {
    pub fn validate(&self, ctx: &Context<Self>) -> Result<()> {
        let vrf_account_info = &ctx.accounts.vrf_account_info;
        VrfAccountData::new(vrf_account_info)
            .map_err(|_| SolscatterError::InvalidSwitchboardVrfAccount)?;
        Ok(())
    }
}

pub fn handler(ctx: Context<InitializeVrf>) -> Result<()> {
    let state = &mut ctx.accounts.vrf_client_state.load_init()?;
    state.max_result = u64::MAX;
    state.vrf = ctx.accounts.vrf_account_info.key().clone();
    state.authority = ctx.accounts.signer.to_account_info().key().clone();
    Ok(())
}
