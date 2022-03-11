use anchor_lang::prelude::*;

#[account(zero_copy)]
pub struct VrfClient {
    pub authority: Pubkey,
    pub max_result: u64,
    pub vrf: Pubkey,
    pub result_buffer: [u8; 32],
    pub result: u128,
    pub last_timestamp: i64,
}

impl Default for VrfClient {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}