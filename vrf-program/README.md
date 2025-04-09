# Verifiable Random Function (VRF) on Solana

This project implements a Verifiable Random Function (VRF) on the Solana blockchain, along with a simple guessing game to demonstrate its functionality.

## What is a VRF?

A Verifiable Random Function (VRF) is a cryptographic primitive that maps inputs to verifiable pseudorandom outputs. When a VRF is used to generate randomness:

1. Anyone can verify that the output was generated correctly from the input
2. The output is unpredictable without knowing the secret key
3. The same input always produces the same output for the same key

These properties make VRFs ideal for blockchain applications that require provably fair randomness.

## Our Implementation

This implementation consists of:

1. **VRF Program**: A Solana program that implements the VRF functionality
2. **Number Guessing Game**: A simple game where players try to guess a number between 0-99 that is determined by the VRF

## How the Game Works

1. A game is initialized with a VRF account
2. Randomness is requested with a seed
3. The VRF oracle (in a real application) or the authority (in our demo) fulfills the randomness with a cryptographic proof
4. The player submits a guess (0-99)
5. The game calculates the winning number from the VRF output and determines if the player won

## Usage

### Prerequisites

- Rust and Cargo
- Solana CLI tools
- A Solana keypair (can be generated with `solana-keygen new`)

### Building the Program

```bash
cd vrf-program
cargo build-bpf
```

### Deploying the Program

```bash
solana program deploy target/deploy/vrf_program.so
```

### Playing the Game

1. Initialize the accounts:

```bash
cargo run --bin cli init --keypair path/to/keypair.json --program-id <PROGRAM_ID>
```

2. Play the game:

```bash
cargo run --bin cli play --keypair path/to/keypair.json --program-id <PROGRAM_ID> --vrf-account <VRF_ACCOUNT> --game-account <GAME_ACCOUNT> --guess <YOUR_GUESS>
```

## Technical Implementation

Our VRF implementation is based on the Ed25519 signature scheme:

1. The seed is hashed to a point on the curve
2. The point is raised to the power of the secret key
3. The resulting point is signed to prove knowledge of the secret key
4. The output is the hash of this point

This is a simplified implementation for demonstration purposes. In a production environment, you would use a more robust VRF implementation like ECVRF.

## Future Improvements

- Implement full ECVRF based on standards like draft-irtf-cfrg-vrf
- Add multi-player game mechanics
- Implement an oracle network to generate and verify VRF proofs
- Add staking and rewards for correct guesses

## License

This project is licensed under the MIT License. 