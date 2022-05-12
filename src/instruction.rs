use borsh::BorshDeserialize;
use solana_program::program_error::ProgramError;

use crate::state::{TransferInput, WithdrawInput};
pub enum TransferInstruction{ 
    /// Create a transfer with a escrow account created and funded by sender
    /// account should have a total_lamport= program_rent_account+amount_to_send.
    ///
    /// Accounts expected:
    ///
    /// `[writable]` escrow account, it will hold all necessary info about the trade.
    /// `[signer]` sender account
    /// `[]` receiver account
    CreateTranfer(TransferInput),


    /// Withdraw for receiver
    ///
    /// Accounts expected:
    ///
    /// `[writable]` escrow account, it will hold all necessary info about the trade.
    /// `[signer]` receiver account
    Withdraw(WithdrawInput)
}

impl TransferInstruction{
    pub fn unpack(instruction_data: &[u8]) -> Result<Self, ProgramError>{
        let (tag, data) = instruction_data
            .split_first()
            .ok_or(ProgramError::InvalidInstructionData)?;
        match tag {
            0 => Ok(TransferInstruction::CreateTranfer(
                TransferInput::try_from_slice(data)?,
            )),
            1 => Ok(TransferInstruction::Withdraw(
                WithdrawInput::try_from_slice(data)?,
            )),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}