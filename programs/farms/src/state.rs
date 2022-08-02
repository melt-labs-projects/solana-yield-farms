use anchor_lang::prelude::*;
use borsh::{BorshDeserialize, BorshSerialize};

#[account]
pub struct Manager {

    pub owner: Pubkey,

    pub crops: u64,

    pub rewarder_bump: u8,

}

impl Manager {
    pub fn space() -> usize {
        1 * 8 + 1 * 32 + 1
    }
}

#[account]
pub struct Crop {

    pub manager: Pubkey,

    pub reward_treasury: Pubkey,

    pub reward_mint: Pubkey,

    pub deposit_treasury: Pubkey,

    pub deposit_mint: Pubkey,

    pub end_timestamp: u64,

    pub deposit_fee: u64,

    pub withdraw_fee: u64,

    pub total_deposited: u64,

    pub fees: u64,

    pub rewards_per_share: u64,

    pub reward_rate: u64,

    pub previous_reward_timestamp: u64,

    pub paused: bool,

    pub bump: u8,

    pub id: u64

}

impl Crop {
    pub fn space() -> usize {
        5 * 32 + 9 * 8 + 2 * 1
    }
}

#[account]
pub struct Plot {

    pub amount: u64,

    pub debt: u64,

    pub bump: u8,

}

impl Plot {
    pub fn space() -> usize {
        2 * 8 + 1
    }
}