pub mod client;
pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;
pub mod vrf;

pub use crate::error::VrfError;
pub use crate::instruction::{GameInstruction, VrfInstruction};
pub use crate::processor::{process_game_instruction, process_instruction as processor_process_instruction};
pub use crate::state::{calculate_game_number, GameAccountState, GameResult, GameState, VrfAccountState};
pub use crate::vrf::Vrf;
pub use crate::client::VrfClient;

#[cfg(not(feature = "no-entrypoint"))]
use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, pubkey::Pubkey,
};

// Entry point
#[cfg(not(feature = "no-entrypoint"))]
entrypoint!(process_instruction);

// This is our wrapper over the processor's implementation
#[cfg(not(feature = "no-entrypoint"))]
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    if instruction_data.len() > 0 && instruction_data[0] == 0 {
        // If the first byte is 0, process as a game instruction
        processor::process_game_instruction(program_id, accounts, &instruction_data[1..])
    } else {
        // Otherwise, process as a VRF instruction
        processor::process_instruction(program_id, accounts, instruction_data)
    }
}
