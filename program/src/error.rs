use {
    num_derive::FromPrimitive,
    num_traits::FromPrimitive,
    solana_program::{decode_error::DecodeError, program_error::ProgramError,program_error::PrintProgramError,msg},
    thiserror::Error,
};

#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum TransferError {
    #[error("Not owned by Transfer Program")]
    NotOwnedByTransfer,
}

pub type TransferErrorResult = Result<(), TransferError>;

impl From<TransferError> for ProgramError {
    fn from(e: TransferError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for TransferError {
    fn type_of() -> &'static str {
        "TransferError"
    }
}


impl PrintProgramError for TransferError {
    fn print<E>(&self)
    where
        E: 'static + std::error::Error + DecodeError<E> + PrintProgramError + FromPrimitive,
    {
        match self {
            TransferError::NotOwnedByTransfer => msg!("Error: account does not have the correct program id!"),
        }
    }
}
