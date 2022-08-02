use anchor_lang::prelude::*;

use crate::state::{Manager, Crop};
use crate::instructions::utils;

#[derive(Accounts)]
pub struct Recultivate<'info> {

    #[account(mut, has_one = owner)]
    pub manager: Account<'info, Manager>,

    #[account(
        mut,
        seeds = [b"crop".as_ref(), crop.manager.as_ref(), &crop.id.to_le_bytes()],
        bump = crop.bump
    )]
    pub crop: Account<'info, Crop>,

    pub owner: Signer<'info>,

}

pub fn handler(
    ctx: Context<Recultivate>, 
    deposit_fee: u64,
    withdraw_fee: u64,
    end_timestamp: u64,
    reward_rate: u64
) -> ProgramResult {

    utils::assert_valid_fee(deposit_fee)?;
    utils::assert_valid_fee(withdraw_fee)?;

    // Construct the new crop
    let crop = &mut ctx.accounts.crop;
    crop.end_timestamp = end_timestamp;
    crop.deposit_fee = deposit_fee;
    crop.withdraw_fee = withdraw_fee;
    crop.reward_rate = reward_rate;

    Ok(())
}
