use anyhow::{anyhow, Result};
use dotenv::dotenv;
use reqwest::blocking::Client as HttpClient;
use serde_json::{json, Value};
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::system_instruction;
use solana_sdk::transaction::Transaction;
use std::env;
use base64::Engine; // for .encode()
use once_cell::sync::OnceCell;
use crate::jito::Jito;

pub static JITO_CLIENT: OnceCell<Jito> = OnceCell::const_new();

fn main() -> Result<()> {
	let jito = match JITO_CLIENT.get() {
		Some(client) => client,
		None => {
			println!("Error: Jito client not initialized");
			return;
		}
	};
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