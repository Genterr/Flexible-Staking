import * as anchor from '@project-serum/anchor';
import { Program } from '@project-serum/anchor';
import { PublicKey, Keypair, SystemProgram } from '@solana/web3.js';
import { TOKEN_PROGRAM_ID, Token } from '@solana/spl-token';
import { assert } from 'chai';
import { GentStaking } from '../target/types/gent_staking';

describe('Gent Staking Tests', () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.GentStaking as Program<GentStaking>;
  
  let stakingPool: PublicKey;
  let stakingPoolBump: number;
  let mintAuthority: Keypair;
  let mint: Token;
  let userTokenAccount: PublicKey;
  let stakeTokenAccount: PublicKey;
  let rewardVault: PublicKey;
  let treasury: PublicKey;

  const user = Keypair.generate();
  const emergencyAdmin = Keypair.generate();

  before(async () => {
    // Airdrop SOL to user
    const signature = await provider.connection.requestAirdrop(
      user.publicKey,
      2 * anchor.web3.LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(signature);

    // Create mint and token accounts
    mintAuthority = Keypair.generate();
    mint = await Token.createMint(
      provider.connection,
      user,
      mintAuthority.publicKey,
      null,
      6,
      TOKEN_PROGRAM_ID
    );

    userTokenAccount = await mint.createAccount(user.publicKey);
    stakeTokenAccount = await mint.createAccount(provider.wallet.publicKey);
    rewardVault = await mint.createAccount(provider.wallet.publicKey);
    treasury = await mint.createAccount(provider.wallet.publicKey);

    // Mint initial tokens to user
    await mint.mintTo(
      userTokenAccount,
      mintAuthority.publicKey,
      [mintAuthority],
      1000000000000 // 1,000,000 tokens
    );

    // Find PDA for staking pool
    [stakingPool, stakingPoolBump] = await PublicKey.findProgramAddress(
      [Buffer.from('staking_pool')],
      program.programId
    );
  });

  it('Initializes the staking pool', async () => {
    const config = {
      earlyAdopterPeriod: new anchor.BN(7 * 24 * 60 * 60), // 7 days
      minStakeDuration: new anchor.BN(30 * 24 * 60 * 60),  // 30 days
      maxStakeDuration: new anchor.BN(365 * 24 * 60 * 60), // 1 year
      rewardsMultiplier: new anchor.BN(100),               // 1%
      treasuryFee: new anchor.BN(500),                     // 5%
    };

    await program.methods
      .initializePool(config)
      .accounts({
        stakingPool,
        authority: provider.wallet.publicKey,
        treasury,
        emergencyAdmin: emergencyAdmin.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const poolAccount = await program.account.stakingPool.fetch(stakingPool);
    assert.ok(poolAccount.authority.equals(provider.wallet.publicKey));
    assert.ok(poolAccount.treasury.equals(treasury));
    assert.ok(poolAccount.emergencyAdmin.equals(emergencyAdmin.publicKey));
  });

  it('Creates a stake account', async () => {
    const [stakerInfo, _] = await PublicKey.findProgramAddress(
      [Buffer.from('staker'), user.publicKey.toBuffer()],
      program.programId
    );

    await program.methods
      .createStakeAccount(null)
      .accounts({
        stakerInfo,
        owner: user.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([user])
      .rpc();

    const stakeAccount = await program.account.stakerInfo.fetch(stakerInfo);
    assert.ok(stakeAccount.owner.equals(user.publicKey));
    assert.equal(stakeAccount.amount.toNumber(), 0);
  });

  it('Stakes tokens', async () => {
    const [stakerInfo, _] = await PublicKey.findProgramAddress(
      [Buffer.from('staker'), user.publicKey.toBuffer()],
      program.programId
    );

    const stakeAmount = new anchor.BN(100000000000); // 100,000 tokens
    const lockPeriod = new anchor.BN(90 * 24 * 60 * 60); // 90 days

    await program.methods
      .stake(stakeAmount, lockPeriod)
      .accounts({
        stakingPool,
        stakerInfo,
        user: user.publicKey,
        userTokenAccount,
        stakeTokenAccount,
        rewardVault,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([user])
      .rpc();

    const stakeAccount = await program.account.stakerInfo.fetch(stakerInfo);
    assert.equal(stakeAccount.amount.toNumber(), stakeAmount.toNumber());
  });

  // Additional tests for unstaking, claiming rewards, etc.
});