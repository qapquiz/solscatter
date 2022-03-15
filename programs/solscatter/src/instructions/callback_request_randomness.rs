use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct CallbackRequestRandomness {}

pub fn handler(_ctx: Context<CallbackRequestRandomness>) -> Result<()> {
    Ok(())
}