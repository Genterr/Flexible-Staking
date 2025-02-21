use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use std::convert::TryFrom;
use crate::state::*;
use crate::errors::*;
use crate::events::*;

const MIN_STAKE_AMOUNT: u64 = 100_000_000; // 100 GENT (with 6 decimals)
const REWARDS_PRECISION: u128 = 1_000_000_000_000;
const SECONDS_PER_DAY: i64 = 86400;

pub fn initialize_pool(
    ctx: Context<InitializePool>,
    config: PoolConfig,
) -> Result<()> {
    let pool = &mut ctx.accounts.staking_pool;
    pool.authority = ctx.accounts.authority.key();
    pool.treasury = ctx.accounts.treasury.key();
    pool.emergency_admin = ctx.accounts.emergency_admin.key();
    pool.launch_timestamp = Clock::get()?.unix_timestamp;
    pool.early_adopter_deadline = pool.launch_timestamp + config.early_adopter_period;
    pool.min_stake_duration = config.min_stake_duration;
    pool.max_stake_duration = config.max_stake_duration;
    pool.rewards_multiplier = config.rewards_multiplier;
    pool.treasury_fee = config.treasury_fee;
    pool.bump = *ctx.bumps.get("staking_pool").unwrap();

    emit!(PoolInitializedEvent {
        authority: pool.authority,
        treasury: pool.treasury,
        launch_timestamp: pool.launch_timestamp,
    });

    Ok(())
}

pub fn create_stake_account(
    ctx: Context<CreateStakeAccount>,
    delegated_authority: Option<Pubkey>,
) -> Result<()> {
    let staker = &mut ctx.accounts.staker_info;
    staker.owner = ctx.accounts.owner.key();
    staker.delegated_authority = delegated_authority;
    staker.bump = *ctx.bumps.get("staker_info").unwrap();

    emit!(StakeAccountCreatedEvent {
        owner: staker.owner,
        delegated_authority,
    });

    Ok(())
}

pub fn stake(
    ctx: Context<Stake>,
    amount: u64,
    lock_period: i64,
) -> Result<()> {
    require!(amount >= MIN_STAKE_AMOUNT, StakingError::BelowMinimumStake);
    require!(
        lock_period >= ctx.accounts.staking_pool.min_stake_duration,
        StakingError::LockPeriodTooShort
    );

    let pool = &ctx.accounts.staking_pool;
    require!(!pool.paused, StakingError::PoolPaused);

    let staker = &mut ctx.accounts.staker_info;
    let clock = Clock::get()?;

    // Update rewards before new stake
    if staker.amount > 0 {
        let pending_rewards = calculate_pending_rewards(
            &ctx.accounts.reward_vault,
            staker,
            clock.unix_timestamp,
        )?;
        staker.accumulated_rewards = staker.accumulated_rewards
            .checked_add(pending_rewards)
            .ok_or(StakingError::CalculationOverflow)?;
    }

    // Transfer tokens to stake account
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_token_account.to_account_info(),
                to: ctx.accounts.stake_token_account.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        amount,
    )?;

    // Update staker info
    staker.amount = staker.amount
        .checked_add(amount)
        .ok_or(StakingError::CalculationOverflow)?;
    staker.start_time = clock.unix_timestamp;
    staker.last_reward_time = clock.unix_timestamp;
    staker.tier = calculate_tier(staker.amount);
    staker.is_early_adopter = clock.unix_timestamp <= pool.early_adopter_deadline;
    staker.lock_period = lock_period;
    staker.unlock_time = clock.unix_timestamp
        .checked_add(lock_period)
        .ok_or(StakingError::CalculationOverflow)?;

    // Update pool stats
    let pool = &mut ctx.accounts.staking_pool;
    pool.total_staked = pool.total_staked
        .checked_add(amount)
        .ok_or(StakingError::CalculationOverflow)?;
    pool.stake_count = pool.stake_count
        .checked_add(1)
        .ok_or(StakingError::CalculationOverflow)?;

    emit!(StakeEvent {
        user: ctx.accounts.user.key(),
        amount,
        timestamp: clock.unix_timestamp,
        tier: staker.tier,
        lock_period,
    });

    Ok(())
}

