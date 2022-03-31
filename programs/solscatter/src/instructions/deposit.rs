use crate::{
    seed::*,
    state::{MainState, Metadata, UserDeposit},
};
use anchor_lang::prelude::*;
use std::cmp::{max};
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
use yi::{cpi::accounts::Stake, YiToken};
use quarry_mine::{cpi::accounts::UserStake, Miner, Quarry, Rewarder};
use quarry_mine::program::QuarryMine;

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub depositor: Signer<'info>,
    #[account(
        mut,
        associated_token::mint = yi_underlying_mint,
        associated_token::authority = depositor,
    )]
    pub depositor_yi_underlying_token_account: Box<Account<'info, TokenAccount>>,
    // ######## PROGRAM STATE ########
    #[account(
        mut,
        seeds = [MAIN_STATE_SEED.as_bytes()],
        bump,
    )]
    pub main_state: Box<Account<'info, MainState>>,
    #[account(
        mut,
        seeds = [METADATA_SEED.as_bytes()],
        bump,
    )]
    pub metadata: Box<Account<'info, Metadata>>,
    /// CHECK: platform authority
    #[account(
        seeds = [PLATFORM_AUTHORITY_SEED.as_bytes()],
        bump,
    )]
    pub platform_authority: AccountInfo<'info>,
    #[account(
        mut,
        constraint = user_deposit.owner == depositor.to_account_info().key(),
    )]
    pub user_deposit: Box<Account<'info, UserDeposit>>,
    // ######## END PROGRAM STATE ########

    // ######## YIELD GENERATOR ########
    #[account(
        mut,
        address = metadata.yi_mint,
    )]
    pub yi_mint: Box<Account<'info, Mint>>,
    #[account(
        address = metadata.yi_underlying_token_account,
    )]
    pub yi_underlying_mint: Box<Account<'info, Mint>>,
    /// CHECK: Yi token program (solUST authority)
    #[account(
        address = yi::program::Yi::id()
    )]
    pub yi_token_program: AccountInfo<'info>,
    pub yi_token: AccountLoader<'info, YiToken>,
    #[account(
        mut,
        associated_token::mint = yi_underlying_mint,
        associated_token::authority = yi_token,
    )]
    pub yi_underlying_token_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = yi_underlying_mint,
        associated_token::authority = platform_authority,
    )]
    pub platform_yi_underlying_token_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = yi_mint,
        associated_token::authority = platform_authority,
    )]
    pub platform_yi_token_account: Box<Account<'info, TokenAccount>>,
    // ######## END YIELD GENERATOR ########

    // ######## QUARRY ########
    pub quarry_program: Program<'info, QuarryMine>,
    #[account(
        address = metadata.quarry_miner
    )]
    pub platform_quarry_miner: Account<'info, Miner>,
    #[account(mut)]
    pub quarry: Box<Account<'info, Quarry>>,
    pub rewarder: Box<Account<'info, Rewarder>>,
    #[account(
        mut,
        associated_token::mint = yi_mint,
        associated_token::authority = platform_quarry_miner,
    )]
    pub miner_vault: Box<Account<'info, TokenAccount>>,
    // ######## NATIVE PROGRAM ########
    pub clock: Sysvar<'info, Clock>,
    pub token_program: Program<'info, Token>,
    // ######## END NATIVE PROGRAM ########
}

