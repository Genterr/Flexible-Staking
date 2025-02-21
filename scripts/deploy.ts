import * as anchor from '@project-serum/anchor';
import { Program } from '@project-serum/anchor';
import { PublicKey, Keypair, Connection } from '@solana/web3.js';
import { TOKEN_PROGRAM_ID, Token } from '@solana/spl-token';
import * as dotenv from 'dotenv';
import { GentStaking } from '../target/types/gent_staking';

dotenv.config();

async function deploy() {
    console.log('Starting deployment...');

    // Initialize connection and provider
    const connection = new Connection(process.env.SOLANA_RPC_URL || 'http://localhost:8899');
    const wallet = new anchor.Wallet(Keypair.fromSecretKey(
        Buffer.from(JSON.parse(process.env.DEPLOYER_PRIVATE_KEY!))
    ));
    const provider = new anchor.AnchorProvider(connection, wallet, {
        commitment: 'confirmed',
    });

    // Deploy program
    const program = new Program(GentStaking.IDL, new PublicKey(process.env.PROGRAM_ID!), provider);

    try {
        // Create staking pool
        const [stakingPool, stakingPoolBump] = await PublicKey.findProgramAddress(
            [Buffer.from('staking_pool')],
            program.programId
        );

        // Create reward vault
        const [rewardVault, rewardVaultBump] = await PublicKey.findProgramAddress(
            [Buffer.from('reward_vault')],
            program.programId
        );

        // Initialize pool with configuration
        const config = {
            earlyAdopterPeriod: new anchor.BN(7 * 24 * 60 * 60), // 7 days
            minStakeDuration: new anchor.BN(30 * 24 * 60 * 60),  // 30 days
            maxStakeDuration: new anchor.BN(365 * 24 * 60 * 60), // 1 year
            rewardsMultiplier: new anchor.BN(100),               // 1%
            treasuryFee: new anchor.BN(500),                     // 5%
        };

        console.log('Initializing staking pool...');
        await program.methods
            .initializePool(config)
            .accounts({
                stakingPool,
                authority: wallet.publicKey,
                treasury: new PublicKey(process.env.TREASURY_ADDRESS!),
                emergencyAdmin: new PublicKey(process.env.EMERGENCY_ADMIN!),
                systemProgram: anchor.web3.SystemProgram.programId,
            })
            .rpc();

        console.log('Deployment completed successfully!');
        console.log('Staking Pool Address:', stakingPool.toString());
        console.log('Reward Vault Address:', rewardVault.toString());

    } catch (error) {
        console.error('Deployment failed:', error);
        process.exit(1);
    }
}

deploy().catch(console.error);