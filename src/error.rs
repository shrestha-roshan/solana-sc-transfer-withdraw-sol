use solana_program::{program_error::ProgramError};
use thiserror::Error;

/// Errors that may be returned by the Token program.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum TokenError {
    #[error("Lamport balance below rent-exempt threshold")]
    NotRentExempt,
    #[error("Public key mismatched")]
    PublicKeyMismatch,
    #[error("Account not associated with Escrow")]
    EscrowMismatch
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum EscrowError {
    #[error("24 hours not passed yet!")]
    WithdrawTimeLimitNotExceed,
}

impl From<TokenError> for ProgramError {
    fn from(e: TokenError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl From<EscrowError> for ProgramError {
    fn from(e: EscrowError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
