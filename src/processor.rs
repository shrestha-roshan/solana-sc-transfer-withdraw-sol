use crate::{
    instruction::TransferInstruction,
    state::{TransferInput, WithdrawInput, Escrow},
};


use super::error::{TokenError, EscrowError};
use borsh::{BorshDeserialize, BorshSerialize};

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
    msg, program_pack::Pack, program::{invoke_signed, invoke},
    system_instruction,
    sysvar::{rent::Rent, Sysvar},
};

pub struct Processor;

impl Processor{
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8]
    ) -> ProgramResult {

        msg!("starts unpacking");

        let instruction = TransferInstruction::unpack(instruction_data)?;

        msg!("unpacking done");

        match instruction {
            TransferInstruction::CreateTranfer(TransferInput { start_time, amount_to_send }) => {
                Self::process_create_transfer(program_id, accounts, start_time, amount_to_send)
            }
            TransferInstruction::Withdraw(WithdrawInput{amount}) => {
                Self::process_withdraw(program_id, accounts, amount)
            }
        }
    }

    fn process_create_transfer(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        start_time: u64, 
        amount_to_send: u64
    ) -> ProgramResult {

        // Get the rent sysvar via syscall
        //let rent = Rent::get()?; //
        
        msg!("INTO create transfer!");
        msg!("start: {:?}", start_time);
        msg!("amount: {:?}", amount_to_send);


        let account_info_iter = &mut accounts.iter();
        let escrow_account = next_account_info(account_info_iter)?;
        let sender_account = next_account_info(account_info_iter)?;
        let receiver_account = next_account_info(account_info_iter)?;
        let system_program = next_account_info(account_info_iter)?;       

        if !sender_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let (_account_address, bump_seed) = Pubkey::find_program_address(
            &[&sender_account.key.to_bytes()],
            program_id,
        );
        let pda_signer_seeds: &[&[_]] = &[
            &sender_account.key.to_bytes(),
            &[bump_seed],
        ];

        msg!("Creating Escrow Account");
        invoke_signed(
            &system_instruction::create_account(
                sender_account.key, 
                escrow_account.key, 
                Rent::get()?.minimum_balance(std::mem::size_of::<Escrow>()),
                81,
                program_id
            ),
            &[sender_account.clone(), escrow_account.clone(),system_program.clone()],
            &[pda_signer_seeds]
        )?;

        msg!("Escrow Data ====> {:?}" , &escrow_account.data.borrow());
        msg!("Escrow Data ====> {:?}" , &escrow_account.data_len());

        msg!("unpacking escrow");
        let mut escrow = Escrow::try_from_slice(&escrow_account.data.borrow())?;
        //let mut escrow = Escrow::unpack_unchecked(&escrow_account.try_borrow_mut_data()?)?;

        escrow.is_initialized = true;
        escrow.start_time = start_time;
        escrow.receiver = *receiver_account.key;
        escrow.amount_to_send = amount_to_send;
        escrow.sender = *sender_account.key;

        msg!("escrow sender confirm {:?}", escrow.sender);
        msg!("escrow amount to send confirm {:?}", escrow.amount_to_send);

        msg!("packing escrow");
        //escrow::pack(escrow, &mut escrow_account.try_borrow_mut_data()?)?; 
        escrow.serialize(&mut &mut escrow_account.data.borrow_mut()[..])?;
        
        msg!("COMPLETED");

        Ok(())
    }

    fn process_withdraw(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        amount: u64,
    ) -> ProgramResult{
        let account_info_iter = &mut accounts.iter();
        let escrow_account = next_account_info(account_info_iter)?; // pda storage
        let sender_account = next_account_info(account_info_iter)?; // sender account
        let receiver_account = next_account_info(account_info_iter)?; // receipent account
        let system_program = next_account_info(account_info_iter)?; // system program

        let escrow_data = Escrow::try_from_slice(&escrow_account.data.borrow()).expect("Failed to seriallize");

        if *receiver_account.key != escrow_data.receiver {
            return Err(ProgramError::IllegalOwner);
        }

        if !receiver_account.is_signer { 
            return Err(ProgramError::MissingRequiredSignature);
        }

        let (account_address, _bump_seed) = Pubkey::find_program_address(
            &[&sender_account.key.to_bytes()],
            program_id,
        );

        if account_address != *escrow_account.key{
            return Err(ProgramError::InvalidAccountData);
        }

        if escrow_data.start_time + 2 > Clock::get()?.unix_timestamp as u64{ // 24 hours not passed yet (24*60*60)
            return Err(EscrowError::WithdrawTimeLimitNotExceed.into());
        }
        
        let (_account_address, bump_seed) = Pubkey::find_program_address(
            &[&sender_account.key.to_bytes()],
            program_id,
        );
        let pda_signer_seeds: &[&[_]] = &[
            &sender_account.key.to_bytes(),
            &[bump_seed],
        ];

        invoke_signed(
            &system_instruction::transfer(
                sender_account.key,
                receiver_account.key,
                amount
            ),
            &[
                sender_account.clone(),
                receiver_account.clone(),
                system_program.clone()
            ],
            &[pda_signer_seeds],
        )?;
        Ok(())
    }
}