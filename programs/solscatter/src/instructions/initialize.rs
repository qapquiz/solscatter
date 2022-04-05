use crate::{
    seed::{
        MAIN_STATE_SEED,
        PLATFORM_AUTHORITY_SEED,
        MINER_SEED,
        METADATA_SEED,
    },
    duration::*,
    state::{MainState, Metadata, Fee},
};
use anchor_lang::prelude::*;
use anchor_spl::{token::TokenAccount, associated_token::AssociatedToken};
use anchor_spl::token::Token;
use quarry_mine::Quarry;
use solana_program::pubkey;
use yi::YiToken;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
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
    /// CHECK: Yi token program
    #[account(
        address = yi::program::Yi::id()
    )]
    pub yi_program: AccountInfo<'info>,
    pub yi_token: AccountLoader<'info, YiToken>,
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
    pub quarry: Box<Account<'info, Quarry>>,
    /// CHECK: alreday checked with quarry.rewarder_key
    #[account(
        address = quarry.rewarder_key,
    )]
    pub rewarder: AccountInfo<'info>,
    #[account(
        associated_token::mint = yi_mint,
        associated_token::authority = miner,
    )]
    pub miner_vault: Box<Account<'info, TokenAccount>>,
    // ######### END QUARRY #########

    // ######### SWITCHBOARD VRF #########
    /// CHECK: This is our VrfAccountData
    pub vrf_account_info: AccountInfo<'info>,
    // ######### END SWITCHBOARD VRF #########

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

        main_state.penalty_early_withdraw_fee = Fee { bips: 500 };
        main_state.platform_fee = Fee { bips: 100 };
        Ok(())
    }

    fn initialize_metadata(&mut self) -> Result<()> {
        let metadata = &mut self.metadata;
        let yi = self.yi_token.load()?;

        msg!("yi_token mint {}", yi.mint);
        msg!("yi_token underlying_token_mint {}", yi.underlying_token_mint);
        msg!("yi_token underlying_tokens {}", yi.underlying_tokens);

        metadata.yi_program = self.yi_program.key();
        metadata.yi_token = self.yi_token.to_account_info().key();
        metadata.yi_underlying_mint = self.yi_underlying_mint.key();
        metadata.yi_underlying_token_account = self.yi_underlying_token_account.to_account_info().key();
        metadata.yi_mint = self.yi_mint.key();
        metadata.yi_mint_token_account = self.yi_mint_token_account.to_account_info().key();
        metadata.platform_authority = self.platform_authority.key();
        metadata.quarry_program = self.quarry_program.key();
        metadata.quarry = self.quarry.to_account_info().key();
        metadata.quarry_miner = self.miner.key();
        metadata.quarry_miner_vault = self.miner_vault.to_account_info().key();
        metadata.quarry_rewarder = self.rewarder.key();
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
