use borsh::BorshDeserialize;
use solana_client::rpc_client::RpcClient;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_instruction,
};
use solana_sdk::{
    commitment_config::CommitmentConfig,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use std::{thread, time::Duration};

use crate::{
    GameAccountState, GameState, VrfAccountState,
};

pub struct VrfClient {
    pub client: RpcClient,
    pub payer: Keypair,
    pub program_id: Pubkey,
}

impl VrfClient {
    pub fn new(url: &str, payer: Keypair, program_id: Pubkey) -> Self {
        let client = RpcClient::new_with_commitment(url.to_string(), CommitmentConfig::confirmed());
        Self {
            client,
            payer,
            program_id,
        }
    }

    pub fn initialize_vrf(&self, vrf_account: &Keypair) -> Result<String, Box<dyn std::error::Error>> {
        // Calculate the rent-exempt minimum balance for the VRF account
        let vrf_account_size = std::mem::size_of::<VrfAccountState>();
        let rent = self.client.get_minimum_balance_for_rent_exemption(vrf_account_size)?;

        // Create a transaction to create the VRF account and initialize it
        let create_vrf_account_ix = system_instruction::create_account(
            &self.payer.pubkey(),
            &vrf_account.pubkey(),
            rent,
            vrf_account_size as u64,
            &self.program_id,
        );

        // Initialize instruction with just the opcode
        let initialize_data = vec![0];  // Just use a simple byte code for the instruction

        let initialize_vrf_ix = Instruction::new_with_bytes(
            self.program_id,
            &initialize_data,
            vec![
                AccountMeta::new(self.payer.pubkey(), true),
                AccountMeta::new(vrf_account.pubkey(), false),
            ],
        );

        let recent_blockhash = self.client.get_latest_blockhash()?;
        let transaction = Transaction::new_signed_with_payer(
            &[create_vrf_account_ix, initialize_vrf_ix],
            Some(&self.payer.pubkey()),
            &[&self.payer, vrf_account],
            recent_blockhash,
        );

        let signature = self.client.send_and_confirm_transaction(&transaction)?;
        Ok(signature.to_string())
    }

    pub fn initialize_game(&self, game_account: &Keypair) -> Result<String, Box<dyn std::error::Error>> {
        // Calculate the rent-exempt minimum balance for the game account
        let game_account_size = std::mem::size_of::<GameAccountState>();
        let rent = self.client.get_minimum_balance_for_rent_exemption(game_account_size)?;

        // Create a transaction to create the game account and initialize it
        let create_game_account_ix = system_instruction::create_account(
            &self.payer.pubkey(),
            &game_account.pubkey(),
            rent,
            game_account_size as u64,
            &self.program_id,
        );

        // Initialize game instruction with opcode
        let initialize_game_data = vec![0, 0];  // First 0 means game instruction, second 0 means InitializeGame

        let initialize_game_ix = Instruction::new_with_bytes(
            self.program_id,
            &initialize_game_data,
            vec![
                AccountMeta::new(self.payer.pubkey(), true),
                AccountMeta::new(game_account.pubkey(), false),
            ],
        );

        let recent_blockhash = self.client.get_latest_blockhash()?;
        let transaction = Transaction::new_signed_with_payer(
            &[create_game_account_ix, initialize_game_ix],
            Some(&self.payer.pubkey()),
            &[&self.payer, game_account],
            recent_blockhash,
        );

        let signature = self.client.send_and_confirm_transaction(&transaction)?;
        Ok(signature.to_string())
    }

    pub fn request_randomness(
        &self,
        vrf_account: &Pubkey,
        game_account: &Pubkey,
        seed: Vec<u8>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // Create instruction data: opcode 1 for RequestRandomness + seed bytes
        let mut instruction_data = vec![1];
        instruction_data.extend_from_slice(&(seed.len() as u32).to_le_bytes());
        instruction_data.extend_from_slice(&seed);

        let request_randomness_ix = Instruction::new_with_bytes(
            self.program_id,
            &instruction_data,
            vec![
                AccountMeta::new(self.payer.pubkey(), true),
                AccountMeta::new(*vrf_account, false),
                AccountMeta::new(*game_account, false),
            ],
        );

        let recent_blockhash = self.client.get_latest_blockhash()?;
        let transaction = Transaction::new_signed_with_payer(
            &[request_randomness_ix],
            Some(&self.payer.pubkey()),
            &[&self.payer],
            recent_blockhash,
        );

        let signature = self.client.send_and_confirm_transaction(&transaction)?;
        Ok(signature.to_string())
    }

    pub fn fulfill_randomness(
        &self,
        vrf_account: &Pubkey,
        game_account: &Pubkey,
        proof: [u8; 64],
    ) -> Result<String, Box<dyn std::error::Error>> {
        // Create instruction data: opcode 2 for FulfillRandomness + proof bytes
        let mut instruction_data = vec![2];
        instruction_data.extend_from_slice(&proof);

        let fulfill_randomness_ix = Instruction::new_with_bytes(
            self.program_id,
            &instruction_data,
            vec![
                AccountMeta::new(self.payer.pubkey(), true),
                AccountMeta::new(*vrf_account, false),
                AccountMeta::new(*game_account, false),
            ],
        );

        let recent_blockhash = self.client.get_latest_blockhash()?;
        let transaction = Transaction::new_signed_with_payer(
            &[fulfill_randomness_ix],
            Some(&self.payer.pubkey()),
            &[&self.payer],
            recent_blockhash,
        );

        let signature = self.client.send_and_confirm_transaction(&transaction)?;
        Ok(signature.to_string())
    }

    pub fn submit_guess(
        &self,
        player: &Keypair,
        game_account: &Pubkey,
        vrf_account: &Pubkey,
        guess: u8,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // Create instruction data: first byte 0 for game instruction, second byte 1 for SubmitGuess, then the guess
        let instruction_data = vec![0, 1, guess];

        let submit_guess_ix = Instruction::new_with_bytes(
            self.program_id,
            &instruction_data,
            vec![
                AccountMeta::new(player.pubkey(), true),
                AccountMeta::new(*game_account, false),
                AccountMeta::new_readonly(*vrf_account, false),
            ],
        );

        let recent_blockhash = self.client.get_latest_blockhash()?;
        let transaction = Transaction::new_signed_with_payer(
            &[submit_guess_ix],
            Some(&player.pubkey()),
            &[player],
            recent_blockhash,
        );

        let signature = self.client.send_and_confirm_transaction(&transaction)?;
        Ok(signature.to_string())
    }

    pub fn get_vrf_account_data(&self, vrf_account: &Pubkey) -> Result<VrfAccountState, Box<dyn std::error::Error>> {
        let account_data = self.client.get_account_data(vrf_account)?;
        let vrf_state = VrfAccountState::try_from_slice(&account_data)?;
        Ok(vrf_state)
    }

    pub fn get_game_account_data(&self, game_account: &Pubkey) -> Result<GameAccountState, Box<dyn std::error::Error>> {
        let account_data = self.client.get_account_data(game_account)?;
        let game_state = GameAccountState::try_from_slice(&account_data)?;
        Ok(game_state)
    }

    pub fn wait_for_game_state(&self, game_account: &Pubkey, expected_state: GameState) -> Result<GameAccountState, Box<dyn std::error::Error>> {
        for _ in 0..30 {
            let game_state = self.get_game_account_data(game_account)?;
            if game_state.game_state == expected_state {
                return Ok(game_state);
            }
            thread::sleep(Duration::from_secs(2));
        }
        Err("Timed out waiting for game state change".into())
    }
} 