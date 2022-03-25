use crate::{
    seed::*,
    state::{main_state::MainState, user_deposit::UserDeposit, metadata::Metadata},
    token::transfer_token
};
use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use spl_token_lending::{
    instruction::deposit_reserve_liquidity_and_obligation_collateral
};
use crate::state::SolendReserve;
use std::cmp::{max};

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(
        mut,
        has_one = owner,
    )]
    pub user_deposit: Account<'info, UserDeposit>,
    #[account(
        mut,
        seeds = [MAIN_STATE_SEED],
        bump,
    )]
    pub main_state: Account<'info, MainState>,
    #[account(
        seeds = [METADATA_SEED],
        bump
    )]
    pub metadata: Box<Account<'info, Metadata>>,
    /// CHECK:
    #[account(
        mut,
        seeds = [PROGRAM_AUTHORITY_SEED],
        bump
    )]
    pub program_authority: AccountInfo<'info>,
    #[account(
        address = metadata.usdc_mint
    )]
    pub usdc_mint: Box<Account<'info, Mint>>,
    #[account(
        mut,
        associated_token::mint = usdc_mint.to_account_info(),
        associated_token::authority = program_authority
    )]
    pub program_usdc_token_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = usdc_mint.to_account_info(),
        associated_token::authority = owner.to_account_info()
    )]
    pub user_usdc_token_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = reserve_collateral_mint,
        associated_token::authority = program_authority,
    )]
    pub program_collateral_token_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        address = metadata.reserve,
    )]
    pub reserve: Box<Account<'info, SolendReserve>>,
    #[account(
        mut,
        address = reserve.liquidity.supply_pubkey
    )]
    pub reserve_liquidity_supply: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        address = reserve.collateral.mint_pubkey
    )]
    pub reserve_collateral_mint: Box<Account<'info, Mint>>,
    /// CHECK:
    #[account(
        address = metadata.lending_market
    )]
    pub lending_market: AccountInfo<'info>,
    /// CHECK:
    #[account(
        seeds = [metadata.lending_market_authority_seed.as_ref()],
        bump,
        seeds::program = metadata.lending_program,
    )]
    pub lending_market_authority: AccountInfo<'info>,
    #[account(mut)]
    pub destination_deposit_collateral: Box<Account<'info, TokenAccount>>,
    /// CHECK:
    #[account(
        mut,
        address = metadata.obligation
    )]
    pub obligation: AccountInfo<'info>,
    /// CHECK:
    #[account(
        address = reserve.liquidity.pyth_oracle_pubkey,
    )]
    pub reserve_liquidity_pyth_oracle: AccountInfo<'info>,
    /// CHECK:
    #[account(
        address = reserve.liquidity.switchboard_oracle_pubkey,
    )]
    pub reserve_liquidity_switchboard_oracle: AccountInfo<'info>,
    /// CHECK:
    #[account(
        address = "ALend7Ketfx5bxh6ghsCDXAoDrhvEmsXT3cynB6aPLgx".parse::< Pubkey > ().unwrap()
    )]
    pub lending_program: AccountInfo<'info>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub clock: Sysvar<'info, Clock>,
    pub token_program: Program<'info, Token>,
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct DepositParams {
    pub ui_amount: f64,
    pub decimals: u8
}

impl<'info> Deposit<'info> {

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
            None => current_timestamp,
            _ => user_deposit.latest_deposit_timestamp.unwrap()
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

    fn deposit_to_platform(&mut self, amount: u64, program_authority_bump: u8) -> Result<()> {
        let ix = deposit_reserve_liquidity_and_obligation_collateral(
            spl_token_lending::id(),
            amount,
            self.program_usdc_token_account.key(),
            self.program_collateral_token_account.key(),
            self.reserve.to_account_info().key(),
            self.reserve_liquidity_supply.key(),
            self.reserve_collateral_mint.key(),
            self.lending_market.key(),
            self.destination_deposit_collateral.key(),
            self.obligation.key(),
            self.program_authority.key(),
            self.reserve_liquidity_pyth_oracle.key(),
            self.reserve_liquidity_switchboard_oracle.key(),
            self.program_authority.key(),
        );

        invoke_signed(
            &ix,
            &[
                self.program_usdc_token_account.to_account_info(),
                self.program_collateral_token_account.to_account_info(),
                self.reserve.to_account_info().clone(),
                self.reserve_liquidity_supply.to_account_info(),
                self.reserve_collateral_mint.to_account_info(),
                self.lending_market.clone(),
                self.lending_market_authority.clone(),
                self.destination_deposit_collateral.to_account_info(),
                self.obligation.clone(),
                self.reserve_liquidity_pyth_oracle.clone(),
                self.reserve_liquidity_switchboard_oracle.clone(),
                self.program_authority.clone(),
                self.lending_program.clone(),
                self.clock.to_account_info(),
                self.token_program.to_account_info()
            ],
            &[
                &[PROGRAM_AUTHORITY_SEED, &[program_authority_bump]]
            ]
        ).map_err(Into::into)
    }

    pub fn deposit(&mut self, params: DepositParams, program_authority_bump: u8) -> Result<()> {

        let deposit_amount = spl_token::ui_amount_to_amount(params.ui_amount, params.decimals);

        self.update_state(deposit_amount)?;

        transfer_token(
            self.user_usdc_token_account.to_account_info().clone(),
            self.program_usdc_token_account.to_account_info().clone(),
            self.owner.to_account_info().clone(),
            &[],
            self.token_program.clone(),
            deposit_amount
        )?;

        self.deposit_to_platform(deposit_amount, program_authority_bump)
    }
}

pub fn handler(ctx: Context<Deposit>, params: DepositParams) -> Result<()> {
    if params.ui_amount == 0_f64{
        return Ok(());
    }

    let program_authority_bump = *ctx.bumps.get("program_authority").unwrap();

    ctx.accounts.deposit(params, program_authority_bump)
}
