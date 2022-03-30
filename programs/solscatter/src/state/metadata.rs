use anchor_lang::prelude::*;

#[account]
pub struct Metadata {
	pub yi_underlying_mint: Pubkey,
	pub yi_mint: Pubkey,
	pub yi_underlying_token_account : Pubkey,
	pub yi_mint_token_account: Pubkey,
	pub platform_authority: Pubkey,
	pub quarry_miner: Pubkey,
}

impl Metadata {
	pub const LEN: usize = 
		8 + // discriminator
		32 + // yi_underlying_mint
		32 + // yi_mint
		32 + // yi_underlying_token_account
		32 + // yi_mint_token_account
		32 + // platform_authority
		32 // quarry_miner
		;
}