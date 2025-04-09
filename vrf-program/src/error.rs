use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Error, Debug, Copy, Clone)]
pub enum VrfError {
    #[error("Invalid instruction")]
    InvalidInstruction,
    
    #[error("Invalid VRF proof")]
    InvalidProof,
    
    #[error("Not a valid authority")]
    InvalidAuthority,
    
    #[error("Game is in an invalid state")]
    InvalidGameState,
    
    #[error("Randomness not yet available")]
    RandomnessNotAvailable,
}

impl From<VrfError> for ProgramError {
    fn from(e: VrfError) -> Self {
        ProgramError::Custom(e as u32)
    }
} 