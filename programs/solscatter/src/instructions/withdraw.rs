use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use solana_program::instruction::Instruction;
use solana_program::program::{invoke, invoke_signed};
use solana_program::sysvar;
use spl_token_lending::instruction::{LendingInstruction, refresh_obligation, refresh_reserve};
use crate::{
	seed::*,
	error::SolscatterError,
	state::{main_state::MainState, metadata::Metadata, user_deposit::UserDeposit},
	instructions::token::transfer_token
};
use crate::state::SolendReserve;

#[derive(Accounts)]
pub struct Withdraw<'info> {
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
	/// CHECK:
	#[account(
		mut,
		address = metadata.obligation
	)]
	pub obligation: AccountInfo<'info>,
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
	#[account(
		mut,
		address = reserve.collateral.mint_pubkey
	)]
	pub reserve_collateral_mint: Box<Account<'info, Mint>>,
	#[account(
		mut,
		address = reserve.collateral.supply_pubkey
	)]
	pub reserve_collateral_supply: Box<Account<'info, TokenAccount>>,
	#[account(
		mut,
		address = reserve.liquidity.supply_pubkey
	)]
	pub reserve_liquidity_supply: Box<Account<'info, TokenAccount>>,
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
pub struct WithdrawParams {
	pub ui_amount: f64,
	pub decimals: u8
}

impl<'info> Withdraw<'info> {

	fn update_state(&mut self, amount: u64) -> Result<()> {
		let user_deposit = &mut self.user_deposit;
		user_deposit.amount = user_deposit.amount - amount;

		let main_state = &mut self.main_state;
		main_state.total_deposit = main_state.total_deposit - amount;
		Ok(())
	}
	
	fn withdraw_from_platform(&mut self, amount: u64, program_authority_bump: u8) -> Result<()> {
		let ix_refresh_reserve_liquidity = refresh_reserve(
			self.metadata.lending_program,
			self.reserve.key(),
			self.reserve_liquidity_pyth_oracle.key(),
			self.reserve_liquidity_switchboard_oracle.key()
		);
		invoke(
			&ix_refresh_reserve_liquidity,
			&[
				self.reserve.to_account_info().clone(),
				self.reserve_liquidity_pyth_oracle.clone(),
				self.reserve_liquidity_switchboard_oracle.clone(),
				self.lending_program.clone(),
				self.clock.to_account_info(),
			],
		)?;

		let ix_refresh_obligation = refresh_obligation(
			self.metadata.lending_program,
			self.obligation.key(),
			vec![self.reserve.key().clone()]
		);

		invoke(
			&ix_refresh_obligation,
			&[
				self.obligation.clone(),
				self.reserve.to_account_info().clone(),
				self.clock.to_account_info()
			]
		)?;

		let ix_withdraw = Instruction {
			program_id: self.metadata.lending_program,
			accounts: vec![
				AccountMeta::new(self.reserve_collateral_supply.key(), false),
				AccountMeta::new(self.program_collateral_token_account.to_account_info().key(), false),
				AccountMeta::new(self.reserve.to_account_info().key(), false),
				AccountMeta::new(self.obligation.key(), false),
				AccountMeta::new_readonly(self.lending_market.key(), false),
				AccountMeta::new_readonly(self.lending_market_authority.key(), false),
				AccountMeta::new(self.program_usdc_token_account.to_account_info().key(), false),
				AccountMeta::new(self.reserve_collateral_mint.to_account_info().key(), false),
				AccountMeta::new(self.reserve_liquidity_supply.to_account_info().key(), false),
				AccountMeta::new_readonly(self.program_authority.key(), true),
				AccountMeta::new_readonly(self.program_authority.key(), true),
				AccountMeta::new_readonly(sysvar::clock::id(), false),
				AccountMeta::new_readonly(Token::id(), false),
			],
			data: LendingInstruction::WithdrawObligationCollateralAndRedeemReserveCollateral { collateral_amount: amount }.pack(),
		};

		invoke_signed(
			&ix_withdraw,
			&[
				self.reserve_collateral_supply.to_account_info().clone(),
				self.program_collateral_token_account.to_account_info(),
				self.reserve.to_account_info().clone(),
				self.obligation.clone(),
				self.lending_market.clone(),
				self.lending_market_authority.clone(),
				self.program_usdc_token_account.to_account_info(),
				self.reserve_collateral_mint.to_account_info(),
				self.reserve_liquidity_supply.to_account_info(),
				self.program_authority.clone(),
				self.clock.to_account_info(),
				self.token_program.to_account_info(),
			],
			&[&[PROGRAM_AUTHORITY_SEED, &[program_authority_bump]]],
		).map_err(Into::into)
		
	}

	pub fn withdraw(&mut self, params: WithdrawParams, program_authority_bump: u8) -> Result<()> {
		let amount = spl_token::ui_amount_to_amount(params.ui_amount, params.decimals);

		if amount > self.user_deposit.amount {
			return Err(error!(SolscatterError::InsufficientAmount))
		}
		
		self.update_state(amount)?;

		self.withdraw_from_platform(amount, program_authority_bump)?;

		let fee = spl_token::ui_amount_to_amount(params.ui_amount * 0.5, params.decimals);
		let withdraw_amount = amount - fee;

		transfer_token(
			self.program_usdc_token_account.to_account_info(),
			self.user_usdc_token_account.to_account_info(),
			self.program_authority.to_account_info(),
			&[&[PROGRAM_AUTHORITY_SEED, &[program_authority_bump]]],
			self.token_program.clone(),
			withdraw_amount
		)
	}
}

pub fn handler(ctx: Context<Withdraw>, params: WithdrawParams) -> Result<()> {

	let program_authority_bump = *ctx.bumps.get("program_authority").unwrap();

	ctx.accounts.withdraw(params, program_authority_bump)
}
