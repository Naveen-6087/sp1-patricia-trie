use crate::types::H256;
use crate::path::{to_nibbles, decode_path};
use crate::rlp_encoding::{keccak256, decode_list, decode_bytes};

/// Verify a Merkle Patricia Trie proof
/// 
/// # Arguments
/// * `root` - The expected root hash of the trie
/// * `key` - The key to verify
/// * `expected_value` - The expected value at the key
/// * `proof` - Vec of RLP-encoded nodes from root to leaf
/// 
/// # Returns
/// * `true` if the proof is valid, `false` otherwise
pub fn verify_proof(
    root: &H256,
    key: &[u8],
    expected_value: &[u8],
    proof: &[Vec<u8>],
) -> bool {
    if proof.is_empty() {
        return false;
    }
    
    // Convert key to nibbles
    let nibbles = to_nibbles(key);
    let mut nibble_idx = 0;
    let mut expected_hash = *root;
    
    for (i, node_rlp) in proof.iter().enumerate() {
        // Verify hash matches expected (skip for root which we trust)
        if i > 0 {
            let node_hash = if node_rlp.len() < 32 {
                // Short nodes are embedded directly
                let mut hash = [0u8; 32];
                if node_rlp.len() <= 32 {
                    hash[..node_rlp.len()].copy_from_slice(node_rlp);
                }
                hash
            } else {
                keccak256(node_rlp)
            };
            
            if node_hash != expected_hash {
                return false;
            }
        }
        
        // Decode RLP node
        let decoded = match decode_list(node_rlp) {
            Ok(d) => d,
            Err(_) => return false,
        };
        
        match decoded.len() {
            // Leaf or Extension node (2 items)
            2 => {
                let path_encoded = match decode_bytes(&decoded[0]) {
                    Ok(p) => p,
                    Err(_) => return false,
                };
                
                let (path, is_leaf) = decode_path(&path_encoded);
                
                if is_leaf {
                    // Leaf node - should be last in proof
                    if i != proof.len() - 1 {
                        return false;
                    }
                    
                    // Check path matches remaining nibbles
                    let remaining = &nibbles[nibble_idx..];
                    if path != remaining {
                        return false;
                    }
                    
                    // Verify value
                    let value = match decode_bytes(&decoded[1]) {
                        Ok(v) => v,
                        Err(_) => return false,
                    };
                    
                    return value == expected_value;
                } else {
                    // Extension node
                    if nibble_idx + path.len() > nibbles.len() {
                        return false;
                    }
                    
                    let remaining = &nibbles[nibble_idx..nibble_idx + path.len()];
                    if path != remaining {
                        return false;
                    }
                    
                    nibble_idx += path.len();
                    
                    // Get next hash
                    let next_node = match decode_bytes(&decoded[1]) {
                        Ok(n) => n,
                        Err(_) => return false,
                    };
                    
                    if next_node.len() == 32 {
                        expected_hash.copy_from_slice(&next_node);
                    } else if next_node.len() < 32 {
                        // Short node embedded
                        expected_hash = [0u8; 32];
                        expected_hash[..next_node.len()].copy_from_slice(&next_node);
                    } else {
                        return false;
                    }
                }
            }
            // Branch node (17 items)
            17 => {
                if nibble_idx > nibbles.len() {
                    return false;
                }
                
                if nibble_idx == nibbles.len() {
                    // Value is in branch node itself (index 16)
                    let value = match decode_bytes(&decoded[16]) {
                        Ok(v) => v,
                        Err(_) => {
                            // Empty value
                            if decoded[16].is_empty() || decoded[16] == vec![0x80] {
                                return false;
                            }
                            return false;
                        }
                    };
                    
                    return value == expected_value;
                }
                
                let nibble = nibbles[nibble_idx] as usize;
                if nibble >= 16 {
                    return false;
                }
                
                nibble_idx += 1;
                
                let child = &decoded[nibble];
                if child.is_empty() || child == &vec![0x80] {
                    // Empty child
                    return false;
                }
                
                // Get next hash
                let next_node = match decode_bytes(child) {
                    Ok(n) => n,
                    Err(_) => {
                        // Might be a raw hash
                        if child.len() == 32 {
                            child.clone()
                        } else {
                            return false;
                        }
                    }
                };
                
                if next_node.len() == 32 {
                    expected_hash.copy_from_slice(&next_node);
                } else if next_node.len() < 32 {
                    // Short node embedded
                    expected_hash = [0u8; 32];
                    expected_hash[..next_node.len()].copy_from_slice(&next_node);
                } else {
                    return false;
                }
            }
            _ => return false,
        }
    }
    
    false
}

/// Get the hash of a node
pub fn hash_node(node_rlp: &[u8]) -> H256 {
    if node_rlp.len() < 32 {
        let mut hash = [0u8; 32];
        hash[..node_rlp.len()].copy_from_slice(node_rlp);
        hash
    } else {
        keccak256(node_rlp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rlp_encoding::encode_bytes;
    use crate::path::encode_path;

    #[test]
    fn test_verify_simple_leaf_proof() {
        // Create a simple proof with just a leaf node
        let key = b"test";
        let value = b"value";
        
        let nibbles = to_nibbles(key);
        let encoded_path = encode_path(&nibbles, true);
        
        let leaf_items = vec![
            encode_bytes(&encoded_path),
            encode_bytes(value),
        ];
        
        let leaf_rlp = crate::rlp_encoding::encode_list(&leaf_items);
        let root = keccak256(&leaf_rlp);
        
        let proof = vec![leaf_rlp];
        
        assert!(verify_proof(&root, key, value, &proof));
    }

    #[test]
    fn test_verify_wrong_value() {
        let key = b"test";
        let value = b"value";
        let wrong_value = b"wrong";
        
        let nibbles = to_nibbles(key);
        let encoded_path = encode_path(&nibbles, true);
        
        let leaf_items = vec![
            encode_bytes(&encoded_path),
            encode_bytes(value),
        ];
        
        let leaf_rlp = crate::rlp_encoding::encode_list(&leaf_items);
        let root = keccak256(&leaf_rlp);
        
        let proof = vec![leaf_rlp];
        
        assert!(!verify_proof(&root, key, wrong_value, &proof));
    }

    #[test]
    fn test_verify_empty_proof() {
        let key = b"test";
        let value = b"value";
        let root = [0u8; 32];
        let proof = vec![];
        
        assert!(!verify_proof(&root, key, value, &proof));
    }
}
