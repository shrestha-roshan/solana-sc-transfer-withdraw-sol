use crate::{
    instruction::TransferInstruction,
    state::{TransferInput, WithdrawInput, TransferData},
};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::{rent::Rent, Sysvar},
};
pub struct Processor;

impl Processor{
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8]
    ) -> ProgramResult {
        let instruction = TransferInstruction::unpack(instruction_data)?;
        match instruction {
            TransferInstruction::CreateTranfer(data) => {
                Self::process_create_transfer(program_id, accounts, data)
            }
            TransferInstruction::Withdraw(data) => {
                Self::process_withdraw(program_id, accounts, data)
            }
        }
    }

    fn process_create_transfer(
        _program_id: &Pubkey,
        accounts: &[AccountInfo],
        data: TransferInput, 
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let escrow_account = next_account_info(account_info_iter)?;
        let sender_account = next_account_info(account_info_iter)?;
        let receiver_account = next_account_info(account_info_iter)?;       

        if !sender_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        if *receiver_account.key != data.receiver {
            return Err(ProgramError::InvalidAccountData);
        }

        **sender_account.try_borrow_mut_lamports()? -= data.amount_to_send;
        **escrow_account.try_borrow_mut_lamports()? += data.amount_to_send;

        if data.amount_to_send + Rent::get()?.minimum_balance(escrow_account.data_len()) != **escrow_account.lamports.borrow() { 
            return Err(ProgramError::InsufficientFunds)
        }

        let escrow_data = TransferData::new(data, *sender_account.key);

        escrow_data.serialize(&mut &mut escrow_account.data.borrow_mut()[..])?; // how does this work?
        Ok(())
    }

    fn process_withdraw(
        _program_id: &Pubkey,
        accounts: &[AccountInfo],
        data: WithdrawInput,
    ) -> ProgramResult{
        let account_info_iter = &mut accounts.iter();
        let escrow_account = next_account_info(account_info_iter)?;
        let receiver_account = next_account_info(account_info_iter)?;

        let escrow_data = TransferData::try_from_slice(&escrow_account.data.borrow()).expect("Failed to seriallize");

        if *receiver_account.key != escrow_data.receiver {
            return Err(ProgramError::IllegalOwner);
        }

        if !receiver_account.is_signer { // Reciever signer???
            return Err(ProgramError::MissingRequiredSignature);
        }

        if escrow_data.start_time + (24*60*60) > Clock::get()?.unix_timestamp{ // 24 hours not passed yet
            return Err(ProgramError::Custom(999)) 
        }

        **escrow_account.try_borrow_mut_lamports()? -= data.amount;
        **receiver_account.try_borrow_mut_lamports()? += data.amount;
        Ok(())
    }
}