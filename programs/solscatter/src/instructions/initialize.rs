use crate::{
    seed::*,
    error::SolscatterError,
    state::{main_state::MainState, metadata::Metadata, VrfClientState, SolendReserve},
};
use spl_token_lending::instruction::init_obligation;
use anchor_lang::{
    prelude::*,
    solana_program::program::*,
    solana_program::pubkey,
};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};
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
    pub metadata: Box<Account<'info, Metadata>>,
    /// CHECK:
    #[account(
        seeds = [PROGRAM_AUTHORITY_SEED],
        bump
    )]
    pub program_authority: AccountInfo<'info>,
    #[account(
        address = pubkey!("zVzi5VAf4qMEwzv7NXECVx5v2pQ7xnqVVjCXZwS9XzA"),
    )]
    pub usdc_mint: Box<Account<'info, Mint>>,
    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = usdc_mint,
        associated_token::authority = program_authority
    )]
    pub usdc_token_account: Box<Account<'info, TokenAccount>>,
    /// CHECK:
    #[account(
        address = pubkey!("FNNkz4RCQezSSS71rW2tvqZH1LCkTzaiG7Nd1LeA5x5y"),
    )]
    pub reserve: Box<Account<'info, SolendReserve>>,
    #[account(
        address = reserve.collateral.mint_pubkey,
    )]
    pub reserve_collateral_mint: Box<Account<'info, Mint>>,
    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = reserve_collateral_mint,
        associated_token::authority = program_authority,
    )]
    pub collateral_token_account: Box<Account<'info, TokenAccount>>,
    /// CHECK:
    #[account(
        init,
        payer = signer,
        space = 1300,
        owner = lending_program.key(),
    )]
    pub obligation: AccountInfo<'info>,
    /// CHECK:
    pub lending_market: AccountInfo<'info>,
    /// CHECK:
    #[account(
        address = pubkey!("ALend7Ketfx5bxh6ghsCDXAoDrhvEmsXT3cynB6aPLgx"),
    )]
    pub lending_program: AccountInfo<'info>,
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
    pub clock: Sysvar<'info, Clock>,
    pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
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

    fn initialize_metadata(&mut self) -> Result<()> {
        let metadata = &mut self.metadata;
        metadata.lending_program = spl_token_lending::id();
        metadata.usdc_mint = self.usdc_mint.key().clone();
        metadata.usdc_token_account = self.usdc_token_account.to_account_info().key().clone();
        metadata.program_authority = self.program_authority.key().clone();
        metadata.obligation = self.obligation.key().clone();
        metadata.reserve = self.reserve.to_account_info().key().clone();
        metadata.collateral_token_account = self.collateral_token_account.to_account_info().key().clone();
        metadata.lending_market = self.reserve.lending_market;
        metadata.lending_market_authority_seed = self.reserve.lending_market;

        Ok(())
    }

    fn initialize_obligation(&mut self, program_authority_bump: u8) -> Result<()> {
        let ix_init_obligation = init_obligation(
            spl_token_lending::id(),
            self.obligation.key().clone(),
            self.lending_market.key().clone(),
            self.program_authority.key().clone(),
        );

        invoke_signed(
            &ix_init_obligation,
            &[
                self.obligation.clone(),
                self.lending_market.clone(),
                self.program_authority.clone(),
                self.lending_program.clone(),
                self.clock.to_account_info().clone(),
                self.rent.to_account_info().clone(),
                self.token_program.to_account_info().clone(),
            ],
            &[&[PROGRAM_AUTHORITY_SEED, &[program_authority_bump]]]
        ).map_err(Into::into)
    }

    pub fn initialize(&mut self, program_authority_bump: u8) -> Result<()> {
        self.initialize_vrf()?;
        self.initialize_main_state()?;
        self.initialize_metadata()?;
        self.initialize_obligation(program_authority_bump)?;
        Ok(())
    }
}

pub fn handler(ctx: Context<Initialize>) -> Result<()> {
    let program_authority_bump = *ctx.bumps.get("program_authority").unwrap();
    ctx.accounts.initialize(program_authority_bump)
}
