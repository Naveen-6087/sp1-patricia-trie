# Merkle Patricia Trie with SP1 Zero-Knowledge Proofs

A Rust implementation of Ethereum's Merkle Patricia Trie (MPT) with zero-knowledge proof verification using [Succinct's SP1 zkVM](https://succinctlabs.github.io/sp1/).

## Overview

This project demonstrates how to:
- Build and manipulate Merkle Patricia Tries in Rust
- Generate cryptographic proofs for key-value pairs in an MPT
- Verify MPT proofs in a zero-knowledge virtual machine
- Create succinct proofs of MPT operations without revealing the entire trie

## Features

- **Complete MPT Implementation**
  - Support for all node types: Leaf, Extension, and Branch nodes
  - Efficient RLP encoding/decoding
  - Keccak256 hashing (Ethereum-compatible)
  - Nibble-based path encoding with compact representation

- **Proof Generation & Verification**
  - Generate Merkle proofs for any key in the trie
  - Verify proofs both natively and in zkVM
  - Batch proof generation for multiple keys

- **SP1 Integration**
  - Zero-knowledge proof generation for MPT verification
  - RISC-V execution in SP1 zkVM
  - Efficient cycle count (~122k cycles for complex tries)

## Project Structure

```
mpt/
├── lib/           # Core MPT library (no_std compatible)
│   └── src/
│       ├── types.rs        # Data structures
│       ├── rlp_encoding.rs # RLP codec
│       ├── path.rs         # Nibble path encoding
│       ├── mpt.rs          # Proof verification
│       └── builder.rs      # Trie construction (std only)
├── program/       # SP1 zkVM program
│   └── src/
│       └── main.rs         # Proof verification in zkVM
└── script/        # Host program
    └── src/
        ├── bin/
        │   ├── main.rs     # Main execution script
        │   ├── evm.rs      # EVM proof generation
        │   └── vkey.rs     # Verification key script
        └── build.rs        # Build script
```

## Getting Started

### Prerequisites

- Rust toolchain (1.91+)
- SP1 toolchain: `cargo install cargo-prove`

### Installation

Clone the repository:
```bash
git clone https://github.com/Naveen-6087/sp1-patricia-trie.git
cd sp1-patricia-trie/mpt
```

### Build

```bash
cargo build --release
```

### Run Tests

```bash
# Run all library tests
cargo test --lib -p mpt-lib

# Run specific test
cargo test --lib -p mpt-lib test_builder_multiple_inserts
```

### Execute Program

Run the MPT verification in SP1 zkVM:
```bash
cargo run --release --bin mpt -- --execute
```

Generate a zero-knowledge proof:
```bash
cargo run --release --bin mpt -- --prove
```

## Usage Examples

### Building a Trie

```rust
use mpt_lib::MPTBuilder;

let mut builder = MPTBuilder::new();

// Insert key-value pairs
builder.insert(b"do", b"verb");
builder.insert(b"dog", b"puppy");
builder.insert(b"doge", b"coin");
builder.insert(b"horse", b"stallion");

// Get the root hash
let root = builder.root().unwrap();

// Retrieve a value
let value = builder.get(b"dog").unwrap();
assert_eq!(value, b"puppy");
```

### Generating and Verifying Proofs

```rust
use mpt_lib::{MPTBuilder, verify_proof};

let mut builder = MPTBuilder::new();
builder.insert(b"key", b"value");

let root = builder.root().unwrap();
let proof = builder.get_proof(b"key").unwrap();

// Verify the proof
let verified = verify_proof(&root, b"key", b"value", &proof);
assert!(verified);
```

### Batch Operations

```rust
// Generate proofs for multiple keys
let keys = vec![b"do", b"dog", b"doge"];
let proofs = builder.get_batch_proofs(&keys);

// Get all entries
let all_entries = builder.get_all_entries();
for (key, value) in all_entries {
    println!("{:?} -> {:?}", key, value);
}
```

## Implementation Details

### Merkle Patricia Trie Structure

The MPT uses three types of nodes:

1. **Leaf Node**: `[encoded_path, value]`
   - Stores the final value for a key
   - Path is encoded with a terminator flag

2. **Extension Node**: `[encoded_path, child_hash]`
   - Optimizes storage by compressing common path prefixes
   - Points to next node in the trie

3. **Branch Node**: `[child_0, child_1, ..., child_15, value]`
   - 16 children for each hex digit (0-F)
   - Optional value if a key terminates at this branch

### RLP Encoding

All nodes are encoded using Recursive Length Prefix (RLP) encoding, which is:
- Deterministic
- Compact
- Ethereum-compatible

### Path Encoding

Keys are converted to nibbles (4-bit values) and encoded with:
- Prefix flags to indicate leaf vs extension
- Odd/even length handling
- Compact representation

### Zero-Knowledge Proofs

The SP1 zkVM program:
1. Receives an MPT proof input (key, value, root, proof nodes)
2. Verifies the proof by traversing nodes
3. Outputs verification result
4. Generates a succinct proof of correct execution

## Performance

- **Trie Building**: O(k * log n) where k is key length, n is number of entries
- **Proof Generation**: O(k * log n)
- **Proof Verification**: O(k * p) where p is proof length
- **zkVM Cycles**: ~20k for simple proofs, ~123k for complex tries

## Testing

The project includes comprehensive tests:
- 24 unit tests covering all functionality
- RLP encoding/decoding tests
- Path encoding tests
- Trie insertion and retrieval tests
- Proof generation and verification tests
- Batch operation tests

Run tests with:
```bash
cargo test --lib -p mpt-lib
```

## Known Limitations

- Certain complex nested extension node configurations may have proof verification issues
- No support for trie deletion (only insertion and retrieval)
- In-memory storage only (no persistence layer)

## API Reference

### MPTBuilder

Main interface for building and querying tries:

- `new()` - Create a new empty trie
- `insert(key, value)` - Insert a key-value pair
- `get(key)` - Retrieve a value by key
- `root()` - Get the current root hash
- `get_proof(key)` - Generate a Merkle proof for a key
- `get_batch_proofs(keys)` - Generate proofs for multiple keys
- `get_all_entries()` - Retrieve all key-value pairs

### Verification Functions

- `verify_proof(root, key, value, proof)` - Verify a single proof
- `verify_batch_proofs(root, proofs)` - Verify multiple proofs
- `verify_all_proofs(root, proofs)` - Check if all proofs are valid

## References

- [Ethereum Yellow Paper](https://ethereum.github.io/yellowpaper/paper.pdf) - MPT specification
- [SP1 Documentation](https://succinctlabs.github.io/sp1/) - Zero-knowledge VM
- [RLP Specification](https://ethereum.org/en/developers/docs/data-structures-and-encoding/rlp/)

## License

MIT License - see [LICENSE-MIT](LICENSE-MIT)

## Contributing

Contributions are welcome! Please submit Pull Requests.

## Acknowledgments

- Built with [SP1](https://github.com/succinctlabs/sp1) by Succinct Labs
- Inspired by Ethereum's MPT implementation
