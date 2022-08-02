use anchor_lang::prelude::*;

use crate::state::Manager;

#[derive(Accounts)]
pub struct Entrust<'info> {

    #[account(
        mut,
        has_one = owner
    )]
    pub manager: ProgramAccount<'info, Manager>,

    pub owner: Signer<'info>,

}

pub fn handler(ctx: Context<Entrust>, trustee: Pubkey) -> ProgramResult {
    ctx.accounts.manager.owner = trustee;
    Ok(())
}
