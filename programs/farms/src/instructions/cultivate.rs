use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint};

use crate::state::{Manager, Crop};
use crate::instructions::utils;

#[derive(Accounts)]
#[instruction(seed: u8)]
pub struct Cultivate<'info> {

    #[account(mut, has_one = owner)]
    pub manager: Account<'info, Manager>,

    #[account(
        seeds = [b"rewarder".as_ref(), manager.key().as_ref()],
        bump = manager.rewarder_bump
    )]
    pub rewarder_pda: AccountInfo<'info>,

    #[account(
        init,
        payer = owner,
        space = 8 + Crop::space(),
        seeds = [b"crop".as_ref(), manager.key().as_ref(), &manager.crops.to_le_bytes()],
        bump = seed
    )]
    pub crop: Account<'info, Crop>,

    #[account(
        init, 
        payer = owner, 
        token::mint = deposit_mint,
        token::authority = crop
    )]
    pub deposit_treasury: Box<Account<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = owner,
        token::mint = reward_mint,
        token::authority = rewarder_pda
    )]
    pub reward_treasury: Account<'info, TokenAccount>,

    #[account(mut)]
    pub deposit_mint: Account<'info, Mint>,

    #[account(mut)]
    pub reward_mint: Account<'info, Mint>,

    pub owner: Signer<'info>,

    pub token_program: Program<'info, Token>,

    pub system_program: Program<'info, System>,

    pub rent: Sysvar<'info, Rent>,

}

pub fn handler(
    ctx: Context<Cultivate>, 
    deposit_fee: u64,
    withdraw_fee: u64,
    end_timestamp: u64,
    reward_rate: u64,
    seed: u8
) -> ProgramResult {

    utils::assert_valid_fee(deposit_fee)?;
    utils::assert_valid_fee(withdraw_fee)?;

    // Construct the new crop
    let crop = &mut ctx.accounts.crop;
    crop.manager = ctx.accounts.manager.key();
    crop.reward_treasury = ctx.accounts.reward_treasury.key();
    crop.reward_mint = ctx.accounts.reward_mint.key();
    crop.deposit_treasury = ctx.accounts.deposit_treasury.key();
    crop.deposit_mint = ctx.accounts.deposit_mint.key();
    crop.end_timestamp = end_timestamp;
    crop.deposit_fee = deposit_fee;
    crop.withdraw_fee = withdraw_fee;
    crop.reward_rate = reward_rate;
    crop.total_deposited = 0u64;
    crop.rewards_per_share = 0u64;
    crop.previous_reward_timestamp = utils::get_current_timestamp()?;
    crop.paused = false;
    crop.bump = seed;
    crop.id = ctx.accounts.manager.crops;

    // Increment the crop count
    ctx.accounts.manager.crops += 1;

    Ok(())
}
