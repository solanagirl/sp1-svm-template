#![cfg(all(target_os = "unknown", not(feature = "no-entrypoint")))]

use {
    crate::processor::Processor,
    stateless_asks::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey},
};

stateless_asks::entrypoint!(process_instruction);
fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    Processor::process(program_id, accounts, instruction_data)
}
