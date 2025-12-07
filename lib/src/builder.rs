use crate::types::H256;
use crate::rlp_encoding::{encode_bytes, encode_list, keccak256, decode_list, decode_bytes};
use crate::path::{to_nibbles, encode_path, decode_path};
use std::collections::HashMap;

#[derive(Clone, Debug)]
enum TrieNode {
    Empty,
    Leaf(Vec<u8>, Vec<u8>),      // (path, value)
    Extension(Vec<u8>, H256),     // (path, child_hash)
    Branch([Option<H256>; 16], Option<Vec<u8>>), // (children, value)
}

/// An in-memory Merkle Patricia Trie builder with full insertion logic
pub struct MPTBuilder {
    // Store nodes by their hash
    nodes: HashMap<H256, Vec<u8>>,
    // In-memory representation of nodes for easier manipulation
    node_cache: HashMap<H256, TrieNode>,
    root: Option<H256>,
}

impl MPTBuilder {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            node_cache: HashMap::new(),
            root: None,
        }
    }
    
    /// Insert a key-value pair and return the new root
    pub fn insert(&mut self, key: &[u8], value: &[u8]) -> H256 {
        let nibbles = to_nibbles(key);
        let new_root = self.insert_at(self.root, &nibbles, value.to_vec());
        self.root = Some(new_root);
        new_root
    }
    
    /// Recursively insert into the trie
    fn insert_at(&mut self, node_hash: Option<H256>, path: &[u8], value: Vec<u8>) -> H256 {
        match node_hash {
            None => {
                // Create a new leaf node
                self.create_leaf(path, value)
            }
            Some(hash) => {
                // Get the node
                let node = self.get_node(&hash);
                
                match node {
                    TrieNode::Empty => {
                        // Replace empty with leaf
                        self.create_leaf(path, value)
                    }
                    TrieNode::Leaf(leaf_path, leaf_value) => {
                        // Find common prefix
                        let common_len = common_prefix_len(&leaf_path, path);
                        
                        if common_len == leaf_path.len() && common_len == path.len() {
                            // Exact match - update value
                            self.create_leaf(path, value)
                        } else {
                            // Need to create a branch
                            let mut branch_children: [Option<H256>; 16] = Default::default();
                            let mut branch_value = None;
                            
                            // Handle the existing leaf
                            if common_len == leaf_path.len() {
                                // Existing leaf path is exhausted, value goes in branch
                                branch_value = Some(leaf_value);
                            } else {
                                // Existing leaf continues past the branch
                                let idx = leaf_path[common_len] as usize;
                                let child = self.create_leaf(&leaf_path[common_len + 1..], leaf_value);
                                branch_children[idx] = Some(child);
                            }
                            
                            // Handle the new value
                            if common_len == path.len() {
                                // New path is exhausted, value goes in branch
                                branch_value = Some(value);
                            } else {
                                // New value continues past the branch
                                let idx = path[common_len] as usize;
                                let child = self.create_leaf(&path[common_len + 1..], value);
                                branch_children[idx] = Some(child);
                            }
                            
                            let branch = self.create_branch_node(branch_children, branch_value);
                            
                            if common_len == 0 {
                                branch
                            } else {
                                self.create_extension(&path[..common_len], branch)
                            }
                        }
                    }
                    TrieNode::Extension(ext_path, child_hash) => {
                        let common_len = common_prefix_len(&ext_path, path);
                        
                        if common_len == ext_path.len() {
                            // Continue down the extension
                            let new_child = self.insert_at(Some(child_hash), &path[common_len..], value);
                            self.create_extension(&ext_path, new_child)
                        } else {
                            // Extension needs to be split
                            let mut branch_children: [Option<H256>; 16] = Default::default();
                            
                            // Add the old extension's child
                            let old_idx = ext_path[common_len] as usize;
                            if common_len + 1 == ext_path.len() {
                                branch_children[old_idx] = Some(child_hash);
                            } else {
                                let new_ext = self.create_extension(&ext_path[common_len + 1..], child_hash);
                                branch_children[old_idx] = Some(new_ext);
                            }
                            
                            // Add new value
                            let new_idx = path[common_len] as usize;
                            if common_len + 1 == path.len() {
                                // Value goes in branch
                                let branch = self.create_branch_node(branch_children, Some(value));
                                return if common_len == 0 {
                                    branch
                                } else {
                                    self.create_extension(&path[..common_len], branch)
                                };
                            } else {
                                let new_child = self.create_leaf(&path[common_len + 1..], value);
                                branch_children[new_idx] = Some(new_child);
                            }
                            
                            let branch = self.create_branch_node(branch_children, None);
                            
                            if common_len == 0 {
                                branch
                            } else {
                                self.create_extension(&path[..common_len], branch)
                            }
                        }
                    }
                    TrieNode::Branch(mut children, branch_value) => {
                        if path.is_empty() {
                            // Update branch value
                            self.create_branch_node(children, Some(value))
                        } else {
                            // Insert into appropriate child
                            let idx = path[0] as usize;
                            let new_child = self.insert_at(children[idx], &path[1..], value);
                            children[idx] = Some(new_child);
                            self.create_branch_node(children, branch_value)
                        }
                    }
                }
            }
        }
    }
    
    /// Create a leaf node
    fn create_leaf(&mut self, path: &[u8], value: Vec<u8>) -> H256 {
        let encoded_path = encode_path(path, true);
        let leaf_items = vec![
            encode_bytes(&encoded_path),
            encode_bytes(&value),
        ];
        let leaf_rlp = encode_list(&leaf_items);
        let hash = keccak256(&leaf_rlp);
        
        self.nodes.insert(hash, leaf_rlp);
        self.node_cache.insert(hash, TrieNode::Leaf(path.to_vec(), value));
        hash
    }
    
    /// Create an extension node
    fn create_extension(&mut self, path: &[u8], child_hash: H256) -> H256 {
        let encoded_path = encode_path(path, false);
        let ext_items = vec![
            encode_bytes(&encoded_path),
            encode_bytes(&child_hash),
        ];
        let ext_rlp = encode_list(&ext_items);
        let hash = keccak256(&ext_rlp);
        
        self.nodes.insert(hash, ext_rlp);
        self.node_cache.insert(hash, TrieNode::Extension(path.to_vec(), child_hash));
        hash
    }
    
    /// Create a branch node
    fn create_branch_node(&mut self, children: [Option<H256>; 16], value: Option<Vec<u8>>) -> H256 {
        let mut items = Vec::with_capacity(17);
        
        for child in &children {
            if let Some(hash) = child {
                items.push(encode_bytes(hash));
            } else {
                items.push(encode_bytes(&[]));
            }
        }
        
        if let Some(v) = &value {
            items.push(encode_bytes(v));
        } else {
            items.push(encode_bytes(&[]));
        }
        
        let branch_rlp = encode_list(&items);
        let hash = keccak256(&branch_rlp);
        
        self.nodes.insert(hash, branch_rlp);
        self.node_cache.insert(hash, TrieNode::Branch(children, value));
        hash
    }
    
    /// Helper to create a branch with a single leaf child
    /// Get a node from cache or decode it
    fn get_node(&mut self, hash: &H256) -> TrieNode {
        if let Some(node) = self.node_cache.get(hash) {
            return node.clone();
        }
        
        // Decode from RLP
        if let Some(rlp) = self.nodes.get(hash) {
            if let Ok(items) = decode_list(rlp) {
                if items.len() == 2 {
                    // Leaf or Extension
                    if let Ok(path_bytes) = decode_bytes(&items[0]) {
                        let (path, is_leaf) = decode_path(&path_bytes);
                        
                        if is_leaf {
                            if let Ok(value) = decode_bytes(&items[1]) {
                                let node = TrieNode::Leaf(path, value);
                                self.node_cache.insert(*hash, node.clone());
                                return node;
                            }
                        } else {
                            if let Ok(child_bytes) = decode_bytes(&items[1]) {
                                if child_bytes.len() == 32 {
                                    let mut child_hash = [0u8; 32];
                                    child_hash.copy_from_slice(&child_bytes);
                                    let node = TrieNode::Extension(path, child_hash);
                                    self.node_cache.insert(*hash, node.clone());
                                    return node;
                                }
                            }
                        }
                    }
                } else if items.len() == 17 {
                    // Branch
                    let mut children: [Option<H256>; 16] = Default::default();
                    for i in 0..16 {
                        if let Ok(child_bytes) = decode_bytes(&items[i]) {
                            if child_bytes.len() == 32 {
                                let mut child_hash = [0u8; 32];
                                child_hash.copy_from_slice(&child_bytes);
                                children[i] = Some(child_hash);
                            }
                        }
                    }
                    
                    let value = decode_bytes(&items[16]).ok().filter(|v| !v.is_empty());
                    let node = TrieNode::Branch(children, value);
                    self.node_cache.insert(*hash, node.clone());
                    return node;
                }
            }
        }
        
        TrieNode::Empty
    }
    
    /// Get the current root hash
    pub fn root(&self) -> Option<H256> {
        self.root
    }
    
    /// Generate a proof for a key (collect all nodes along the path)
    pub fn get_proof(&self, key: &[u8]) -> Option<Vec<Vec<u8>>> {
        let nibbles = to_nibbles(key);
        let mut proof = Vec::new();
        let mut current_hash = self.root?;
        let mut remaining_path = &nibbles[..];
        
        loop {
            let node_rlp = self.nodes.get(&current_hash)?;
            proof.push(node_rlp.clone());
            
            // Decode and determine next step
            let items = decode_list(node_rlp).ok()?;
            
            if items.len() == 2 {
                // Leaf or Extension
                let path_bytes = decode_bytes(&items[0]).ok()?;
                let (path, is_leaf) = decode_path(&path_bytes);
                
                if is_leaf {
                    // Reached a leaf
                    return Some(proof);
                } else {
                    // Extension - continue to child
                    if !remaining_path.starts_with(&path) {
                        return None;
                    }
                    remaining_path = &remaining_path[path.len()..];
                    let child_bytes = decode_bytes(&items[1]).ok()?;
                    if child_bytes.len() != 32 {
                        return None;
                    }
                    current_hash.copy_from_slice(&child_bytes);
                }
            } else if items.len() == 17 {
                // Branch
                if remaining_path.is_empty() {
                    // Value is in the branch itself
                    return Some(proof);
                }
                
                let idx = remaining_path[0] as usize;
                let child_bytes = decode_bytes(&items[idx]).ok()?;
                
                if child_bytes.is_empty() {
                    return None;
                }
                
                if child_bytes.len() != 32 {
                    return None;
                }
                
                current_hash.copy_from_slice(&child_bytes);
                remaining_path = &remaining_path[1..];
            } else {
                return None;
            }
        }
    }
    
    /// Get a value by key
    pub fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        let nibbles = to_nibbles(key);
        let mut current_hash = self.root?;
        let mut remaining_path = &nibbles[..];
        
        loop {
            let node_rlp = self.nodes.get(&current_hash)?;
            let items = decode_list(node_rlp).ok()?;
            
            if items.len() == 2 {
                // Leaf or Extension
                let path_bytes = decode_bytes(&items[0]).ok()?;
                let (path, is_leaf) = decode_path(&path_bytes);
                
                if is_leaf {
                    // Check if path matches
                    if path == remaining_path {
                        return decode_bytes(&items[1]).ok();
                    } else {
                        return None;
                    }
                } else {
                    // Extension
                    if !remaining_path.starts_with(&path) {
                        return None;
                    }
                    remaining_path = &remaining_path[path.len()..];
                    let child_bytes = decode_bytes(&items[1]).ok()?;
                    if child_bytes.len() != 32 {
                        return None;
                    }
                    current_hash.copy_from_slice(&child_bytes);
                }
            } else if items.len() == 17 {
                // Branch
                if remaining_path.is_empty() {
                    // Value is in the branch
                    let value = decode_bytes(&items[16]).ok()?;
                    return if value.is_empty() { None } else { Some(value) };
                }
                
                let idx = remaining_path[0] as usize;
                let child_bytes = decode_bytes(&items[idx]).ok()?;
                
                if child_bytes.is_empty() {
                    return None;
                }
                
                if child_bytes.len() != 32 {
                    return None;
                }
                
                current_hash.copy_from_slice(&child_bytes);
                remaining_path = &remaining_path[1..];
            } else {
                return None;
            }
        }
    }
}

