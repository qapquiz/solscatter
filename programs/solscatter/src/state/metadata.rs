use anchor_lang::prelude::*;

#[account]
pub struct Metadata {
	pub yi_underlying_mint: Pubkey,
	pub yi_mint: Pubkey,
	pub yi_underlying_token_account : Pubkey,
	pub yi_mint_token_account: Pubkey,
	pub platform_authority: Pubkey,
}

impl Metadata {
	pub const LEN: usize = 8 + (32 * 3);
}