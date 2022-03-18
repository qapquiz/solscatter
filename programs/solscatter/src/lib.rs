mod error;
mod events;
mod instructions;
mod state;

use anchor_lang::prelude::*;
use instructions::*;

declare_id!("DFyrxbvQJANhzH8Q6xLYFfEVKRyF6HzhVeWKGYjGzCfS");

pub const STATE_SEED: &[u8] = b"STATE";
pub const MAIN_STATE_SEED: &[u8] = b"main_state";
pub const PLATFORM_AUTHORITY_SEED: &[u8] = b"PLATFORM_AUTHORITY";

#[program]
pub mod solscatter {
    use super::*;

    #[access_control(ctx.accounts.validate(&ctx))]
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        instructions::initialize::handler(ctx)
    }

    pub fn callback_request_randomness(ctx: Context<CallbackRequestRandomness>) -> Result<()> {
        instructions::callback_request_randomness::handler(ctx)
    }

    #[access_control(ctx.accounts.validate(&ctx, &params))]
    pub fn request_randomness(ctx: Context<RequestRanmdomness>, params: RequestRandomnessParams) -> Result<()> {
        RequestRanmdomness::handler(&ctx, &params)
    }

    pub fn deposit_initialize(ctx: Context<DepositInitialize>) -> Result<()> {
        instructions::deposit_initialize::handler(ctx)
    }

    pub fn deposit(ctx: Context<Deposit>, params: DepositParams) -> Result<()> {
        instructions::deposit::handler(ctx, params)
    }

    pub fn start_drawing_phase(ctx: Context<StartDrawingPhase>, number_of_rewards: u8, random_numbers: Vec<u64>) -> Result<()> {
        instructions::start_drawing_phase::handler(ctx, number_of_rewards, random_numbers)
    }

    pub fn drawing(ctx: Context<Drawing>) -> Result<()> {
        instructions::drawing::handler(ctx)
    }
}
