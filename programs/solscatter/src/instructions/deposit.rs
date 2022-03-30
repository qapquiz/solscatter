use crate::{
    seed::*,
    state::{MainState, UserDeposit},
};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
use solana_program::pubkey;
use yi::{cpi::accounts::Stake, YiToken};

#[derive(Accounts)]
pub struct Deposit<'info> {
    /// CHECK: platform authority
    #[account(
        seeds = [PLATFORM_AUTHORITY_SEED.as_bytes()],
        bump,
    )]
    pub platform_authority: AccountInfo<'info>,
    #[account(mut)]
    pub depositor: Signer<'info>,
    #[account(
        mut,
        associated_token::mint = yi_underlying_mint,
        associated_token::authority = depositor,
    )]
    pub depositor_yi_mint_token_account: Box<Account<'info, TokenAccount>>,

    // ######## PROGRAM STATE ########
    #[account(
        mut,
        seeds = [MAIN_STATE_SEED.as_bytes()],
        bump,
    )]
    pub main_state: Box<Account<'info, MainState>>,
    #[account(
        mut,
        constraint = user_deposit.owner == depositor.to_account_info().key(),
    )]
    pub user_deposit: Box<Account<'info, UserDeposit>>,
    // ######## END PROGRAM STATE ########

    // ######## YIELD GENERATOR ########
    #[account(
        mut,
        address = pubkey!("6XyygxFmUeemaTvA9E9mhH9FvgpynZqARVyG3gUdCMt7"),
    )]
    pub yi_mint: Box<Account<'info, Mint>>,
    #[account(
        address = pubkey!("5fjG31cbSszE6FodW37UJnNzgVTyqg5WHWGCmL3ayAvA"),
    )]
    pub yi_underlying_mint: Box<Account<'info, Mint>>,
    /// CHECK: Yi token program (solUST authority)
    #[account( address = yi::program::Yi::id() )]
    pub yi_token_program: AccountInfo<'info>,
    pub yi_token: AccountLoader<'info, YiToken>,
    #[account(
        mut,
        associated_token::mint = yi_underlying_mint,
        associated_token::authority = platform_authority,
    )]
    pub source_tokens: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = yi_underlying_mint,
        associated_token::authority = yi_token,
    )]
    pub yi_underlying_tokens: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = yi_mint,
        associated_token::authority = platform_authority,
    )]
    pub destination_yi_tokens: Box<Account<'info, TokenAccount>>,
    // ######## END YIELD GENERATOR ########

    // ######## NATIVE PROGRAM ########
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    // ######## END NATIVE PROGRAM ########
}

impl<'info> Deposit<'info> {
    fn into_stake_cpi_context(&self) -> CpiContext<'_, '_, '_, 'info, Stake<'info>> {
        CpiContext::new(
            self.yi_token_program.to_account_info(),
            Stake {
                yi_token: self.yi_token.to_account_info(),
                yi_mint: self.yi_mint.to_account_info(),
                source_tokens: self.source_tokens.to_account_info(),
                source_authority: self.platform_authority.to_account_info(),
                yi_underlying_tokens: self.yi_underlying_tokens.to_account_info(),
                destination_yi_tokens: self.destination_yi_tokens.to_account_info(),
                token_program: self.token_program.to_account_info(),
            },
        )
    }

    fn transfer_yi_underlying_to_platform(&self, amount: u64) -> Result<()> {
        let cpi_account = Transfer {
            from: self.depositor_yi_mint_token_account.to_account_info(),
            to: self.source_tokens.to_account_info(),
            authority: self.depositor.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(self.token_program.to_account_info(), cpi_account);
        token::transfer(cpi_ctx, amount)
    }

    fn stake_to_yield_generator(&self, amount: u64, platform_authority_bump: u8) -> Result<()> {
        // platform stake yi_underlying to solUST
        yi::cpi::stake(
            self.into_stake_cpi_context()
                .with_signer(&[&[PLATFORM_AUTHORITY_SEED.as_bytes(), &[platform_authority_bump]]]),
            amount,
        )
    }

    fn update_state(&mut self, amount: u64) -> Result<()> {
        let user_deposit = &mut self.user_deposit;
        user_deposit.amount = user_deposit.amount + amount;
        user_deposit.latest_deposit_timestamp = Some(Clock::get().unwrap().unix_timestamp);

        let main_state = &mut self.main_state;
        main_state.total_deposit = main_state.total_deposit + amount;
        Ok(())
    }

    pub fn deposit(&mut self, params: DepositParams, platform_authority_bump: u8) -> Result<()> {
        self.transfer_yi_underlying_to_platform(params.amount)?;
        self.stake_to_yield_generator(params.amount, platform_authority_bump)?;
        self.update_state(params.amount)?;
        Ok(())
    }
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct DepositParams {
    pub amount: u64,
}

pub fn handler(ctx: Context<Deposit>, params: DepositParams) -> Result<()> {
    if params.amount <= 0 {
        return Ok(());
    }

    let platform_authority_bump = *ctx.bumps.get("platform_authority").unwrap();
    ctx.accounts.deposit(params, platform_authority_bump)
}
