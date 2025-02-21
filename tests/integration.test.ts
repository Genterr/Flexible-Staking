import * as anchor from '@project-serum/anchor';
import { Program } from '@project-serum/anchor';
import { PublicKey, Keypair, SystemProgram } from '@solana/web3.js';
import { TOKEN_PROGRAM_ID, Token } from '@solana/spl-token';
import { assert } from 'chai';
import { GentStaking } from '../target/types/gent_staking';

describe('Gent Staking Integration Tests', () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.GentStaking as Program<GentStaking>;
  
  // Test accounts
  let stakingPool: PublicKey;
  let rewardVault: PublicKey;
  let treasury: PublicKey;
  
  // Test users
  const users = Array(5).fill(0).map(() => ({
    keypair: Keypair.generate(),
    tokenAccount: null as PublicKey | null,
    stakerInfo: null as PublicKey | null,
  }));

  const mint = Keypair.generate();
  const emergencyAdmin = Keypair.generate();

  before(async () => {
    // Setup initial state
    for (const user of users) {
      const signature = await provider.connection.requestAirdrop(
        user.keypair.publicKey,
        2 * anchor.web3.LAMPORTS_PER_SOL
      );
      await provider.connection.confirmTransaction(signature);
    }

    // Initialize token mint and accounts
    // ... (similar to staking.test.ts)
  });

  describe('Full staking lifecycle', () => {
    it('Completes full staking cycle for multiple users', async () => {
      // Initialize pool
      const config = {
        earlyAdopterPeriod: new anchor.BN(7 * 24 * 60 * 60),
        minStakeDuration: new anchor.BN(30 * 24 * 60 * 60),
        maxStakeDuration: new anchor.BN(365 * 24 * 60 * 60),
        rewardsMultiplier: new anchor.BN(100),
        treasuryFee: new anchor.BN(500),
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

      // Create stake accounts for all users
      for (const user of users) {
        const [stakerInfo, _] = await PublicKey.findProgramAddress(
          [Buffer.from('staker'), user.keypair.publicKey.toBuffer()],
          program.programId
        );
        user.stakerInfo = stakerInfo;

        await program.methods
          .createStakeAccount(null)
          .accounts({
            stakerInfo,
            owner: user.keypair.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([user.keypair])
          .rpc();
      }

      // Simulate different staking scenarios
      const scenarios = [
        { amount: new anchor.BN(5000000000), duration: new anchor.BN(30 * 24 * 60 * 60) },  // 5,000 tokens, 30 days
        { amount: new anchor.BN(10000000000), duration: new anchor.BN(90 * 24 * 60 * 60) }, // 10,000 tokens, 90 days
        { amount: new anchor.BN(50000000000), duration: new anchor.BN(180 * 24 * 60 * 60) },// 50,000 tokens, 180 days
        { amount: new anchor.BN(100000000000), duration: new anchor.BN(365 * 24 * 60 * 60) },// 100,000 tokens, 365 days
      ];

      // Execute staking for each user with different scenarios
      for (let i = 0; i < users.length - 1; i++) {
        const user = users[i];
        const scenario = scenarios[i];

        await program.methods
          .stake(scenario.amount, scenario.duration)
          .accounts({
            stakingPool,
            stakerInfo: user.stakerInfo!,
            user: user.keypair.publicKey,
            userTokenAccount: user.tokenAccount!,
            stakeTokenAccount,
            rewardVault,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([user.keypair])
          .rpc();
      }

      // Wait for some time to accumulate rewards
      await new Promise(resolve => setTimeout(resolve, 5000));

      // Check rewards and unstake for each user
      for (const user of users) {
        if (!user.stakerInfo) continue;

        const stakerAccount = await program.account.stakerInfo.fetch(user.stakerInfo);
        if (stakerAccount.amount.toNumber() > 0) {
          // Claim rewards
          await program.methods
            .claimRewards()
            .accounts({
              stakingPool,
              stakerInfo: user.stakerInfo,
              user: user.keypair.publicKey,
              userTokenAccount: user.tokenAccount!,
              rewardVault,
              rewardVaultToken: rewardVault,
              treasuryAccount: treasury,
              tokenProgram: TOKEN_PROGRAM_ID,
            })
            .signers([user.keypair])
            .rpc();
        }
      }
    });
  });

  // Error cases and edge scenarios
  describe('Error cases', () => {
    it('Prevents unauthorized access', async () => {
      // Test unauthorized access attempts
    });

    it('Handles invalid stake amounts', async () => {
      // Test invalid stake amount scenarios
    });

    it('Enforces lock periods', async () => {
      // Test lock period enforcement
    });
  });
});