use crate::error::EscrowError;
use bytemuck::{Pod, Zeroable};
use solana_program::program_error::ProgramError;

#[derive(Debug, Clone, Copy)]
pub enum EscrowInstruction {
    Make(Make),
    Take,
    Refund,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Make {
    pub seed: u64,
    pub amount: u64,
    pub receive: u64,
}

impl EscrowInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (&variant, rest) = input
            .split_first()
            .ok_or(EscrowError::InvalidInstruction)?;

        Ok(match variant {
            0 => {
                let make = Make::try_from(rest)?;
                Self::Make(make)
            }
            1 => Self::Take,
            2 => Self::Refund,
            _ => return Err(EscrowError::InvalidInstruction.into()),
        })
    }
}

impl TryFrom<&[u8]> for Make {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        bytemuck::try_from_bytes(data)
            .map(|&make| make)
            .map_err(|_| EscrowError::InvalidInstruction.into())
    }
}