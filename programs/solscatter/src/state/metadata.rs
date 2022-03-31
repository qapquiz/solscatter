use anchor_lang::prelude::*;

#[account]
pub struct Metadata {
	pub yi_underlying_mint: Pubkey,
	pub yi_mint: Pubkey,
	pub yi_underlying_token_account : Pubkey,
	pub yi_mint_token_account: Pubkey,
	pub platform_authority: Pubkey,
	pub quarry: Pubkey,
	pub quarry_miner: Pubkey,
	pub quarry_miner_vault: Pubkey,
	pub quarry_rewarder: Pubkey,
}

impl Metadata {
	pub const LEN: usize = 
		8 + // discriminator
		32 + // yi_underlying_mint
		32 + // yi_mint
		32 + // yi_underlying_token_account
		32 + // yi_mint_token_account
		32 + // platform_authority
		32 + // quarry_miner
		32 + // quarry_miner_vault
		32 // quarry
		;
}