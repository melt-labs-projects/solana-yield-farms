use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};


use crate::state::{Manager, Crop, Plot};
use crate::instructions::utils;

#[derive(Accounts)]
pub struct Uproot<'info> {

    #[account(mut)]
    pub manager: Account<'info, Manager>,

    #[account(
        seeds = [b"rewarder".as_ref(), crop.manager.as_ref()],
        bump = manager.rewarder_bump
    )]
    pub rewarder_pda: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [b"crop".as_ref(), crop.manager.as_ref(), &crop.id.to_le_bytes()],
        bump = crop.bump,
        has_one = deposit_treasury,
        has_one = reward_treasury
    )]
    pub crop: Box<Account<'info, Crop>>,

    #[account(
        mut,
        seeds = [b"plot".as_ref(), crop.manager.as_ref(), farmer.key().as_ref(), &crop.id.to_le_bytes()],
        bump = plot.bump
    )]
    pub plot: Account<'info, Plot>,

    #[account(mut)]
    pub deposit_treasury: Account<'info, TokenAccount>,

    #[account(mut)]
    pub reward_treasury: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = crop.deposit_mint == deposit_token_account.mint.key()
    )]
    pub deposit_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = crop.reward_mint == reward_token_account.mint.key()
    )]
    pub reward_token_account: Account<'info, TokenAccount>,

    pub farmer: Signer<'info>,

    pub token_program: Program<'info, Token>,

}

impl<'info> Uproot<'info> {

    fn transfer_from_treasury(&self, amount: u64) -> ProgramResult {
        token::transfer(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(), 
                Transfer {
                    from: self.deposit_treasury.to_account_info(),
                    to: self.deposit_token_account.to_account_info(),
                    authority: self.crop.to_account_info()
                },
                &[&[b"crop".as_ref(), self.crop.manager.as_ref(), &self.crop.id.to_le_bytes(), &[self.crop.bump]]]
            ), 
            amount
        )?;
        Ok(())
    }

    fn transfer_rewards(&self, amount: u64) -> ProgramResult {
        token::transfer(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(), 
                Transfer {
                    from: self.reward_treasury.to_account_info(),
                    to: self.reward_token_account.to_account_info(),
                    authority: self.rewarder_pda.to_account_info()
                },
                &[&[b"rewarder".as_ref(), self.crop.manager.as_ref(), &[self.manager.rewarder_bump]]]
            ), 
            amount
        )?;
        Ok(())
    }

}

pub fn handler(ctx: Context<Uproot>, amount: u64) -> ProgramResult {

    msg!("amount: {}", ctx.accounts.plot.amount);

    // check amount isnt more than plot has
    utils::assert_enough_to_uproot(&ctx.accounts.plot, amount)?;

    utils::update_crop(&mut ctx.accounts.crop)?;

    // Transfer rewards?
    let rewards = utils::calculate_plot_rewards(&ctx.accounts.crop, &ctx.accounts.plot)?;
    ctx.accounts.transfer_rewards(rewards)?;

    if amount > 0 {

        ctx.accounts.plot.amount -= amount;
        ctx.accounts.plot.debt = utils::calculate_reward_debt(
            ctx.accounts.crop.rewards_per_share,
            ctx.accounts.plot.amount
        )?;

        ctx.accounts.crop.total_deposited -= amount;

        let fee_amount = utils::calculate_fee(ctx.accounts.crop.withdraw_fee, amount)?;
        let withdraw_amount = amount - fee_amount;

        ctx.accounts.transfer_from_treasury(withdraw_amount)?;
        ctx.accounts.crop.fees += fee_amount;
    }

    Ok(())
}