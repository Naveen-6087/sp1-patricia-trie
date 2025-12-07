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
