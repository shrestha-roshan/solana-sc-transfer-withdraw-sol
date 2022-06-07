use crate::{
    instruction::TransferInstruction,
    state::{TransferInput, WithdrawInput, Escrow},
};


use solana_program::{
    account_info::{next_account_info, AccountInfo},
    // clock::Clock,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
    msg, program_pack::Pack, program::invoke,
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
        let rent = Rent::get()?; //
        
        msg!("INTO create transfer!");
        msg!("start: {:?}", start_time);

        let account_info_iter = &mut accounts.iter();
        let escrow_account = next_account_info(account_info_iter)?;
        let sender_account = next_account_info(account_info_iter)?;
        let receiver_account = next_account_info(account_info_iter)?;
        let system_program = next_account_info(account_info_iter)?;       

        if !sender_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // **sender_account.try_borrow_mut_lamports()? -= amount_to_send;
        // **escrow_account.try_borrow_mut_lamports()? += amount_to_send;
        // Sending transaction fee to recipient. So, he can withdraw the streamed fund
       
        invoke(
            &system_instruction::create_account(
                sender_account.key,
                escrow_account.key,
                rent.minimum_balance(std::mem::size_of::<Escrow>()),
                std::mem::size_of::<Escrow>() as u64,
                program_id
            ),
            &[
                sender_account.clone(),
                escrow_account.clone(),
                system_program.clone(),
            ],
        )?;

        msg!("unpacking escrow");
        let mut escrow = Escrow::unpack_unchecked(&escrow_account.try_borrow_mut_data()?)?;
        escrow.is_initialized = true;
        escrow.start_time = start_time;
        escrow.receiver = *receiver_account.key;
        escrow.amount_to_send = amount_to_send;
        escrow.sender = *sender_account.key;

        msg!("packing escrow");
        Escrow::pack(escrow, &mut escrow_account.try_borrow_mut_data()?)?; 

        msg!("COMPLETED");

        Ok(())
    }

    fn process_withdraw(
        _program_id: &Pubkey,
        accounts: &[AccountInfo],
        amount: u64,
    ) -> ProgramResult{

        msg!("INTO withdraw function");
        msg!("Data {:?}", amount);

        let account_info_iter = &mut accounts.iter();
        let escrow_account = next_account_info(account_info_iter)?;
        let receiver_account = next_account_info(account_info_iter)?;

        let escrow_data = Escrow::unpack_unchecked(&escrow_account.data.borrow()).expect("Failed to seriallize");

        if *receiver_account.key != escrow_data.receiver {
            return Err(ProgramError::IllegalOwner);
        }

        if !receiver_account.is_signer { // Reciever signer???
            return Err(ProgramError::MissingRequiredSignature);
        }

        **escrow_account.try_borrow_mut_lamports()? -= amount;
        **receiver_account.try_borrow_mut_lamports()? += amount;

        msg!("COMPLETED");

        Ok(())
    }
}