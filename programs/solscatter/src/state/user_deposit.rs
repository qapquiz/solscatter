use std::cmp::max;
use anchor_lang::prelude::*;
use solana_program::clock::UnixTimestamp;

#[account]
pub struct UserDeposit {
	pub slot: u64,
	pub amount: u64,
	pub penalty_fee: f64,
	pub owner: Pubkey,
	pub latest_deposit_timestamp: Option<i64>,
}

impl UserDeposit {
	pub const LEN: usize = 8 + 8 + 8 + 8 + 32 + 9;

	pub fn refresh_penalty_fee(&mut self, current_timestamp: UnixTimestamp, penalty_period: i64) -> Result<()> {
		if self.latest_deposit_timestamp == None { return Ok(()); }

		let duration_to_penalty_period = max((self.latest_deposit_timestamp.unwrap() + penalty_period) - current_timestamp, 0) as f64;

		let refreshed_penalty_fee = (duration_to_penalty_period / penalty_period as f64) * self.penalty_fee;

		self.penalty_fee = refreshed_penalty_fee;

		msg!("refreshed_penalty_fee : {} ", refreshed_penalty_fee);

		Ok(())
	}

	pub fn update_penalty_fee(&mut self, amount: u64, current_timestamp: UnixTimestamp, penalty_period: i64, platform_penalty_fee: f64) -> Result<()> {
		self.refresh_penalty_fee(current_timestamp, penalty_period)?;

		let new_penalty_fee = ((self.amount as f64 * self.penalty_fee) + (amount as f64 * platform_penalty_fee)) / (self.amount + amount) as f64;

		self.penalty_fee = new_penalty_fee;

		msg!("new_penalty_fee : {} ", new_penalty_fee);

		Ok(())
	}
}
