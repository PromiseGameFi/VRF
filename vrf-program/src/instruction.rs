use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, PartialEq)]
pub enum VrfInstruction {
    /// Initialize a new VRF account
    /// Accounts expected:
    /// 0. `[signer, writable]` Authority account that will manage the VRF
    /// 1. `[writable]` The VRF account to initialize
    Initialize,
    
    /// Request randomness with a seed
    /// Accounts expected:
    /// 0. `[signer]` Authority account
    /// 1. `[writable]` The VRF account
    /// 2. `[writable]` Game account that will use the randomness
    RequestRandomness {
        seed: Vec<u8>,
    },
    
    /// Fulfill randomness with proof
    /// Accounts expected:
    /// 0. `[signer]` Authority account
    /// 1. `[writable]` The VRF account
    /// 2. `[writable]` Game account that will use the randomness
    FulfillRandomness {
        proof: [u8; 64],
    },
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, PartialEq)]
pub enum GameInstruction {
    /// Initialize a new game account
    /// Accounts expected:
    /// 0. `[signer, writable]` Authority account
    /// 1. `[writable]` The game account to initialize
    InitializeGame,
    
    /// Submit a player's guess
    /// Accounts expected:
    /// 0. `[signer]` Player account
    /// 1. `[writable]` The game account
    /// 2. `[]` The VRF account (readonly)
    SubmitGuess {
        guess: u8,
    },
} 