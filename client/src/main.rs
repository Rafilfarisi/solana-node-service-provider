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
use std::env;

fn main() -> Result<()> {
	// Load .env
	dotenv().ok();

	// Env config
	let rpc_endpoint = env::var("RPC_ENDPOINT").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string());
	let jito_url = env::var("JITO_URL").unwrap_or_else(|_| "https://ny.mainnet.block-engine.jito.wtf/api/v1/transactions".to_string());
	let jito_bearer = env::var("JITO_BEARER").ok(); // optional
	let mk1 = env::var("mk1").map_err(|_| anyhow!("Missing mk1 in .env"))?;
	let mk2 = env::var("mk2").map_err(|_| anyhow!("Missing mk2 in .env"))?;

	// Keys
	let sender = parse_keypair(&mk1)?;
	let recipient = parse_keypair(&mk2)?;
	println!("Sender: {}", sender.pubkey());
	println!("Recipient: {}", recipient.pubkey());

	// Build transfer transaction (1000 lamports)
	let rpc = RpcClient::new_with_commitment(rpc_endpoint.clone(), CommitmentConfig::confirmed());
	let latest_blockhash = rpc.get_latest_blockhash()?;
	let ix = system_instruction::transfer(&sender.pubkey(), &recipient.pubkey(), 1000);
	let tx = Transaction::new_signed_with_payer(&[ix], Some(&sender.pubkey()), &[&sender], latest_blockhash);

	// Encode transaction to base64
	let tx_bytes = bincode::serialize(&tx)?;
	let tx_base64 = base64::engine::general_purpose::STANDARD.encode(tx_bytes);

	// JSON-RPC payload for Jito Block Engine
	let payload = json!({
		"jsonrpc": "2.0",
		"id": 1,
		"method": "sendTransaction",
		"params": [tx_base64, {"encoding": "base64"}]
	});

	let http = HttpClient::new();
	let mut req = http.post(jito_url).header(CONTENT_TYPE, "application/json").json(&payload);
	if let Some(token) = jito_bearer.as_ref() {
		req = req.header(AUTHORIZATION, format!("Bearer {}", token));
	}
	let resp = req.send()?;
	let status = resp.status();
	let body = resp.text()?;
	if !status.is_success() {
		println!("Jito returned error {}: {}", status, body);
		return Err(anyhow!("jito error"));
	}

	let v: Value = serde_json::from_str(&body).unwrap_or_else(|_| json!({"raw": body}));
	if let Some(sig) = v.get("result").and_then(|s| s.as_str()) {
		println!("Jito signature: {}", sig);
	} else {
		println!("Jito response: {}", body);
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