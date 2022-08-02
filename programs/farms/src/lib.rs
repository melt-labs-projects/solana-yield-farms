use anchor_lang::prelude::*;

pub mod errors;
pub mod state;
pub mod instructions;

use instructions::*;

declare_id!("H4USCP7cY9Rpsu6j3N6uLQ7tF1aNN9rLF9WfvjFGDfLe");

#[program]
pub mod farms {
    use super::*;

    // Create manager
    pub fn appoint(ctx: Context<Appoint>, rewarder_bump: u8) -> ProgramResult {
        instructions::appoint::handler(ctx, rewarder_bump)
    }

    // Transfer ownership
    pub fn entrust(ctx: Context<Entrust>, trustee: Pubkey) -> ProgramResult {
        instructions::entrust::handler(ctx, trustee)
    }

    // Create farm
    pub fn cultivate(
        ctx: Context<Cultivate>, 
        seed: u8,
        deposit_fee: u64,
        withdraw_fee: u64,
        end_timestamp: u64,
        reward_rate: u64,
    ) -> ProgramResult {
        instructions::cultivate::handler(
            ctx,
            deposit_fee,
            withdraw_fee,
            end_timestamp,
            reward_rate,
            seed
        )
    }

    // Update farm
    pub fn recultivate(
        ctx: Context<Recultivate>, 
        deposit_fee: u64,
        withdraw_fee: u64,
        end_timestamp: u64,
        reward_rate: u64
    ) -> ProgramResult {
        instructions::recultivate::handler(
            ctx,
            deposit_fee,
            withdraw_fee,
            end_timestamp,
            reward_rate
        )
    }

    // Create user account
    pub fn till(ctx: Context<Till>, seed: u8, crop_id: u64) -> ProgramResult {
        instructions::till::handler(ctx, seed)
    }

    // Deposit
    pub fn sow(ctx: Context<Sow>, amount: u64) -> ProgramResult {
        instructions::sow::handler(ctx, amount)
    }

    // Withdraw
    pub fn uproot(ctx: Context<Uproot>, amount: u64) -> ProgramResult {
        instructions::uproot::handler(ctx, amount)
    }

    // Collect fees 
    pub fn collect(ctx: Context<Collect>) -> ProgramResult {
        instructions::collect::handler(ctx)
    }

}
