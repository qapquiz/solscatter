use crate::{
    MAIN_STATE_SEED,
    STATE_SEED,
    PLATFORM_AUTHORITY_SEED,
    error::SolscatterError,
    state::{main_state::MainState, VrfClientState},
};
use anchor_lang::prelude::*;
use anchor_spl::{token::{TokenAccount, Mint}, associated_token::AssociatedToken};
use anchor_spl::token::Token;
use solana_program::pubkey;
use switchboard_v2::VrfAccountData;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = signer,
        space = MainState::LEN,
        seeds = [MAIN_STATE_SEED],
        bump,
    )]
    pub main_state: Account<'info, MainState>,
    /// CHECK: platform_authority will stake token on behalf of user to collect all yield
    #[account(
        seeds = [
            PLATFORM_AUTHORITY_SEED,
        ],
        bump
    )]
    pub platform_authority: AccountInfo<'info>,

    // ######### YIELD GENERATOR #########

    // DEVNET MINT = 5fjG31cbSszE6FodW37UJnNzgVTyqg5WHWGCmL3ayAvA
    #[account(
        address = pubkey!("5fjG31cbSszE6FodW37UJnNzgVTyqg5WHWGCmL3ayAvA"),
    )]
    pub yi_underlying_mint: Box<Account<'info, Mint>>,
    // DEVNET MINT = 6XyygxFmUeemaTvA9E9mhH9FvgpynZqARVyG3gUdCMt7
    #[account(
        address = pubkey!("6XyygxFmUeemaTvA9E9mhH9FvgpynZqARVyG3gUdCMt7"),
    )]
    pub yi_mint: Box<Account<'info, Mint>>,

    #[account(
        init,
        payer = signer,
        associated_token::mint = yi_mint,
        associated_token::authority = platform_authority,
    )]
    pub yi_mint_token_account: Account<'info, TokenAccount>,
    #[account(
        init,
        payer = signer,
        associated_token::mint = yi_underlying_mint,
        associated_token::authority = platform_authority,
    )]
    pub yi_underlying_token_account: Account<'info, TokenAccount>,

    // ######### END YIELD GENERATOR #########

    // ######### SWITCHBOARD VRF #########
    
    #[account(
        init,
        payer = signer,
        seeds = [
            STATE_SEED,
            vrf_account_info.key().as_ref(),
            signer.to_account_info().key().as_ref(),
        ],
        bump,
        space = VrfClientState::LEN,
    )]
    pub vrf_client_state: AccountLoader<'info, VrfClientState>,
    /// CHECK: This is our VrfAccountData
    pub vrf_account_info: AccountInfo<'info>,
    
    // ######### END SWITCHBOARD VRF #########

    #[account(mut)]
    pub signer: Signer<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> Initialize<'info> {
    pub fn validate(&self, ctx: &Context<Self>) -> Result<()> {
        let vrf_account_info = &ctx.accounts.vrf_account_info;
        VrfAccountData::new(vrf_account_info)
            .map_err(|_| SolscatterError::InvalidSwitchboardVrfAccount)?;
        Ok(())
    }

    fn initialize_vrf(&mut self) -> Result<()> {
        let state = &mut self.vrf_client_state.load_init()?;
        state.max_result = u64::MAX;
        state.vrf = self.vrf_account_info.key().clone();
        state.authority = self.signer.to_account_info().key().clone(); 
        Ok(())
    }

    fn initialize_main_state(&mut self) -> Result<()> {
        let main_state = &mut self.main_state;
        main_state.current_slot = 0;
        main_state.current_round = 1;
        main_state.total_deposit = 0;
        main_state.vrf_account_pubkey = self.vrf_account_info.key();
        Ok(())
    }

    pub fn initialize(&mut self) -> Result<()> {
        self.initialize_vrf()?;
        self.initialize_main_state()?;
        Ok(())
    }
}

pub fn handler(ctx: Context<Initialize>) -> Result<()> {
    ctx.accounts.initialize()
}
