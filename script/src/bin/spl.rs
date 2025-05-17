//! An end-to-end example of using the SP1 SDK to generate a proof of a program that can have an
//! offer-Compatible proof generated which can be verified on-chain.
//!
//! You can run this script using the following command:
//! ```shell
//! RUST_LOG=info cargo run --release --bin offer
//! ```

use borsh::{BorshDeserialize, BorshSerialize};
use clap::Parser;
use solana_program::pubkey::Pubkey;
use solana_zk_offers::zk_offers::PublicValuesStruct;
use sp1_sdk::{HashableKey, ProverClient, SP1ProofWithPublicValues, SP1Stdin, SP1VerifyingKey};
use std::path::PathBuf;
use std::{fs, io::stdin};
///
///  linkable format) file for the Succinct RISC-V zkVM.
pub const ZKVM_ELF: &[u8] = include_bytes!("../../../elf/riscv32im-succinct-zkvm-elf");

/// The arguments for the offer command.
#[derive(Parser, Debug, Clone, BorshDeserialize, BorshSerialize)]
#[clap()]
pub struct ZKAskArgs {
    pub public_values: PublicValuesStruct,
    maker_wallet: Pubkey,
    maker_size: u64,
}

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize, serde::Serialize)]
struct SP1ZK {
    vkey: String,
    public_values: PublicValuesStruct,
    proof: String,
}

fn main() {
    // Setup the logger.
    sp1_sdk::utils::setup_logger();

    // Parse the command line arguments.
    let args: ZKAskArgs = ZKAskArgs::parse();

    // Setup the prover client.
    let client = ProverClient::new();

    // Setup the program.
    let (pk, vk) = client.setup(ZKVM_ELF);

    // Serialize the public values into the SP1Stdin format.
    // let mut stdin: SP1Stdin = SP1Stdin::new();
    // stdin.write(
    //     args.serialize()
    //         .try_to_vec()
    //         .expect("Failed to serialize public values"),
    // );

    println!("ZK Offer for Mint: {:#?}", args.public_values);

    let sp1_stdin: SP1Stdin = SP1Stdin::new(); // Wrap Stdin in SP1Stdin

    // Generate the proof.
    let proof = client
        .prove(&pk, sp1_stdin)
        .plonk()
        .run()
        .expect("failed to generate proof");

    create_plonk_fixture(&proof, &vk);
}

/// Create a fixture for the given proof.
fn create_plonk_fixture(proof: &SP1ProofWithPublicValues, vk: &SP1VerifyingKey) {
    // Deserialize the public values.
    let bytes: &[u8] = proof.public_values.as_slice();
    let serialized_data = bytes.to_vec();
    let data = PublicValuesStruct::try_from_slice(&serialized_data).unwrap();

    // Create the testing fixture so we can test things end-to-end.
    let fixture = SP1ZK {
        vkey: vk.bytes32().to_string(),
        public_values: data,
        proof: hex::encode(proof.bytes()),
    };

    // The verification key is used to verify that the proof corresponds to the execution of the
    // program on the given input.
    //
    // Note that the verification key stays the same regardless of the input.
    println!("Verification Key: {}", fixture.vkey);

    // The public values are the values which are publicly committed to by the zkVM.
    //
    // If you need to expose the inputs or outputs of your program, you should commit them in
    // the public values.
    println!("Public Values: {:#?}", fixture.public_values);

    // The proof proves to the verifier that the program was executed with some inputs that led to
    // the give public values.
    println!("Proof Bytes: {}", fixture.proof);

    // Save the fixture to a file.
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../contracts/src/fixtures");
    std::fs::create_dir_all(&fixture_path).expect("failed to create fixture path");

    // Serialize to JSON
    let serialized_fixture =
        serde_json::to_string_pretty(&fixture).expect("Failed to serialize fixture to JSON");

    // Write JSON data to a file
    fs::write(fixture_path.join("fixture.json"), serialized_fixture)
        .expect("Failed to write fixture");
}
