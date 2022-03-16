use anchor_lang::prelude::*;
use switchboard_v2::VrfAccountData;

use crate::{state::VrfClientState, events::ReceivedVrfEvent};

#[derive(Accounts)]
pub struct CallbackRequestRandomness<'info> {
    #[account(mut)]
    pub state: AccountLoader<'info, VrfClientState>,
    /// CHECK: this is vrf acocunt data
    pub vrf: AccountInfo<'info>,
}

pub fn handler(ctx: Context<CallbackRequestRandomness>) -> Result<()> {
    let vrf_account_info = &ctx.accounts.vrf;
    let vrf = VrfAccountData::new(vrf_account_info)?;
    let result_buffer = vrf.get_result()?;
    if result_buffer == [0u8; 32] {
        msg!("vrf buffer empty");
        return Ok(());
    }

    let state = &mut ctx.accounts.state.load_mut()?;
    let max_result = state.max_result;
    if result_buffer == state.result_buffer {
        msg!("existing result_buffer");
        return Ok(());
    }

    msg!("Result buffer is {:?}", result_buffer);
    let value: &[u128] = bytemuck::cast_slice(&result_buffer[..]);
    msg!("u128 buffer {:?}", value);
    let result = value[0] % max_result as u128;
    msg!("Crurrent VRF Value [0 - {}]) = {}!", max_result, result);

    let clock = Clock::get().unwrap();

    if state.result != result {
        state.result = result;
        state.result_buffer = result_buffer;
        state.last_timestamp = clock.unix_timestamp; 
    }

    emit!(ReceivedVrfEvent {
        received_timestamp: clock.unix_timestamp,
        counter: vrf.counter,
    });

    Ok(())
}