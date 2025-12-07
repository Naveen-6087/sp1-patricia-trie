//! MPT Proof Generation and Verification Script
//!
//! You can run this script using the following command:
//! ```shell
//! RUST_LOG=info cargo run --release -- --execute
//! ```
//! or
//! ```shell
//! RUST_LOG=info cargo run --release -- --prove
//! ```

use clap::Parser;
use mpt_lib::{MPTProofInput, MPTVerificationResult, MPTBuilder};
use sp1_sdk::{include_elf, ProverClient, SP1Stdin};

/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM.
pub const MPT_ELF: &[u8] = include_elf!("mpt-program");

/// The arguments for the command.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long)]
    execute: bool,

    #[arg(long)]
    prove: bool,
}

fn main() {
    // Setup the logger.
    sp1_sdk::utils::setup_logger();
    dotenv::dotenv().ok();

    // Parse the command line arguments.
    let args = Args::parse();

    if args.execute == args.prove {
        eprintln!("Error: You must specify either --execute or --prove");
        std::process::exit(1);
    }

    // Setup the prover client.
    let client = ProverClient::from_env();

    // Build a simple MPT with one key-value pair
    let mut builder = MPTBuilder::new();
    let key = b"hello";
    let value = b"world";
    
    println!("Building MPT...");
    let root = builder.insert(key, value);
    println!("  Root: {}", hex::encode(root));
    
    // Generate proof
    let proof = builder.get_proof(key).expect("Failed to generate proof");
    println!("  Proof nodes: {}", proof.len());
    
    // Create MPT proof input
    let input = MPTProofInput {
        key: key.to_vec(),
        value: value.to_vec(),
        proof,
        root,
    };

    // Setup the inputs.
    let mut stdin = SP1Stdin::new();
    stdin.write(&input);

    println!("\nMPT Proof Input:");
    println!("  Key: {}", hex::encode(&input.key));
    println!("  Value: {}", String::from_utf8_lossy(&input.value));
    println!("  Root: {}", hex::encode(&input.root));

    if args.execute {
        // Execute the program
        let (mut output, report) = client.execute(MPT_ELF, &stdin).run().unwrap();
        println!("\nProgram executed successfully.");

        // Read the output.
        let result: MPTVerificationResult = output.read();
        println!("\nVerification Result:");
        println!("  Verified: {}", result.verified);
        println!("  Key: {}", hex::encode(&result.key));
        println!("  Value: {}", String::from_utf8_lossy(&result.value));
        println!("  Root: {}", hex::encode(&result.root));

        // Record the number of cycles executed.
        println!("\nNumber of cycles: {}", report.total_instruction_count());
    } else {
        // Setup the program for proving.
        let (pk, vk) = client.setup(MPT_ELF);

        // Generate the proof
        let mut proof = client
            .prove(&pk, &stdin)
            .run()
            .expect("failed to generate proof");

        println!("\nSuccessfully generated proof!");

        // Read the output from the proof
        let result: MPTVerificationResult = proof.public_values.read();
        println!("\nVerification Result:");
        println!("  Verified: {}", result.verified);
        println!("  Key: {}", hex::encode(&result.key));
        println!("  Value: {}", String::from_utf8_lossy(&result.value));

        // Verify the proof.
        client.verify(&proof, &vk).expect("failed to verify proof");
        println!("\nSuccessfully verified proof!");
    }
}
