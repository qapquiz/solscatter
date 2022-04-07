use anchor_lang::prelude::*;
use anchor_spl::{token::{Mint, Token, TokenAccount}, associated_token::{AssociatedToken}};
use quarry_mine::{Miner, Quarry, Rewarder, cpi::accounts::UserStake};
use yi::{YiToken, cpi::accounts::Unstake};
use crate::{state::{drawing_result::{DrawingResult, DrawingState}, main_state::MainState, Metadata}};
use crate::error::SolscatterError;
use crate::seed::{
    METADATA_SEED,
    DRAWING_RESULT_SEED,
    DRAWING_PDA_SEED,
    MAIN_STATE_SEED,
    PLATFORM_AUTHORITY_SEED
};

#[derive(Accounts)]
#[instruction(number_of_rewards: u8, random_numbers: Vec<u64>)]
pub struct StartDrawingPhase<'info> {
    /// CHECK: platform authority
    #[account(
        seeds = [PLATFORM_AUTHORITY_SEED.as_bytes()],
        bump,
    )]
    pub platform_authority: AccountInfo<'info>,
    #[account(
        seeds = [MAIN_STATE_SEED.as_bytes()],
        bump
    )]
    pub main_state: Account<'info, MainState>,
    #[account(
        seeds = [METADATA_SEED.as_bytes()],
        bump,
    )]
    pub metadata: Box<Account<'info, Metadata>>,
    #[account(
        init,
        payer = signer,
        space = DrawingResult::space(number_of_rewards)?,
        seeds = [
            DRAWING_RESULT_SEED.as_bytes(),
            main_state.current_round.to_le_bytes().as_ref(),
        ],
        bump,
    )]
    pub drawing_result: Account<'info, DrawingResult>,
    /// CHECK: drawing authority for each round
    #[account(
        seeds = [
            DRAWING_PDA_SEED.as_bytes(),
            main_state.current_round.to_le_bytes().as_ref(),
        ],
        bump,
    )]
    pub drawing_pda: AccountInfo<'info>,
    #[account(
        init,
        payer = signer,
        associated_token::mint = yi_underlying_mint,
        associated_token::authority = drawing_pda,
    )]
    pub drawing_pda_yi_underlying_token_account: Account<'info, TokenAccount>,
    // ######## YI ########
    /// CHECK: Yi token program
    #[account(
        address = yi::program::Yi::id(),
    )]
    pub yi_program: AccountInfo<'info>,
    pub yi_token: AccountLoader<'info, YiToken>,
    #[account(
        mut,
        associated_token::mint = yi_underlying_mint,
        associated_token::authority = yi_token,
    )]
    pub yi_underlying_token_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = yi_mint,
        associated_token::authority = platform_authority,
    )]
    pub platform_yi_token_account: Box<Account<'info, TokenAccount>>,
    
    // ######## QUARRY ########
    /// CHECK: Quarry program
    #[account(
        address = quarry_mine::program::QuarryMine::id(),
    )]
    pub quarry_program: AccountInfo<'info>,
    /// CHECK: this is miner of platform
    #[account(
        address = metadata.quarry_miner,
    )]
    pub miner: Box<Account<'info, Miner>>,
    #[account(
		mut,
		address = metadata.quarry
	)]
	pub quarry: Box<Account<'info, Quarry>>,
	#[account(
		address = metadata.quarry_rewarder
	)]
	pub rewarder: Box<Account<'info, Rewarder>>,
	#[account(
		mut,
		associated_token::mint = yi_mint,
		associated_token::authority = miner,
	)]
	pub miner_vault: Box<Account<'info, TokenAccount>>,

    pub yi_underlying_mint: Box<Account<'info, Mint>>,
    pub yi_mint: Box<Account<'info, Mint>>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> StartDrawingPhase<'info> {
    pub fn validate(&self, number_of_rewards: u8, random_numbers: &Vec<u64>) -> Result<()> {
        require_gt!(number_of_rewards, 0, SolscatterError::NumberOfRewardsMustMoreThanZero);
        require_neq!(random_numbers.len(), number_of_rewards as usize, SolscatterError::NumberOfRandomNumbersNotMatchWithNumberOfRewards);
        Ok(())
    }

    pub fn withdraw_from_quarry(&self, withdraw_yi_amount: u64, platform_authority_bump: u8) -> Result<()> {
        let signer_seeds: &[&[&[u8]]] = &[&[PLATFORM_AUTHORITY_SEED.as_bytes(), &[platform_authority_bump]]];
        let cpi_context = CpiContext::new_with_signer(
            self.quarry_program.to_account_info(),
            UserStake {
                authority: self.platform_authority.to_account_info(),
                miner: self.miner.to_account_info(),
                quarry: self.quarry.to_account_info(),
                miner_vault: self.miner_vault.to_account_info(),
                token_account: self.platform_yi_token_account.to_account_info(),
                token_program: self.token_program.to_account_info(),
                rewarder: self.rewarder.to_account_info(),
            },
            signer_seeds,
        );

        quarry_mine::cpi::withdraw_tokens(cpi_context, withdraw_yi_amount)
    }

    pub fn unstake_from_solust(&self, withdraw_yi_amount: u64, platform_authority_bump: u8) -> Result<()> {
        let signer_seeds: &[&[&[u8]]] = &[&[PLATFORM_AUTHORITY_SEED.as_bytes(), &[platform_authority_bump]]];
        let cpi_context = CpiContext::new_with_signer(
            self.yi_program.to_account_info(),
            Unstake {
                yi_token: self.yi_token.to_account_info(),
                yi_mint: self.yi_mint.to_account_info(),
                source_yi_tokens: self.platform_yi_token_account.to_account_info(),
                source_authority: self.platform_authority.to_account_info(),
                yi_underlying_tokens: self.yi_underlying_token_account.to_account_info(),
                destination_underlying_tokens: self.drawing_pda_yi_underlying_token_account.to_account_info(),
                token_program: self.token_program.to_account_info(),
            },
            signer_seeds
        );

        yi::cpi::unstake(cpi_context, withdraw_yi_amount)
    }

    pub fn calculate_yield(&self) -> Result<u64> {
        let yi_token = self.yi_token.load()?;
        let miner = &self.miner;
        let yi_amount = miner.balance;
        let yi_underlying_amount = yi_token.calculate_underlying_for_yitokens(
            yi_amount,
            self.yi_underlying_token_account.amount,
            self.yi_mint.supply,
        ).unwrap();

        let reward_underlying_amount = yi_underlying_amount.checked_sub(
            self.main_state.total_deposit
        ).unwrap();

        require_gt!(reward_underlying_amount, 0);

        let reward_yi_amount = yi_token.calculate_yitokens_for_underlying(
            reward_underlying_amount,
            self.yi_underlying_token_account.amount,
            self.yi_mint.supply,
        ).unwrap();

        Ok(reward_yi_amount)
    }

    pub fn claim_yield(&mut self, platform_authority_bump: u8) -> Result<()> {
        let withdraw_yi_amount = self.calculate_yield()?;
        self.withdraw_from_quarry(withdraw_yi_amount, platform_authority_bump)?;
        self.unstake_from_solust(withdraw_yi_amount, platform_authority_bump)?;

        self.drawing_pda_yi_underlying_token_account.reload()?;

        require_gt!(self.drawing_pda_yi_underlying_token_account.amount, 0);
        Ok(())
    }
}

pub fn handler(ctx: Context<StartDrawingPhase>, number_of_rewards: u8, random_numbers: Vec<u64>) -> Result<()> {
    let platform_authority_bump = *ctx.bumps.get("platform_authority").unwrap();
    ctx.accounts.claim_yield(platform_authority_bump)?;

    let main_state = &ctx.accounts.main_state;
    let reward_token_account = &ctx.accounts.drawing_pda_yi_underlying_token_account;
    let drawing_result = &mut ctx.accounts.drawing_result;
    drawing_result.round = main_state.current_round;
    drawing_result.state = DrawingState::Processing;
    drawing_result.reward_token_account_pubkey = reward_token_account.key();
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