impl<'info> Deposit<'info> {
    fn into_yi_stake_cpi_context(&self) -> CpiContext<'_, '_, '_, 'info, Stake<'info>> {
        CpiContext::new(
            self.yi_token_program.to_account_info(),
            Stake {
                yi_token: self.yi_token.to_account_info(),
                yi_mint: self.yi_mint.to_account_info(),
                source_tokens: self.platform_yi_underlying_token_account.to_account_info(),
                source_authority: self.platform_authority.to_account_info(),
                yi_underlying_tokens: self.yi_underlying_token_account.to_account_info(),
                destination_yi_tokens: self.platform_yi_token_account.to_account_info(),
                token_program: self.token_program.to_account_info(),
            },
        )
    }

    fn into_quarry_stake_cpi_context(&self) -> CpiContext<'_, '_, '_, 'info, UserStake<'info>> {
        CpiContext::new(
            self.quarry_program.to_account_info(),
            UserStake {
                authority: self.platform_authority.to_account_info(),
                miner: self.platform_quarry_miner.to_account_info(),
                quarry: self.quarry.to_account_info(),
                miner_vault: self.miner_vault.to_account_info(),
                token_account: self.platform_yi_token_account.to_account_info(),
                token_program: self.token_program.to_account_info(),
                rewarder: self.rewarder.to_account_info()
            }
        )
    }

    fn transfer_yi_underlying_to_platform(&self, amount: u64) -> Result<()> {
        let cpi_account = Transfer {
            from: self.depositor_yi_underlying_token_account.to_account_info(),
            to: self.platform_yi_underlying_token_account.to_account_info(),
            authority: self.depositor.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(self.token_program.to_account_info(), cpi_account);
        token::transfer(cpi_ctx, amount)
    }

    fn stake_to_yield_generator(&self, amount: u64, platform_authority_bump: u8) -> Result<()> {
        // platform stake yi_underlying to solUST
        yi::cpi::stake(
            self.into_yi_stake_cpi_context()
                .with_signer(&[&[PLATFORM_AUTHORITY_SEED.as_bytes(), &[platform_authority_bump]]]),
            amount,
        )
    }

    fn stake_to_quarry(&mut self, platform_authority_bump: u8) -> Result<()> {
        self.platform_yi_token_account.reload()?;

        quarry_mine::cpi::stake_tokens(
            self.into_quarry_stake_cpi_context()
                .with_signer(&[&[PLATFORM_AUTHORITY_SEED.as_bytes(), &[platform_authority_bump]]]),
            self.platform_yi_token_account.amount
        )
    }

    fn update_state(&mut self, amount: u64) -> Result<()> {
        let user_deposit = &mut self.user_deposit;
        user_deposit.amount = user_deposit.amount + amount;

        let main_state = &mut self.main_state;
        main_state.total_deposit = main_state.total_deposit + amount;

        self.update_penalty_fee(amount)
    }

    fn update_penalty_fee(&mut self, amount: u64) -> Result<()> {
        let current_timestamp = self.clock.unix_timestamp;
        let penalty_period = self.main_state.penalty_period;
        let penalty_fee = self.main_state.penalty_fee;
        let user_deposit = &mut self.user_deposit;

        let last_deposit_timestamp = match user_deposit.latest_deposit_timestamp {
            Some(deposit_timestamp) => deposit_timestamp,
            None => current_timestamp,
        };

        let seconds_diff_from_penalty_period = max((last_deposit_timestamp + penalty_period) - current_timestamp, 0) as f64;

        let previous_penalty_fee = (seconds_diff_from_penalty_period / penalty_period as f64) * user_deposit.penalty_fee;

        let new_penalty_fee = ((user_deposit.amount as f64 * previous_penalty_fee) + (amount as f64 * penalty_fee)) / (user_deposit.amount + amount) as f64;

        msg!("previous_penalty_fee : {} ", previous_penalty_fee);
        msg!("new_penalty_fee : {} ", new_penalty_fee);

        user_deposit.penalty_fee = new_penalty_fee;
        user_deposit.latest_deposit_timestamp = Some(current_timestamp);

        Ok(())
    }

    pub fn deposit(&mut self, params: DepositParams, platform_authority_bump: u8) -> Result<()> {
        self.transfer_yi_underlying_to_platform(params.amount)?;
        self.stake_to_yield_generator(params.amount, platform_authority_bump)?;
        self.stake_to_quarry(platform_authority_bump)?;
        self.update_state(params.amount)
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
