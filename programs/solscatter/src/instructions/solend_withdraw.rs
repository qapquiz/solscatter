use anchor_lang::prelude::*;
use anchor_spl::token::Token;
use solana_program::{instruction::Instruction, program::invoke_signed};
use spl_token_lending::instruction::LendingInstruction;

#[derive(Accounts)]
pub struct SolendWithdraw<'info> {
    /// CHECK: JUST SOLEND CPI ACCOUNT
    #[account(mut)]
    pub source_collateral: AccountInfo<'info>,
    /// CHECK: JUST SOLEND CPI ACCOUNT
    #[account(mut)]
    pub destination_collateral: AccountInfo<'info>,
    /// CHECK: JUST SOLEND CPI ACCOUNT
    #[account(mut)]
    pub withdraw_reserve: AccountInfo<'info>,
    /// CHECK: JUST SOLEND CPI ACCOUNT
    #[account(mut)]
    pub obligation: AccountInfo<'info>,
    /// CHECK: JUST SOLEND CPI ACCOUNT
    pub lending_market: AccountInfo<'info>,
    /// CHECK: JUST SOLEND CPI ACCOUNT
    pub lending_market_authority: AccountInfo<'info>,
    /// CHECK: JUST SOLEND CPI ACCOUNT
    #[account(mut)]
    pub destination_liquidity: AccountInfo<'info>,
    /// CHECK: JUST SOLEND CPI ACCOUNT
    #[account(mut)]
    pub reserve_collateral_mint: AccountInfo<'info>,
    /// CHECK: JUST SOLEND CPI ACCOUNT
    #[account(mut)]
    pub reserve_liquidity_supply: AccountInfo<'info>,
    /// CHECK: JUST SOLEND CPI ACCOUNT
    #[account(mut, signer)]
    pub obligation_owner: AccountInfo<'info>,
    /// CHECK: JUST SOLEND CPI ACCOUNT
    #[account(mut, signer)]
    pub transfer_authority: AccountInfo<'info>,
    pub clock: Sysvar<'info, Clock>,
    pub token_program: Program<'info, Token>,
    /// CHECK: JUST SOLEND CPI ACCOUNT
    pub solend_program_address: AccountInfo<'info>,
}

pub fn solend_withdraw<'a, 'b, 'c, 'info>(
    ctx: CpiContext<'a, 'b, 'c, 'info, SolendWithdraw<'info>>,
    amount: u64,
) -> Result<()> {
    let ix = Instruction {
        program_id: ctx.accounts.solend_program_address.key(),
        accounts: vec![
            AccountMeta::new(ctx.accounts.source_collateral.key(), false),
            AccountMeta::new(ctx.accounts.destination_collateral.key(), false),
            AccountMeta::new(ctx.accounts.withdraw_reserve.key(), false),
            AccountMeta::new(ctx.accounts.obligation.key(), false),
            AccountMeta::new_readonly(ctx.accounts.lending_market.key(), false),
            AccountMeta::new_readonly(ctx.accounts.lending_market_authority.key(), false),
            AccountMeta::new(ctx.accounts.destination_liquidity.key(), false),
            AccountMeta::new(ctx.accounts.reserve_collateral_mint.key(), false),
            AccountMeta::new(ctx.accounts.reserve_liquidity_supply.key(), false),
            AccountMeta::new(ctx.accounts.obligation_owner.key(), true),
            AccountMeta::new(ctx.accounts.transfer_authority.key(), true),
            AccountMeta::new_readonly(ctx.accounts.clock.key(), false),
            AccountMeta::new_readonly(ctx.accounts.token_program.key(), false),
        ],
        data: LendingInstruction::WithdrawObligationCollateralAndRedeemReserveCollateral { collateral_amount: amount }.pack(),
    };

    invoke_signed(
        &ix,
        &[
            ctx.accounts.source_collateral.to_account_info(),
            ctx.accounts.destination_collateral.to_account_info(),
            ctx.accounts.withdraw_reserve.to_account_info(),
            ctx.accounts.obligation.to_account_info(),
            ctx.accounts.lending_market.to_account_info(),
            ctx.accounts.lending_market_authority.to_account_info(),
            ctx.accounts.destination_liquidity.to_account_info(),
            ctx.accounts.reserve_collateral_mint.to_account_info(),
            ctx.accounts.reserve_liquidity_supply.to_account_info(),
            ctx.accounts.obligation_owner.to_account_info(),
            ctx.accounts.transfer_authority.to_account_info(),
            ctx.accounts.clock.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.solend_program_address.to_account_info(),
        ],
        ctx.signer_seeds,
    ).map_err(Into::into)
}