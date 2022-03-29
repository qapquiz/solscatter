use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount, Token, Transfer};

use crate::{state::{DrawingResult, DrawingState, Winner}, error::SolscatterError};

#[derive(Accounts)]
#[instruction(round: u64)]
pub struct ClaimReward<'info> {
    pub collateral_mint: Account<'info, Mint>,
    #[account(
        mut,
        seeds = [b"drawing_result".as_ref(), round.to_le_bytes().as_ref()],
        bump,
    )]
    pub drawing_result: Account<'info, DrawingResult>,
    /// CHECK: This is drawing pda for each round derive from program so do not worry
    #[account(
        seeds = [b"drawing_pda".as_ref(), round.to_le_bytes().as_ref()],
        bump,
    )]
    pub drawing_pda: AccountInfo<'info>,
    #[account(
        mut,
        associated_token::mint = collateral_mint,
        associated_token::authority = drawing_pda,
    )]
    pub drawing_reward_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        associated_token::mint = collateral_mint,
        associated_token::authority = user
    )]
    pub user_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<ClaimReward>, round: u64) -> Result<()> {
    let drawing_result = &mut ctx.accounts.drawing_result;
    let user = &ctx.accounts.user;
    let drawing_pda_bump = *ctx.bumps.get("drawing_pda").unwrap();

    if drawing_result.state != DrawingState::Finished {
        return Err(error!(SolscatterError::DrawingIsNotFinished));
    }

    let mut total_receive_amount: u64 = 0;
    let each_reward_amount =  drawing_result.reward_amount / (drawing_result.number_of_rewards as u64);

    for option_winner in drawing_result.winners.iter_mut() {
        *option_winner = match option_winner {
            Some(winner) => {
                if winner.pubkey == user.key() && winner.can_claim {
                    // claim
                    total_receive_amount = total_receive_amount.checked_add(each_reward_amount).unwrap();
                    winner.can_claim = false
                }

                Some(Winner {
                    pubkey: winner.pubkey,
                    can_claim: winner.can_claim,
                })
            },
            None => None,
        }
    }

    if total_receive_amount != 0 {
        anchor_spl::token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.drawing_reward_token_account.to_account_info(),
                    to: ctx.accounts.user_token_account.to_account_info(),
                    authority: ctx.accounts.drawing_pda.to_account_info(),
                },
                &[&[
                    b"drawing_pda".as_ref(),
                    round.to_le_bytes().as_ref(),
                    &[drawing_pda_bump],
                ]]
            ),
            total_receive_amount,
        )?;
    }

    Ok(())
}
