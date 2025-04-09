use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct VrfAccountState {
    pub authority: Pubkey,
    pub is_initialized: bool,
    pub seed: Option<Vec<u8>>,
    pub randomness: Option<[u8; 32]>,
    pub proof: Option<[u8; 64]>,
    pub public_key: [u8; 32],
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct GameAccountState {
    pub authority: Pubkey,
    pub is_initialized: bool,
    pub randomness: Option<[u8; 32]>,
    pub game_state: GameState,
    pub player_guess: Option<u8>,
    pub result: Option<GameResult>,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub enum GameState {
    AwaitingRandomness,
    AwaitingPlayerGuess,
    GameComplete,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub enum GameResult {
    Win,
    Lose,
}

/// Helper function to calculate a game number from randomness
pub fn calculate_game_number(randomness: &[u8; 32]) -> u8 {
    // Use first byte of randomness modulo 100 to get a number between 0-99
    randomness[0] % 100
} 