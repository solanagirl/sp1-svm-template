pub mod anon_offers {
    use borsh::{BorshDeserialize, BorshSerialize};
    use solana_pubkey::Pubkey;

    #[derive(BorshSerialize, BorshDeserialize, Debug)]
    pub struct PublicValuesStruct {
        pub n: u32,
        pub a: u32,
        pub b: u32,
    }

    pub struct OfferInfoStruct<'a> {
        program_id: &'a Pubkey,
        maker_wallet: &'a Pubkey,
        taker_wallet: &'a Pubkey,
        maker_src_account: &'a Pubkey,
        maker_dst_account: &'a Pubkey,
        taker_src_account: &'a Pubkey,
        taker_dst_account: &'a Pubkey,
        maker_mint: &'a Pubkey,
        taker_mint: &'a Pubkey,
        authority: &'a Pubkey,
        token_program_id: &'a Pubkey,
        is_native: bool,
        maker_size: u64,
        taker_size: u64,
        bump_seed: u8,
    }
    pub fn place_offer(n: u32) -> (u32, u32) {
        let mut a = 0u32;
        let mut b = 1u32;

        for _ in 0..n {
            let temp = a + b;
            a = b;
            b = temp;
        }

        (a, b)
    }
}
