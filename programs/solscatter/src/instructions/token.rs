use anchor_lang::prelude::*;
use anchor_spl::token::{Token, Transfer};

pub fn transfer_token<'a, 'b, 'c,'info>(
	from: AccountInfo<'info>,
	to: AccountInfo<'info>,
	authority: AccountInfo<'info>,
	signer_seeds: &'a [&'b [&'c [u8]]],
	token_program: Program<'info, Token>,
	amount: u64
) -> Result<()> {

	let ix = Transfer {
		from: from.to_account_info().clone(),
		to: to.to_account_info().clone(),
		authority: authority.to_account_info().clone()
	};

	let cpi_context = CpiContext::new_with_signer(
		token_program.to_account_info().clone(),
		ix,
		signer_seeds
	);

	anchor_spl::token::transfer(cpi_context, amount)
}