use anchor_lang::prelude::*;

declare_id!("GxPZABaCGhe9MizXWLGy4Si2gwEt7FPUhjAiHkTWGp4u");

#[program]
pub mod loki {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let main_state = &mut ctx.accounts.main_state;
        main_state.current_slot = 0;
        main_state.current_round = 1;
        main_state.total_deposit = 0;
        main_state.switchboard_pubkey = ctx.accounts.switchboard.key();

        let treasury = &mut ctx.accounts.treasury;
        treasury.bags = vec![];
        Ok(())
    }

    pub fn receive_from_remaining_accounts(_ctx: Context<ReceiveFromRemainingAccounts>) -> Result<()> {
        Ok(())
    }

    pub fn deposit_initialize(ctx: Context<DepositInitialize>) -> Result<()> {
        let user_deposit = &mut ctx.accounts.user_deposit;
        let main_state = &mut ctx.accounts.main_state;
        let depositor = &ctx.accounts.depositor;

        user_deposit.slot = main_state.current_slot + 1;
        user_deposit.amount = 0;
        user_deposit.owner = depositor.key().clone();

        main_state.current_slot = main_state.current_slot + 1;
        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        let user_deposit = &mut ctx.accounts.user_deposit;
        user_deposit.amount = user_deposit.amount + amount;

        let main_state = &mut ctx.accounts.main_state;
        main_state.total_deposit = main_state.total_deposit + amount;

        Ok(())
    }

    pub fn start_drawing_phaase(ctx: Context<StartDrawingPhase>, random_number: u64) -> Result<()> {
        let main_state = &ctx.accounts.main_state;
        let drawing_result = &mut ctx.accounts.drawing_result;
        drawing_result.round = main_state.current_round;
        drawing_result.state = DrawingState::Processing;
        drawing_result.winner = None;
        drawing_result.random_number = random_number;
        drawing_result.total_deposit = main_state.total_deposit;
        drawing_result.last_processed_slot = 0;
        drawing_result.finished_timestamp = None;

        Ok(())
    }


    pub fn drawing(ctx: Context<Drawing>) -> Result<()> {
        let drawing_result = &mut ctx.accounts.drawing_result;
        
        let main_state = &mut ctx.accounts.main_state;
        let user_deposit = &ctx.accounts.user_deposit;

        if drawing_result.random_number < user_deposit.amount {
            // Found the winner
            drawing_result.winner = Some(user_deposit.owner);
            // Set finished timestamp
            drawing_result.state = DrawingState::Finished;

            main_state.current_round = main_state.current_round + 1;
            return Ok(());
        }

        drawing_result.random_number = drawing_result.random_number - user_deposit.amount;
        drawing_result.last_processed_slot = drawing_result.last_processed_slot + 1;
        return Ok(());
    }
}

#[derive(Accounts)]
pub struct Initialize<'info>  {
    #[account(
        init,
        payer = signer,
        space = MainState::LEN,
        seeds = [b"main_state"],
        bump,
    )]
    pub main_state: Account<'info, MainState>,
    #[account(
        init,
        payer = signer,
        space = Treasury::space(20_u16),
        seeds = [b"treasury"],
        bump
    )]
    pub treasury: Account<'info, Treasury>,
    /// CHECK: This is switchboard pubkey will check with address =
    pub switchboard: AccountInfo<'info>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DepositInitialize<'info> {
    #[account(
        init,
        payer = depositor,
        seeds = [(main_state.current_slot + 1).to_le_bytes().as_ref()],
        bump,
        space = UserDeposit::LEN,
    )]
    pub user_deposit: Account<'info, UserDeposit>,
    #[account(
        mut,
        seeds = [b"main_state"],
        bump,
    )]
    pub main_state: Account<'info, MainState>,
    #[account(mut)]
    pub depositor: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(
        mut,
        has_one = owner,
    )]
    pub user_deposit: Account<'info, UserDeposit>,
    #[account(
        mut,
        seeds = [b"main_state"],
        bump,
    )]
    pub main_state: Account<'info, MainState>,
    pub owner: Signer<'info>,
}

#[derive(Accounts)]
pub struct StartDrawingPhase<'info> {
    #[account(
        init,
        payer = signer,
        space = DrawingResult::LEN,
        seeds = [
            b"drawing_result", 
            main_state.current_round.to_le_bytes().as_ref(),
        ],
        bump,
    )]
    pub drawing_result: Account<'info, DrawingResult>,
    #[account(
        seeds = [b"main_state"],
        bump
    )]
    pub main_state: Account<'info, MainState>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Drawing<'info> {
    #[account(
        mut,
        seeds = [b"main_state"],
        bump
    )]
    pub main_state: Account<'info, MainState>,
    #[account(
        mut,
        seeds = [
            b"drawing_state",
            main_state.current_round.to_le_bytes().as_ref(),
        ],
        bump,
        constraint = drawing_result.state == DrawingState::Processing,
    )]
    pub drawing_result: Account<'info, DrawingResult>,
    #[account(
        seeds = [(drawing_result.last_processed_slot + 1).to_le_bytes().as_ref()],
        bump,
    )]
    pub user_deposit: Account<'info, UserDeposit>,
}

#[derive(Accounts)]
pub struct ReceiveFromRemainingAccounts {

}

#[account]
pub struct MainState {
    pub current_slot: u64,
    pub current_round: u64,
    pub total_deposit: u64,
    pub switchboard_pubkey: Pubkey,
}

impl MainState {
    pub const LEN: usize = 8 + 8 + 8 + 8 + 32;
}

#[account]
pub struct DrawingResult {
    pub round: u64,
    pub state: DrawingState,
    pub winner: Option<Pubkey>,
    pub random_number: u64,
    pub total_deposit: u64,
    pub last_processed_slot: u64,
    pub finished_timestamp: Option<u64>,
}

impl DrawingResult {
    pub const LEN: usize = 8 + 8 + 1 + 33 + 8 + 8 + 8 + 9;
}

#[derive(Debug, PartialEq, Eq, Clone, AnchorSerialize, AnchorDeserialize)]
pub enum DrawingState {
    Processing,
    Finished,
}

#[account]
pub struct UserDeposit {
    pub slot: u64,
    pub amount: u64,
    pub owner: Pubkey,
}

impl UserDeposit {
    pub const LEN: usize = 8 + 8 + 8 + 32;
}

#[account]
pub struct Treasury {
    pub bags: Vec<Bag>,
}

impl Treasury {
    fn space(vector_capacity: u16) -> usize {
        return 8 + 4 * (Bag::LEN * vector_capacity as usize) 
    }
}

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct Bag {
    owner: Pubkey,
    amount: u64,
}

impl Bag {
    pub const LEN: usize = 32 + 8; 
}
