use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::state::{Manager, Crop};

#[derive(Accounts)]
pub struct Collect<'info> {

    #[account(mut, has_one = owner)]
    pub manager: Account<'info, Manager>,

    #[account(
        mut,
        seeds = [b"crop".as_ref(), crop.manager.as_ref(), &crop.id.to_le_bytes()],
        bump = crop.bump,
        has_one = deposit_treasury
    )]
    pub crop: Account<'info, Crop>,

    #[account(mut)]
    pub deposit_treasury: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = crop.deposit_mint == deposit_token_account.mint.key()
    )]
    pub deposit_token_account: Account<'info, TokenAccount>,

    pub owner: Signer<'info>,

    pub token_program: Program<'info, Token>,

}

impl<'info> Collect<'info> {

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

}

pub fn handler(ctx: Context<Collect>) -> ProgramResult {

    let fees = ctx.accounts.crop.fees;
    ctx.accounts.crop.fees = 0;
    ctx.accounts.transfer_from_treasury(fees)?;

    Ok(())
}
