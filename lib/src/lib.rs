//! # Merkle Patricia Trie Library
//!
//! A Rust implementation of Ethereum's Merkle Patricia Trie with SP1 zkVM integration.
//!
//! This library provides a complete implementation of the MPT data structure used in Ethereum,
//! with support for zero-knowledge proof generation and verification using SP1.
//!
//! ## Features
//!
//! - **Complete MPT Implementation**: Support for Leaf, Extension, and Branch nodes
//! - **RLP Encoding**: Ethereum-compatible encoding/decoding
//! - **Proof Generation**: Generate Merkle proofs for any key
//! - **zkVM Integration**: Verify proofs in SP1's zero-knowledge virtual machine
//! - **no_std Compatible**: Core functionality works without the standard library
//!
//! ## Example
//!
//! ```ignore
//! use mpt_lib::MPTBuilder;
//!
//! // Build a trie
//! let mut builder = MPTBuilder::new();
//! builder.insert(b"key", b"value");
//!
//! // Generate and verify a proof
//! let root = builder.root().unwrap();
//! let proof = builder.get_proof(b"key").unwrap();
//! let verified = verify_proof(&root, b"key", b"value", &proof);
//! assert!(verified);
//! ```

use alloy_sol_types::sol;

pub mod types;
pub mod rlp_encoding;
pub mod path;
pub mod mpt;

#[cfg(feature = "std")]
pub mod builder;

pub use types::*;
pub use rlp_encoding::*;
pub use path::*;
pub use mpt::*;

#[cfg(feature = "std")]
pub use builder::*;

sol! {
    /// The public values encoded as a struct for Solidity verification.
    struct MPTProofOutput {
        bool verified;
        bytes32 root;
        bytes key;
        bytes value;
    }
}
