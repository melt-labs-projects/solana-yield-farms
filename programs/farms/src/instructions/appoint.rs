use anchor_lang::prelude::*;

use crate::state::Manager;

#[derive(Accounts)]
#[instruction(rewarder_bump: u8)]
pub struct Appoint<'info> {

    #[account(
        init,
        payer = signer,
        space = 8 + Manager::space(),
    )]
    pub manager: ProgramAccount<'info, Manager>,

    #[account(
        seeds = [b"rewarder".as_ref(), manager.key().as_ref()],
        bump = rewarder_bump
    )]
    pub rewarder_pda: AccountInfo<'info>,

    pub signer: Signer<'info>,

    pub system_program: Program<'info, System>,

    pub rent: Sysvar<'info, Rent>,

}

pub fn handler(ctx: Context<Appoint>, rewarder_bump: u8) -> ProgramResult {
    let manager = &mut ctx.accounts.manager;
    manager.owner = ctx.accounts.signer.key();
    manager.crops = 0;
    manager.rewarder_bump = rewarder_bump;
    Ok(())
}
