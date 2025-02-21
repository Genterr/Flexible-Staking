use anchor_lang::prelude::*;

pub mod state;
pub mod instructions;
pub mod context;
pub mod events;
pub mod errors;

use instructions::*;

declare_id!("your_program_id");

#[program]
pub mod gent_staking {
    use super::*;

    pub fn initialize_pool(
        ctx: Context<InitializePool>,
        config: PoolConfig,
    ) -> Result<()> {
        instructions::initialize_pool(ctx, config)
    }

    pub fn create_stake_account(
        ctx: Context<CreateStakeAccount>,
        delegated_authority: Option<Pubkey>,
    ) -> Result<()> {
        instructions::create_stake_account(ctx, delegated_authority)
    }

    pub fn stake(
        ctx: Context<Stake>,
        amount: u64,
        lock_period: i64,
    ) -> Result<()> {
        instructions::stake(ctx, amount, lock_period)
    }

    pub fn unstake(
        ctx: Context<Unstake>,
        amount: u64,
    ) -> Result<()> {
        instructions::unstake(ctx, amount)
    }

    pub fn claim_rewards(ctx: Context<ClaimRewards>) -> Result<()> {
        instructions::claim_rewards(ctx)
    }

    pub fn emergency_unstake(ctx: Context<EmergencyUnstake>) -> Result<()> {
        instructions::emergency_unstake(ctx)
    }
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct PoolConfig {
    pub early_adopter_period: i64,
    pub min_stake_duration: i64,
    pub max_stake_duration: i64,
    pub rewards_multiplier: u64,
    pub treasury_fee: u64,
}