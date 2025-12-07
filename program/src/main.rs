//! Merkle Patricia Trie proof verification program for SP1 zkVM.
//!
//! This program verifies MPT proofs inside the zkVM.

#![no_main]
sp1_zkvm::entrypoint!(main);

use mpt_lib::{MPTProofInput, MPTVerificationResult, verify_proof};

pub fn main() {
    // Read the proof input from the host
    let input: MPTProofInput = sp1_zkvm::io::read();
    
    // Verify the MPT proof
    let verified = verify_proof(
        &input.root,
        &input.key,
        &input.value,
        &input.proof,
    );
    
    // Create the verification result
    let result = MPTVerificationResult {
        verified,
        key: input.key,
        value: input.value,
        root: input.root,
    };
    
    // Commit the verification result
    sp1_zkvm::io::commit(&result);
}