pub fn unstake(
    ctx: Context<Unstake>,
    amount: u64,
) -> Result<()> {
    let staker = &mut ctx.accounts.staker_info;
    let clock = Clock::get()?;

    require!(
        clock.unix_timestamp >= staker.unlock_time,
        StakingError::StakeLocked
    );
    require!(amount <= staker.amount, StakingError::InsufficientStakeBalance);

    // Claim any pending rewards before unstaking
    let pending_rewards = calculate_pending_rewards(
        &ctx.accounts.reward_vault,
        staker,
        clock.unix_timestamp,
    )?;

    if pending_rewards > 0 {
        staker.accumulated_rewards = staker.accumulated_rewards
            .checked_add(pending_rewards)
            .ok_or(StakingError::CalculationOverflow)?;
    }

    // Transfer unstaked tokens back to user
    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.stake_token_account.to_account_info(),
                to: ctx.accounts.user_token_account.to_account_info(),
                authority: ctx.accounts.staking_pool.to_account_info(),
            },
            &[&[
                b"staking_pool".as_ref(),
                &[ctx.accounts.staking_pool.bump],
            ]],
        ),
        amount,
    )?;

    // Update staker info
    staker.amount = staker.amount
        .checked_sub(amount)
        .ok_or(StakingError::CalculationOverflow)?;
    staker.last_reward_time = clock.unix_timestamp;

    // Update pool stats
    let pool = &mut ctx.accounts.staking_pool;
    pool.total_staked = pool.total_staked
        .checked_sub(amount)
        .ok_or(StakingError::CalculationOverflow)?;

    emit!(UnstakeEvent {
        user: ctx.accounts.user.key(),
        amount,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}

pub fn claim_rewards(ctx: Context<ClaimRewards>) -> Result<()> {
    let staker = &mut ctx.accounts.staker_info;
    let clock = Clock::get()?;

    let pending_rewards = calculate_pending_rewards(
        &ctx.accounts.reward_vault,
        staker,
        clock.unix_timestamp,
    )?;

    let total_rewards = staker.accumulated_rewards
        .checked_add(pending_rewards)
        .ok_or(StakingError::CalculationOverflow)?;

    require!(total_rewards > 0, StakingError::NoRewardsToClaim);

    // Calculate treasury fee
    let treasury_fee = total_rewards
        .checked_mul(ctx.accounts.staking_pool.treasury_fee as u64)
        .ok_or(StakingError::CalculationOverflow)?
        .checked_div(10000)
        .ok_or(StakingError::CalculationOverflow)?;

    let user_reward = total_rewards
        .checked_sub(treasury_fee)
        .ok_or(StakingError::CalculationOverflow)?;

    // Transfer rewards
    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.reward_vault_token.to_account_info(),
                to: ctx.accounts.user_token_account.to_account_info(),
                authority: ctx.accounts.reward_vault.to_account_info(),
            },
            &[&[
                b"reward_vault".as_ref(),
                &[ctx.accounts.reward_vault.bump],
            ]],
        ),
        user_reward,
    )?;

    // Transfer treasury fee
    if treasury_fee > 0 {
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.reward_vault_token.to_account_info(),
                    to: ctx.accounts.treasury_account.to_account_info(),
                    authority: ctx.accounts.reward_vault.to_account_info(),
                },
                &[&[
                    b"reward_vault".as_ref(),
                    &[ctx.accounts.reward_vault.bump],
                ]],
            ),
            treasury_fee,
        )?;
    }

    // Update staker info
    staker.accumulated_rewards = 0;
    staker.last_reward_time = clock.unix_timestamp;
    staker.rewards_claimed = staker.rewards_claimed
        .checked_add(total_rewards)
        .ok_or(StakingError::CalculationOverflow)?;

    emit!(RewardsClaimedEvent {
        user: ctx.accounts.user.key(),
        amount: user_reward,
        treasury_fee,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}

pub fn emergency_unstake(ctx: Context<EmergencyUnstake>) -> Result<()> {
    let staker = &mut ctx.accounts.staker_info;
    let amount = staker.amount;

    require!(amount > 0, StakingError::NoStakeToUnstake);

    // Transfer all staked tokens back to user
    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.stake_token_account.to_account_info(),
                to: ctx.accounts.user_token_account.to_account_info(),
                authority: ctx.accounts.staking_pool.to_account_info(),
            },
            &[&[
                b"staking_pool".as_ref(),
                &[ctx.accounts.staking_pool.bump],
            ]],
        ),
        amount,
    )?;

    // Reset staker info
    staker.amount = 0;
    staker.accumulated_rewards = 0;
    staker.last_reward_time = Clock::get()?.unix_timestamp;

    // Update pool stats
    let pool = &mut ctx.accounts.staking_pool;
    pool.total_staked = pool.total_staked
        .checked_sub(amount)
        .ok_or(StakingError::CalculationOverflow)?;

    emit!(EmergencyUnstakeEvent {
        user: ctx.accounts.user.key(),
        amount,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

// Helper functions
fn calculate_pending_rewards(
    reward_vault: &RewardVault,
    staker: &StakerInfo,
    current_time: i64,
) -> Result<u64> {
    if staker.amount == 0 || current_time <= staker.last_reward_time {
        return Ok(0);
    }

    let time_delta = (current_time - staker.last_reward_time) as u64;
    let base_rewards = reward_vault.rewards_per_second
        .checked_mul(time_delta)
        .ok_or(StakingError::CalculationOverflow)?
        .checked_mul(staker.amount)
        .ok_or(StakingError::CalculationOverflow)?;

    let mut multiplier = 10000; // Base multiplier (100%)

    // Add tier bonus
    multiplier = multiplier
        .checked_add(get_tier_bonus(staker.tier))
        .ok_or(StakingError::CalculationOverflow)?;

    // Add early adopter bonus
    if staker.is_early_adopter {
        multiplier = multiplier
            .checked_add(1000) // 10% bonus
            .ok_or(StakingError::CalculationOverflow)?;
    }

    // Add lock period bonus
    let lock_bonus = calculate_lock_bonus(staker.lock_period)?;
    multiplier = multiplier
        .checked_add(lock_bonus)
        .ok_or(StakingError::CalculationOverflow)?;

    let rewards = base_rewards
        .checked_mul(multiplier)
        .ok_or(StakingError::CalculationOverflow)?
        .checked_div(10000)
        .ok_or(StakingError::CalculationOverflow)?;

    Ok(rewards as u64)
}

fn calculate_tier(amount: u64) -> u8 {
    if amount >= 500_000_000_000 { // 500,000 GENT
        4 // Diamond
    } else if amount >= 100_000_000_000 { // 100,000 GENT
        3 // Platinum
    } else if amount >= 50_000_000_000 { // 50,000 GENT
        2 // Gold
    } else if amount >= 10_000_000_000 { // 10,000 GENT
        1 // Silver
    } else {
        0 // Bronze
    }
}

fn get_tier_bonus(tier: u8) -> u64 {
    match tier {
        0 => 0,      // Bronze: 0%
        1 => 500,    // Silver: 5%
        2 => 1000,   // Gold: 10%
        3 => 2000,   // Platinum: 20%
        4 => 3000,   // Diamond: 30%
        _ => 0,
    }
}

fn calculate_lock_bonus(lock_period: i64) -> Result<u64> {
    let days = lock_period
        .checked_div(SECONDS_PER_DAY)
        .ok_or(StakingError::CalculationOverflow)?;

    let bonus = match days {
        0..=29 => 0,         // 0%
        30..=89 => 500,      // 5%
        90..=179 => 1000,    // 10%
        180..=364 => 2000,   // 20%
        _ => 3000,           // 30%
    };

    Ok(bonus)
}