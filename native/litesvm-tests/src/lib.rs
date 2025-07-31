#[cfg(test)]
mod minimal_test;

#[cfg(test)]
mod tests {
    use bytemuck::{from_bytes, Pod, Zeroable};
    use litesvm::LiteSVM;
    use solana_sdk::{
        account::Account,
        instruction::{AccountMeta, Instruction},
        signature::{Keypair, Signer},
        message::Message,
        pubkey::Pubkey,
        system_instruction,
        system_program,
        transaction::Transaction,
        program_pack::Pack,
    };
    use spl_associated_token_account::get_associated_token_address;
    use spl_token::state::{Account as TokenAccount, Mint};

    // This should match the program ID from target/deploy/native_escrow-keypair.json
    const ESCROW_PROGRAM_ID: &str = "EZtrKpnfRjtMpFgKdJE4ui6ei1D8zqQZEWqB4a6kMhnF";

    #[repr(C)]
    #[derive(Clone, Copy, Debug, Pod, Zeroable)]
    pub struct Escrow {
        pub seed: u64,
        pub maker: Pubkey,
        pub mint_a: Pubkey,
        pub mint_b: Pubkey,
        pub receive: u64,
        pub bump: u8,
        pub _padding: [u8; 7],
    }

    #[repr(C)]
    #[derive(Clone, Copy, Debug, Pod, Zeroable)]
    pub struct Make {
        pub seed: u64,
        pub amount: u64,
        pub receive: u64,
    }

    fn read_escrow_program() -> Vec<u8> {
        let path = "../../target/deploy/native_escrow.so";
        std::fs::read(path).unwrap_or_else(|e| {
            panic!(
                "Failed to read native_escrow.so from {}. \
                Make sure to run 'cargo build-sbf' from the root directory first. \
                Error: {}",
                path, e
            )
        })
    }

    fn payer_keypair(svm: &mut LiteSVM) -> Keypair {
        // LiteSVM stores an internal airdrop keypair that we need to reconstruct
        // This is a workaround since litesvm doesn't expose the payer directly
        let payer = Keypair::new();
        svm.airdrop(&payer.pubkey(), 100_000_000_000).unwrap();
        payer
    }

    fn create_mint(svm: &mut LiteSVM, mint_authority: &Keypair, decimals: u8) -> Pubkey {
        let mint_keypair = Keypair::new();
        let mint_pubkey = mint_keypair.pubkey();
        let mint_len = Mint::LEN;
        let mint_rent = svm.minimum_balance_for_rent_exemption(mint_len);

        let create_mint_account_ix = system_instruction::create_account(
            &mint_authority.pubkey(),
            &mint_pubkey,
            mint_rent,
            mint_len as u64,
            &spl_token::id(),
        );

        let init_mint_ix = spl_token::instruction::initialize_mint(
            &spl_token::id(),
            &mint_pubkey,
            &mint_authority.pubkey(),
            None,
            decimals,
        )
        .unwrap();

        let msg = Message::new(
            &[create_mint_account_ix, init_mint_ix],
            Some(&mint_authority.pubkey()),
        );

        let tx = Transaction::new(
            &[mint_authority, &mint_keypair],
            msg,
            svm.latest_blockhash(),
        );

        svm.send_transaction(tx).unwrap();
        mint_pubkey
    }

    fn create_token_account(
        svm: &mut LiteSVM,
        mint: &Pubkey,
        owner: &Pubkey,
        payer: &Keypair,
    ) -> Pubkey {
        let ata = get_associated_token_address(owner, mint);

        let create_ata_ix = spl_associated_token_account::instruction::create_associated_token_account(
            &payer.pubkey(),
            owner,
            mint,
            &spl_token::id(),
        );

        let msg = Message::new(&[create_ata_ix], Some(&payer.pubkey()));
        let tx = Transaction::new(&[payer], msg, svm.latest_blockhash());
        svm.send_transaction(tx).unwrap();

        ata
    }

