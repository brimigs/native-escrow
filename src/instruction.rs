use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program_error::ProgramError;

use crate::error::EscrowError::InvalidInstruction;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct InitEscrowArgs {
    pub amount: u64,
}

pub enum EscrowInstruction {
    InitEscrow { amount: u64 },
    Exchange { amount: u64 },
    Cancel,
}

impl EscrowInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;

        Ok(match tag {
            0 => {
                let args = InitEscrowArgs::try_from_slice(rest)?;
                Self::InitEscrow { amount: args.amount }
            }
            1 => {
                let amount = u64::from_le_bytes(
                    rest.get(..8)
                        .and_then(|slice| slice.try_into().ok())
                        .ok_or(InvalidInstruction)?,
                );
                Self::Exchange { amount }
            }
            2 => Self::Cancel,
            _ => return Err(InvalidInstruction.into()),
        })
    }
}