use borsh::{BorshDeserialize, BorshSerialize};
use sha3::{Digest, Sha3_256};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{
    error::VrfError,
    instruction::{GameInstruction, VrfInstruction},
    state::{
        calculate_game_number, GameAccountState, GameResult, GameState, VrfAccountState,
    },
};

// Program logic
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = VrfInstruction::try_from_slice(instruction_data)
        .map_err(|_| VrfError::InvalidInstruction)?;

    match instruction {
        VrfInstruction::Initialize => initialize_vrf(program_id, accounts),
        VrfInstruction::RequestRandomness { seed } => request_randomness(program_id, accounts, seed),
        VrfInstruction::FulfillRandomness { proof } => fulfill_randomness(program_id, accounts, proof),
    }
}

// Initialize a new VRF account
fn initialize_vrf(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let authority = next_account_info(account_info_iter)?;
    let vrf_account = next_account_info(account_info_iter)?;

    if !authority.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if vrf_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    // Generate VRF key pair (in a real implementation, the authority should provide this)
    // Here we're using a dummy key for simplicity
    let public_key = [1u8; 32]; // In a real implementation, this would be a proper ed25519 public key

    let vrf_state = VrfAccountState {
        authority: *authority.key,
        is_initialized: true,
        seed: None,
        randomness: None,
        proof: None,
        public_key,
    };

    vrf_state.serialize(&mut *vrf_account.data.borrow_mut())?;
    msg!("VRF account initialized");
    Ok(())
}

// Request randomness with a seed
fn request_randomness(program_id: &Pubkey, accounts: &[AccountInfo], seed: Vec<u8>) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let authority = next_account_info(account_info_iter)?;
    let vrf_account = next_account_info(account_info_iter)?;
    let game_account = next_account_info(account_info_iter)?;

    if !authority.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if vrf_account.owner != program_id || game_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    let mut vrf_state = VrfAccountState::try_from_slice(&vrf_account.data.borrow())?;
    
    if vrf_state.authority != *authority.key {
        return Err(VrfError::InvalidAuthority.into());
    }

    // Initialize or validate game account
    let mut game_state = if game_account.data_is_empty() {
        GameAccountState {
            authority: *authority.key,
            is_initialized: true,
            randomness: None,
            game_state: GameState::AwaitingRandomness,
            player_guess: None,
            result: None,
        }
    } else {
        GameAccountState::try_from_slice(&game_account.data.borrow())?
    };

    // Set the seed for VRF
    vrf_state.seed = Some(seed);
    vrf_state.randomness = None;
    vrf_state.proof = None;

    // Update game state
    game_state.game_state = GameState::AwaitingRandomness;
    game_state.randomness = None;
    game_state.player_guess = None;
    game_state.result = None;

    vrf_state.serialize(&mut *vrf_account.data.borrow_mut())?;
    game_state.serialize(&mut *game_account.data.borrow_mut())?;

    msg!("Randomness requested");
    Ok(())
}

// Fulfill randomness with proof
fn fulfill_randomness(program_id: &Pubkey, accounts: &[AccountInfo], proof: [u8; 64]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let authority = next_account_info(account_info_iter)?;
    let vrf_account = next_account_info(account_info_iter)?;
    let game_account = next_account_info(account_info_iter)?;

    if !authority.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if vrf_account.owner != program_id || game_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    let mut vrf_state = VrfAccountState::try_from_slice(&vrf_account.data.borrow())?;
    
    if vrf_state.authority != *authority.key {
        return Err(VrfError::InvalidAuthority.into());
    }

    // Verify the VRF proof (simplified for this example)
    // In a real implementation, we would verify the proof cryptographically
    let seed = vrf_state.seed.as_ref().ok_or(ProgramError::InvalidArgument)?;
    
    // For this example, we'll generate randomness by hashing the proof and seed
    let mut hasher = Sha3_256::new();
    hasher.update(&proof);
    hasher.update(seed);
    let randomness = hasher.finalize();
    
    let mut randomness_bytes = [0u8; 32];
    randomness_bytes.copy_from_slice(&randomness);

    // Update VRF state
    vrf_state.proof = Some(proof);
    vrf_state.randomness = Some(randomness_bytes);

    // Update game state
    let mut game_state = GameAccountState::try_from_slice(&game_account.data.borrow())?;
    game_state.randomness = Some(randomness_bytes);
    game_state.game_state = GameState::AwaitingPlayerGuess;

    vrf_state.serialize(&mut *vrf_account.data.borrow_mut())?;
    game_state.serialize(&mut *game_account.data.borrow_mut())?;

    msg!("Randomness fulfilled: {:?}", randomness_bytes);
    Ok(())
}

// Game-specific process instruction
pub fn process_game_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = GameInstruction::try_from_slice(instruction_data)
        .map_err(|_| VrfError::InvalidInstruction)?;

    match instruction {
        GameInstruction::InitializeGame => initialize_game(program_id, accounts),
        GameInstruction::SubmitGuess { guess } => submit_guess(program_id, accounts, guess),
    }
}

// Initialize a new game account
fn initialize_game(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let authority = next_account_info(account_info_iter)?;
    let game_account = next_account_info(account_info_iter)?;

    if !authority.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if game_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    let game_state = GameAccountState {
        authority: *authority.key,
        is_initialized: true,
        randomness: None,
        game_state: GameState::AwaitingRandomness,
        player_guess: None,
        result: None,
    };

    game_state.serialize(&mut *game_account.data.borrow_mut())?;
    msg!("Game account initialized");
    Ok(())
}

// Process a player's guess
fn submit_guess(program_id: &Pubkey, accounts: &[AccountInfo], guess: u8) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let player = next_account_info(account_info_iter)?;
    let game_account = next_account_info(account_info_iter)?;
    let _vrf_account = next_account_info(account_info_iter)?;

    if !player.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if game_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    // Get game state
    let mut game_state = GameAccountState::try_from_slice(&game_account.data.borrow())?;
    
    // Check game is in correct state
    if game_state.game_state != GameState::AwaitingPlayerGuess {
        return Err(VrfError::InvalidGameState.into());
    }
    
    // Get randomness from game state
    let randomness = game_state.randomness.ok_or(VrfError::RandomnessNotAvailable)?;
    
    // Calculate game number from randomness
    let game_number = calculate_game_number(&randomness);
    
    // Determine result
    let result = if guess == game_number {
        GameResult::Win
    } else {
        GameResult::Lose
    };
    
    // Update game state
    game_state.player_guess = Some(guess);
    game_state.result = Some(result.clone());
    game_state.game_state = GameState::GameComplete;
    
    game_state.serialize(&mut *game_account.data.borrow_mut())?;
    
    msg!("Game complete! Number was: {}, Player guessed: {}, Result: {:?}", 
         game_number, guess, result);
    
    Ok(())
} 