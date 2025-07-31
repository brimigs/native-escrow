use litesvm::LiteSVM;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    signature::{Keypair, Signer},
    message::Message,
    pubkey::Pubkey,
    transaction::Transaction,
};

#[test]
fn test_program_loads() {
    let mut svm = LiteSVM::new();
    let program_id = "EZtrKpnfRjtMpFgKdJE4ui6ei1D8zqQZEWqB4a6kMhnF".parse::<Pubkey>().unwrap();
    
    let program_bytes = std::fs::read("../../target/deploy/native_escrow.so").unwrap();
    svm.add_program(program_id, &program_bytes);
    
    // Create a simple invalid instruction to see if the program runs at all
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 1_000_000_000).unwrap();
    
    let ix = Instruction::new_with_bytes(
        program_id,
        &[99], // Invalid instruction
        vec![AccountMeta::new(payer.pubkey(), true)],
    );
    
    let msg = Message::new(&[ix], Some(&payer.pubkey()));
    let tx = Transaction::new(&[&payer], msg, svm.latest_blockhash());
    
    match svm.send_transaction(tx) {
        Ok(_) => panic!("Expected error"),
        Err(e) => {
            println!("Got expected error: {:?}", e);
        }
    }
}