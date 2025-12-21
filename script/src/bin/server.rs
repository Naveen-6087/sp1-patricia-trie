//! Oracle Server for SP1 Storage Proofs
//!
//! This server mimics a "Relayer" or "Oracle" that:
//! 1. Listens to a Source Chain (via RPC).
//! 2. Fetches a Storage Proof for a specific Account and Slot.
//! 3. Generates a ZK Proof (using SP1) that the storage value is correct.
//! 4. Submits the proof/update to a Destination Chain.
//!
//! Usage:
//! RUST_LOG=info cargo run --bin server

use alloy::providers::{Provider, ProviderBuilder};
use alloy::rpc::types::{BlockId, BlockNumberOrTag};
use alloy::primitives::{Address, B256, keccak256};
use sp1_sdk::{include_elf, ProverClient, SP1Stdin, HashableKey};
use std::env;
use std::str::FromStr;
use std::time::Duration;
use tokio::time::sleep;
use mpt_lib::{MPTProofInput, MPTVerificationResult};
use url::Url;

/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM.
pub const MPT_ELF: &[u8] = include_elf!("mpt-program");

#[derive(Clone)]
struct OracleConfig {
    source_rpc_url: Url,
    target_contract_address: Address,
    target_storage_slot: B256,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup logging
    sp1_sdk::utils::setup_logger();
    dotenv::dotenv().ok();

    // -- Configuration (Mock values for demo if env vars missing) --
    let source_rpc_url = env::var("SOURCE_RPC_URL")
        .unwrap_or_else(|_| "https://eth-mainnet.g.alchemy.com/v2/demo".to_string());
    
    // Default: Vitalik's address, and some slot
    let target_contract_addr_str = env::var("TARGET_CONTRACT")
        .unwrap_or_else(|_| "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045".to_string());
    let target_slot_str = env::var("TARGET_SLOT")
        .unwrap_or_else(|_| "0x0000000000000000000000000000000000000000000000000000000000000000".to_string());

    let config = OracleConfig {
        source_rpc_url: Url::parse(&source_rpc_url).expect("Invalid RPC URL"),
        target_contract_address: Address::from_str(&target_contract_addr_str).expect("Invalid address"),
        target_storage_slot: B256::from_str(&target_slot_str).expect("Invalid slot"),
    };

    println!("Starting Oracle Server...");
    // println!("Source: {}", config.source_rpc_url); // Don't print API keys
    println!("Target Account: {}", config.target_contract_address);
    println!("Target Slot: {}", config.target_storage_slot);

    // Setup SP1 Client
    let client = ProverClient::from_env();
    let (pk, vk) = client.setup(MPT_ELF); // pk used now
    println!("SP1 Setup Complete. Verification Key: {}", vk.bytes32());

    // Setup Alloy Provider
    let provider = ProviderBuilder::new().on_http(config.source_rpc_url);

    // Main Loop
    loop {
        println!("\nFetching latest block...");
        
        // 1. Get latest block number
        let latest_block = match provider.get_block_number().await {
            Ok(n) => n,
            Err(e) => {
                eprintln!("Failed to get block number: {}, retrying...", e);
                sleep(Duration::from_secs(5)).await;
                continue;
            }
        };
        println!("Latest Block: {}", latest_block);

        // 2. Fetch Storage Proof (eth_getProof)
        // alloy's get_proof arguments: address, keys, block_id
        // We request the proof for the raw slot.
        let proof_response = match provider.get_proof(
            config.target_contract_address,
            vec![config.target_storage_slot],
        ).block_id(BlockId::Number(BlockNumberOrTag::Number(latest_block))).await {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Failed to get proof: {}", e);
                sleep(Duration::from_secs(5)).await;
                continue;
            }
        };

        // 3. Parse Proof for SP1
        if let Some(storage_proof) = proof_response.storage_proof.first() {
            println!("Storage Value: {}", storage_proof.value);
            
            // Convert bytes/RLP to vectors expected by MPTProofInput
            // Note: alloy returns `JsonStorageProof` where `proof` is Vec<Bytes>
            let proof_nodes: Vec<Vec<u8>> = storage_proof.proof.iter()
                .map(|b| b.to_vec())
                .collect();

            // The root we are proving against is the *StorageHash* of the account.
            let storage_root = proof_response.storage_hash;
            
            // Ethereum Storage Trie Key is keccak256(slot)
            let trie_key = keccak256(config.target_storage_slot);

            // MPTProofInput expects:
            // key: The path in the trie (which is the hashed slot)
            // value: The RLP-encoded value
            // root: The root hash
            
            let input = MPTProofInput {
                key: trie_key.to_vec(), 
                value: rlp::encode(&storage_proof.value.to_be_bytes::<32>().to_vec()).to_vec(),
                proof: proof_nodes,
                root: storage_root.0,
            };

            // 4. Generate SP1 Proof
            println!("Generating SP1 Proof...");
            let mut stdin = SP1Stdin::new();
            stdin.write(&input);

            // Check for execution mode
            let use_real_proof = env::var("USE_REAL_PROOF").unwrap_or_else(|_| "false".to_string()) == "true";

            if use_real_proof {
                println!("Generating Real SP1 Proof (PLONK)... This may take time and requires Docker.");
                 match client.prove(&pk, &stdin).plonk().run() {
                     Ok(mut proof) => {
                        println!("Proof Generation Successful!");
                        let result: MPTVerificationResult = proof.public_values.read();
                        println!("Verified Root: {}", hex::encode(result.root));
                        println!("Verified Key: {}", hex::encode(result.key));
                        println!("Verified Value: {}", hex::encode(result.value));
                        println!("(Mock) Submitting result to Destination Chain Contract...");
                     }
                     Err(e) => eprintln!("Proving failed (ensure Docker is running): {}", e),
                 }
            } else {
                println!("Generating Mock SP1 Proof (Execute)...");
                match client.execute(MPT_ELF, &stdin).run() {
                     Ok((mut output, _report)) => {
                        println!("Execution Successful (Mock Proof)!");
                        let result: MPTVerificationResult = output.read();
                        println!("Verified Root: {}", hex::encode(result.root));
                        println!("Verified Key: {}", hex::encode(result.key));
                        println!("Verified Value: {}", hex::encode(result.value));
                        println!("(Mock) Submitting result to Destination Chain Contract...");
                     }
                     Err(e) => eprintln!("Execution failed: {}", e),
                 }
            }

        } else {
            eprintln!("No storage proof found for slot");
        }

        println!("Sleeping for 12 seconds...");
        sleep(Duration::from_secs(12)).await;
    }
}
