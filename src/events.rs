use anchor_lang::prelude::*;

#[event]
pub struct PoolInitializedEvent {
    pub authority: Pubkey,
    pub treasury: Pubkey,
    pub launch_timestamp: i64,
}

#[event]
pub struct StakeAccountCreatedEvent {
    pub owner: Pubkey,
    pub delegated_authority: Option<Pubkey>,
}

#[event]
pub struct StakeEvent {
    pub user: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
    pub tier: u8,
    pub lock_period: i64,
}

#[event]
pub struct UnstakeEvent {
    pub user: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct RewardsClaimedEvent {
    pub user: Pubkey,
    pub amount: u64,
    pub treasury_fee: u64,
    pub timestamp: i64,
}

#[event]
pub struct EmergencyUnstakeEvent {
    pub user: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct StakingPoolConfigUpdatedEvent {
    pub authority: Pubkey,
    pub min_stake_duration: i64,
    pub max_stake_duration: i64,
    pub rewards_multiplier: u64,
    pub treasury_fee: u64,
    pub timestamp: i64,
}

#[event]
pub struct RewardVaultUpdatedEvent {
    pub authority: Pubkey,
    pub rewards_per_second: u64,
    pub timestamp: i64,
}