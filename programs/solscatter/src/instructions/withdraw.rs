use crate::{
	seed::*,
	error::SolscatterError,
	state::{MainState, Metadata, UserDeposit},
};
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount, Transfer};
use quarry_mine::{cpi::accounts::UserStake, Miner, Quarry, Rewarder};
use yi::{cpi::accounts::Unstake, YiToken};


#[derive(Accounts)]
pub struct Withdraw<'info> {
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
	// ######## YIELD GENERATOR ########
	#[account(
		mut,
		address = metadata.yi_mint,
	)]
	pub yi_mint: Box<Account<'info, Mint>>,
	#[account(
		address = metadata.yi_underlying_mint,
	)]
	pub yi_underlying_mint: Box<Account<'info, Mint>>,
	/// CHECK: Yi token program (solUST authority)
	#[account(
		address = yi::program::Yi::id()
	)]
	pub yi_program: AccountInfo<'info>,
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
	// ######## QUARRY ########
	/// CHECK: this is quarry program already checked with address =
	#[account(
		address = quarry_mine::program::QuarryMine::id(),
	)]
	pub quarry_program: AccountInfo<'info>,
	/// CHECK: this is miner check with seed
	#[account(
		mut,
		address = metadata.quarry_miner
	)]
	pub miner: Account<'info, Miner>,
	#[account(
		mut,
		address = metadata.quarry
	)]
	pub quarry: Box<Account<'info, Quarry>>,
	#[account(
		address = metadata.quarry_rewarder
	)]
	pub rewarder: Box<Account<'info, Rewarder>>,
	#[account(
		mut,
		associated_token::mint = yi_mint,
		associated_token::authority = miner,
	)]
	pub miner_vault: Box<Account<'info, TokenAccount>>,
	// ######## NATIVE PROGRAM ########
	pub clock: Sysvar<'info, Clock>,
	pub token_program: Program<'info, Token>,
}

impl<'info> Withdraw<'info> {
	pub fn validate(&self, amount: u64) -> Result<()> {
		let user_deposit = &self.user_deposit;

		require_gt!(amount, user_deposit.amount, SolscatterError::InsufficientBalance);

		if user_deposit.latest_deposit_timestamp.is_none() {
			return Err(error!(SolscatterError::NeverDeposited));
		}

		Ok(())
	}

	fn into_quarry_stake_cpi_context(&self) -> CpiContext<'_, '_, '_, 'info, UserStake<'info>> {
		CpiContext::new(
			self.quarry_program.to_account_info(),
			UserStake {
				authority: self.platform_authority.to_account_info(),
				miner: self.miner.to_account_info(),
				quarry: self.quarry.to_account_info(),
				miner_vault: self.miner_vault.to_account_info(),
				token_account: self.platform_yi_token_account.to_account_info(),
				token_program: self.token_program.to_account_info(),
				rewarder: self.rewarder.to_account_info()
			}
		)
	}

	fn into_yi_unstake_cpi_context(&self) -> CpiContext<'_, '_, '_, 'info, Unstake<'info>> {
		CpiContext::new(
			self.yi_program.to_account_info(),
			Unstake {
				yi_token: self.yi_token.to_account_info(),
				yi_mint: self.yi_mint.to_account_info(),
				source_yi_tokens: self.platform_yi_token_account.to_account_info(),
				source_authority: self.platform_authority.to_account_info(),
				yi_underlying_tokens: self.yi_underlying_token_account.to_account_info(),
				destination_underlying_tokens: self.platform_yi_underlying_token_account.to_account_info(),
				token_program: self.token_program.to_account_info(),
			},
		)
	}

	fn update_state(&mut self, amount: u64) -> Result<()> {
		let user_deposit = &mut self.user_deposit;
		user_deposit.amount = user_deposit.amount.checked_sub(amount).unwrap();

		let main_state = &mut self.main_state;
		main_state.total_deposit = main_state.total_deposit.checked_sub(amount).unwrap();

		Ok(())
	}

	fn withdraw_from_quarry(&mut self, yi_amount: u64, platform_authority_bump: u8) -> Result<()> {
		quarry_mine::cpi::withdraw_tokens(
			self.into_quarry_stake_cpi_context()
				.with_signer(&[&[PLATFORM_AUTHORITY_SEED.as_bytes(), &[platform_authority_bump]]]),
			yi_amount
		)
	}

	fn unstake_from_yield_generator(&mut self, yi_amount: u64, platform_authority_bump: u8) -> Result<()> {
		yi::cpi::unstake(
			self.into_yi_unstake_cpi_context()
				.with_signer(&[&[PLATFORM_AUTHORITY_SEED.as_bytes(), &[platform_authority_bump]]]),
			yi_amount,
		)
	}

	fn transfer_yi_underlying_to_depositor(&self, amount: u64, platform_authority_bump: u8) -> Result<()> {
		let cpi_account = Transfer {
			from: self.platform_yi_underlying_token_account.to_account_info(),
			to: self.depositor_yi_underlying_token_account.to_account_info(),
			authority: self.platform_authority.to_account_info(),
		};

		let cpi_ctx = CpiContext::new(self.token_program.to_account_info(), cpi_account);

		anchor_spl::token::transfer(
			cpi_ctx
				.with_signer(&[&[PLATFORM_AUTHORITY_SEED.as_bytes(), &[platform_authority_bump]]]),
			amount
		)
	}

	fn withdraw(&mut self, params: WithdrawParams, platform_authority_bump: u8) -> Result<()> {
		let amount = spl_token::ui_amount_to_amount(params.ui_amount, params.decimals);
		let yi_amount = self.yi_token.load()?.calculate_yitokens_for_underlying(
			amount,
			self.yi_underlying_token_account.amount,
			self.yi_mint.supply
		).unwrap();

		self.validate(amount)?;
		self.withdraw_from_quarry(yi_amount, platform_authority_bump)?;
		self.unstake_from_yield_generator(yi_amount, platform_authority_bump)?;
		self.user_deposit.refresh_penalty_fee(self.clock.unix_timestamp, self.main_state.penalty_period)?;

        let penalty_fee = self.main_state.penalty_early_withdraw_fee.calculate_fee_with_discount_peroid(
            amount,
            self.main_state.penalty_period as u64,
            self.user_deposit.latest_deposit_timestamp.unwrap(),
            self.clock.unix_timestamp,
        )?;
        let receive_amount = amount.checked_sub(penalty_fee).unwrap();

        if penalty_fee != 0 {
            // @todo transfer to fee collector token account
        }

		let fee = (self.user_deposit.penalty_fee + self.main_state.default_fee) / 100_f64;
		let fee_amount = spl_token::ui_amount_to_amount(params.ui_amount * fee, params.decimals);
		let withdraw_amount = amount - fee_amount;
        
        require_eq!(fee_amount, penalty_fee);
        require_eq!(withdraw_amount, receive_amount);

		self.transfer_yi_underlying_to_depositor(withdraw_amount, platform_authority_bump)?;
		self.update_state(amount)
	}
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct WithdrawParams {
	pub ui_amount: f64,
	pub decimals: u8,
}

pub fn handler(ctx: Context<Withdraw>, params: WithdrawParams) -> Result<()> {
	let platform_authority_bump = *ctx.bumps.get("platform_authority").unwrap();
	ctx.accounts.withdraw(params, platform_authority_bump)
}
