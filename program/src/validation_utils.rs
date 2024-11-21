use {
    solana_program::{
        entrypoint_deprecated::ProgramResult,
        program_error::ProgramError,
        program_pack::{IsInitialized, Pack, Sealed},
        sysvar::slot_history::AccountInfo,
    },
    solana_pubkey::Pubkey,
    spl_associated_token_account::get_associated_token_address,
    spl_token::{self, state::Account},
};

pub fn assert_is_ata(ata: &AccountInfo, wallet: &Pubkey, mint: &Pubkey) -> ProgramResult {
    // Ensure the account is owned by the SPL Token program
    assert_owned_by(ata, &spl_token::id())?;

    // Ensure the account is initialized
    let ata_account: Account = assert_initialized(ata)?;

    // Validate the account owner
    assert_keys_equal(ata_account.owner, *wallet)?;

    let expected_ata_address = get_associated_token_address(wallet, mint);
    assert_keys_equal(expected_ata_address, *ata.key)?;

    Ok(())
}

pub fn assert_keys_equal(key1: Pubkey, key2: Pubkey) -> ProgramResult {
    assert_eq!(key1, key2, "PublicKeyMismatch");
    Ok(())
}

pub fn assert_initialized<T: Pack + IsInitialized>(
    account_info: &AccountInfo,
) -> Result<T, ProgramError> {
    // Try to deserialize the account
    let account: T = T::unpack_unchecked(&account_info.data.borrow())
        .map_err(|_e| ProgramError::UninitializedAccount)?;

    // Check if the account is initialized
    if !account.is_initialized() {
        Err(ProgramError::UninitializedAccount)
    } else {
        Ok(account)
    }
}

pub fn assert_owned_by(account: &AccountInfo, owner: &Pubkey) -> ProgramResult {
    assert_eq!(account.owner, owner, "IncorrectOwner");
    Ok(())
}
