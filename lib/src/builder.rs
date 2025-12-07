use crate::types::H256;
use crate::rlp_encoding::{encode_bytes, encode_list, keccak256};
use crate::path::{to_nibbles, encode_path};
use std::collections::HashMap;

/// A simple in-memory Merkle Patricia Trie builder
pub struct MPTBuilder {
    // Store nodes by their hash
    nodes: HashMap<H256, Vec<u8>>,
    root: Option<H256>,
}

impl MPTBuilder {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            root: None,
        }
    }
    
    /// Insert a key-value pair and return the new root
    pub fn insert(&mut self, key: &[u8], value: &[u8]) -> H256 {
        let nibbles = to_nibbles(key);
        
        // For simplicity, create a single leaf node
        // In a full implementation, this would handle branches and extensions
        let encoded_path = encode_path(&nibbles, true);
        
        let leaf_items = vec![
            encode_bytes(&encoded_path),
            encode_bytes(value),
        ];
        
        let leaf_rlp = encode_list(&leaf_items);
        let hash = keccak256(&leaf_rlp);
        
        self.nodes.insert(hash, leaf_rlp);
        self.root = Some(hash);
        
        hash
    }
    
    /// Get the current root hash
    pub fn root(&self) -> Option<H256> {
        self.root
    }
    
    /// Generate a proof for a key
    pub fn get_proof(&self, _key: &[u8]) -> Option<Vec<Vec<u8>>> {
        let root = self.root?;
        let node_rlp = self.nodes.get(&root)?;
        
        // For simple single-node trie, the proof is just the leaf
        Some(vec![node_rlp.clone()])
    }
    
    /// Get a value by key
    pub fn get(&self, _key: &[u8]) -> Option<Vec<u8>> {
        let root = self.root?;
        let node_rlp = self.nodes.get(&root)?;
        
        // Decode the leaf node
        let decoded = crate::rlp_encoding::decode_list(node_rlp).ok()?;
        if decoded.len() != 2 {
            return None;
        }
        
        let value = crate::rlp_encoding::decode_bytes(&decoded[1]).ok()?;
        Some(value)
    }
}

impl Default for MPTBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_insert_and_get() {
        let mut builder = MPTBuilder::new();
        
        let key = b"test_key";
        let value = b"test_value";
        
        builder.insert(key, value);
        
        let retrieved = builder.get(key).unwrap();
        assert_eq!(retrieved, value);
    }

    #[test]
    fn test_builder_proof_generation() {
        let mut builder = MPTBuilder::new();
        
        let key = b"test_key";
        let value = b"test_value";
        
        let root = builder.insert(key, value);
        let proof = builder.get_proof(key).unwrap();
        
        assert!(!proof.is_empty());
        
        // Verify the proof
        let verified = crate::mpt::verify_proof(&root, key, value, &proof);
        assert!(verified);
    }
}
