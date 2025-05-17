use borsh::{BorshDeserialize, BorshSerialize};
use clap::Parser;
use hex::{encode, ToHex};
use serde::Serialize;
use solana_instruction::{AccountMeta, Instruction as V2_instruction};
use solana_message::{legacy, v0::Message, VersionedMessage};
use solana_program::{hash::Hash, instruction::Instruction, pubkey::Pubkey};
use solana_pubkey::Pubkey as V2_pubkey;
use solana_transaction::{versioned::VersionedTransaction, Transaction};
use solana_zk_offers::zk_offers::{
    approve_delegation, cancel_delegation, compute_maker_src_account, compute_offer_pda,
    create_offer_transaction, PublicValuesStruct,
};

use sp1_sdk::{ProverClient, SP1Stdin};
use spl_token::ID as TOKEN_PROGRAM_ID;
use std::str::FromStr;

/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM.
pub const ZKVM_ELF: &[u8] = include_bytes!("../../../elf/riscv32im-succinct-zkvm-elf");

fn parse_pubkey(s: &str) -> Result<Pubkey, String> {
    Pubkey::from_str(s).map_err(|e| format!("Invalid Pubkey: {}", e))
}

fn parse_public_values(s: &str) -> Result<PublicValuesStruct, String> {
    let parts: Vec<&str> = s.split(',').map(|s| s.trim()).collect();
    if parts.len() != 5 {
        return Err("Expected 5 comma-separated values".to_string());
    }

    let maker_mint = parse_pubkey(parts[0])?;
    let taker_mint = if parts[1].eq_ignore_ascii_case("null") {
        None
    } else {
        Some(parse_pubkey(parts[1])?)
    };
    let is_native = parts[2]
        .parse::<bool>()
        .map_err(|_| "Invalid boolean value for is_native".to_string())?;
    let maker_size = parts[3]
        .parse::<u64>()
        .map_err(|_| "Invalid integer value for maker_size".to_string())?;
    let taker_size = if parts[4].eq_ignore_ascii_case("null") {
        None
    } else {
        Some(
            parts[4]
                .parse::<u64>()
                .map_err(|_| "Invalid integer value for taker_size".to_string())?,
        )
    };

    Ok(PublicValuesStruct {
        maker_mint,
        taker_mint,
        is_native,
        maker_size,
        taker_size,
    })
}

/// The arguments for the command.
#[derive(Clone, Parser, Debug, BorshDeserialize, BorshSerialize, serde::Serialize)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(long)]
    execute: bool,

    #[clap(long)]
    prove: bool,

    #[clap(long, value_parser = parse_public_values)]
    public_values: PublicValuesStruct,

    #[clap(long, value_parser = parse_pubkey)]
    maker_wallet: Pubkey,

    #[clap(long, default_value = "So11111111111111111111111111111111111111112")]
    program_id: Pubkey,
}

struct Application {
    args: Args,
}

impl Application {
    /// Initialize the application
    fn new() -> Self {
        // Setup the logger
        sp1_sdk::utils::setup_logger();

        // Parse command-line arguments
        let args = Args::parse();

        // Validate arguments
        if args.execute == args.prove {
            eprintln!("Error: You must specify either --execute or --prove");
            std::process::exit(1);
        }

        Self { args }
    }

    /// Compute the PDA
    fn compute_pda(&self) -> Pubkey {
        compute_offer_pda(
            &self.args.program_id,
            &self.args.maker_wallet,
            &self.args.public_values.maker_mint,
        )
    }

    fn approve_listing(&self) -> Instruction {
        // Compute maker_src_account from maker wallet and maker mint

        let maker_src_account: Pubkey = compute_maker_src_account(
            &self.args.program_id,
            &self.args.maker_wallet,
            &self.args.public_values.maker_mint,
        );

        let pda: &Pubkey = &self.compute_pda();
        approve_delegation(
            &self.args.maker_wallet,
            &maker_src_account,
            pda,
            self.args.public_values.maker_size,
            &TOKEN_PROGRAM_ID,
        )
    }

    /// Build the transaction
    fn build_transaction(&self, offer: Vec<u8>) -> Instruction {
        let pda: Pubkey = self.compute_pda();
        create_offer_transaction(&self.args.program_id, offer, pda)
    }

    /// Run the application
    fn run(&self) {
        let offer = self.approve_listing();
        // let transaction = self.build_transaction(pda, offer);

        let data_buffer = hex::encode(&offer.data);
        println!("Created Offer Instruction: {:?}", data_buffer);

        // For a dummy transaction, use a dummy signer
        let dummy_signer = Pubkey::new_unique();
        let recent_blockhash = Hash::from_str("11111111111111111111111111111111").unwrap(); // Replace with actual blockhash

        let v2_instruction = V2_instruction {
            program_id: offer.program_id,
            accounts: offer
                .accounts
                .iter()
                .map(|account| AccountMeta {
                    pubkey: account.pubkey,
                    is_signer: account.is_signer,
                    is_writable: account.is_writable,
                })
                .collect(),
            data: offer.data.clone(),
        };
        let result = Message::try_compile(&dummy_signer, &[v2_instruction], &[], recent_blockhash);

        // Handle the Result
        let message: Message = match result {
            Ok(msg) => msg,
            Err(err) => {
                eprintln!("Failed to compile message: {}", err);
                return;
            }
        };

        let versioned_message: VersionedMessage = solana_message::VersionedMessage::V0(message);
        let transaction = VersionedTransaction {
            signatures: [].to_vec(),
            message: versioned_message,
        };

        println!("", );

        println!("Transaction: {:?}", transaction);
    }
}
fn main() {
    let app = Application::new(); // Initialize application
    app.run(); // Execute main logic

    let client = ProverClient::new();

    // Serialize the public values into the SP1Stdin format.
    let mut stdin = SP1Stdin::new();

    if app.args.execute {
        // Execute the program
        let (output, report) = client.execute(ZKVM_ELF, stdin).run().unwrap();
        println!("Program executed successfully.");
        let mut output_reader: &[u8] = output.as_slice();

        // Deserialize the output into PublicValuesStruct
        let decoded: PublicValuesStruct = BorshDeserialize::deserialize(&mut output_reader)
            .expect("Failed to deserialize with Borsh");
        println!("Decoded PublicValuesStruct: {:?}", decoded);
    } else {
        // Setup the program for proving.
        let (pk, vk) = client.setup(ZKVM_ELF);

        // Generate the proof
        let proof = client
            .prove(&pk, stdin)
            .run()
            .expect("Failed to generate proof");

        println!("Successfully generated proof!");

        // Verify the proof.
        client.verify(&proof, &vk).expect("Failed to verify proof");
        println!("Successfully verified proof!");
    }
}
