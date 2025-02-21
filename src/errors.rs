use anchor_lang::prelude::*;

#[error_code]
pub enum StakingError {
    #[msg("Provided amount is below minimum stake requirement")]
    BelowMinimumStake,

    #[msg("Lock period is shorter than minimum required duration")]
    LockPeriodTooShort,

    #[msg("Staking pool is currently paused")]
    PoolPaused,

    #[msg("Calculation resulted in overflow")]
    CalculationOverflow,

    #[msg("Unauthorized access")]
    UnauthorizedAccess,

    #[msg("Invalid token account")]
    InvalidTokenAccount,

    #[msg("Stake is still locked")]
    StakeLocked,

    #[msg("Insufficient stake balance")]
    InsufficientStakeBalance,

    #[msg("No rewards to claim")]
    NoRewardsToClaim,

    #[msg("No stake to unstake")]
    NoStakeToUnstake,

    #[msg("Invalid stake tier")]
    InvalidStakeTier,

    #[msg("Invalid lock period")]
    InvalidLockPeriod,

    #[msg("Invalid rewards multiplier")]
    InvalidRewardsMultiplier,

    #[msg("Invalid treasury fee")]
    InvalidTreasuryFee,

    #[msg("Invalid reward rate")]
    InvalidRewardRate,
}