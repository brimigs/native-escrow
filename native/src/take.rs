use crate::state::Escrow;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
};
use spl_token::state::Account as TokenAccount;

pub fn process_take(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    
    let taker = next_account_info(account_info_iter)?;
    let taker_token_a = next_account_info(account_info_iter)?;
    let taker_token_b = next_account_info(account_info_iter)?;
    let maker_token_b = next_account_info(account_info_iter)?;
    let escrow = next_account_info(account_info_iter)?;
    let vault = next_account_info(account_info_iter)?;
    let token_program = next_account_info(account_info_iter)?;

    if !taker.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let escrow_data = escrow.try_borrow_data()?;
    let escrow_state = bytemuck::from_bytes::<Escrow>(&escrow_data);

    let (vault_pda, vault_bump) = Pubkey::find_program_address(
        &[b"vault", escrow.key.as_ref()],
        program_id,
    );

    if vault_pda != *vault.key {
        return Err(ProgramError::InvalidAccountData);
    }

    let taker_token_b_info = TokenAccount::unpack(&taker_token_b.data.borrow())?;
    if taker_token_b_info.mint != escrow_state.mint_b {
        return Err(ProgramError::InvalidAccountData);
    }

    if taker_token_b_info.amount < escrow_state.receive {
        return Err(ProgramError::InsufficientFunds);
    }

    let vault_token_info = TokenAccount::unpack(&vault.data.borrow())?;
    let transfer_amount = vault_token_info.amount;

    invoke_signed(
        &spl_token::instruction::transfer(
            token_program.key,
            vault.key,
            taker_token_a.key,
            &vault_pda,
            &[],
            transfer_amount,
        )?,
        &[vault.clone(), taker_token_a.clone(), token_program.clone()],
        &[&[b"vault", escrow.key.as_ref(), &[vault_bump]]],
    )?;

    invoke(
        &spl_token::instruction::transfer(
            token_program.key,
            taker_token_b.key,
            maker_token_b.key,
            taker.key,
            &[],
            escrow_state.receive,
        )?,
        &[taker_token_b.clone(), maker_token_b.clone(), taker.clone(), token_program.clone()],
    )?;

    invoke_signed(
        &spl_token::instruction::close_account(
            token_program.key,
            vault.key,
            taker.key,
            &vault_pda,
            &[],
        )?,
        &[vault.clone(), taker.clone(), token_program.clone()],
        &[&[b"vault", escrow.key.as_ref(), &[vault_bump]]],
    )?;

    **taker.try_borrow_mut_lamports()? = taker
        .lamports()
        .checked_add(escrow.lamports())
        .ok_or(ProgramError::ArithmeticOverflow)?;
    **escrow.try_borrow_mut_lamports()? = 0;
    escrow.try_borrow_mut_data()?.fill(0);

    Ok(())
}