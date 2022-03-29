use anchor_lang::prelude::*;

#[account(zero_copy)]
pub struct VrfClientState {
    pub authority: Pubkey,
    pub max_result: u64,
    pub vrf: Pubkey,
    pub result_buffer: [u8; 32],
    pub result: u128,
    pub last_timestamp: i64,
}

impl VrfClientState {
    pub const LEN: usize = 8 + 32 + 8 + 32 + (1 * 32) + 16 + 8;
}

impl Default for VrfClientState {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}
