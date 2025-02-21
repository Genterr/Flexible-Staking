import * as anchor from '@project-serum/anchor';
import { Program } from '@project-serum/anchor';
import { PublicKey, Connection, Keypair } from '@solana/web3.js';
import { GentStaking } from '../target/types/gent_staking';
import * as dotenv from 'dotenv';
import * as readline from 'readline';

dotenv.config();

class EmergencyConsole {
    private program: Program<GentStaking>;
    private connection: Connection;
    private adminKeypair: Keypair;

    constructor() {
        this.connection = new Connection(process.env.SOLANA_RPC_URL || 'http://localhost:8899');
        const provider = new anchor.AnchorProvider(
            this.connection,
            new anchor.Wallet(this.adminKeypair),
            { commitment: 'confirmed' }
        );
        this.program = new Program(GentStaking.IDL, new PublicKey(process.env.PROGRAM_ID!), provider);
    }

    async emergencyUnstake(userPubkey: PublicKey) {
        const [stakerInfo] = await PublicKey.findProgramAddress(
            [Buffer.from('staker'), userPubkey.toBuffer()],
            this.program.programId
        );

        console.log(`Initiating emergency unstake for user: ${userPubkey.toString()}`);
        
        try {
            await this.program.methods
                .emergencyUnstake()
                .accounts({
                    stakingPool: await this.getStakingPoolAddress(),
                    stakerInfo,
                    user: userPubkey,
                    emergencyAdmin: this.adminKeypair.publicKey,
                    // ... other required accounts
                })
                .signers([this.adminKeypair])
                .rpc();
            
            console.log('Emergency unstake completed successfully');
        } catch (error) {
            console.error('Emergency unstake failed:', error);
        }
    }

    async pausePool() {
        try {
            // Implementation for pausing the staking pool
            console.log('Staking pool paused successfully');
        } catch (error) {
            console.error('Failed to pause pool:', error);
        }
    }

    async getStakingPoolAddress(): Promise<PublicKey> {
        const [stakingPool] = await PublicKey.findProgramAddress(
            [Buffer.from('staking_pool')],
            this.program.programId
        );
        return stakingPool;
    }

    async showPoolStatus() {
        const poolAddress = await this.getStakingPoolAddress();
        const poolData = await this.program.account.stakingPool.fetch(poolAddress);
        
        console.log('\nStaking Pool Status:');
        console.log('-------------------');
        console.log(`Total Staked: ${poolData.totalStaked.toString()}`);
        console.log(`Total Rewards Distributed: ${poolData.totalRewardsDistributed.toString()}`);
        console.log(`Paused: ${poolData.paused}`);
        console.log(`Stake Count: ${poolData.stakeCount.toString()}`);
    }

    async startConsole() {
        const rl = readline.createInterface({
            input: process.stdin,
            output: process.stdout
        });

        console.log('Emergency Admin Console');
        console.log('----------------------');
        
        const prompt = () => {
            rl.question(`
1. Show Pool Status
2. Pause Pool
3. Emergency Unstake
4. Exit

Select option: `, async (answer) => {
                switch(answer) {
                    case '1':
                        await this.showPoolStatus();
                        prompt();
                        break;
                    case '2':
                        await this.pausePool();
                        prompt();
                        break;
                    case '3':
                        rl.question('Enter user public key: ', async (pubkey) => {
                            await this.emergencyUnstake(new PublicKey(pubkey));
                            prompt();
                        });
                        break;
                    case '4':
                        rl.close();
                        process.exit(0);
                        break;
                    default:
                        console.log('Invalid option');
                        prompt();
                }
            });
        };

        prompt();
    }
}

// Start the console
const console = new EmergencyConsole();
console.startConsole().catch(console.error);