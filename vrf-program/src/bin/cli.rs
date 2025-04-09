use clap::{App, Arg, SubCommand};
use rand::Rng;
use solana_program::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, read_keypair_file};
use solana_sdk::signature::Signer;
use std::{error::Error, str::FromStr};
use vrf_program::{GameState, VrfClient};

fn main() -> Result<(), Box<dyn Error>> {
    let matches = App::new("VRF Game CLI")
        .version("1.0")
        .author("Your Name")
        .about("A CLI for the VRF Game on Solana")
        .subcommand(
            SubCommand::with_name("init")
                .about("Initialize the VRF and game accounts")
                .arg(
                    Arg::with_name("keypair")
                        .short("k")
                        .long("keypair")
                        .value_name("KEYPAIR")
                        .help("Path to keypair file")
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    Arg::with_name("program_id")
                        .short("p")
                        .long("program-id")
                        .value_name("PUBKEY")
                        .help("The program ID of the deployed VRF program")
                        .takes_value(true)
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("play")
                .about("Play the VRF guessing game")
                .arg(
                    Arg::with_name("keypair")
                        .short("k")
                        .long("keypair")
                        .value_name("KEYPAIR")
                        .help("Path to keypair file")
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    Arg::with_name("program_id")
                        .short("p")
                        .long("program-id")
                        .value_name("PUBKEY")
                        .help("The program ID of the deployed VRF program")
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    Arg::with_name("vrf_account")
                        .short("v")
                        .long("vrf-account")
                        .value_name("PUBKEY")
                        .help("The VRF account pubkey")
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    Arg::with_name("game_account")
                        .short("g")
                        .long("game-account")
                        .value_name("PUBKEY")
                        .help("The game account pubkey")
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    Arg::with_name("guess")
                        .short("n")
                        .long("guess")
                        .value_name("NUMBER")
                        .help("Your guess (0-99)")
                        .takes_value(true)
                        .required(true),
                ),
        )
        .get_matches();

    // Connect to Solana devnet
    let url = "https://api.devnet.solana.com";

    if let Some(matches) = matches.subcommand_matches("init") {
        let keypair_path = matches.value_of("keypair").unwrap();
        let payer = read_keypair_file(keypair_path)
            .map_err(|_| format!("Failed to read keypair from {}", keypair_path))?;
        
        let program_id = Pubkey::from_str(matches.value_of("program_id").unwrap())?;
        
        let client = VrfClient::new(url, payer, program_id);
        
        // Generate new keypairs for VRF and game accounts
        let vrf_account = Keypair::new();
        let game_account = Keypair::new();
        
        println!("Initializing VRF account: {}", vrf_account.pubkey());
        let signature = client.initialize_vrf(&vrf_account)?;
        println!("VRF account initialized. Signature: {}", signature);
        
        println!("Initializing game account: {}", game_account.pubkey());
        let signature = client.initialize_game(&game_account)?;
        println!("Game account initialized. Signature: {}", signature);
        
        println!("\nAccounts created successfully!");
        println!("VRF Account: {}", vrf_account.pubkey());
        println!("Game Account: {}", game_account.pubkey());
        println!("\nUse these account addresses with the 'play' command to start playing.");
        
    } else if let Some(matches) = matches.subcommand_matches("play") {
        let keypair_path = matches.value_of("keypair").unwrap();
        let payer = read_keypair_file(keypair_path)
            .map_err(|_| format!("Failed to read keypair from {}", keypair_path))?;
        
        let program_id = Pubkey::from_str(matches.value_of("program_id").unwrap())?;
        let vrf_account = Pubkey::from_str(matches.value_of("vrf_account").unwrap())?;
        let game_account = Pubkey::from_str(matches.value_of("game_account").unwrap())?;
        let guess = matches.value_of("guess").unwrap().parse::<u8>()?;
        
        if guess > 99 {
            return Err("Guess must be between 0 and 99".into());
        }
        
        let client = VrfClient::new(url, payer, program_id);
        
        // Generate a random seed
        let mut rng = rand::thread_rng();
        let seed: Vec<u8> = (0..32).map(|_| rng.gen::<u8>()).collect();
        
        println!("Requesting randomness with seed: {:?}", seed);
        let signature = client.request_randomness(&vrf_account, &game_account, seed.clone())?;
        println!("Randomness requested. Signature: {}", signature);
        
        // In a real VRF implementation, this would be done by an oracle or some other entity
        // that has the secret key. For our demo, we're doing it in the client.
        println!("Fulfilling randomness with dummy proof...");
        let dummy_proof = [1u8; 64]; // In a real implementation, this would be a valid VRF proof
        let signature = client.fulfill_randomness(&vrf_account, &game_account, dummy_proof)?;
        println!("Randomness fulfilled. Signature: {}", signature);
        
        // Wait for the game state to be updated
        println!("Waiting for game state to be ready for player guess...");
        client.wait_for_game_state(&game_account, GameState::AwaitingPlayerGuess)?;
        
        println!("Submitting guess: {}", guess);
        let signature = client.submit_guess(&client.payer, &game_account, &vrf_account, guess)?;
        println!("Guess submitted. Signature: {}", signature);
        
        // Wait for the game to complete and get the result
        println!("Waiting for game to complete...");
        let final_state = client.wait_for_game_state(&game_account, GameState::GameComplete)?;
        
        match final_state.result {
            Some(result) => {
                match result {
                    vrf_program::GameResult::Win => println!("Congratulations! You won!"),
                    vrf_program::GameResult::Lose => println!("Sorry, you lost. Better luck next time!"),
                }
                
                // Get the VRF account data to see the randomness
                let vrf_data = client.get_vrf_account_data(&vrf_account)?;
                if let Some(randomness) = vrf_data.randomness {
                    let game_number = vrf_program::calculate_game_number(&randomness);
                    println!("The winning number was: {}", game_number);
                }
            },
            None => println!("Game did not complete properly"),
        }
    }

    Ok(())
} 