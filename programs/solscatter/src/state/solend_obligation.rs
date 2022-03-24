use std::{io::Write, ops::Deref};

use anchor_lang::prelude::*;
use solana_program::{program_pack::Pack, pubkey};
use spl_token_lending::state::Obligation;

#[derive(Clone)]
pub struct SolendObligation(Obligation);

impl anchor_lang::AccountDeserialize for SolendObligation {
    fn try_deserialize(buf: &mut &[u8]) -> Result<Self> {
        SolendObligation::try_deserialize_unchecked(buf)
    }

    fn try_deserialize_unchecked(buf: &mut &[u8]) -> Result<Self> {
        Obligation::unpack(buf)
            .map(SolendObligation)
            .map_err(Into::into)
    }
}

impl anchor_lang::AccountSerialize for SolendObligation {
    fn try_serialize<W: Write>(&self, _writer: &mut W) -> Result<()> {
        Ok(())
    }
}

impl anchor_lang::Owner for SolendObligation {
    fn owner() -> Pubkey {
        pubkey!("ALend7Ketfx5bxh6ghsCDXAoDrhvEmsXT3cynB6aPLgx")
    }
}

impl Deref for SolendObligation {
    type Target = Obligation;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
