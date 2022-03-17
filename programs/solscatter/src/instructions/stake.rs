use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount, Token};
use yi::cpi::accounts::Stake;

#[derive(Accounts)]
pub struct SolUSTStake<'info> {
    /// CHECK: sol_ust_authority
    pub sol_ust_authority: AccountInfo<'info>,
    #[account(mut)]
    pub yi_sol_ust_mint: Account<'info, Mint>,
    #[account(mut)]
    pub sol_ust_depositor_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub depositor: Signer<'info>,
    #[account(mut)]
    pub sol_ust_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub yi_sol_ust_depositor_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

impl<'info> SolUSTStake<'info> {
    fn into_stake_cpi_context(&self) -> CpiContext<'_, '_, '_, 'info, Stake<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Stake {
                yi_token: self.sol_ust_authority.clone(),
                yi_mint: self.yi_sol_ust_mint.to_account_info(),
                source_tokens: self.sol_ust_depositor_token_account.to_account_info(),
                source_authority: self.depositor.to_account_info(),
                yi_underlying_tokens: self.sol_ust_token_account.to_account_info(),
                destination_yi_tokens: self.yi_sol_ust_depositor_token_account.to_account_info(),
                token_program: self.token_program.to_account_info(),
            },
        )
    }
}

pub fn handler(ctx: Context<SolUSTStake>, amount: u64) -> Result<()> {
    // solUST mint = 5fjG31cbSszE6FodW37UJnNzgVTyqg5WHWGCmL3ayAvA
    // yi-solUST mint = 6XyygxFmUeemaTvA9E9mhH9FvgpynZqARVyG3gUdCMt7
    yi::cpi::stake(ctx.accounts.into_stake_cpi_context(), amount)
}