use alloy_sol_types::sol;

pub mod types;
pub mod rlp_encoding;
pub mod path;

pub use types::*;
pub use rlp_encoding::*;
pub use path::*;

sol! {
    /// The public values encoded as a struct for Solidity verification.
    struct MPTProofOutput {
        bool verified;
        bytes32 root;
        bytes key;
        bytes value;
    }
}
