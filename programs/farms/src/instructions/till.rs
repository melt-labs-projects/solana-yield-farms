use anchor_lang::prelude::*;


use crate::state::{Plot, Manager};

#[derive(Accounts)]
#[instruction(seed: u8, crop_id: u64)]
pub struct Till<'info> {

    pub manager: Account<'info, Manager>,

    #[account(
        init,
        payer = farmer,
        space = 8 + Plot::space(),
        seeds = [b"plot".as_ref(), manager.key().as_ref(), farmer.key().as_ref(), &crop_id.to_le_bytes()],
        bump = seed
    )]
    pub plot: Account<'info, Plot>,

    pub farmer: Signer<'info>,

    pub system_program: Program<'info, System>,

    pub rent: Sysvar<'info, Rent>,

}

pub fn handler(ctx: Context<Till>, seed: u8) -> ProgramResult {
    ctx.accounts.plot.bump = seed;
    Ok(())
}