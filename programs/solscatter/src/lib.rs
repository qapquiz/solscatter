mod error;
mod events;
mod instructions;
mod state;
mod seed;
mod duration;

use anchor_lang::prelude::*;
use instructions::*;

declare_id!("FWjLfARtqXmqfFejHLwEZqCxQGiTwZ6H4mTWjbZjeTMX");

#[program]
pub mod solscatter {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        instructions::initialize::handler(ctx)
    }

    #[access_control(ctx.accounts.validate(&ctx))]
    pub fn initialize_vrf(ctx: Context<InitializeVrf>) -> Result<()> {
        instructions::initialize_vrf::handler(ctx)
    }

    pub fn initialize_quarry(ctx: Context<InitializeQuarry>) -> Result<()> {
        instructions::initialize_quarry::handler(ctx)
    }

    pub fn callback_request_randomness(ctx: Context<CallbackRequestRandomness>) -> Result<()> {
        instructions::callback_request_randomness::handler(ctx)
    }

    pub fn request_randomness(ctx: Context<RequestRanmdomness>, params: RequestRandomnessParams) -> Result<()> {
        RequestRanmdomness::handler(&ctx, &params)
    }

    pub fn deposit_initialize(ctx: Context<DepositInitialize>, slot_number: u64) -> Result<()> {
        instructions::deposit_initialize::handler(ctx, slot_number)
    }

    pub fn deposit(ctx: Context<Deposit>, params: DepositParams) -> Result<()> {
        instructions::deposit::handler(ctx, params)
    }

    pub fn start_drawing_phase(ctx: Context<StartDrawingPhase>, number_of_rewards: u8, random_numbers: Vec<u64>) -> Result<()> {
        instructions::start_drawing_phase::handler(ctx, number_of_rewards, random_numbers)
    }

    pub fn drawing(ctx: Context<Drawing>, processing_slot: u64) -> Result<()> {
        instructions::drawing::handler(ctx, processing_slot)
    }
}