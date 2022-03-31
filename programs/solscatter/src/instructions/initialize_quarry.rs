use anchor_lang::prelude::*;
use anchor_spl::{token::{TokenAccount, Token}, associated_token::AssociatedToken};
use quarry_mine::{Quarry, cpi::accounts::CreateMiner};
use solana_program::pubkey;
use crate::seed::{
    PLATFORM_AUTHORITY_SEED,
    MINER_SEED,
};

#[derive(Accounts)]
pub struct InitializeQuarry<'info> {
    /// CHECK: platform_authority will stake token on behalf of user to collect all yield
    #[account(
        seeds = [
            PLATFORM_AUTHORITY_SEED.as_bytes(),
        ],
        bump
    )]
    pub platform_authority: AccountInfo<'info>,
    /// CHECK: staked solust
    #[account(
        address = pubkey!("6XyygxFmUeemaTvA9E9mhH9FvgpynZqARVyG3gUdCMt7"),
    )]
    pub yi_mint: AccountInfo<'info>,
    /// CHECK: this is quarry program already checked with address =
    #[account(
        address = quarry_mine::program::QuarryMine::id(),
    )]
    pub quarry_program: AccountInfo<'info>,
    /// CHECK: this is miner check with seed it will init with Quarry program
    #[account(
        mut,
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
    /// CHECK: rewarder from Quarry
    #[account(
        address = quarry.rewarder_key,
    )]
    pub rewarder: AccountInfo<'info>,
    #[account(
        init,
        payer = signer,
        associated_token::mint = yi_mint,
        associated_token::authority = miner,
    )]
    pub miner_vault: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> InitializeQuarry<'info> {
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
}

pub fn handler(ctx: Context<InitializeQuarry>) -> Result<()> {
    let platform_authority_bump = *ctx.bumps.get("platform_authority").unwrap();
    let miner_bump = *ctx.bumps.get("miner").unwrap();
    quarry_mine::cpi::create_miner(
        ctx.accounts.into_create_miner_cpi_context()
            .with_signer(&[&[PLATFORM_AUTHORITY_SEED.as_bytes(), &[platform_authority_bump]]]),
        miner_bump 
    )
}
