use crate::{
    seed::{
        MAIN_STATE_SEED,
        PLATFORM_AUTHORITY_SEED,
        MINER_SEED,
        STATE_SEED,
        METADATA_SEED,
    },
    duration::*,
    error::SolscatterError,
    state::{MainState, Metadata, VrfClientState},
};
use anchor_lang::prelude::*;
use anchor_spl::{token::{TokenAccount, Mint}, associated_token::AssociatedToken};
use anchor_spl::token::Token;
use quarry_mine::{cpi::accounts::CreateMiner, Quarry, Rewarder};
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
    #[account(
        init,
        payer = signer,
        space = Metadata::LEN,
        seeds = [METADATA_SEED],
        bump,
    )]
    pub metadata: Account<'info, Metadata>,
    /// CHECK: platform_authority will stake token on behalf of user to collect all yield
    #[account(
        seeds = [PLATFORM_AUTHORITY_SEED],
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

    // ######### QUARRY #########
    /// CHECK: this is quarry program already checked with address =
    #[account(
        address = quarry_mine::program::QuarryMine::id(),
    )]
    pub quarry_program: AccountInfo<'info>,
    /// CHECK: this is miner check with seed it will init with Quarry program
    #[account(
        mut,
        seeds = [
            MINER_SEED.as_ref(),
            quarry.key().to_bytes().as_ref(),
            platform_authority.key().to_bytes().as_ref(),
        ],
        bump,
    )]
    pub miner: AccountInfo<'info>,
    #[account(mut)]
    pub quarry: Box<Account<'info, Quarry>>,
    pub rewarder: Box<Account<'info, Rewarder>>,
    #[account(
        mut,
        associated_token::mint = yi_mint,
        associated_token::authority = miner,
    )]
    pub miner_vault: Box<Account<'info, TokenAccount>>,
    // ######### END QUARRY #########

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

    pub fn into_create_miner_cpi_context(&self) -> CpiContext<'_, '_, '_, 'info, CreateMiner<'info>> {
        CpiContext::new(
            self.quarry_program.to_account_info(),
            CreateMiner {
                authority: self.platform_authority.to_account_info(),
                miner: self.miner.to_account_info(),
                quarry: self.quarry.to_account_info(),
                rewarder: self.rewarder.to_account_info(),
                system_program: self.system_program.to_account_info(),
                payer: self.signer.to_account_info(),
                token_mint: self.yi_mint.to_account_info(),
                miner_vault: self.miner_vault.to_account_info(),
                token_program: self.token_program.to_account_info(),
            },
        )
    }

    fn initialize_vrf(&mut self) -> Result<()> {
        let state = &mut self.vrf_client_state.load_init()?;
        state.max_result = u64::MAX;
        state.vrf = self.vrf_account_info.key().clone();
        state.authority = self.signer.to_account_info().key().clone(); 
        Ok(())
    }

    fn initialize_quarry(&self, platform_authority_bump: u8, miner_bump: u8) -> Result<()> {
        quarry_mine::cpi::create_miner(
            self.into_create_miner_cpi_context()
                .with_signer(&[&[PLATFORM_AUTHORITY_SEED, &[platform_authority_bump]]]),
            miner_bump 
        )
    } 

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
        Ok(())
    }

    pub fn initialize(&mut self, platform_authority_bump: u8, miner_bump: u8) -> Result<()> {
        self.initialize_vrf()?;
        self.initialize_quarry(platform_authority_bump, miner_bump)?;
        self.initialize_main_state()?;
        self.initialize_metadata()?;
        Ok(())
    }
}

pub fn handler(ctx: Context<Initialize>) -> Result<()> {
    let platform_authority_bump = *ctx.bumps.get("platform_authority").unwrap();
    let miner_bump = *ctx.bumps.get("miner").unwrap();
    ctx.accounts.initialize(platform_authority_bump, miner_bump)
}
