// These two lines are necessary for the program to properly compile.

#![no_main]
sp1_zkvm::entrypoint!(main);

use borsh::BorshSerialize;
use solana_program::pubkey::Pubkey;
use solana_zk_offers::zk_offers::PublicValuesStruct;
use std::process;

fn read_pubkey() -> Pubkey {
    let input = sp1_zkvm::io::read::<String>(); // Read a string
    let bytes = bs58::decode(&input).into_vec().unwrap_or_else(|_| {
        eprintln!("Error: Failed to decode Base58 string");
        process::exit(1);
    });

    if bytes.len() != 32 {
        eprintln!("Error: Decoded Pubkey is not 32 bytes long");
        process::exit(1);
    }

    // Convert the byte vector into a fixed-size array
    let mut byte_array = [0u8; 32];
    byte_array.copy_from_slice(&bytes);

    // Create a Pubkey from the byte array
    Pubkey::new_from_array(byte_array)
}

fn read_input() -> u64 {
    sp1_zkvm::io::read::<u64>()
}

fn serialize_public_values(public_values: &PublicValuesStruct) -> Vec<u8> {
    let mut bytes = Vec::new();
    public_values
        .serialize(&mut bytes)
        .expect("Failed to serialize public values");
    bytes
}

/// Commit public values to the zkVM
fn commit_public_values(bytes: &[u8]) {
    sp1_zkvm::io::commit_slice(bytes);
}

pub fn main() {
    // Read an input to the program.
    //
    // Behind the scenes, this compiles down to a custom system call which handles reading inputs
    // from the prover.

    let maker_mint: Pubkey = read_pubkey();

    let maker_size: u64 = read_input();

    // Encode the public values of the program.
    let public_values: PublicValuesStruct = PublicValuesStruct {
        maker_mint,
        maker_size,
        is_native: true,
        taker_mint: None,
        taker_size: None,
    };
    let mut bytes = Vec::new();
    public_values
        .serialize(&mut bytes)
        .expect("Failed to serialize");

    // Commit to the public values of the program. The final proof will have a commitment to all the
    // bytes that were committed to.
    sp1_zkvm::io::commit_slice(&bytes);
}
