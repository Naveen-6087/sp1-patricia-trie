use serde::{Deserialize, Serialize};

/// 32-byte hash type
pub type H256 = [u8; 32];

/// MPT Node types
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Node {
    /// Empty node
    Empty,
    
    /// Leaf node: [encoded_path, value]
    Leaf(Vec<u8>, Vec<u8>),
    
    /// Extension node: [encoded_path, next_hash]
    Extension(Vec<u8>, H256),
    
    /// Branch node: 16 children + optional value
    Branch([Option<H256>; 16], Option<Vec<u8>>),
}

/// Input for MPT proof verification
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MPTProofInput {
    pub key: Vec<u8>,
    pub value: Vec<u8>,
    pub proof: Vec<Vec<u8>>, // RLP-encoded nodes
    pub root: H256,
}

/// Output from MPT proof verification
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MPTVerificationResult {
    pub verified: bool,
    pub key: Vec<u8>,
    pub value: Vec<u8>,
    pub root: H256,
}

/// Batch proof input for multiple key-value pairs
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MPTBatchProofInput {
    pub proofs: Vec<MPTProofInput>,
    pub root: H256,
}

/// Batch verification result
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MPTBatchVerificationResult {
    pub all_verified: bool,
    pub individual_results: Vec<bool>,
    pub root: H256,
    pub count: usize,
}
