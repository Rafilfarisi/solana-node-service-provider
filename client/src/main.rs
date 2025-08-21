use anyhow::{anyhow, Result};
use base64::Engine; // for .encode()
use dotenv::dotenv;
use reqwest::blocking::Client as HttpClient;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde_json::{json, Value};
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::system_instruction;
use solana_sdk::transaction::Transaction;
use solana_sdk::pubkey::Pubkey;
use rand::seq::SliceRandom;
use std::str::FromStr;
use std::env;
use std::thread;
use std::time::Duration;

fn main() -> Result<()> {
	// Load .env
	dotenv().ok();

	// Env config
	let rpc_endpoint = env::var("RPC_ENDPOINT").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string());
	let service_url = env::var("SERVICE_URL").unwrap_or_else(|_| "https://ny.mainnet.block-engine.jito.wtf/api/v1/transactions".to_string());
	let service_bearer = env::var("SERVICE_BEARER").ok(); // optional
	let mk1 = env::var("mk1").map_err(|_| anyhow!("Missing mk1 in .env"))?;
	let mk2 = env::var("mk2").map_err(|_| anyhow!("Missing mk2 in .env"))?;

	// Keys
	let sender = parse_keypair(&mk1)?;
	let recipient = parse_keypair(&mk2)?;
	println!("Sender: {}", sender.pubkey());
	println!("Recipient: {}", recipient.pubkey());

	// Build transfer instructions (1000 lamports)
	let rpc = RpcClient::new_with_commitment(rpc_endpoint.clone(), CommitmentConfig::confirmed());
	let latest_blockhash = rpc.get_latest_blockhash()?;
	let ix = system_instruction::transfer(&sender.pubkey(), &recipient.pubkey(), 1000);

	// tip_ix: send SOL from sender to a random one of tip accounts
	const TIP_ACCOUNTS: [&str; 4] = [
		"3DpmFFACtWVbkmuMEE6SfVC3JoqHnZmFe5KeBV7Ux8M9",
		"Ex2kh7BnjbUdD6HFXrtMPq2QVPgPNxxo1y1aV17zcuXV",
		"EoVbZM9raES9obgXtsMpEBeDPLiK7S8Y16z3uekpQLvm",
		"GifL6PrDJTKSmucMhFJ8vdgnYNtaiavEGZyv2GLnsUW2",
	];
	 let mut rng = rand::thread_rng();
	 let tip_lamports: u64 = 10000000; // 0.000001 SOL
	// Send multiple requests at 2 per second rate
	let requests_per_second = 2;
	let delay_ms = 1000 / requests_per_second;
	//println!("Sending {} requests per second ({}ms delay between requests)", requests_per_second, delay_ms);

	for i in 1..=1 { // Send 10 requests total
	 	println!("Sending request #{}", i);
		
		let latest_blockhash = rpc.get_latest_blockhash()?;
		let ix = system_instruction::transfer(&sender.pubkey(), &recipient.pubkey(), 1000);
		let tip_str = TIP_ACCOUNTS.choose(&mut rng).ok_or_else(|| anyhow!("No tip accounts configured"))?;
		let tip_pubkey = Pubkey::from_str(tip_str)?;
		let tip_ix = system_instruction::transfer(&sender.pubkey(), &tip_pubkey, tip_lamports);
		let tx = Transaction::new_signed_with_payer(&[ix, tip_ix], Some(&sender.pubkey()), &[&sender], latest_blockhash);
		//let tx = Transaction::new_signed_with_payer(&[ix], Some(&sender.pubkey()), &[&sender], latest_blockhash);
		send_transaction(&tx, &service_url, &service_bearer)?;
		if i < 10 {
			thread::sleep(Duration::from_millis(delay_ms));
		}
	}

	Ok(())
}

fn send_transaction(tx: &Transaction, service_url: &str, service_bearer: &Option<String>) -> Result<()> {
	// Encode transaction to base64
	let tx_bytes = bincode::serialize(tx)?;
	let tx_base64 = base64::engine::general_purpose::STANDARD.encode(tx_bytes);

	// JSON-RPC payload for Service Block Engine
	let payload = json!({
		"jsonrpc": "2.0",
		"id": 1,
		"method": "sendTransaction",
		"params": [tx_base64, {"encoding": "base64"}]
	});

	let http = HttpClient::new();
	let mut req = http.post(service_url).header(CONTENT_TYPE, "application/json").json(&payload);
	if let Some(token) = service_bearer.as_ref() {
		req = req.header(AUTHORIZATION, format!("Bearer {}", token));
	}
	let resp = req.send()?;
	let status = resp.status();
	let body = resp.text()?;
	if !status.is_success() {
		println!("Service returned error {}: {}", status, body);
		return Err(anyhow!("Service error"));
	}

	let v: Value = serde_json::from_str(&body).unwrap_or_else(|_| json!({"raw": body}));
	if let Some(sig) = v.get("result").and_then(|s| s.as_str()) {
		println!("Service signature: {}", sig);
	} else {
		println!("Service response: {}", body);
	}
	Ok(())
}

fn parse_keypair(input: &str) -> Result<Keypair> {
	// Try JSON array format first (common for Solana)
	if let Ok(bytes) = serde_json::from_str::<Vec<u8>>(input) {
		return Ok(Keypair::from_bytes(&bytes)?);
	}
	// Try base58
	if let Ok(bytes) = bs58::decode(input).into_vec() {
		if let Ok(kp) = Keypair::from_bytes(&bytes) {
			return Ok(kp);
		}
	}
	// Try hex
	if let Ok(bytes) = hex::decode(input) {
		if let Ok(kp) = Keypair::from_bytes(&bytes) {
			return Ok(kp);
		}
	}
	Err(anyhow!("Unsupported private key format for: {}", &mask(input)))
}

fn mask(s: &str) -> String {
	let len = s.len();
	if len <= 8 { return "****".to_string(); }
	format!("{}****{}", &s[..4], &s[len-4..])
}