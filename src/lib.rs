//! Caesar Economic System
//!
//! Complete economic backend for HyperMesh providing:
//! - Real-time token balance tracking
//! - Transaction processing and validation
//! - Reward calculation based on resource sharing
//! - Staking mechanisms with APY calculations
//! - Exchange operations and market rates

use anyhow::{Result, anyhow};
use async_trait::async_trait;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, put, delete},
    Router,
};
use chrono::{DateTime, Utc, Duration};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};
use uuid::Uuid;
use std::collections::HashMap;

// HyperMesh Asset System integration
#[cfg(feature = "hypermesh")]
use hypermesh::assets::core::{AssetManager, AssetId, AssetType, ConsensusProof, AssetAllocationRequest, ResourceRequirements};

pub mod models;
pub mod storage;
pub mod rewards;
pub mod staking;
pub mod exchange;
pub mod transactions;
pub mod analytics;
pub mod banking_interop_bridge;
pub mod banking_providers;
pub mod crypto_exchange_providers;
pub mod cross_chain_bridge;

use models::*;
use storage::CaesarStorage;
use rewards::RewardCalculator;
use staking::StakingManager;
use exchange::ExchangeEngine;
use transactions::TransactionProcessor;
use analytics::AnalyticsEngine;
use cross_chain_bridge::CrossChainBridge;

