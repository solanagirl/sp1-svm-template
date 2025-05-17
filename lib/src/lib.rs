use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program, sysvar,
};
use spl_token::instruction::approve;
use std::str::FromStr;

pub mod zk_offers {
    use super::*;

    /// Struct representing the offer details for off-chain advertisement.
    #[derive(BorshSerialize, BorshDeserialize)]
    struct PrivateOfferStruct {
        pub maker_wallet: Pubkey,
        pub taker_wallet: Pubkey,
        pub maker_src_account: Pubkey,
        pub maker_dst_account: Pubkey,
        pub taker_src_account: Pubkey,
        pub taker_dst_account: Pubkey,
        pub bump_seed: u8,
    }
    #[derive(Clone, BorshSerialize, BorshDeserialize, Debug, serde::Serialize)]
    pub struct PublicValuesStruct {
        pub maker_mint: Pubkey,
        pub taker_mint: Option<Pubkey>,
        pub is_native: bool,
        pub maker_size: u64,
        pub taker_size: Option<u64>,
    }

    impl FromStr for PublicValuesStruct {
        type Err = String;
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let parts: Vec<&str> = s.split(',').collect();
            if parts.len() != 5 {
                return Err("Expected 5 comma-separated values".to_string());
            }

            let maker_mint = parts[0].parse::<Pubkey>().map_err(|e| e.to_string())?;
            let taker_mint = if parts[1].is_empty() {
                None
            } else {
                Some(parts[1].parse::<Pubkey>().map_err(|e| e.to_string())?)
            };
            let is_native = parts[2].parse::<bool>().map_err(|e| e.to_string())?;
            let maker_size = parts[3].parse::<u64>().map_err(|e| e.to_string())?;
            let taker_size = if parts[4].is_empty() {
                None
            } else {
                Some(parts[4].parse::<u64>().map_err(|e| e.to_string())?)
            };

            Ok(PublicValuesStruct {
                maker_mint,
                taker_mint,
                is_native,
                maker_size,
                taker_size,
            })
        }
    }

    #[derive(BorshSerialize, BorshDeserialize)]
    pub struct OfferStruct {
        private_offer: PrivateOfferStruct,
        pub public_values: PublicValuesStruct,
    }

    pub fn compute_offer_pda(
        program_id: &Pubkey,
        maker_wallet: &Pubkey,
        maker_mint: &Pubkey,
    ) -> Pubkey {
        let seeds = &[maker_wallet.as_ref(), maker_mint.as_ref(), b"offer"];
        let (pubkey, _) = Pubkey::find_program_address(seeds, program_id); // Ignore the bump seed
        pubkey
    }

    pub fn compute_maker_src_account(
        program_id: &Pubkey,
        maker_wallet: &Pubkey,
        maker_mint: &Pubkey,
    ) -> Pubkey {
        let seeds = &[maker_wallet.as_ref(), maker_mint.as_ref(), b"zk-ata"];
        let (pda, _) = Pubkey::find_program_address(seeds, program_id);
        pda
    }

    /// Approve token delegation to the PDA.
    pub fn approve_delegation(
        maker_wallet: &Pubkey,
        maker_src_account: &Pubkey,
        pda: &Pubkey,
        maker_size: u64,
        token_program_id: &Pubkey,
    ) -> Instruction {
        approve(
            token_program_id,
            maker_src_account,
            pda,
            maker_wallet,
            &[],
            maker_size,
        )
        .unwrap()
    }

    /// Create an offer transaction for the taker.
    pub fn create_offer_transaction(
        program_id: &Pubkey,
        offer: Vec<u8>,
        pda: Pubkey,
    ) -> Instruction {
        let OfferStruct {
            private_offer,
            public_values,
        } = OfferStruct::try_from_slice(&offer).expect("Failed to deserialize offer_info");

        let accounts = vec![
            AccountMeta::new(private_offer.maker_wallet, true),
            AccountMeta::new(private_offer.taker_wallet, true),
            AccountMeta::new(private_offer.maker_src_account, false),
            AccountMeta::new(private_offer.maker_dst_account, false),
            AccountMeta::new(private_offer.taker_src_account, false),
            AccountMeta::new(private_offer.taker_dst_account, false),
            AccountMeta::new_readonly(public_values.maker_mint, false),
            AccountMeta::new_readonly(public_values.taker_mint.unwrap_or(Pubkey::default()), false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
            AccountMeta::new_readonly(system_program::id(), false),
            AccountMeta::new_readonly(pda, false),
        ];

        Instruction {
            program_id: *program_id,
            accounts,
            data: offer,
        }
    }

    /// Cancel the offer by revoking the delegation.
    pub fn cancel_delegation(
        maker_wallet: &Pubkey,
        maker_src_account: &Pubkey,
        token_program_id: &Pubkey,
    ) -> Instruction {
        spl_token::instruction::revoke(token_program_id, maker_src_account, maker_wallet, &[])
            .unwrap()
    }
}
