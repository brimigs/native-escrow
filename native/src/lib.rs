mod error;
mod instructions;
mod make;
mod processor;
mod refund;
mod state;
mod take;

use solana_program::{
    account_info::AccountInfo,
    entrypoint,
    entrypoint::ProgramResult,
    pubkey::Pubkey,
};

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    processor::process(program_id, accounts, instruction_data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{instructions::{EscrowInstruction, Make}, state::Escrow};
    use mollusk_svm::{Mollusk, MolluskError};
    use solana_sdk::{
        account::{AccountSharedData, WritableAccount},
        instruction::{AccountMeta, Instruction},
        program_pack::Pack,
        pubkey::Pubkey,
        system_program,
    };
    use spl_token::state::{Account as TokenAccount, Mint};

    #[test]
    fn test_make_instruction() {
        let program_id = Pubkey::new_unique();
        let mut mollusk = Mollusk::new(&program_id, "../../target/deploy/native_escrow");

        let maker = Pubkey::new_unique();
        let mint_a = Pubkey::new_unique();
        let mint_b = Pubkey::new_unique();
        let seed = 42u64;
        let amount = 1000u64;
        let receive = 2000u64;

        let (escrow_pda, _bump) = Pubkey::find_program_address(
            &[b"escrow", maker.as_ref(), seed.to_le_bytes().as_ref()],
            &program_id,
        );

        let (vault_pda, _vault_bump) = Pubkey::find_program_address(
            &[b"vault", escrow_pda.as_ref()],
            &program_id,
        );

        let mut maker_token_a_data = vec![0u8; TokenAccount::LEN];
        let maker_token_a_state = TokenAccount {
            mint: mint_a,
            owner: maker,
            amount: 5000,
            state: spl_token::state::AccountState::Initialized,
            ..TokenAccount::default()
        };
        TokenAccount::pack(maker_token_a_state, &mut maker_token_a_data).unwrap();

        let mut vault_data = vec![0u8; TokenAccount::LEN];
        let vault_state = TokenAccount {
            mint: mint_a,
            owner: vault_pda,
            state: spl_token::state::AccountState::Initialized,
            ..TokenAccount::default()
        };
        TokenAccount::pack(vault_state, &mut vault_data).unwrap();

        let maker_token_a = Pubkey::new_unique();
        let maker_token_b = Pubkey::new_unique();

        let make_data = Make { seed, amount, receive };
        let mut instruction_data = vec![0u8];
        instruction_data.extend_from_slice(bytemuck::bytes_of(&make_data));

        let instruction = Instruction::new_with_bytes(
            program_id,
            &instruction_data,
            vec![
                AccountMeta::new(maker, true),
                AccountMeta::new(maker_token_a, false),
                AccountMeta::new_readonly(maker_token_b, false),
                AccountMeta::new(escrow_pda, false),
                AccountMeta::new(vault_pda, false),
                AccountMeta::new_readonly(mint_a, false),
                AccountMeta::new_readonly(mint_b, false),
                AccountMeta::new_readonly(spl_token::id(), false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
        );

        let mut maker_account = AccountSharedData::new(10_000_000_000, 0, &system_program::id());
        let mut escrow_account = AccountSharedData::new(0, 0, &system_program::id());
        let mut maker_token_a_account = AccountSharedData::new(
            2_039_280,
            TokenAccount::LEN,
            &spl_token::id(),
        );
        maker_token_a_account.set_data(maker_token_a_data);

        let mut vault_account = AccountSharedData::new(
            2_039_280,
            TokenAccount::LEN,
            &spl_token::id(),
        );
        vault_account.set_data(vault_data);

        mollusk.process_and_validate_instruction(
            &instruction,
            &vec![
                (maker, maker_account),
                (maker_token_a, maker_token_a_account),
                (maker_token_b, AccountSharedData::default()),
                (escrow_pda, escrow_account),
                (vault_pda, vault_account),
                (mint_a, AccountSharedData::default()),
                (mint_b, AccountSharedData::default()),
                (spl_token::id(), AccountSharedData::default()),
                (system_program::id(), AccountSharedData::default()),
            ],
            &vec![],
        );
    }
}