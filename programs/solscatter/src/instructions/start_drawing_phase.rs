use anchor_lang::prelude::*;
use anchor_spl::{token::{TokenAccount, Mint, Token}, associated_token::AssociatedToken};
use crate::{state::{drawing_result::{DrawingResult, DrawingState}, main_state::MainState, SolendObligation, SolendReserve}, instructions::solend_withdraw::solend_withdraw};
use crate::error::SolscatterError;
use crate::seed::PROGRAM_AUTHORITY_SEED;

use super::solend_withdraw::SolendWithdraw;

#[derive(Accounts)]
#[instruction(number_of_rewards: u8, random_numbers: Vec<u64>)]
pub struct StartDrawingPhase<'info> {
    #[account(
        init,
        payer = signer,
        space = DrawingResult::space(number_of_rewards)?,
        seeds = [
            b"drawing_result".as_ref(), 
            main_state.current_round.to_le_bytes().as_ref(),
        ],
        bump,
    )]
    pub drawing_result: Box<Account<'info, DrawingResult>>,
    #[account(
        seeds = [b"main_state".as_ref()],
        bump
    )]
    pub main_state: Box<Account<'info, MainState>>,
    pub collateral_mint: Box<Account<'info, Mint>>,
    /// CHECK: PDA Authority for reward_token_account
    #[account(
        seeds = [
            b"drawing_pda".as_ref(),
            main_state.current_round.to_le_bytes().as_ref(),
        ],
        bump,
    )]
    pub drawing_pda: AccountInfo<'info>,
    #[account(
        init,
        payer = signer,
        associated_token::mint = collateral_mint,
        associated_token::authority = drawing_pda,
    )]
    pub drawing_reward_token_account: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,

    /// CHECK: JUST SOLEND CPI ACCOUNT
    #[account(mut)]
    pub source_collateral: AccountInfo<'info>,
    /// CHECK: JUST SOLEND CPI ACCOUNT
    #[account(mut)]
    pub destination_collateral: AccountInfo<'info>,
    /// CHECK: JUST SOLEND CPI ACCOUNT
    #[account(mut)]
    pub withdraw_reserve: Box<Account<'info, SolendReserve>>,
    /// CHECK: JUST SOLEND CPI ACCOUNT
    #[account(mut)]
    pub obligation: Box<Account<'info, SolendObligation>>,
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
    #[account(
        mut,
        seeds = [PROGRAM_AUTHORITY_SEED],
        bump,
    )]
    pub obligation_owner: AccountInfo<'info>,
    /// CHECK: JUST SOLEND CPI ACCOUNT
    #[account(
        mut,
        seeds = [PROGRAM_AUTHORITY_SEED],
        bump,
    )]
    pub transfer_authority: AccountInfo<'info>,
    pub clock: Sysvar<'info, Clock>,
    pub token_program: Program<'info, Token>,
    /// CHECK: JUST SOLEND CPI ACCOUNT
    pub solend_program_address: AccountInfo<'info>,
}

impl<'info> StartDrawingPhase<'info> {
    fn claim_reward(&self, program_authority_bump: u8) -> Result<u64> {
        let main_state = self.main_state.clone();
        let obligation = self.obligation.clone();
        let c_token_exchange_rate = self.withdraw_reserve.collateral_exchange_rate()?;
        let deposit_plus_interest_amount = obligation.deposits.get(0);
        let reward_amount: u64;
        match deposit_plus_interest_amount {
            Some(deposit) => {
                let interest_amount = c_token_exchange_rate.collateral_to_liquidity(deposit.deposited_amount)? - main_state.total_deposit;
                let withdraw_collateral_amount = c_token_exchange_rate.liquidity_to_collateral(interest_amount)?;
                reward_amount = withdraw_collateral_amount;

                if interest_amount <= 0 {
                    return Err(error!(SolscatterError::YieldLessOrEqualZero));
                }

                if interest_amount >= main_state.total_deposit {
                    msg!("collateral deposited_amount: {}", deposit.deposited_amount.to_string().as_str());
                    msg!("liquidity deposited_amount: {}", c_token_exchange_rate.collateral_to_liquidity(deposit.deposited_amount)?.to_string().as_str());
                    msg!("total_deposit: {}", main_state.total_deposit.to_string().as_str());
                    msg!("withdraw_collateral_amount: {}", withdraw_collateral_amount.to_string().as_str());
                    msg!("interest_amount: {}", interest_amount.to_string().as_str());
                    return Err(error!(SolscatterError::YieldMoreThanTotalDeposit));
                }

                // claim here
                solend_withdraw(
                    CpiContext::new_with_signer(
                        self.solend_program_address.to_account_info(),
                        SolendWithdraw {
                            source_collateral: self.source_collateral.to_account_info(), 
                            destination_collateral: self.destination_collateral.to_account_info(), 
                            withdraw_reserve: self.withdraw_reserve.to_account_info(), 
                            obligation: self.obligation.to_account_info(), 
                            lending_market: self.lending_market.to_account_info(), 
                            lending_market_authority: self.lending_market_authority.to_account_info(), 
                            destination_liquidity: self.destination_liquidity.to_account_info(), 
                            reserve_collateral_mint: self.reserve_collateral_mint.to_account_info(), 
                            reserve_liquidity_supply: self.reserve_liquidity_supply.to_account_info(), 
                            obligation_owner: self.obligation_owner.to_account_info(), 
                            transfer_authority: self.transfer_authority.to_account_info(), 
                            clock: self.clock.clone(), 
                            token_program: self.token_program.clone(), 
                            solend_program_address: self.solend_program_address.to_account_info()
                        },
                        &[&[PROGRAM_AUTHORITY_SEED, &[program_authority_bump]]],  
                    ),
                    withdraw_collateral_amount 
                )?;
            },
            None => {
                return Err(error!(SolscatterError::NoDepositInObligation));
            },
        }
        Ok(reward_amount)
    }
}

pub fn handler(ctx: Context<StartDrawingPhase>, number_of_rewards: u8, random_numbers: Vec<u64>) -> Result<()> {
    if number_of_rewards <= 0 {
        return Err(error!(SolscatterError::NumberOfRewardsMustMoreThanZero));
    }

    if random_numbers.len() != number_of_rewards as usize {
        return Err(error!(SolscatterError::NumberOfRandomNumbersNotMatchWithNumberOfRewards));
    }

    let program_authority_bump = *ctx.bumps.get("obligation_owner").unwrap();

    let reward_amount = ctx.accounts.claim_reward(program_authority_bump)?;

    let main_state = &ctx.accounts.main_state;
    let drawing_result = &mut ctx.accounts.drawing_result;
    drawing_result.round = main_state.current_round;
    drawing_result.state = DrawingState::Processing;
    drawing_result.reward_amount = reward_amount;
    drawing_result.number_of_rewards = number_of_rewards;
    drawing_result.winners = vec!(); 
    drawing_result.random_numbers = random_numbers;
    drawing_result.total_deposit = main_state.total_deposit;
    drawing_result.last_processed_slot = 0;
    drawing_result.finished_timestamp = None;

    for _ in 0..number_of_rewards {
        drawing_result.winners.push(None);
    }

    Ok(())
}
