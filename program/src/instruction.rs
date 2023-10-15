use {
    borsh::{BorshDeserialize, BorshSchema, BorshSerialize},
    solana_program::{
        borsh::try_from_slice_unchecked,
        instruction::{AccountMeta, Instruction},
        program_error::ProgramError,
        pubkey::Pubkey,
    },
};

/// Instructions supported by the generic Name Registry program
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq, BorshSchema)]
pub enum TransferInstruction {
    /// Accounts expected by this instruction:
    ///
    ///   * Single owner/delegate
    ///   0. `[writable]` The source account.
    ///   1. `[writable]` The valut account.
    ///   2. `[writable]` The valut spl token account.
    ///   3. `[signer]` The source account's owner/delegate.
    ///   4. `[]` the SPL's mint account.
    ///   5. `[]` the SPL Token Program account.
    ///   6. `[]` the system program account.
    ///   7. `[]` the rent program account.
    Transfer{
        to: String,
        amount: u64,
    },

    /// Accounts expected by this instruction:
    ///
    ///   * Single owner/delegate
    ///   0. `[writable]` The valut account.
    ///   1. `[writable]` The valut spl token account.
    ///   2. `[signer]` The withdraw account's spl token account.
    ///   3. `[]` the SPL's mint account.
    ///   4. `[]` the SPL Token Program account.
    Withdraw {
        amount: u64,
        nonce: u64,
        slot:u64,
        signature: String,
    }
    
}
