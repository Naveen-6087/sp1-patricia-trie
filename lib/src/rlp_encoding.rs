use sha3::{Digest, Keccak256};
use crate::types::H256;

/// Compute Keccak256 hash
pub fn keccak256(data: &[u8]) -> H256 {
    let mut hasher = Keccak256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Encode a byte string using RLP
pub fn encode_bytes(data: &[u8]) -> Vec<u8> {
    if data.len() == 1 && data[0] < 0x80 {
        // Single byte less than 128: encode as itself
        data.to_vec()
    } else if data.len() < 56 {
        // Short string (0-55 bytes)
        let mut encoded = vec![0x80 + data.len() as u8];
        encoded.extend_from_slice(data);
        encoded
    } else {
        // Long string (56+ bytes)
        let len_bytes = length_to_bytes(data.len());
        let mut encoded = vec![0xb7 + len_bytes.len() as u8];
        encoded.extend_from_slice(&len_bytes);
        encoded.extend_from_slice(data);
        encoded
    }
}

/// Encode a list using RLP
pub fn encode_list(items: &[Vec<u8>]) -> Vec<u8> {
    let mut payload = Vec::new();
    for item in items {
        payload.extend_from_slice(item);
    }
    
    if payload.len() < 56 {
        // Short list
        let mut encoded = vec![0xc0 + payload.len() as u8];
        encoded.extend_from_slice(&payload);
        encoded
    } else {
        // Long list
        let len_bytes = length_to_bytes(payload.len());
        let mut encoded = vec![0xf7 + len_bytes.len() as u8];
        encoded.extend_from_slice(&len_bytes);
        encoded.extend_from_slice(&payload);
        encoded
    }
}

/// Convert length to big-endian bytes
fn length_to_bytes(len: usize) -> Vec<u8> {
    if len == 0 {
        return vec![0];
    }
    
    let mut bytes = Vec::new();
    let mut n = len;
    while n > 0 {
        bytes.push((n & 0xff) as u8);
        n >>= 8;
    }
    bytes.reverse();
    bytes
}

/// Decode RLP-encoded data into a list of byte vectors
pub fn decode_list(data: &[u8]) -> Result<Vec<Vec<u8>>, &'static str> {
    if data.is_empty() {
        return Err("Empty input");
    }
    
    let prefix = data[0];
    
    // Handle list
    if prefix >= 0xc0 {
        let (payload_start, payload_len) = if prefix <= 0xf7 {
            // Short list
            (1, (prefix - 0xc0) as usize)
        } else {
            // Long list
            let len_of_len = (prefix - 0xf7) as usize;
            if data.len() < 1 + len_of_len {
                return Err("Invalid RLP: insufficient data");
            }
            let payload_len = bytes_to_length(&data[1..1 + len_of_len]);
            (1 + len_of_len, payload_len)
        };
        
        if data.len() < payload_start + payload_len {
            return Err("Invalid RLP: payload too short");
        }
        
        // Parse items from payload
        let mut items = Vec::new();
        let mut pos = payload_start;
        let end = payload_start + payload_len;
        
        while pos < end {
            let item_prefix = data[pos];
            
            let item_len = if item_prefix < 0x80 {
                // Single byte
                1
            } else if item_prefix <= 0xb7 {
                // Short string
                1 + (item_prefix - 0x80) as usize
            } else if item_prefix <= 0xbf {
                // Long string
                let len_of_len = (item_prefix - 0xb7) as usize;
                let data_len = bytes_to_length(&data[pos + 1..pos + 1 + len_of_len]);
                1 + len_of_len + data_len
            } else if item_prefix <= 0xf7 {
                // Short list
                1 + (item_prefix - 0xc0) as usize
            } else {
                // Long list
                let len_of_len = (item_prefix - 0xf7) as usize;
                let data_len = bytes_to_length(&data[pos + 1..pos + 1 + len_of_len]);
                1 + len_of_len + data_len
            };
            
            if pos + item_len > end {
                return Err("Invalid RLP: item exceeds payload");
            }
            
            items.push(data[pos..pos + item_len].to_vec());
            pos += item_len;
        }
        
        Ok(items)
    } else {
        // Single item, wrap in list
        Ok(vec![data.to_vec()])
    }
}

/// Decode a single RLP-encoded byte string
pub fn decode_bytes(data: &[u8]) -> Result<Vec<u8>, &'static str> {
    if data.is_empty() {
        return Err("Empty input");
    }
    
    let prefix = data[0];
    
    if prefix < 0x80 {
        // Single byte
        Ok(data.to_vec())
    } else if prefix <= 0xb7 {
        // Short string
        let len = (prefix - 0x80) as usize;
        if data.len() < 1 + len {
            return Err("Invalid RLP: insufficient data");
        }
        Ok(data[1..1 + len].to_vec())
    } else if prefix <= 0xbf {
        // Long string
        let len_of_len = (prefix - 0xb7) as usize;
        if data.len() < 1 + len_of_len {
            return Err("Invalid RLP: insufficient length bytes");
        }
        let str_len = bytes_to_length(&data[1..1 + len_of_len]);
        if data.len() < 1 + len_of_len + str_len {
            return Err("Invalid RLP: insufficient data");
        }
        Ok(data[1 + len_of_len..1 + len_of_len + str_len].to_vec())
    } else {
        Err("Not a byte string (it's a list)")
    }
}

/// Convert big-endian bytes to length
fn bytes_to_length(bytes: &[u8]) -> usize {
    let mut len = 0;
    for &byte in bytes {
        len = (len << 8) | byte as usize;
    }
    len
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_single_byte() {
        let data = vec![0x42];
        let encoded = encode_bytes(&data);
        assert_eq!(encoded, vec![0x42]);
    }

    #[test]
    fn test_encode_short_string() {
        let data = b"dog";
        let encoded = encode_bytes(data);
        assert_eq!(encoded, vec![0x83, b'd', b'o', b'g']);
    }

    #[test]
    fn test_encode_empty() {
        let data = b"";
        let encoded = encode_bytes(data);
        assert_eq!(encoded, vec![0x80]);
    }

    #[test]
    fn test_encode_list() {
        let item1 = encode_bytes(b"cat");
        let item2 = encode_bytes(b"dog");
        let encoded = encode_list(&[item1, item2]);
        // Should be: 0xc8, 0x83, 'c', 'a', 't', 0x83, 'd', 'o', 'g'
        assert_eq!(encoded[0], 0xc8);
    }

    #[test]
    fn test_keccak256() {
        let data = b"hello";
        let hash = keccak256(data);
        assert_eq!(hash.len(), 32);
    }
}
