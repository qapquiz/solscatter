use crate::{
    seed::*,
    state::{main_state::MainState, user_deposit::UserDeposit},
};
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use yi::{cpi::accounts::Stake, YiToken};

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(
        mut,
        has_one = owner,
    )]
    pub user_deposit: Account<'info, UserDeposit>,
    #[account(
        mut,
        seeds = [MAIN_STATE_SEED],
        bump,
    )]
    pub main_state: Account<'info, MainState>,
    pub owner: Signer<'info>,
    pub clock: Sysvar<'info, Clock>,

    // solUST mint = 5fjG31cbSszE6FodW37UJnNzgVTyqg5WHWGCmL3ayAvA
    // yi-solUST mint = 6XyygxFmUeemaTvA9E9mhH9FvgpynZqARVyG3gUdCMt7
    /// CHECK: yi token program
    #[account(address = yi::program::Yi::id())]
    pub yi_token_program: AccountInfo<'info>,
    /// CHECK: sol_ust_authority
    pub sol_ust_authority: AccountLoader<'info, YiToken>,
    /// [YiToken::mint]. [Mint] of the [YiToken].
    #[account(mut)]
    pub yi_mint: Account<'info, Mint>,
    /// Tokens to be staked into the [YiToken].
    #[account(
        mut,
        // associated_token::mint = "5fjG31cbSszE6FodW37UJnNzgVTyqg5WHWGCmL3ayAvA".parse::<Pubkey>(),
        // associated_token::authority = source_authority.to_account_info().key(),
    )]
    pub source_tokens: Box<Account<'info, TokenAccount>>,
    /// Thes [TokenAccount::owner] of [Self::source_tokens].
    #[account(mut)]
    pub source_authority: Signer<'info>,
    /// [YiToken::underlying_tokens].
    #[account(
        mut,
        // associated_token::mint = "5fjG31cbSszE6FodW37UJnNzgVTyqg5WHWGCmL3ayAvA".parse::<Pubkey>(),
        // associated_token::authority = sol_ust_authority.to_account_info().key(),
    )]
    pub yi_underlying_tokens: Box<Account<'info, TokenAccount>>,
    /// The [TokenAccount] receiving the minted [YiToken]s.
    #[account(
        mut,
        // associated_token::mint = "6XyygxFmUeemaTvA9E9mhH9FvgpynZqARVyG3gUdCMt7".parse::<Pubkey>(),
        // associated_token::authority = source_authority.to_account_info().key(),
    )]
    pub destination_yi_tokens: Box<Account<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>,
    // Quarry
    // #[account(
    //     seeds = [
    //         b"Miner".as_ref(),
    //         quarry.key().as_ref(),
    //         source_authority.key().as_ref(),
    //     ],
    //     bump,
    // )]
    // pub miner: Box<Account<'info, Miner>>,
    // pub quarry_mine_program: Box<Program<'info, QuarryMine>>,
    // #[account(mut)]
    // pub quarry: Box<Account<'info, quarry_mine::Quarry>>,
    // // [Rewarder]: 57fEKyj9C7dhdcrFCXFGQZp68CmjfVS4sgNecy2PdnTC
    // pub rewarder: Box<Account<'info, Rewarder>>,
    // pub system_program: Box<Program<'info, System>>,
    // pub token_mint: Box<Account<'info, Mint>>,
    // #[account(mut)]
    // pub miner_vault: Box<Account<'info, TokenAccount>>,
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct DepositParams {
    pub amount: u64,
}

impl<'info> Deposit<'info> {
    fn into_stake_cpi_context(&self) -> CpiContext<'_, '_, '_, 'info, Stake<'info>> {
        CpiContext::new(
            self.yi_token_program.to_account_info(),
            Stake {
                yi_token: self.sol_ust_authority.to_account_info(),
                yi_mint: self.yi_mint.to_account_info(),
                source_tokens: self.source_tokens.to_account_info(),
                source_authority: self.source_authority.to_account_info(),
                yi_underlying_tokens: self.yi_underlying_tokens.to_account_info(),
                destination_yi_tokens: self.destination_yi_tokens.to_account_info(),
                token_program: self.token_program.to_account_info(),
            },
        )
    }

    // fn into_create_miner_context(&self) -> CpiContext<'_, '_, '_, 'info, CreateMiner<'info>> {
    //     let (miner_pda, miner_bump) = Pubkey::find_program_address(
    //         &[
    //             b"Miner",
    //             self.quarry.to_account_info().key().as_ref(),
    //             self.source_authority.to_account_info().key().as_ref(),
    //         ],
    //         &quarry_mine::id(),
    //     );

    //     CpiContext::new(
    //         // quarry_mine: QMNeHCGYnLVDn1icRAfQZpjPLBNkfGbSKRB83G5d8KB
    //         quarry_mine::id(),
    //         CreateMiner {
    //             authority: self.source_authority.to_account_info(),
    //             miner: self.miner.to_account_info(),
    //             quarry: self.quarry.to_account_info(),
    //             rewarder: self.rewarder.to_account_info(),
    //             system_program: self.system_program.to_account_info(),
    //             payer: self.source_authority.to_account_info(),
    //             token_mint: self.token_mint.to_account_info(),
    //             miner_vault: self.miner_vault.to_account_info(),
    //             token_program: self.token_program.to_account_info(),
    //         },
    //     )
    // }

    fn update_state(&mut self, amount: u64) -> Result<()> {
        let user_deposit = &mut self.user_deposit;
        user_deposit.amount = user_deposit.amount + amount;
        user_deposit.latest_deposit_timestamp = Some(self.clock.unix_timestamp);

        let main_state = &mut self.main_state;
        main_state.total_deposit = main_state.total_deposit + amount;
        Ok(())
    }

    fn stake_sol_ust(&self, amount: u64) -> Result<()> {
        yi::cpi::stake(self.into_stake_cpi_context(), amount)
    }

    // fn create_miner(&self, ctx: &Context<Self>) -> Result<()> {
    //     // quarry
    //     let bump = *ctx.bumps.get("miner").unwrap();
    //     quarry_mine::cpi::create_miner(self.into_create_miner_context(), bump)
    // }

    pub fn deposit(&mut self, params: DepositParams) -> Result<()> {
        self.update_state(params.amount)?;
        self.stake_sol_ust(params.amount)?;
        // self.create_miner(ctx)?;
        Ok(())
    }
}

pub fn handler(ctx: Context<Deposit>, params: DepositParams) -> Result<()> {
    if params.amount == 0 {
        return Ok(());
    }

    ctx.accounts.deposit(params)
}
