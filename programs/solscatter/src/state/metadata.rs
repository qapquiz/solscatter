use anchor_lang::prelude::*;

#[account]
pub struct Metadata {
	pub lending_program: Pubkey,
	pub usdc_mint: Pubkey,
	pub usdc_token_account: Pubkey,
	pub program_authority: Pubkey,
	pub obligation: Pubkey,
	pub reserve: Pubkey,
	pub collateral: Pubkey,
	pub lending_market: Pubkey,
	pub lending_market_authority_seed: Pubkey,
}

impl Metadata {
	pub const LEN: usize = 8 + (32 * 9);
}