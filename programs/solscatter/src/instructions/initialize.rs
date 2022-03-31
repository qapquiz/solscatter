use crate::{
    seed::{
        MAIN_STATE_SEED,
        PLATFORM_AUTHORITY_SEED,
        MINER_SEED,
        METADATA_SEED,
    },
    duration::*,
    state::{MainState, Metadata},
};
use anchor_lang::prelude::*;
use anchor_spl::{token::TokenAccount, associated_token::AssociatedToken};
use anchor_spl::token::Token;
use quarry_mine::Quarry;
use solana_program::pubkey;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = signer,
        space = MainState::LEN,
        seeds = [
            MAIN_STATE_SEED.as_bytes(),
        ],
        bump,
    )]
    pub main_state: Box<Account<'info, MainState>>,
    #[account(
        init,
        payer = signer,
        space = Metadata::LEN,
        seeds = [
            METADATA_SEED.as_bytes()
        ],
        bump,
    )]
    pub metadata: Box<Account<'info, Metadata>>,
    /// CHECK: platform_authority will stake token on behalf of user to collect all yield
    #[account(
        seeds = [
            PLATFORM_AUTHORITY_SEED.as_bytes(),
        ],
        bump
    )]
    pub platform_authority: AccountInfo<'info>,

    // ######### YIELD GENERATOR #########
    /// CHECK: solust
    #[account(
        address = pubkey!("5fjG31cbSszE6FodW37UJnNzgVTyqg5WHWGCmL3ayAvA"),
    )]
    pub yi_underlying_mint: AccountInfo<'info>,
    /// CHECK: staked solust
    #[account(
        address = pubkey!("6XyygxFmUeemaTvA9E9mhH9FvgpynZqARVyG3gUdCMt7"),
    )]
    pub yi_mint: AccountInfo<'info>,

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

    // ######### QUARRY #########
    /// CHECK: this is quarry program already checked with address =
    #[account(
        address = quarry_mine::program::QuarryMine::id(),
    )]
    pub quarry_program: AccountInfo<'info>,
    /// CHECK: this is miner check with seed it will init with Quarry program
    #[account(
        seeds = [
            MINER_SEED.as_bytes(),
            quarry.key().to_bytes().as_ref(),
            platform_authority.key().to_bytes().as_ref(),
        ],
        bump,
        seeds::program = quarry_program.key(),
    )]
    pub miner: AccountInfo<'info>,
    #[account(mut)]
    pub quarry: Box<Account<'info, Quarry>>,
    // ######### END QUARRY #########

    // ######### SWITCHBOARD VRF #########
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
    fn initialize_main_state(&mut self) -> Result<()> {
        let main_state = &mut self.main_state;
        main_state.current_slot = 0;
        main_state.current_round = 1;
        main_state.total_deposit = 0;
        main_state.vrf_account_pubkey = self.vrf_account_info.key();
        main_state.penalty_period = 7 * SECS_PER_DAY;
        main_state.penalty_fee = 5_f64;
        main_state.default_fee = 1_f64;
        Ok(())
    }

    fn initialize_metadata(&mut self) -> Result<()> {
        let metadata = &mut self.metadata;
        metadata.yi_underlying_mint = self.yi_underlying_mint.key();
        metadata.yi_underlying_token_account = self.yi_underlying_token_account.to_account_info().key();
        metadata.yi_mint = self.yi_mint.key();
        metadata.yi_mint_token_account = self.yi_mint_token_account.to_account_info().key();
        metadata.platform_authority = self.platform_authority.key();
        metadata.quarry_miner = self.miner.key();
        Ok(())
    }

    pub fn initialize(&mut self) -> Result<()> {
        self.initialize_main_state()?;
        self.initialize_metadata()?;
        Ok(())
    }
}

pub fn handler(ctx: Context<Initialize>) -> Result<()> {
    ctx.accounts.initialize()
}
