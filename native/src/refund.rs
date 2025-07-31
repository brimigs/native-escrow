use crate::state::Escrow;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program::{invoke_signed},
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
};
use spl_token::state::Account as TokenAccount;

pub fn process_refund(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    
    let maker = next_account_info(account_info_iter)?;
    let maker_token_a = next_account_info(account_info_iter)?;
    let escrow = next_account_info(account_info_iter)?;
    let vault = next_account_info(account_info_iter)?;
    let token_program = next_account_info(account_info_iter)?;

    if !maker.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let escrow_data = escrow.try_borrow_data()?;
    let escrow_state = bytemuck::from_bytes::<Escrow>(&escrow_data);

    if escrow_state.maker != *maker.key {
        return Err(ProgramError::InvalidAccountData);
    }

    let (vault_pda, vault_bump) = Pubkey::find_program_address(
        &[b"vault", escrow.key.as_ref()],
        program_id,
    );

    if vault_pda != *vault.key {
        return Err(ProgramError::InvalidAccountData);
    }

    let vault_token_info = TokenAccount::unpack(&vault.data.borrow())?;
    let refund_amount = vault_token_info.amount;

    invoke_signed(
        &spl_token::instruction::transfer(
            token_program.key,
            vault.key,
            maker_token_a.key,
            &vault_pda,
            &[],
            refund_amount,
        )?,
        &[vault.clone(), maker_token_a.clone(), token_program.clone()],
        &[&[b"vault", escrow.key.as_ref(), &[vault_bump]]],
    )?;

    invoke_signed(
        &spl_token::instruction::close_account(
            token_program.key,
            vault.key,
            maker.key,
            &vault_pda,
            &[],
        )?,
        &[vault.clone(), maker.clone(), token_program.clone()],
        &[&[b"vault", escrow.key.as_ref(), &[vault_bump]]],
    )?;

    **maker.try_borrow_mut_lamports()? = maker
        .lamports()
        .checked_add(escrow.lamports())
        .ok_or(ProgramError::ArithmeticOverflow)?;
    **escrow.try_borrow_mut_lamports()? = 0;
    escrow.try_borrow_mut_data()?.fill(0);

    Ok(())
}