/// Helper function to find common prefix length
fn common_prefix_len(a: &[u8], b: &[u8]) -> usize {
    a.iter().zip(b.iter()).take_while(|(x, y)| x == y).count()
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
        
        builder.insert(key, value);
        
        let proof = builder.get_proof(key).unwrap();
        assert!(!proof.is_empty());
    }
    
    #[test]
    fn test_builder_multiple_inserts() {
        let mut builder = MPTBuilder::new();
        
        // Insert multiple key-value pairs
        builder.insert(b"do", b"verb");
        builder.insert(b"dog", b"puppy");
        builder.insert(b"doge", b"coin");
        builder.insert(b"horse", b"stallion");
        
        // Verify all values can be retrieved
        assert_eq!(builder.get(b"do").unwrap(), b"verb");
        assert_eq!(builder.get(b"dog").unwrap(), b"puppy");
        assert_eq!(builder.get(b"doge").unwrap(), b"coin");
        assert_eq!(builder.get(b"horse").unwrap(), b"stallion");
    }
    
    #[test]
    fn test_builder_overwrite_value() {
        let mut builder = MPTBuilder::new();
        
        builder.insert(b"key", b"value1");
        assert_eq!(builder.get(b"key").unwrap(), b"value1");
        
        builder.insert(b"key", b"value2");
        assert_eq!(builder.get(b"key").unwrap(), b"value2");
    }
    
    #[test]
    fn test_builder_branch_node() {
        let mut builder = MPTBuilder::new();
        
        // These keys will create a branch at the first nibble
        builder.insert(b"a", b"value_a");
        builder.insert(b"b", b"value_b");
        
        assert_eq!(builder.get(b"a").unwrap(), b"value_a");
        assert_eq!(builder.get(b"b").unwrap(), b"value_b");
    }
    
    #[test]
    fn test_builder_extension_node() {
        let mut builder = MPTBuilder::new();
        
        // These will create an extension node (common prefix "do")
        builder.insert(b"dog", b"puppy");
        builder.insert(b"dodge", b"car");
        
        assert_eq!(builder.get(b"dog").unwrap(), b"puppy");
        assert_eq!(builder.get(b"dodge").unwrap(), b"car");
    }
    
    #[test]
    fn test_builder_get_nonexistent() {
        let mut builder = MPTBuilder::new();
        
        builder.insert(b"key", b"value");
        
        assert!(builder.get(b"nonexistent").is_none());
    }
    
    #[test]
    fn test_builder_complex_proof() {
        let mut builder = MPTBuilder::new();
        
        // Build a complex trie
        builder.insert(b"do", b"verb");
        builder.insert(b"dog", b"puppy");
        builder.insert(b"doge", b"coin");
        
        // Get proof for each key
        let proof1 = builder.get_proof(b"do").unwrap();
        let proof2 = builder.get_proof(b"dog").unwrap();
        let proof3 = builder.get_proof(b"doge").unwrap();
        
        // All proofs should exist
        assert!(!proof1.is_empty());
        assert!(!proof2.is_empty());
        assert!(!proof3.is_empty());
        
        // Proofs for longer keys should have more nodes
        assert!(proof2.len() >= proof1.len());
        assert!(proof3.len() >= proof2.len());
    }
}
