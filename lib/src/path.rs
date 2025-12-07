/// Encode path with compact encoding
/// First nibble contains: odd_flag (bit 0) and leaf_flag (bit 1)
pub fn encode_path(nibbles: &[u8], is_leaf: bool) -> Vec<u8> {
    let mut encoded = Vec::new();
    let odd_len = nibbles.len() % 2 == 1;
    
    // Prefix encoding:
    // 0x0: extension, even length
    // 0x1: extension, odd length
    // 0x2: leaf, even length
    // 0x3: leaf, odd length
    let prefix = match (odd_len, is_leaf) {
        (true, true) => 0x3,   // 0011
        (false, true) => 0x2,  // 0010
        (true, false) => 0x1,  // 0001
        (false, false) => 0x0, // 0000
    };
    
    if odd_len {
        // Odd length: prefix + first nibble in first byte
        encoded.push((prefix << 4) | nibbles[0]);
        // Pack remaining nibbles
        for i in (1..nibbles.len()).step_by(2) {
            encoded.push((nibbles[i] << 4) | nibbles[i + 1]);
        }
    } else {
        // Even length: prefix + padding in first byte
        encoded.push(prefix << 4);
        // Pack all nibbles
        for i in (0..nibbles.len()).step_by(2) {
            encoded.push((nibbles[i] << 4) | nibbles[i + 1]);
        }
    }
    
    encoded
}

/// Decode compact-encoded path
/// Returns (nibbles, is_leaf)
pub fn decode_path(encoded: &[u8]) -> (Vec<u8>, bool) {
    if encoded.is_empty() {
        return (Vec::new(), false);
    }
    
    let first = encoded[0];
    let prefix = first >> 4;
    let is_leaf = (prefix & 0x2) != 0;
    let odd_len = (prefix & 0x1) != 0;
    
    let mut nibbles = Vec::new();
    
    if odd_len {
        // First nibble is in the first byte
        nibbles.push(first & 0x0F);
    }
    
    // Unpack remaining bytes into nibbles
    for &byte in &encoded[1..] {
        nibbles.push(byte >> 4);
        nibbles.push(byte & 0x0F);
    }
    
    (nibbles, is_leaf)
}

/// Convert bytes to nibbles (hex digits)
pub fn to_nibbles(data: &[u8]) -> Vec<u8> {
    let mut nibbles = Vec::with_capacity(data.len() * 2);
    for &byte in data {
        nibbles.push(byte >> 4);
        nibbles.push(byte & 0x0F);
    }
    nibbles
}

/// Convert nibbles back to bytes
pub fn from_nibbles(nibbles: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity((nibbles.len() + 1) / 2);
    for i in (0..nibbles.len()).step_by(2) {
        if i + 1 < nibbles.len() {
            bytes.push((nibbles[i] << 4) | nibbles[i + 1]);
        } else {
            bytes.push(nibbles[i] << 4);
        }
    }
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_path_leaf_odd() {
        let nibbles = vec![1, 2, 3, 4, 5];
        let encoded = encode_path(&nibbles, true);
        let (decoded, is_leaf) = decode_path(&encoded);
        
        assert_eq!(nibbles, decoded);
        assert!(is_leaf);
    }

    #[test]
    fn test_encode_decode_path_leaf_even() {
        let nibbles = vec![1, 2, 3, 4];
        let encoded = encode_path(&nibbles, true);
        let (decoded, is_leaf) = decode_path(&encoded);
        
        assert_eq!(nibbles, decoded);
        assert!(is_leaf);
    }

    #[test]
    fn test_encode_decode_path_extension_odd() {
        let nibbles = vec![1, 2, 3];
        let encoded = encode_path(&nibbles, false);
        let (decoded, is_leaf) = decode_path(&encoded);
        
        assert_eq!(nibbles, decoded);
        assert!(!is_leaf);
    }

    #[test]
    fn test_to_nibbles() {
        let data = vec![0x12, 0x34, 0xab];
        let nibbles = to_nibbles(&data);
        assert_eq!(nibbles, vec![1, 2, 3, 4, 10, 11]);
    }

    #[test]
    fn test_from_nibbles() {
        let nibbles = vec![1, 2, 3, 4, 10, 11];
        let data = from_nibbles(&nibbles);
        assert_eq!(data, vec![0x12, 0x34, 0xab]);
    }
}
