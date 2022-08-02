use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};


use crate::state::{Crop, Plot};
use crate::instructions::utils;

#[derive(Accounts)]
pub struct Sow<'info> {

    #[account(
        mut,
        seeds = [b"crop".as_ref(), crop.manager.as_ref(), &crop.id.to_le_bytes()],
        bump = crop.bump,
        has_one = deposit_treasury,
    )]
    pub crop: Account<'info, Crop>,

    #[account(
        mut,
        seeds = [b"plot".as_ref(), crop.manager.key().as_ref(), farmer.key().as_ref(), &crop.id.to_le_bytes()],
        bump = plot.bump
    )]
    pub plot: Account<'info, Plot>,

    #[account(mut)]
    pub deposit_treasury: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = crop.deposit_mint == from_token_account.mint.key()
    )]
    pub from_token_account: Account<'info, TokenAccount>,

    pub farmer: Signer<'info>,

    pub token_program: Program<'info, Token>,

}

impl<'info> Sow<'info> {

    fn transfer_to_treasury(&self, amount: u64) -> ProgramResult {
        token::transfer(
            CpiContext::new(
                self.token_program.to_account_info(), 
                Transfer {
                    from: self.from_token_account.to_account_info(),
                    to: self.deposit_treasury.to_account_info(),
                    authority: self.farmer.to_account_info()
                }
            ), 
            amount
        )?;
        Ok(())
    }

}

pub fn handler(ctx: Context<Sow>, amount: u64) -> ProgramResult {
    let from_token_account = &ctx.accounts.from_token_account;

    utils::assert_non_zero(amount)?;
    utils::assert_sufficient_funds(from_token_account, amount)?;

    // transfer the user's tokens to the treasury
    ctx.accounts.transfer_to_treasury(amount)?;

    // handle deposit fee
    let fee_amount = utils::calculate_fee(ctx.accounts.crop.deposit_fee, amount)?;
    let deposit_amount = amount - fee_amount;

    // // update last reward 
    utils::update_crop(&mut ctx.accounts.crop)?;
    
    // Update the users information
    ctx.accounts.plot.debt += utils::calculate_reward_debt(
        ctx.accounts.crop.rewards_per_share,
        deposit_amount
    )?;
    ctx.accounts.plot.amount += deposit_amount;

    // Update the farm
    ctx.accounts.crop.total_deposited += deposit_amount;
    ctx.accounts.crop.fees += fee_amount;

    Ok(())
}