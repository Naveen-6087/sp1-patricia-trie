# SP1 Merkle Patricia Trie

A high-performance implementation of Ethereum's Merkle Patricia Trie with [SP1](https://github.com/succinctlabs/sp1) zkVM integration for zero-knowledge proof generation and verification.

## Features

- **Complete MPT Implementation**: Support for Leaf, Extension, and Branch nodes
- **Optimized Performance**: Uses SP1 Keccak256 precompile for 65% faster execution (42,620 cycles vs 122,917)
- **Proof Generation**: Generate and verify Merkle proofs for any key-value pair
- **Batch Operations**: Efficient batch proof generation and verification
- **Zero-Knowledge Proofs**: Full SP1 zkVM integration with STARK proof generation
- **EVM Compatibility**: Generate Groth16/PLONK proofs for on-chain verification
- **no_std Compatible**: Core library works without the standard library

## Requirements

- [Rust](https://rustup.rs/) (1.91.1 or later)
- [SP1](https://docs.succinct.xyz/docs/sp1/getting-started/install) (v5.0.8)
- [Docker](https://www.docker.com/) (for EVM proof generation)

## Project Structure

```
mpt/
├── lib/           # Core MPT library (no_std compatible)
│   ├── types.rs         # Data structures (H256, Node, MPTProofInput)
│   ├── rlp_encoding.rs  # RLP codec with Keccak256 (SP1 precompile)
│   ├── path.rs          # Nibble path utilities
│   ├── mpt.rs           # Proof verification logic
│   └── builder.rs       # MPT construction and proof generation
├── program/       # zkVM program (proves MPT verification)
└── script/        # Host scripts for execution and proving
    ├── main.rs    # Execute/prove MPT verification
    ├── evm.rs     # Generate EVM-compatible proofs
    └── vkey.rs    # Extract verification key
```

## Running the Project

There are 3 main ways to run this project: execute the program, generate a core proof, and
generate an EVM-compatible proof.

### Build the Program

The program is automatically built through `script/build.rs` when the script is built.

### Execute the Program

To run the program without generating a proof (fast execution with cycle count):

```sh
cd script
RUST_LOG=info cargo run --release --bin mpt -- --execute
```

**Output Example:**
```
Number of cycles: 42620
Verification Result:
  Verified: true
  Key: 646f67
  Value: puppy
```

This executes the MPT verification in the zkVM and shows performance metrics.

### Generate an SP1 Core Proof

To generate an SP1 [core proof](https://docs.succinct.xyz/docs/sp1/generating-proofs/proof-types#core-default) (STARK proof) for your MPT verification:

```sh
cd script
RUST_LOG=info cargo run --release --bin mpt -- --prove
```

This generates and verifies a zero-knowledge proof that the MPT verification was executed correctly.

**Current Implementation:**
- Verifies 4 key-value pairs: `do`, `dog`, `doge`, `horse`
- Generates proof for the `dog → puppy` entry
- Execution time: ~30 seconds (local CPU proving)
- Proof size: STARK proof (not EVM-compatible)

### Generate an EVM-Compatible Proof

> [!WARNING]
> You will need at least 16GB RAM to generate a Groth16 or PLONK proof locally. View the [SP1 docs](https://docs.succinct.xyz/docs/sp1/getting-started/hardware-requirements#local-proving) for more information. **Using the SP1 Prover Network is highly recommended.**

Generating a proof that is cheap to verify on the EVM (e.g. Groth16 or PLONK) is more intensive than generating a core proof.

**Prerequisites:**
- Docker must be running (`docker info` to verify)
- At least 16GB RAM available
- Alternatively, use the SP1 Prover Network (see below)

To generate a Groth16 proof (most gas-efficient on-chain):

```sh
cd script
RUST_LOG=info cargo run --release --bin evm -- --system groth16
```

To generate a PLONK proof (faster to generate):

```sh
cd script
RUST_LOG=info cargo run --release --bin evm -- --system plonk
```

These commands will generate fixtures at `../contracts/src/fixtures/` containing:
- `proof`: Hex-encoded proof bytes for on-chain verification
- `publicValues`: Committed MPT verification result  
- `vkey`: Verification key (consistent across all proofs from this program)

### Retrieve the Verification Key

To retrieve your `programVKey` for your on-chain contract, run the following command:

```sh
cd script
cargo run --release --bin vkey
```

## Performance Optimizations

This implementation uses SP1's Keccak256 precompile for optimal performance:

- **Cycle Count**: 42,620 cycles (65% reduction from baseline)
- **Precompile**: Uses `tiny-keccak` with SP1 patch for automatic KECCAK_PERMUTE syscall
- **Verification**: The SP1-patched version is confirmed via `cargo tree -p tiny-keccak`

**To verify the precompile is active:**
```sh
cd lib
cargo tree -p tiny-keccak
# Should show: tiny-keccak v2.0.2 (https://github.com/sp1-patches/tiny-keccak...)
```

## Using the Prover Network

We highly recommend using the [Succinct Prover Network](https://docs.succinct.xyz/docs/network/introduction) for EVM proof generation or any compute-intensive proving. The network handles the heavy computation and is essential for production use.

**Why use the prover network:**
- Avoids local memory issues (EVM proving requires 16GB+ RAM)
- Much faster than local proving
- No Docker setup required
- Production-ready infrastructure

**Setup:**

1. Sign up at [https://network.succinct.xyz/](https://network.succinct.xyz/)
2. Get your API key from the dashboard
3. Copy the example environment file:

```sh
cp .env.example .env
```

4. Set your environment variables in `.env`:

```sh
SP1_PROVER=network
SP1_PRIVATE_KEY=your_api_key_here
```

**Generate an EVM proof using the network:**

```sh
cd script
RUST_LOG=info cargo run --release --bin evm -- --system plonk
```

The prover network will handle all the heavy computation remotely.

## Testing

Run the full test suite:

```sh
cd lib
cargo test
```

**Test Coverage:**
- 24 tests passing
- RLP encoding/decoding
- Path/nibble conversion
- MPT node operations
- Proof generation and verification
- Batch proof operations

## Documentation

See [DOCUMENTATION.md](DOCUMENTATION.md) for comprehensive API documentation, examples, and implementation details.
