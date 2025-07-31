use crate::{instructions::Make, state::Escrow};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};
use spl_token::state::Account as TokenAccount;

pub fn process_make(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    make: Make,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    
    let maker = next_account_info(account_info_iter)?;
    let maker_token_a = next_account_info(account_info_iter)?;
    let _maker_token_b = next_account_info(account_info_iter)?;
    let escrow = next_account_info(account_info_iter)?;
    let vault = next_account_info(account_info_iter)?;
    let mint_a = next_account_info(account_info_iter)?;
    let mint_b = next_account_info(account_info_iter)?;
    let token_program = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;

    if !maker.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let (escrow_pda, bump) = Pubkey::find_program_address(
        &[b"escrow", maker.key.as_ref(), make.seed.to_le_bytes().as_ref()],
        program_id,
    );

    if escrow_pda != *escrow.key {
        return Err(ProgramError::InvalidAccountData);
    }

    let rent = Rent::get()?;
    let escrow_lamports = rent.minimum_balance(Escrow::LEN);

    invoke_signed(
        &system_instruction::create_account(
            maker.key,
            escrow.key,
            escrow_lamports,
            Escrow::LEN as u64,
            program_id,
        ),
        &[maker.clone(), escrow.clone(), system_program.clone()],
        &[&[b"escrow", maker.key.as_ref(), make.seed.to_le_bytes().as_ref(), &[bump]]],
    )?;

    let mut escrow_data = escrow.try_borrow_mut_data()?;
    let escrow_state = bytemuck::from_bytes_mut::<Escrow>(&mut escrow_data);
    escrow_state.init(make.seed, *maker.key, *mint_a.key, *mint_b.key, make.receive, bump);

    let (vault_pda, _vault_bump) = Pubkey::find_program_address(
        &[b"vault", escrow.key.as_ref()],
        program_id,
    );

    if vault_pda != *vault.key {
        return Err(ProgramError::InvalidAccountData);
    }

    let token_account_info = TokenAccount::unpack(&maker_token_a.data.borrow())?;
    if token_account_info.mint != *mint_a.key {
        return Err(ProgramError::InvalidAccountData);
    }

    if token_account_info.amount < make.amount {
        return Err(ProgramError::InsufficientFunds);
    }

    invoke(
        &spl_token::instruction::transfer(
            token_program.key,
            maker_token_a.key,
            vault.key,
            maker.key,
            &[],
            make.amount,
        )?,
        &[maker_token_a.clone(), vault.clone(), maker.clone(), token_program.clone()],
    )?;

    Ok(())
}