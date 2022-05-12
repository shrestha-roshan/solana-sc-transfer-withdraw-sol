use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{clock::UnixTimestamp, pubkey::Pubkey};


#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize)]
pub struct TransferInput {
    pub start_time: UnixTimestamp,
    pub receiver: Pubkey,
    pub amount_to_send: u64,
}

#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize)]
pub struct WithdrawInput{
    pub amount: u64,
}

#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize)]
pub struct TransferData {
    pub start_time: UnixTimestamp,
    pub receiver: Pubkey,
    pub amount_to_send: u64,
    pub sender: Pubkey,
}

impl TransferData {
    pub fn new(data: TransferInput, sender: Pubkey) -> Self {
        TransferData { 
            start_time: data.start_time, 
            receiver: data.receiver, 
            amount_to_send: data.amount_to_send, 
            sender,  
        }
    }
}