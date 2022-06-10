use solana_program::program_error::ProgramError;
use solana_program::{
    msg, 
    program_error,
};

use crate::state::{TransferInput, WithdrawInput, InitTokenInput, WithdrawTokenInput};
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
    Withdraw(WithdrawInput),

    TransferToken(InitTokenInput),

    WithdrawToken(WithdrawTokenInput),
}

impl TransferInstruction{
    pub fn unpack(instruction_data: &[u8]) -> Result<Self, ProgramError>{
        msg!("Unpacking Instruction Data");
        let (tag, rest) = instruction_data
            .split_first()
            .ok_or(ProgramError::InvalidInstructionData)?;
        match tag {
            0 => {
                let (start_time, rest) = rest.split_at(8);
                let (amount_to_send, _rest) = rest.split_at(8);
                msg!("start time byte: {:?}", start_time);
                msg!("amount byte: {:?}", amount_to_send);

                let start_time = start_time.try_into().map(u64::from_le_bytes).or(Err(program_error::INVALID_INSTRUCTION_DATA))?;
                let amount_to_send = amount_to_send.try_into().map(u64::from_le_bytes).or(Err(program_error::INVALID_INSTRUCTION_DATA))?;
                
                Ok(TransferInstruction::CreateTranfer(TransferInput{start_time, amount_to_send}))
            },
            1 =>{
                let (amount, _rest) = rest.split_at(8);

                let amount = amount.try_into().map(u64::from_le_bytes).or(Err(program_error::INVALID_INSTRUCTION_DATA))?;

                Ok(TransferInstruction::Withdraw(WithdrawInput{amount}))
            },
            2 => {
                let (start_time, rest) = rest.split_at(8);
                let (amount, _rest) = rest.split_at(8);
                
                let start_time = start_time.try_into().map(u64::from_le_bytes).or(Err(program_error::INVALID_INSTRUCTION_DATA))?;
                let amount = amount.try_into().map(u64::from_le_bytes).or(Err(program_error::INVALID_INSTRUCTION_DATA))?;

                Ok(TransferInstruction::TransferToken(InitTokenInput{start_time , amount}))
            }

            3 => {
                let (amount, _rest) = rest.split_at(8);
                
                let amount = amount.try_into().map(u64::from_le_bytes).or(Err(program_error::INVALID_INSTRUCTION_DATA))?;

                Ok(TransferInstruction::WithdrawToken(WithdrawTokenInput{amount}))
            }
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }

}