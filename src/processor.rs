use spl_associated_token_account;

use crate::{
    instruction::TransferInstruction,
    state::{TransferInput, WithdrawInput, Escrow, InitTokenInput, WithdrawTokenInput, TransferToken},
};

use crate::{PREFIX,PREFIX_TOKEN};
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
            },
            TransferInstruction::TransferToken(InitTokenInput{start_time, amount}) => {
                Self::process_create_token_transfer(program_id, accounts, start_time, amount)
            },
            TransferInstruction::WithdrawToken(WithdrawTokenInput{amount}) => {
                Self::process_withdraw_token(program_id, accounts, amount)
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
        // let rent = Rent::get()?; //
        
        msg!("INTO CREATE TRANSFER NATIVE!");
        msg!("start: {:?}", start_time);
        msg!("amount: {:?}", amount_to_send);


        let account_info_iter = &mut accounts.iter();
        let escrow_account = next_account_info(account_info_iter)?;
        let sender_account = next_account_info(account_info_iter)?;
        let receiver_account = next_account_info(account_info_iter)?;
        let system_program = next_account_info(account_info_iter)?;    
        let vault = next_account_info(account_info_iter)?;

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

        msg!("Escrow Buffer ====> {:?}" , &escrow_account.data.borrow());
        msg!("Escrow Buffer length ====> {:?}" , &escrow_account.data_len());

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

        invoke_signed(
            &system_instruction::transfer(
                sender_account.key,
                vault.key,
                amount_to_send
            ),
            &[
                sender_account.clone(),
                vault.clone(),
                system_program.clone()
            ],
            &[pda_signer_seeds],
        )?;
        
        msg!("COMPLETED");

        Ok(())
    }

    fn process_withdraw(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        amount: u64,
    ) -> ProgramResult{
        msg!("INTO PROCESS WITHDRAW NATIVE!!");
        msg!("amount {:?}", amount);
        
        let account_info_iter = &mut accounts.iter();            
        let escrow_account = next_account_info(account_info_iter)?;     //  pda storage   \\
        let sender_account = next_account_info(account_info_iter)?;    //  sender account  \\
        let receiver_account = next_account_info(account_info_iter)?; //  receipent account \\
        let system_program = next_account_info(account_info_iter)?;  //   system program     \\
        let vault = next_account_info(account_info_iter)?;          //     vault account      \\

        let escrow_data = Escrow::try_from_slice(&escrow_account.data.borrow()).expect("Failed to seriallize");

        if *receiver_account.key != escrow_data.receiver {
            return Err(ProgramError::IllegalOwner);
        }

        if !receiver_account.is_signer { 
            return Err(ProgramError::MissingRequiredSignature);
        }

        // let (account_address, _bump_seed) = Pubkey::find_program_address(
        //     &[&sender_account.key.to_bytes()],
        //     program_id,
        // );

        // if account_address != *escrow_account.key{
        //     return Err(ProgramError::InvalidAccountData);
        // }

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
                vault.key,
                receiver_account.key,
                amount
            ),
            &[
                vault.clone(),
                receiver_account.clone(),
                system_program.clone()
            ],
            &[pda_signer_seeds],
        )?;
        Ok(())
    }

    fn process_create_token_transfer(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        start_time: u64,
        amount: u64,
    ) -> ProgramResult{

        msg!("INTO CREATE TRANSFER SPL!");
        msg!("start: {:?}", start_time);
        msg!("amount: {:?}", amount);

        let account_info_iter = &mut accounts.iter();
        let escrow_account = next_account_info(account_info_iter)?; // pda data storage
        let sender_account = next_account_info(account_info_iter)?; //sender
        let receiver_account = next_account_info(account_info_iter)?; // receiver
        let system_program = next_account_info(account_info_iter)?;  // system program
        let token_mint_info = next_account_info(account_info_iter)?; 
        let token_program_info = next_account_info(account_info_iter)?; 
        let sender_associated_info = next_account_info(account_info_iter)?; // TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA
        let vault_associated_info = next_account_info(account_info_iter)?;
        let rent_info = next_account_info(account_info_iter)?; 
        let associated_token_info = next_account_info(account_info_iter)?; // Associated token master {ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL}
        let vault = next_account_info(account_info_iter)?;

        if token_program_info.key != &spl_token::id() {
            return Err(ProgramError::IncorrectProgramId);
        }    
        // Since we are performing system_instruction source account must be signer
        if !sender_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature); 
        }

        let (_account_address, bump_seed) = Pubkey::find_program_address(
            &[
                PREFIX_TOKEN.as_bytes(),
                &sender_account.key.to_bytes()
                ],
            program_id,
        );

        // if account_address != *escrow_account.key{
        //     return Err(ProgramError::InvalidAccountData);
        // }

        invoke_signed(
            &system_instruction::create_account(
                sender_account.key, 
                escrow_account.key,
                Rent::get()?.minimum_balance(std::mem::size_of::<InitTokenInput>()),
                112,
                program_id
            ),
            &[sender_account.clone(), escrow_account.clone(),system_program.clone()],
            &[&[sender_account.key.as_ref(),&[bump_seed]]]
        )?;

        let mut escrow = TransferToken::try_from_slice(&escrow_account.data.borrow())?;
        
        escrow.start_time = start_time;
        escrow.amount = amount;
        escrow.token_mint = *token_mint_info.key;
        escrow.sender = *sender_account.key;
        escrow.receiver = *receiver_account.key;

        escrow.serialize(&mut &mut escrow_account.data.borrow_mut()[..])?;

        //creating associated token program for receiver to transfer token
        invoke(
            &spl_associated_token_account::create_associated_token_account(
                sender_account.key,
                vault.key,
                token_mint_info.key
            ), 
            &[
                sender_account.clone(),
                vault_associated_info.clone(),
                vault.clone(),
                token_mint_info.clone(),
                system_program.clone(),
                token_program_info.clone(),
                rent_info.clone(),
                associated_token_info.clone(),
            ]
        )?;

        let (_account_address, bump) = Pubkey::find_program_address(
            &[&sender_account.key.to_bytes()], 
            program_id
        );

        let pda_signer_seeds: &[&[_]] = &[&sender_account.key.to_bytes(), &[bump]];

        invoke_signed(
            &spl_token::instruction::transfer(
                token_program_info.key,
                sender_associated_info.key,
                vault_associated_info.key,
                sender_account.key,
                &[sender_account.key],
                amount,
            )?,
            &[
                token_program_info.clone(),
                vault_associated_info.clone(),
                sender_associated_info.clone(),
                sender_account.clone(),
                system_program.clone(),
            ],&[&pda_signer_seeds],
        )?;

        Ok(())
    }

    fn process_withdraw_token(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        amount: u64,
    ) -> ProgramResult {

        msg!("INTO PROCESS WITHDRAW SPL TOKEN!!");
        msg!("amount {:?}", amount);

        let account_info_iter = &mut accounts.iter();
        let escrow_account = next_account_info(account_info_iter)?;
        let sender_account = next_account_info(account_info_iter)?;
        let vault = next_account_info(account_info_iter)?;
        let receiver_account = next_account_info(account_info_iter)?;
        let system_program = next_account_info(account_info_iter)?; 
        let token_mint_info = next_account_info(account_info_iter)?;
        let token_program_info = next_account_info(account_info_iter)?; // TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA
        let vault_associated_info = next_account_info(account_info_iter)?; 
        let receiver_associated_info = next_account_info(account_info_iter)?;
        let rent_info = next_account_info(account_info_iter)?; 
        let associated_token_info = next_account_info(account_info_iter)?; // Associated token master {ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL}


        if *escrow_account.owner != *program_id {
            return Err(ProgramError::InvalidArgument);
        }

        if token_program_info.key != &spl_token::id() {
            return Err(ProgramError::IncorrectProgramId);
        }

        if !receiver_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature); 
        }

        if escrow_account.data_is_empty(){
            return Err(ProgramError::UninitializedAccount);
        }

        let escrow = TransferToken::try_from_slice(&escrow_account.data.borrow())?;

        if escrow.token_mint != *token_mint_info.key {
            return Err(TokenError::PublicKeyMismatch.into());
        }

        if *receiver_account.key != escrow.receiver {
            return Err(TokenError::EscrowMismatch.into());
        }

        if escrow.start_time + 2 > Clock::get()?.unix_timestamp as u64{ // 24 hours not passed yet
            return Err(EscrowError::WithdrawTimeLimitNotExceed.into());
        }

        //creating associated token program for receiver to transfer token
        invoke(
            &spl_associated_token_account::create_associated_token_account(
                receiver_account.key,
                receiver_account.key,
                token_mint_info.key
            ), 
            &[
                receiver_account.clone(),
                receiver_associated_info.clone(),
                receiver_account.clone(),
                token_mint_info.clone(),
                system_program.clone(),
                token_program_info.clone(),
                rent_info.clone(),
                associated_token_info.clone(),
            ]
        )?;

        let (_account_address, bump) = Pubkey::find_program_address(
            &[&sender_account.key.to_bytes()], 
            program_id
        );

        let pda_signer_seeds: &[&[_]] = &[&sender_account.key.to_bytes(), &[bump]];
        
        //transfering token to receiver_associated_info
        invoke_signed(
            &spl_token::instruction::transfer(
                token_program_info.key,
                vault_associated_info.key,
                receiver_associated_info.key,
                vault.key,
                &[vault.key],
                amount,
            )?,
            &[
                token_program_info.clone(),
                vault_associated_info.clone(),
                receiver_associated_info.clone(),
                vault.clone(),
                system_program.clone()
            ],&[&pda_signer_seeds],
        )?;

        Ok(())

    }
}