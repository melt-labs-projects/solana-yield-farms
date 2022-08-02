use anchor_lang::prelude::*;
use anchor_lang::solana_program::sysvar::clock::Clock;
use anchor_spl::token::TokenAccount;
use std::result::Result;
use num_traits::Zero;
use std::cmp;

use crate::errors::*;
use crate::state::{Crop, Plot};

type IntegerResult = Result<u64, ProgramError>;

pub const MAX_FEE: u64 = 1_000_000_000;
pub const ONE_REWARD: u64 = 1_000_000_000;

pub fn get_current_timestamp() -> IntegerResult {
    Ok(Clock::get()?.unix_timestamp as u64)
}

pub fn calculate_fee(fee: u64, amount: u64) -> IntegerResult  {
    Ok(match (amount as u128).checked_mul(fee as u128) {
        Some(x) => match x.checked_div(MAX_FEE as u128) {
            Some(y) => y as u64,
            None => return Err(FarmError::NumericalOverflowError.into()),
        },
        None => return Err(FarmError::NumericalOverflowError.into()),
    })
}

pub fn calculate_reward_debt(rewards_per_share: u64, amount: u64) -> IntegerResult {
    Ok(match (amount as u128).checked_mul(rewards_per_share as u128) {
        Some(x) => match x.checked_div(ONE_REWARD as u128) {
            Some(y) => y as u64,
            None => return Err(FarmError::NumericalOverflowError.into()),
        },
        None => return Err(FarmError::NumericalOverflowError.into()),
    })
}

pub fn calculate_additional_rewards(
    seconds: u64, 
    rewards_per_second: u64,
    total_deposited: u64
) -> IntegerResult {
    
    let rewards = match (seconds as u128).checked_mul(rewards_per_second as u128) {
        Some(x) => x as u128,
        None => return Err(FarmError::NumericalOverflowError.into()),
    };
    Ok(match (rewards as u128).checked_mul(ONE_REWARD as u128) {
        Some(x) => match x.checked_div(total_deposited as u128) {
            Some(y) => y as u64,
            None => return Err(FarmError::NumericalOverflowError.into()),
        },
        None => return Err(FarmError::NumericalOverflowError.into()),
    }) 
}

pub fn calculate_plot_rewards(crop: &Crop, plot: &Plot) -> IntegerResult {
    Ok(match (plot.amount as u128).checked_mul(crop.rewards_per_share as u128) {
        Some(x) => match x.checked_sub(plot.debt as u128) {
            Some(y) => y as u64,
            None => return Err(FarmError::NumericalOverflowError.into()),
        },
        None => return Err(FarmError::NumericalOverflowError.into()),
    })
}

pub fn update_crop(crop: &mut Crop) -> ProgramResult {
    let current_timestamp = get_current_timestamp()?;
    if current_timestamp > crop.previous_reward_timestamp {

        // Check how many seconds have passed since the last reward update, but make sure to not go
        // past the configured end timestamp.
        let current_reward_timestamp = cmp::min(current_timestamp, crop.end_timestamp);
        let time_elapsed = current_reward_timestamp - crop.previous_reward_timestamp;

        // Only update the rewards if there are at least some tokens deposited
        if crop.total_deposited > 0 {
            crop.rewards_per_share += calculate_additional_rewards(
                time_elapsed, 
                crop.reward_rate,
                crop.total_deposited
            )?;
        }

        // Update the last timestamp where rewards were distributed
        crop.previous_reward_timestamp = current_reward_timestamp;
    }
    
    Ok(())
}

pub fn assert_non_zero<T: Zero>(amount: T) -> ProgramResult {
    if amount.is_zero() {
        return Err(FarmError::AmountIsZero.into());
    }
    Ok(())
}

pub fn assert_sufficient_funds(token_account: &TokenAccount, amount: u64) -> ProgramResult {
    if token_account.amount < amount {
        return Err(FarmError::InsufficientFunds.into());
    }
    Ok(())
}

pub fn assert_enough_to_uproot(plot: &Plot, amount: u64) -> ProgramResult {
    if plot.amount < amount {
        return Err(FarmError::AmountIsTooLarge.into());
    }
    Ok(())
}

pub fn assert_valid_fee(proposed_fee: u64) -> ProgramResult {
    if proposed_fee > MAX_FEE {
        return Err(FarmError::InvalidFee.into());
    }
    Ok(())
}