    fn mint_tokens(
        svm: &mut LiteSVM,
        mint: &Pubkey,
        destination: &Pubkey,
        authority: &Keypair,
        amount: u64,
    ) {
        let mint_to_ix = spl_token::instruction::mint_to(
            &spl_token::id(),
            mint,
            destination,
            &authority.pubkey(),
            &[],
            amount,
        )
        .unwrap();

        let msg = Message::new(&[mint_to_ix], Some(&authority.pubkey()));
        let tx = Transaction::new(&[authority], msg, svm.latest_blockhash());
        svm.send_transaction(tx).unwrap();
    }

    #[test]
    fn test_make_escrow() {
        let mut svm = LiteSVM::new();
        let program_id = ESCROW_PROGRAM_ID.parse::<Pubkey>().unwrap();
        
        svm.add_program(program_id, &read_escrow_program());

        let maker = Keypair::new();
        let payer = payer_keypair(&mut svm);

        svm.airdrop(&maker.pubkey(), 10_000_000_000).unwrap();
        svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

        let mint_a = create_mint(&mut svm, &payer, 9);
        let mint_b = create_mint(&mut svm, &payer, 9);

        let maker_ata_a = create_token_account(&mut svm, &mint_a, &maker.pubkey(), &payer);
        let maker_ata_b = create_token_account(&mut svm, &mint_b, &maker.pubkey(), &payer);

        mint_tokens(&mut svm, &mint_a, &maker_ata_a, &payer, 1_000_000_000);

        let seed = 42u64;
        let deposit_amount = 100_000_000u64;
        let receive_amount = 200_000_000u64;

        let (escrow_pda, _bump) = Pubkey::find_program_address(
            &[b"escrow", maker.pubkey().as_ref(), seed.to_le_bytes().as_ref()],
            &program_id,
        );

        let (vault_pda, _vault_bump) = Pubkey::find_program_address(
            &[b"vault", escrow_pda.as_ref()],
            &program_id,
        );

        let make_data = Make {
            seed,
            amount: deposit_amount,
            receive: receive_amount,
        };

        let mut instruction_data = vec![0u8]; // Make instruction
        instruction_data.extend_from_slice(bytemuck::bytes_of(&make_data));
        
        println!("Instruction data length: {}", instruction_data.len());
        println!("Instruction data: {:?}", instruction_data);

        let make_ix = Instruction::new_with_bytes(
            program_id,
            &instruction_data,
            vec![
                AccountMeta::new(maker.pubkey(), true),
                AccountMeta::new(maker_ata_a, false),
                AccountMeta::new_readonly(maker_ata_b, false),
                AccountMeta::new(escrow_pda, false),
                AccountMeta::new(vault_pda, false),
                AccountMeta::new_readonly(mint_a, false),
                AccountMeta::new_readonly(mint_b, false),
                AccountMeta::new_readonly(spl_token::id(), false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
        );

        let msg = Message::new(&[make_ix], Some(&maker.pubkey()));
        let tx = Transaction::new(&[&maker], msg, svm.latest_blockhash());
        
        let result = svm.send_transaction(tx);
        assert!(result.is_ok(), "Make transaction failed: {:?}", result);

        let escrow_account = svm.get_account(&escrow_pda).unwrap();
        assert_eq!(escrow_account.owner, program_id);
        
        let escrow_data: &Escrow = from_bytes(&escrow_account.data);
        assert_eq!(escrow_data.seed, seed);
        assert_eq!(escrow_data.maker, maker.pubkey());
        assert_eq!(escrow_data.mint_a, mint_a);
        assert_eq!(escrow_data.mint_b, mint_b);
        assert_eq!(escrow_data.receive, receive_amount);

        let vault_account = svm.get_account(&vault_pda).unwrap();
        let vault_token_account = TokenAccount::unpack(&vault_account.data).unwrap();
        assert_eq!(vault_token_account.amount, deposit_amount);
        assert_eq!(vault_token_account.mint, mint_a);
    }

    #[test]
    fn test_take_escrow() {
        let mut svm = LiteSVM::new();
        let program_id = ESCROW_PROGRAM_ID.parse::<Pubkey>().unwrap();
        
        svm.add_program(program_id, &read_escrow_program());

        let maker = Keypair::new();
        let taker = Keypair::new();
        let payer = payer_keypair(&mut svm);

        svm.airdrop(&maker.pubkey(), 10_000_000_000).unwrap();
        svm.airdrop(&taker.pubkey(), 10_000_000_000).unwrap();
        svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

        let mint_a = create_mint(&mut svm, &payer, 9);
        let mint_b = create_mint(&mut svm, &payer, 9);

        let maker_ata_a = create_token_account(&mut svm, &mint_a, &maker.pubkey(), &payer);
        let maker_ata_b = create_token_account(&mut svm, &mint_b, &maker.pubkey(), &payer);
        let taker_ata_a = create_token_account(&mut svm, &mint_a, &taker.pubkey(), &payer);
        let taker_ata_b = create_token_account(&mut svm, &mint_b, &taker.pubkey(), &payer);

        mint_tokens(&mut svm, &mint_a, &maker_ata_a, &payer, 1_000_000_000);
        mint_tokens(&mut svm, &mint_b, &taker_ata_b, &payer, 1_000_000_000);

        let seed = 42u64;
        let deposit_amount = 100_000_000u64;
        let receive_amount = 200_000_000u64;

        let (escrow_pda, _bump) = Pubkey::find_program_address(
            &[b"escrow", maker.pubkey().as_ref(), seed.to_le_bytes().as_ref()],
            &program_id,
        );

        let (vault_pda, _vault_bump) = Pubkey::find_program_address(
            &[b"vault", escrow_pda.as_ref()],
            &program_id,
        );

        // First, make the escrow
        let make_data = Make {
            seed,
            amount: deposit_amount,
            receive: receive_amount,
        };

        let mut instruction_data = vec![0u8]; // Make instruction
        instruction_data.extend_from_slice(bytemuck::bytes_of(&make_data));
        
        println!("Instruction data length: {}", instruction_data.len());
        println!("Instruction data: {:?}", instruction_data);

        let make_ix = Instruction::new_with_bytes(
            program_id,
            &instruction_data,
            vec![
                AccountMeta::new(maker.pubkey(), true),
                AccountMeta::new(maker_ata_a, false),
                AccountMeta::new_readonly(maker_ata_b, false),
                AccountMeta::new(escrow_pda, false),
                AccountMeta::new(vault_pda, false),
                AccountMeta::new_readonly(mint_a, false),
                AccountMeta::new_readonly(mint_b, false),
                AccountMeta::new_readonly(spl_token::id(), false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
        );

        let msg = Message::new(&[make_ix], Some(&maker.pubkey()));
        let tx = Transaction::new(&[&maker], msg, svm.latest_blockhash());
        svm.send_transaction(tx).unwrap();

        // Now take the escrow
        let take_ix = Instruction::new_with_bytes(
            program_id,
            &[1u8], // Take instruction
            vec![
                AccountMeta::new(taker.pubkey(), true),
                AccountMeta::new(taker_ata_a, false),
                AccountMeta::new(taker_ata_b, false),
                AccountMeta::new(maker_ata_b, false),
                AccountMeta::new(escrow_pda, false),
                AccountMeta::new(vault_pda, false),
                AccountMeta::new_readonly(spl_token::id(), false),
            ],
        );

        let msg = Message::new(&[take_ix], Some(&taker.pubkey()));
        let tx = Transaction::new(&[&taker], msg, svm.latest_blockhash());
        
        let result = svm.send_transaction(tx);
        assert!(result.is_ok(), "Take transaction failed: {:?}", result);

        // Verify taker received Token A
        let taker_ata_a_account = svm.get_account(&taker_ata_a).unwrap();
        let taker_token_a = TokenAccount::unpack(&taker_ata_a_account.data).unwrap();
        assert_eq!(taker_token_a.amount, deposit_amount);

        // Verify maker received Token B
        let maker_ata_b_account = svm.get_account(&maker_ata_b).unwrap();
        let maker_token_b = TokenAccount::unpack(&maker_ata_b_account.data).unwrap();
        assert_eq!(maker_token_b.amount, receive_amount);

        // Verify escrow and vault are closed
        assert!(svm.get_account(&escrow_pda).is_none());
        assert!(svm.get_account(&vault_pda).is_none());
    }

    #[test]
    fn test_refund_escrow() {
        let mut svm = LiteSVM::new();
        let program_id = ESCROW_PROGRAM_ID.parse::<Pubkey>().unwrap();
        
        svm.add_program(program_id, &read_escrow_program());

        let maker = Keypair::new();
        let payer = payer_keypair(&mut svm);

        svm.airdrop(&maker.pubkey(), 10_000_000_000).unwrap();
        svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

        let mint_a = create_mint(&mut svm, &payer, 9);
        let mint_b = create_mint(&mut svm, &payer, 9);

        let maker_ata_a = create_token_account(&mut svm, &mint_a, &maker.pubkey(), &payer);
        let maker_ata_b = create_token_account(&mut svm, &mint_b, &maker.pubkey(), &payer);

        mint_tokens(&mut svm, &mint_a, &maker_ata_a, &payer, 1_000_000_000);

        let seed = 42u64;
        let deposit_amount = 100_000_000u64;
        let receive_amount = 200_000_000u64;

        let (escrow_pda, _bump) = Pubkey::find_program_address(
            &[b"escrow", maker.pubkey().as_ref(), seed.to_le_bytes().as_ref()],
            &program_id,
        );

        let (vault_pda, _vault_bump) = Pubkey::find_program_address(
            &[b"vault", escrow_pda.as_ref()],
            &program_id,
        );

        // First, make the escrow
        let make_data = Make {
            seed,
            amount: deposit_amount,
            receive: receive_amount,
        };

        let mut instruction_data = vec![0u8]; // Make instruction
        instruction_data.extend_from_slice(bytemuck::bytes_of(&make_data));
        
        println!("Instruction data length: {}", instruction_data.len());
        println!("Instruction data: {:?}", instruction_data);

        let make_ix = Instruction::new_with_bytes(
            program_id,
            &instruction_data,
            vec![
                AccountMeta::new(maker.pubkey(), true),
                AccountMeta::new(maker_ata_a, false),
                AccountMeta::new_readonly(maker_ata_b, false),
                AccountMeta::new(escrow_pda, false),
                AccountMeta::new(vault_pda, false),
                AccountMeta::new_readonly(mint_a, false),
                AccountMeta::new_readonly(mint_b, false),
                AccountMeta::new_readonly(spl_token::id(), false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
        );

        let msg = Message::new(&[make_ix], Some(&maker.pubkey()));
        let tx = Transaction::new(&[&maker], msg, svm.latest_blockhash());
        svm.send_transaction(tx).unwrap();

        // Get initial balance
        let initial_balance = {
            let account = svm.get_account(&maker_ata_a).unwrap();
            TokenAccount::unpack(&account.data).unwrap().amount
        };

        // Now refund the escrow
        let refund_ix = Instruction::new_with_bytes(
            program_id,
            &[2u8], // Refund instruction
            vec![
                AccountMeta::new(maker.pubkey(), true),
                AccountMeta::new(maker_ata_a, false),
                AccountMeta::new(escrow_pda, false),
                AccountMeta::new(vault_pda, false),
                AccountMeta::new_readonly(spl_token::id(), false),
            ],
        );

        let msg = Message::new(&[refund_ix], Some(&maker.pubkey()));
        let tx = Transaction::new(&[&maker], msg, svm.latest_blockhash());
        
        let result = svm.send_transaction(tx);
        assert!(result.is_ok(), "Refund transaction failed: {:?}", result);

        // Verify maker got tokens back
        let final_account = svm.get_account(&maker_ata_a).unwrap();
        let final_balance = TokenAccount::unpack(&final_account.data).unwrap().amount;
        assert_eq!(final_balance - initial_balance, deposit_amount);

        // Verify escrow and vault are closed
        assert!(svm.get_account(&escrow_pda).is_none());
        assert!(svm.get_account(&vault_pda).is_none());
    }
}