/// Caesar Economic System Configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CaesarConfig {
    /// Token economics configuration
    pub economics: EconomicsConfig,

    /// Reward calculation settings
    pub rewards: RewardConfig,

    /// Staking configuration
    pub staking: StakingConfig,

    /// Exchange settings
    pub exchange: ExchangeConfig,

    /// Database configuration
    pub database: DatabaseConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EconomicsConfig {
    /// Total token supply
    pub total_supply: Decimal,

    /// Initial distribution percentage
    pub initial_distribution: Decimal,

    /// Minimum transaction amount
    pub min_transaction: Decimal,

    /// Maximum transaction amount
    pub max_transaction: Decimal,

    /// Transaction fee percentage
    pub transaction_fee: Decimal,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RewardConfig {
    /// Base reward rate per hour for resource sharing
    pub base_rate_per_hour: Decimal,

    /// CPU sharing multiplier
    pub cpu_multiplier: Decimal,

    /// Memory sharing multiplier
    pub memory_multiplier: Decimal,

    /// Storage sharing multiplier
    pub storage_multiplier: Decimal,

    /// Network validation multiplier
    pub validation_multiplier: Decimal,

    /// Asset hosting multiplier
    pub hosting_multiplier: Decimal,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StakingConfig {
    /// Base APY percentage
    pub base_apy: Decimal,

    /// Minimum staking amount
    pub min_stake: Decimal,

    /// Maximum staking amount per wallet
    pub max_stake: Decimal,

    /// Unstaking cooldown period in hours
    pub unstaking_cooldown_hours: u32,

    /// Compound frequency in hours
    pub compound_frequency_hours: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExchangeConfig {
    /// CSR to USD exchange rate
    pub csr_usd_rate: Decimal,

    /// Market volatility percentage
    pub volatility: Decimal,

    /// Liquidity pool size
    pub liquidity_pool: Decimal,

    /// Slippage tolerance percentage
    pub slippage_tolerance: Decimal,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseConfig {
    /// Database URL
    pub url: String,

    /// Redis cache URL (optional)
    pub redis_url: Option<String>,

    /// Connection pool size
    pub pool_size: u32,
}

/// Main Caesar Economic System
pub struct CaesarEconomicSystem {
    /// Configuration
    config: Arc<CaesarConfig>,

    /// Storage layer
    storage: Arc<CaesarStorage>,

    /// Reward calculator
    rewards: Arc<RewardCalculator>,

    /// Staking manager
    staking: Arc<StakingManager>,

    /// Exchange engine
    exchange: Arc<ExchangeEngine>,

    /// Transaction processor
    transactions: Arc<TransactionProcessor>,

    /// Analytics engine
    analytics: Arc<AnalyticsEngine>,

    /// Cross-chain bridge for "mostly-stable" token
    bridge: Arc<CrossChainBridge>,

    /// Active sessions cache
    sessions: Arc<RwLock<HashMap<String, UserSession>>>,

    /// HyperMesh Asset Manager integration
    #[cfg(feature = "hypermesh")]
    asset_manager: Option<Arc<AssetManager>>,
}

/// User session for real-time data
#[derive(Debug, Clone)]
pub struct UserSession {
    pub wallet_id: String,
    pub last_update: DateTime<Utc>,
    pub cached_balance: Decimal,
    pub cached_rewards: Decimal,
    pub active_stakes: Vec<StakeInfo>,
}

impl CaesarEconomicSystem {
    /// Create new Caesar economic system
    pub async fn new(config: CaesarConfig) -> Result<Self> {
        info!("💰 Initializing Caesar Economic System");

        let config = Arc::new(config);

        // Initialize storage
        let storage = Arc::new(CaesarStorage::new(config.database.clone()).await?);

        // Initialize components
        let rewards = Arc::new(RewardCalculator::new(config.rewards.clone(), storage.clone()));
        let staking = Arc::new(StakingManager::new(config.staking.clone(), storage.clone()).await?);
        let exchange = Arc::new(ExchangeEngine::new(config.exchange.clone()));
        let transactions = Arc::new(TransactionProcessor::new(config.economics.clone(), storage.clone()).await?);
        let analytics = Arc::new(AnalyticsEngine::new(storage.clone()).await?);
        let bridge = Arc::new(CrossChainBridge::new().await?);

        let sessions = Arc::new(RwLock::new(HashMap::new()));

        Ok(Self {
            config,
            storage,
            rewards,
            staking,
            exchange,
            transactions,
            analytics,
            bridge,
            sessions,
            #[cfg(feature = "hypermesh")]
            asset_manager: None, // Will be set when integrated with HyperMesh
        })
    }

    /// Create API router for Caesar endpoints
    pub fn create_router(self: Arc<Self>) -> Router {
        Router::new()
            // Wallet endpoints
            .route("/api/v1/caesar/wallet", get(Self::get_wallet))
            .route("/api/v1/caesar/wallet/:id/balance", get(Self::get_balance))
            .route("/api/v1/caesar/wallet/create", post(Self::create_wallet))

            // Transaction endpoints
            .route("/api/v1/caesar/transactions", get(Self::get_transactions))
            .route("/api/v1/caesar/transactions/:id", get(Self::get_transaction))
            .route("/api/v1/caesar/transactions/send", post(Self::send_transaction))

            // Reward endpoints
            .route("/api/v1/caesar/rewards", get(Self::get_rewards))
            .route("/api/v1/caesar/rewards/claim", post(Self::claim_rewards))
            .route("/api/v1/caesar/rewards/history", get(Self::get_reward_history))
            .route("/api/v1/caesar/rewards/calculate", post(Self::calculate_rewards))

            // Staking endpoints
            .route("/api/v1/caesar/staking", get(Self::get_staking_info))
            .route("/api/v1/caesar/staking/stake", post(Self::stake_tokens))
            .route("/api/v1/caesar/staking/unstake", post(Self::unstake_tokens))
            .route("/api/v1/caesar/staking/rewards", get(Self::get_staking_rewards))

            // Exchange endpoints
            .route("/api/v1/caesar/exchange/rates", get(Self::get_exchange_rates))
            .route("/api/v1/caesar/exchange/swap", post(Self::swap_tokens))
            .route("/api/v1/caesar/exchange/liquidity", get(Self::get_liquidity_info))

            // Analytics endpoints
            .route("/api/v1/caesar/analytics/overview", get(Self::get_analytics_overview))
            .route("/api/v1/caesar/analytics/earnings", get(Self::get_earnings_breakdown))

            .with_state(self)
    }

    /// Get wallet information
    async fn get_wallet(
        State(caesar): State<Arc<CaesarEconomicSystem>>,
        Query(params): Query<HashMap<String, String>>,
    ) -> Result<Json<WalletResponse>, StatusCode> {
        let wallet_id = params.get("wallet_id")
            .ok_or(StatusCode::BAD_REQUEST)?;

        match caesar.get_wallet_info(wallet_id).await {
            Ok(wallet) => Ok(Json(wallet)),
            Err(e) => {
                error!("Failed to get wallet: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    /// Get wallet balance
    async fn get_balance(
        State(caesar): State<Arc<CaesarEconomicSystem>>,
        Path(wallet_id): Path<String>,
    ) -> Result<Json<BalanceResponse>, StatusCode> {
        match caesar.get_wallet_balance(&wallet_id).await {
            Ok(balance) => Ok(Json(balance)),
            Err(e) => {
                error!("Failed to get balance: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    /// Create new wallet
    async fn create_wallet(
        State(caesar): State<Arc<CaesarEconomicSystem>>,
        Json(request): Json<CreateWalletRequest>,
    ) -> Result<Json<WalletResponse>, StatusCode> {
        match caesar.create_new_wallet(request).await {
            Ok(wallet) => Ok(Json(wallet)),
            Err(e) => {
                error!("Failed to create wallet: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    /// Get transactions
    async fn get_transactions(
        State(caesar): State<Arc<CaesarEconomicSystem>>,
        Query(params): Query<HashMap<String, String>>,
    ) -> Result<Json<TransactionsResponse>, StatusCode> {
        let wallet_id = params.get("wallet_id")
            .ok_or(StatusCode::BAD_REQUEST)?;

        match caesar.get_wallet_transactions(wallet_id).await {
            Ok(transactions) => Ok(Json(transactions)),
            Err(e) => {
                error!("Failed to get transactions: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    /// Get specific transaction
    async fn get_transaction(
        State(caesar): State<Arc<CaesarEconomicSystem>>,
        Path(tx_id): Path<String>,
    ) -> Result<Json<TransactionResponse>, StatusCode> {
        match caesar.get_transaction_details(&tx_id).await {
            Ok(transaction) => Ok(Json(transaction)),
            Err(e) => {
                error!("Failed to get transaction: {}", e);
                Err(StatusCode::NOT_FOUND)
            }
        }
    }

    /// Send transaction
    async fn send_transaction(
        State(caesar): State<Arc<CaesarEconomicSystem>>,
        Json(request): Json<SendTransactionRequest>,
    ) -> Result<Json<TransactionResponse>, StatusCode> {
        match caesar.process_transaction(request).await {
            Ok(transaction) => Ok(Json(transaction)),
            Err(e) => {
                error!("Failed to send transaction: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    /// Get rewards information
    async fn get_rewards(
        State(caesar): State<Arc<CaesarEconomicSystem>>,
        Query(params): Query<HashMap<String, String>>,
    ) -> Result<Json<RewardsResponse>, StatusCode> {
        let wallet_id = params.get("wallet_id")
            .ok_or(StatusCode::BAD_REQUEST)?;

        match caesar.get_rewards_info(wallet_id).await {
            Ok(rewards) => Ok(Json(rewards)),
            Err(e) => {
                error!("Failed to get rewards: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    /// Claim pending rewards
    async fn claim_rewards(
        State(caesar): State<Arc<CaesarEconomicSystem>>,
        Json(request): Json<ClaimRewardsRequest>,
    ) -> Result<Json<ClaimRewardsResponse>, StatusCode> {
        match caesar.claim_pending_rewards(request).await {
            Ok(response) => Ok(Json(response)),
            Err(e) => {
                error!("Failed to claim rewards: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    /// Get reward history
    async fn get_reward_history(
        State(caesar): State<Arc<CaesarEconomicSystem>>,
        Query(params): Query<HashMap<String, String>>,
    ) -> Result<Json<RewardHistoryResponse>, StatusCode> {
        let wallet_id = params.get("wallet_id")
            .ok_or(StatusCode::BAD_REQUEST)?;

        match caesar.get_reward_history_for_wallet(wallet_id).await {
            Ok(history) => Ok(Json(history)),
            Err(e) => {
                error!("Failed to get reward history: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    /// Calculate rewards based on resource sharing
    async fn calculate_rewards(
        State(caesar): State<Arc<CaesarEconomicSystem>>,
        Json(request): Json<CalculateRewardsRequest>,
    ) -> Result<Json<CalculateRewardsResponse>, StatusCode> {
        match caesar.calculate_resource_rewards(request).await {
            Ok(response) => Ok(Json(response)),
            Err(e) => {
                error!("Failed to calculate rewards: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    /// Get staking information
    async fn get_staking_info(
        State(caesar): State<Arc<CaesarEconomicSystem>>,
        Query(params): Query<HashMap<String, String>>,
    ) -> Result<Json<StakingInfoResponse>, StatusCode> {
        let wallet_id = params.get("wallet_id")
            .ok_or(StatusCode::BAD_REQUEST)?;

        match caesar.get_staking_details(wallet_id).await {
            Ok(info) => Ok(Json(info)),
            Err(e) => {
                error!("Failed to get staking info: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    /// Stake tokens
    async fn stake_tokens(
        State(caesar): State<Arc<CaesarEconomicSystem>>,
        Json(request): Json<StakeRequest>,
    ) -> Result<Json<StakeResponse>, StatusCode> {
        match caesar.stake_tokens_for_wallet(request).await {
            Ok(response) => Ok(Json(response)),
            Err(e) => {
                error!("Failed to stake tokens: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    /// Unstake tokens
    async fn unstake_tokens(
        State(caesar): State<Arc<CaesarEconomicSystem>>,
        Json(request): Json<UnstakeRequest>,
    ) -> Result<Json<UnstakeResponse>, StatusCode> {
        match caesar.unstake_tokens_for_wallet(request).await {
            Ok(response) => Ok(Json(response)),
            Err(e) => {
                error!("Failed to unstake tokens: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    /// Get staking rewards
    async fn get_staking_rewards(
        State(caesar): State<Arc<CaesarEconomicSystem>>,
        Query(params): Query<HashMap<String, String>>,
    ) -> Result<Json<StakingRewardsResponse>, StatusCode> {
        let wallet_id = params.get("wallet_id")
            .ok_or(StatusCode::BAD_REQUEST)?;

        match caesar.calculate_staking_rewards(wallet_id).await {
            Ok(rewards) => Ok(Json(rewards)),
            Err(e) => {
                error!("Failed to get staking rewards: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    /// Get exchange rates
    async fn get_exchange_rates(
        State(caesar): State<Arc<CaesarEconomicSystem>>,
    ) -> Result<Json<ExchangeRatesResponse>, StatusCode> {
        match caesar.get_current_exchange_rates().await {
            Ok(rates) => Ok(Json(rates)),
            Err(e) => {
                error!("Failed to get exchange rates: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    /// Swap tokens
    async fn swap_tokens(
        State(caesar): State<Arc<CaesarEconomicSystem>>,
        Json(request): Json<SwapRequest>,
    ) -> Result<Json<SwapResponse>, StatusCode> {
        match caesar.execute_token_swap(request).await {
            Ok(response) => Ok(Json(response)),
            Err(e) => {
                error!("Failed to swap tokens: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    /// Get liquidity information
    async fn get_liquidity_info(
        State(caesar): State<Arc<CaesarEconomicSystem>>,
    ) -> Result<Json<LiquidityInfoResponse>, StatusCode> {
        match caesar.get_liquidity_pool_info().await {
            Ok(info) => Ok(Json(info)),
            Err(e) => {
                error!("Failed to get liquidity info: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    /// Get analytics overview
    async fn get_analytics_overview(
        State(caesar): State<Arc<CaesarEconomicSystem>>,
        Query(params): Query<HashMap<String, String>>,
    ) -> Result<Json<AnalyticsOverviewResponse>, StatusCode> {
        let wallet_id = params.get("wallet_id");

        match caesar.get_analytics_data(wallet_id).await {
            Ok(analytics) => Ok(Json(analytics)),
            Err(e) => {
                error!("Failed to get analytics: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    /// Get earnings breakdown
    async fn get_earnings_breakdown(
        State(caesar): State<Arc<CaesarEconomicSystem>>,
        Query(params): Query<HashMap<String, String>>,
    ) -> Result<Json<EarningsBreakdownResponse>, StatusCode> {
        let wallet_id = params.get("wallet_id")
            .ok_or(StatusCode::BAD_REQUEST)?;

        match caesar.get_earnings_details(wallet_id).await {
            Ok(earnings) => Ok(Json(earnings)),
            Err(e) => {
                error!("Failed to get earnings breakdown: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    // Public implementation methods for integration

    pub async fn get_wallet_info(&self, wallet_id: &str) -> Result<WalletResponse> {
        let wallet = self.storage.get_wallet(wallet_id).await?;
        let balance = self.storage.get_balance(wallet_id).await?;
        let pending_rewards = self.rewards.get_pending_rewards(wallet_id).await?;
        let staked = self.staking.get_staked_amount(wallet_id).await?;

        // Calculate USD value
        let total_csr = balance + pending_rewards + staked;
        let usd_value = self.exchange.calculate_usd_value(total_csr)?;

        Ok(WalletResponse {
            wallet_id: wallet_id.to_string(),
            balance,
            pending_rewards,
            staked_amount: staked,
            total_value_usd: usd_value,
            created_at: wallet.created_at,
            last_activity: wallet.last_activity,
        })
    }

    pub async fn get_wallet_balance(&self, wallet_id: &str) -> Result<BalanceResponse> {
        let balance = self.storage.get_balance(wallet_id).await?;
        let pending = self.rewards.get_pending_rewards(wallet_id).await?;
        let staked = self.staking.get_staked_amount(wallet_id).await?;

        Ok(BalanceResponse {
            available: balance,
            pending,
            staked,
            total: balance + pending + staked,
            updated_at: Utc::now(),
        })
    }

    pub async fn create_new_wallet(&self, request: CreateWalletRequest) -> Result<WalletResponse> {
        let wallet = self.storage.create_wallet(request).await?;
        self.get_wallet_info(&wallet.wallet_id).await
    }

    pub async fn get_wallet_transactions(&self, wallet_id: &str) -> Result<TransactionsResponse> {
        let transactions = self.storage.get_transactions(wallet_id, 50).await?;
        let total_count = transactions.len();

        Ok(TransactionsResponse {
            wallet_id: wallet_id.to_string(),
            transactions,
            total_count,
        })
    }

    pub async fn get_transaction_details(&self, tx_id: &str) -> Result<TransactionResponse> {
        self.storage.get_transaction(tx_id).await
    }

    pub async fn process_transaction(&self, request: SendTransactionRequest) -> Result<TransactionResponse> {
        self.transactions.process(request).await
    }

    pub async fn get_rewards_info(&self, wallet_id: &str) -> Result<RewardsResponse> {
        let pending = self.rewards.get_pending_rewards(wallet_id).await?;
        let earned_today = self.rewards.get_today_earnings(wallet_id).await?;
        let sources = self.rewards.get_earning_sources(wallet_id).await?;

        Ok(RewardsResponse {
            wallet_id: wallet_id.to_string(),
            pending_rewards: pending,
            earned_today,
            earning_sources: sources,
            next_payout: self.rewards.get_next_payout_time(),
        })
    }

    pub async fn claim_pending_rewards(&self, request: ClaimRewardsRequest) -> Result<ClaimRewardsResponse> {
        let claimed = self.rewards.claim_rewards(&request.wallet_id).await?;

        Ok(ClaimRewardsResponse {
            wallet_id: request.wallet_id,
            claimed_amount: claimed,
            transaction_id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
        })
    }

    pub async fn get_reward_history_for_wallet(&self, wallet_id: &str) -> Result<RewardHistoryResponse> {
        let history = self.storage.get_reward_history(wallet_id, 30).await?;

        Ok(RewardHistoryResponse {
            wallet_id: wallet_id.to_string(),
            history,
        })
    }

    pub async fn calculate_resource_rewards(&self, request: CalculateRewardsRequest) -> Result<CalculateRewardsResponse> {
        let rewards = self.rewards.calculate(request).await?;

        Ok(rewards)
    }

    pub async fn get_staking_details(&self, wallet_id: &str) -> Result<StakingInfoResponse> {
        let stakes = self.staking.get_stakes(wallet_id).await?;
        let total_staked = self.staking.get_staked_amount(wallet_id).await?;
        let apy = self.staking.get_current_apy();
        let rewards = self.staking.calculate_rewards(wallet_id).await?;

        Ok(StakingInfoResponse {
            wallet_id: wallet_id.to_string(),
            total_staked,
            active_stakes: stakes,
            current_apy: apy,
            accumulated_rewards: rewards,
        })
    }

    pub async fn stake_tokens_for_wallet(&self, request: StakeRequest) -> Result<StakeResponse> {
        self.staking.stake(request).await
    }

    pub async fn unstake_tokens_for_wallet(&self, request: UnstakeRequest) -> Result<UnstakeResponse> {
        self.staking.unstake(request).await
    }

    pub async fn calculate_staking_rewards(&self, wallet_id: &str) -> Result<StakingRewardsResponse> {
        let rewards = self.staking.calculate_rewards(wallet_id).await?;
        let breakdown = self.staking.get_rewards_breakdown(wallet_id).await?;

        Ok(StakingRewardsResponse {
            wallet_id: wallet_id.to_string(),
            total_rewards: rewards,
            breakdown,
            last_calculated: Utc::now(),
        })
    }

    pub async fn get_current_exchange_rates(&self) -> Result<ExchangeRatesResponse> {
        self.exchange.get_rates().await
    }

    pub async fn execute_token_swap(&self, request: SwapRequest) -> Result<SwapResponse> {
        self.exchange.swap(request).await
    }

    pub async fn get_liquidity_pool_info(&self) -> Result<LiquidityInfoResponse> {
        self.exchange.get_liquidity_info().await
    }

    pub async fn get_analytics_data(&self, wallet_id: Option<&String>) -> Result<AnalyticsOverviewResponse> {
        self.analytics.get_overview(wallet_id).await
    }

    pub async fn get_earnings_details(&self, wallet_id: &str) -> Result<EarningsBreakdownResponse> {
        self.analytics.get_earnings_breakdown(wallet_id).await
    }
}

/// Default configuration for development
impl Default for CaesarConfig {
    fn default() -> Self {
        Self {
            economics: EconomicsConfig {
                total_supply: dec!(1000000000), // 1 billion tokens
                initial_distribution: dec!(0.1), // 10% initial distribution
                min_transaction: dec!(0.01),
                max_transaction: dec!(1000000),
                transaction_fee: dec!(0.001), // 0.1% fee
            },
            rewards: RewardConfig {
                base_rate_per_hour: dec!(1.0),
                cpu_multiplier: dec!(2.0),
                memory_multiplier: dec!(1.5),
                storage_multiplier: dec!(1.2),
                validation_multiplier: dec!(3.0),
                hosting_multiplier: dec!(1.8),
            },
            staking: StakingConfig {
                base_apy: dec!(4.2),
                min_stake: dec!(10.0),
                max_stake: dec!(100000.0),
                unstaking_cooldown_hours: 72,
                compound_frequency_hours: 24,
            },
            exchange: ExchangeConfig {
                csr_usd_rate: dec!(1.48),
                volatility: dec!(0.05),
                liquidity_pool: dec!(10000000),
                slippage_tolerance: dec!(0.02),
            },
            database: DatabaseConfig {
                url: "sqlite::memory:".to_string(),
                redis_url: None,
                pool_size: 10,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_caesar_initialization() {
        let config = CaesarConfig::default();
        let caesar = CaesarEconomicSystem::new(config).await;
        assert!(caesar.is_ok());
    }

    #[tokio::test]
    async fn test_wallet_creation() {
        let config = CaesarConfig::default();
        let caesar = Arc::new(CaesarEconomicSystem::new(config).await.unwrap());

        let request = CreateWalletRequest {
            user_id: "test_user".to_string(),
            initial_balance: Some(dec!(100)),
        };

        let wallet = caesar.create_new_wallet(request).await;
        assert!(wallet.is_ok());
    }
}
// Type aliases for ethers compatibility
pub type BalanceAmount = rust_decimal::Decimal;
pub type SignerMiddleware = ();
pub type Address = String;
pub type Abi = String;
pub type U256 = u256;

#[derive(Clone, Copy, Debug)]
pub struct u256(pub u128, pub u128);
