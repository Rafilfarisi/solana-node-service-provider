use solana_sdk::{
    transaction::Transaction,
    system_instruction,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};
use std::str::FromStr;

fn main() {
    println!("Solana Transaction Service Example");
    println!("==================================");
    
    // Create test keypairs
    let from_keypair = Keypair::new();
    let to_pubkey = Pubkey::new_unique();
    let tip_account = Pubkey::from_str("11111111111111111111111111111111").unwrap();
    
    println!("From account: {}", from_keypair.pubkey());
    println!("To account: {}", to_pubkey);
    println!("Tip account: {}", tip_account);
    
    // Create a transfer instruction
    let transfer_instruction = system_instruction::transfer(
        &from_keypair.pubkey(),
        &to_pubkey,
        1000000, // 0.001 SOL
    );
    
    // Create a tip instruction
    let tip_instruction = system_instruction::transfer(
        &from_keypair.pubkey(),
        &tip_account,
        1000000, // 0.001 SOL tip
    );
    
    // Create transaction
    let transaction = Transaction::new_signed_with_payer(
        &[transfer_instruction, tip_instruction],
        Some(&from_keypair.pubkey()),
        &[&from_keypair],
        solana_sdk::hash::Hash::default(),
    );
    
    // Encode transaction
    let encoded_transaction = base64::encode(bincode::serialize(&transaction).unwrap());
    
    println!("\nEncoded transaction:");
    println!("{}", encoded_transaction);
    
    println!("\nTo test the service, send a POST request to:");
    println!("http://localhost:3000/simulate");
    println!("\nWith the following JSON body:");
    println!("{{");
    println!("  \"transaction\": \"{}\",", encoded_transaction);
    println!("  \"tip_account\": \"{}\",", tip_account);
    println!("  \"minimum_tip_amount\": 0.001");
    println!("}}");
    
    println!("\nOr use curl:");
    println!("curl -X POST http://localhost:3000/simulate \\");
    println!("  -H \"Content-Type: application/json\" \\");
    println!("  -d '{{");
    println!("    \"transaction\": \"{}\",", encoded_transaction);
    println!("    \"tip_account\": \"{}\",", tip_account);
    println!("    \"minimum_tip_amount\": 0.001");
    println!("  }}'");
}
