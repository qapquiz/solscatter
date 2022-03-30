use anchor_lang::prelude::*;

#[account]
pub struct UserDepositReference {
	pub slot: u64,
}

impl UserDepositReference {
	pub const LEN: usize = 8 + 8;
}