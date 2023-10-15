use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

/// Define the type of state stored in accounts
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct SPLVault {
    /// number of greetings
    pub state: u8,
    pub mint: Pubkey,
    pub amount: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct NonceBank {
    pub state: u8,
    pub nonce: u64,
    pub withdrawer: Pubkey,
}

pub const SPL_VAULT_LEN : usize= 32+8+1;

pub const AUTHORIZED_PUBLIC_KEY: &[u8;64] = b"fc12ad814631ba689f7abe671016f75c54c607f082ae6b0881fac0abeda21781";
