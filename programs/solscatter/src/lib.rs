mod error;
mod instructions;
mod state;

use anchor_lang::prelude::*;
use instructions::*;

declare_id!("HXPrjwxnsK6PAw6N528qec5xR4WDC4WMKzXFBVCcxRM6");

#[program]
pub mod solscatter {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        instructions::initialize::handler(ctx)
    }

    pub fn deposit_initialize(ctx: Context<DepositInitialize>) -> Result<()> {
        instructions::deposit_initialize::handler(ctx)
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        instructions::deposit::handler(ctx, amount)
    }

    pub fn start_drawing_phase(ctx: Context<StartDrawingPhase>, number_of_rewards: u8, random_numbers: Vec<u64>) -> Result<()> {
        instructions::start_drawing_phase::handler(ctx, number_of_rewards, random_numbers)
    }

    pub fn drawing(ctx: Context<Drawing>) -> Result<()> {
        instructions::drawing::handler(ctx)
    }
}
