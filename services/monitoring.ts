import * as anchor from '@project-serum/anchor';
import { Program } from '@project-serum/anchor';
import { Connection, PublicKey } from '@solana/web3.js';
import { GentStaking } from '../target/types/gent_staking';
import * as dotenv from 'dotenv';

dotenv.config();

class StakingMonitor {
    private program: Program<GentStaking>;
    private connection: Connection;
    private stakingPool: PublicKey;

    constructor() {
        this.connection = new Connection(process.env.SOLANA_RPC_URL || 'http://localhost:8899');
        const provider = new anchor.AnchorProvider(
            this.connection,
            new anchor.Wallet(anchor.web3.Keypair.generate()),
            { commitment: 'confirmed' }
        );
        this.program = new Program(GentStaking.IDL, new PublicKey(process.env.PROGRAM_ID!), provider);
    }

    async initialize() {
        [this.stakingPool] = await PublicKey.findProgramAddress(
            [Buffer.from('staking_pool')],
            this.program.programId
        );
    }

    async monitorPoolMetrics() {
        try {
            const poolData = await this.program.account.stakingPool.fetch(this.stakingPool);
            
            const metrics = {
                totalStaked: poolData.totalStaked.toString(),
                totalRewardsDistributed: poolData.totalRewardsDistributed.toString(),
                stakeCount: poolData.stakeCount.toString(),
                isPaused: poolData.paused,
                timestamp: new Date().toISOString()
            };

            console.log('\nPool Metrics:', metrics);
            
            // Here you could send metrics to a monitoring service
            // await this.sendMetricsToService(metrics);

        } catch (error) {
            console.error('Failed to fetch pool metrics:', error);
        }
    }

    async watchStakingEvents() {
        console.log('Watching for staking events...');
        
        this.connection.onProgramAccountChange(
            this.program.programId,
            async (accountInfo) => {
                try {
                    // Process account changes
                    const account = await this.program.coder.accounts.decode(
                        'stakingPool',
                        accountInfo.accountInfo.data
                    );
                    
                    console.log('Staking pool updated:', {
                        totalStaked: account.totalStaked.toString(),
                        stakeCount: account.stakeCount.toString(),
                        timestamp: new Date().toISOString()
                    });
                } catch (error) {
                    console.error('Failed to process account change:', error);
                }
            }
        );
    }

    async startMonitoring(interval: number = 60000) { // Default 1 minute
        await this.initialize();
        
        // Initial metrics
        await this.monitorPoolMetrics();
        
        // Start watching for events
        await this.watchStakingEvents();
        
        // Regular monitoring interval
        setInterval(async () => {
            await this.monitorPoolMetrics();
        }, interval);
    }
}

// Start the monitoring service
const monitor = new StakingMonitor();
monitor.startMonitoring().catch(console.error);