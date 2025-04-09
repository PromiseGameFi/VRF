use solana_program::{
    account_info::AccountInfo,
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};
use borsh::{BorshDeserialize, BorshSerialize};
use sha2::{Sha256, Digest};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use std::convert::TryInto;

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct VRFState {
    pub seed: [u8; 32],
    pub counter: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct GameState {
    pub player_balance: u64,
    pub last_random_number: u64,
    pub vrf_state: VRFState,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            player_balance: 1000, // Starting balance
            last_random_number: 0,
            vrf_state: VRFState {
                seed: [0; 32],
                counter: 0,
            },
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum GameInstruction {
    Initialize,
    Play(u64), // Bet amount
    Verify(u64), // Random number to verify
}

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = GameInstruction::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    match instruction {
        GameInstruction::Initialize => {
            msg!("Initializing game state");
            let game_state = GameState::new();
            // Store game state in account data
            let mut account_data = accounts[0].data.borrow_mut();
            game_state.serialize(&mut *account_data)?;
        }
        GameInstruction::Play(bet_amount) => {
            msg!("Playing game with bet: {}", bet_amount);
            let mut account_data = accounts[0].data.borrow_mut();
            let mut game_state = GameState::deserialize(&mut *account_data)?;
            
            if game_state.player_balance < bet_amount {
                return Err(ProgramError::InsufficientFunds);
            }

            // Generate random number using VRF
            let random_number = generate_random_number(&mut game_state.vrf_state);
            game_state.last_random_number = random_number;

            // Simple game logic: 50% chance to win
            if random_number % 2 == 0 {
                game_state.player_balance += bet_amount;
                msg!("You won! New balance: {}", game_state.player_balance);
            } else {
                game_state.player_balance -= bet_amount;
                msg!("You lost! New balance: {}", game_state.player_balance);
            }

            game_state.serialize(&mut *account_data)?;
        }
        GameInstruction::Verify(number) => {
            msg!("Verifying random number");
            let account_data = accounts[0].data.borrow();
            let game_state = GameState::deserialize(&mut &*account_data)?;
            
            if verify_random_number(&game_state.vrf_state, number) {
                msg!("Number verified successfully!");
            } else {
                msg!("Number verification failed!");
            }
        }
    }

    Ok(())
}

fn generate_random_number(vrf_state: &mut VRFState) -> u64 {
    // Increment counter
    vrf_state.counter += 1;
    
    // Create a hash of seed + counter
    let mut hasher = Sha256::new();
    hasher.update(&vrf_state.seed);
    hasher.update(&vrf_state.counter.to_le_bytes());
    let hash = hasher.finalize();
    
    // Use hash as seed for RNG
    let mut rng = StdRng::from_seed(hash.into());
    rng.gen::<u64>()
}

fn verify_random_number(vrf_state: &VRFState, number: u64) -> bool {
    // Recreate the hash
    let mut hasher = Sha256::new();
    hasher.update(&vrf_state.seed);
    hasher.update(&vrf_state.counter.to_le_bytes());
    let hash = hasher.finalize();
    
    // Use hash as seed for RNG
    let mut rng = StdRng::from_seed(hash.into());
    let expected_number = rng.gen::<u64>();
    
    number == expected_number
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
