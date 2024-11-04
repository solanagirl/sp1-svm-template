pub mod fibonacci_lib {
    use borsh::{BorshDeserialize, BorshSerialize};

    #[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
    pub struct PublicValuesStruct {
        pub n: u32,
        pub a: u32,
        pub b: u32,
    }

    pub fn fibonacci(n: u32) -> (u32, u32) {
        let mut a = 0u32;
        let mut b = 1u32;
        for _ in 0..n {
            let c = a.wrapping_add(b);
            a = b;
            b = c;
        }
        (a, b)
    }
}
