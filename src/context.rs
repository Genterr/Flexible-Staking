use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};
use crate::state::*;

#[derive(Accounts)]
pub struct InitializePool<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 32 + 32 + 8 + 8 + 8 + 8 + 1 + 8 + 8 + 8 + 8 + 8 + 1,
        seeds = [b"staking_pool"],
        bump
    )]
    pub staking_pool: Account<'info, StakingPool>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    /// CHECK: Safe because this read-only account only stores funds
    pub treasury: AccountInfo<'info>,
    
    /// CHECK: Safe because this read-only account is only used for emergency functions
    pub emergency_admin: AccountInfo<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreateStakeAccount<'info> {
    #[account(
        init,
        payer = owner,
        space = 8 + 32 + 32 + 8 + 8 + 8 + 8 + 8 + 1 + 1 + 8 + 8 + 8 + 8 + 8 + 1,
        seeds = [b"staker", owner.key().as_ref()],
        bump
    )]
    pub staker_info: Account<'info, StakerInfo>,
    
    #[account(mut)]
    pub owner: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    pub staking_pool: Account<'info, StakingPool>,
    
    #[account(
        mut,
        seeds = [b"staker", user.key().as_ref()],
        bump = staker_info.bump,
        has_one = owner @ StakingError::UnauthorizedAccess,
    )]
    pub staker_info: Account<'info, StakerInfo>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        mut,
        constraint = user_token_account.owner == user.key() @ StakingError::UnauthorizedAccess
    )]
    pub user_token_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        constraint = stake_token_account.owner == staking_pool.key() @ StakingError::InvalidTokenAccount
    )]
    pub stake_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub reward_vault: Account<'info, RewardVault>,
    
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Unstake<'info> {
    #[account(mut)]
    pub staking_pool: Account<'info, StakingPool>,
    
    #[account(
        mut,
        seeds = [b"staker", user.key().as_ref()],
        bump = staker_info.bump,
        has_one = owner @ StakingError::UnauthorizedAccess,
    )]
    pub staker_info: Account<'info, StakerInfo>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        mut,
        constraint = user_token_account.owner == user.key() @ StakingError::UnauthorizedAccess
    )]
    pub user_token_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        constraint = stake_token_account.owner == staking_pool.key() @ StakingError::InvalidTokenAccount
    )]
    pub stake_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub reward_vault: Account<'info, RewardVault>,
    
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct ClaimRewards<'info> {
    #[account(mut)]
    pub staking_pool: Account<'info, StakingPool>,
    
    #[account(
        mut,
        seeds = [b"staker", user.key().as_ref()],
        bump = staker_info.bump,
        has_one = owner @ StakingError::UnauthorizedAccess,
    )]
    pub staker_info: Account<'info, StakerInfo>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        mut,
        constraint = user_token_account.owner == user.key() @ StakingError::UnauthorizedAccess
    )]
    pub user_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub reward_vault: Account<'info, RewardVault>,
    
    #[account(
        mut,
        constraint = reward_vault_token.owner == reward_vault.key() @ StakingError::InvalidTokenAccount
    )]
    pub reward_vault_token: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        constraint = treasury_account.owner == staking_pool.treasury @ StakingError::InvalidTokenAccount
    )]
    pub treasury_account: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct EmergencyUnstake<'info> {
    #[account(
        mut,
        has_one = emergency_admin @ StakingError::UnauthorizedAccess
    )]
    pub staking_pool: Account<'info, StakingPool>,
    
    #[account(
        mut,
        seeds = [b"staker", user.key().as_ref()],
        bump = staker_info.bump,
        has_one = owner @ StakingError::UnauthorizedAccess,
    )]
    pub staker_info: Account<'info, StakerInfo>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    pub emergency_admin: Signer<'info>,
    
    #[account(
        mut,
        constraint = user_token_account.owner == user.key() @ StakingError::UnauthorizedAccess
    )]
    pub user_token_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        constraint = stake_token_account.owner == staking_pool.key() @ StakingError::InvalidTokenAccount
    )]
    pub stake_token_account: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
}