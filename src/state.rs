use borsh::{BorshDeserialize, BorshSerialize};

use solana_program::{
    program_error::ProgramError,
    pubkey::Pubkey,
    program_pack::{IsInitialized, Pack, Sealed},
};

use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};

#[derive(BorshDeserialize, BorshSerialize, PartialEq, Debug)]
pub struct TransferInput {
    pub start_time: u64,
    pub amount_to_send: u64,
}

#[derive(BorshDeserialize, BorshSerialize, PartialEq, Debug)]
pub struct WithdrawInput{
    pub amount: u64,
}

#[derive(BorshDeserialize, BorshSerialize, PartialEq, Debug)]
pub struct Escrow {
    pub is_initialized:bool,
    pub start_time: u64,
    pub receiver: Pubkey,
    pub amount_to_send: u64,
    pub sender: Pubkey,
}

impl Sealed for Escrow {}

impl IsInitialized for Escrow {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize)]
pub struct InitTokenInput {
    pub start_time: u64,
    pub amount: u64,
}

#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize)]
pub struct WithdrawTokenInput{
    pub amount: u64
}

#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize)]
pub struct TransferToken {
    pub start_time: u64,
    pub amount: u64,
    pub token_mint: Pubkey,
    pub sender: Pubkey,
    pub receiver:Pubkey,
}

impl Pack for Escrow {
    const LEN: usize = 81;
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, Escrow::LEN];
        let (
            is_initialized,
            start_time,
            receiver,
            amount_to_send,
            sender,
        ) = array_refs![src, 1, 8, 32, 8, 32];

        let is_initialized = match is_initialized {
            [0] => false,
            [1] => true,
            _ => return Err(ProgramError::InvalidAccountData),
        };

        Ok(Escrow {
            is_initialized,
            start_time: u64::from_le_bytes(*start_time),
            receiver: Pubkey::new_from_array(*receiver),
            amount_to_send: u64::from_le_bytes(*amount_to_send),
            sender: Pubkey::new_from_array(*sender),
        })
    }

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, Escrow::LEN];
        let (
            is_initialized_dst,
            start_time_dst,
            receiver_dst,
            amount_to_send_dst,
            sender_dst,
        ) = mut_array_refs![dst,1, 8, 32, 8, 32];
        
        let Escrow{
            is_initialized,
            start_time,
            receiver,
            amount_to_send,
            sender,
        } = self;

        is_initialized_dst[0] = *is_initialized as u8;
        *start_time_dst = start_time.to_le_bytes();
        receiver_dst.copy_from_slice(receiver.as_ref());
        *amount_to_send_dst = amount_to_send.to_le_bytes();
        sender_dst.copy_from_slice(sender.as_ref());
    }
}