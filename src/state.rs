use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;

#[account]
pub struct StakingPool {
    pub authority: Pubkey,
    pub treasury: Pubkey,
    pub emergency_admin: Pubkey,
    pub total_staked: u64,
    pub total_rewards_distributed: u64,
    pub launch_timestamp: i64,
    pub early_adopter_deadline: i64,
    pub paused: bool,
    pub min_stake_duration: i64,
    pub max_stake_duration: i64,
    pub stake_count: u64,
    pub rewards_multiplier: u64,  // Basis points (1 = 0.01%)
    pub treasury_fee: u64,        // Basis points
    pub bump: u8,
}

#[account]
#[derive(Default)]
pub struct StakerInfo {
    pub owner: Pubkey,
    pub delegated_authority: Option<Pubkey>,
    pub amount: u64,
    pub start_time: i64,
    pub last_reward_time: i64,
    pub last_compound_time: i64,
    pub rewards_claimed: u64,
    pub tier: u8,
    pub is_early_adopter: bool,
    pub accumulated_rewards: u64,
    pub lock_period: i64,
    pub unlock_time: i64,
    pub boost_factor: u64,        // Additional multiplier for special events
    pub staking_points: u64,      // Loyalty points system
    pub bump: u8,
}

#[account]
pub struct RewardVault {
    pub authority: Pubkey,
    pub token_account: Pubkey,
    pub rewards_per_second: u64,
    pub last_update_time: i64,
    pub accumulated_rewards_per_share: u128,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum StakingTier {
    Bronze,    // 5,000 GENT
    Silver,    // 10,000 GENT
    Gold,      // 50,000 GENT
    Platinum,  // 100,000 GENT
    Diamond,   // 500,000 GENT
}