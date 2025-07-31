use crate::{
    instructions::EscrowInstruction,
    make::process_make,
    refund::process_refund,
    take::process_take,
};
use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    pubkey::Pubkey,
};

pub fn process(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = EscrowInstruction::unpack(instruction_data)?;

    match instruction {
        EscrowInstruction::Make(make) => process_make(program_id, accounts, make),
        EscrowInstruction::Take => process_take(program_id, accounts),
        EscrowInstruction::Refund => process_refund(program_id, accounts),
    }
}