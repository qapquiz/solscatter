use anchor_lang::prelude::*;

#[event]
pub struct ReceivedVrfEvent {
    pub received_timestamp: i64,
    #[index]
    pub counter: u128,
}