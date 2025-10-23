//! Caesar Storage Layer - Database operations

use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use rust_decimal::prelude::FromStr;
use sqlx::{SqlitePool, PgPool, Pool, Sqlite, Postgres, Any};
use std::sync::Arc;
use std::str::FromStr as StdFromStr;
use tracing::{info, error, debug};
use uuid::Uuid;

use crate::models::*;
use crate::DatabaseConfig;

/// Storage layer for Caesar economic system
pub struct CaesarStorage {
    pool: SqlitePool, // Using SQLite for now, can switch to Postgres in production
}

impl CaesarStorage {
    pub async fn new(config: DatabaseConfig) -> Result<Self> {
        info!("Initializing Caesar storage layer");

        // Create connection pool
        let pool = SqlitePool::connect(&config.url).await?;

        // Initialize database schema
        Self::initialize_schema(&pool).await?;

        Ok(Self { pool })
    }

    async fn initialize_schema(pool: &SqlitePool) -> Result<()> {
        // Create wallets table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS wallets (
                wallet_id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                balance REAL NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                last_activity TEXT NOT NULL,
                is_active BOOLEAN NOT NULL DEFAULT 1
            )
            "#
        )
        .execute(pool)
        .await?;

        // Create transactions table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS transactions (
                transaction_id TEXT PRIMARY KEY,
                from_wallet TEXT NOT NULL,
                to_wallet TEXT NOT NULL,
                amount REAL NOT NULL,
                transaction_type TEXT NOT NULL,
                status TEXT NOT NULL,
                fee REAL NOT NULL,
                description TEXT,
                timestamp TEXT NOT NULL,
                block_height INTEGER,
                FOREIGN KEY (from_wallet) REFERENCES wallets(wallet_id),
                FOREIGN KEY (to_wallet) REFERENCES wallets(wallet_id)
            )
            "#
        )
        .execute(pool)
        .await?;

        // Create rewards table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS rewards (
                reward_id TEXT PRIMARY KEY,
                wallet_id TEXT NOT NULL,
                amount REAL NOT NULL,
                reward_type TEXT NOT NULL,
                source TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                claimed BOOLEAN NOT NULL DEFAULT 0,
                FOREIGN KEY (wallet_id) REFERENCES wallets(wallet_id)
            )
            "#
        )
        .execute(pool)
        .await?;

        // Create stakes table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS stakes (
                stake_id TEXT PRIMARY KEY,
                wallet_id TEXT NOT NULL,
                amount REAL NOT NULL,
                start_date TEXT NOT NULL,
                lock_period_days INTEGER,
                apy REAL NOT NULL,
                accumulated_rewards REAL NOT NULL DEFAULT 0,
                is_active BOOLEAN NOT NULL DEFAULT 1,
                FOREIGN KEY (wallet_id) REFERENCES wallets(wallet_id)
            )
            "#
        )
        .execute(pool)
        .await?;

        // Create indices for performance
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_transactions_wallet ON transactions(from_wallet, to_wallet)")
            .execute(pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_rewards_wallet ON rewards(wallet_id)")
            .execute(pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_stakes_wallet ON stakes(wallet_id)")
            .execute(pool)
            .await?;

        info!("Database schema initialized");
        Ok(())
    }

    // Wallet operations

    pub async fn create_wallet(&self, request: CreateWalletRequest) -> Result<Wallet> {
        let wallet_id = format!("CSR_{}", Uuid::new_v4().to_string().replace("-", "").to_uppercase()[..12].to_string());
        let now = Utc::now();
        let initial_balance = request.initial_balance.unwrap_or(Decimal::ZERO);

        sqlx::query(
            r#"
            INSERT INTO wallets (wallet_id, user_id, balance, created_at, last_activity, is_active)
            VALUES (?, ?, ?, ?, ?, 1)
            "#
        )
        .bind(&wallet_id)
        .bind(&request.user_id)
        .bind(initial_balance.to_string())
        .bind(now.to_rfc3339())
        .bind(now.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(Wallet {
            wallet_id: wallet_id.clone(),
            user_id: request.user_id,
            balance: initial_balance,
            created_at: now,
            last_activity: now,
            is_active: true,
        })
    }

    pub async fn get_wallet(&self, wallet_id: &str) -> Result<Wallet> {
        let row = sqlx::query_as::<_, (String, String, f64, String, String, bool)>(
            "SELECT wallet_id, user_id, balance, created_at, last_activity, is_active FROM wallets WHERE wallet_id = ?"
        )
        .bind(wallet_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|_| anyhow!("Wallet not found"))?;

        Ok(Wallet {
            wallet_id: row.0,
            user_id: row.1,
            balance: Decimal::from_f64_retain(row.2).unwrap_or(Decimal::ZERO),
            created_at: DateTime::parse_from_rfc3339(&row.3)?.with_timezone(&Utc),
            last_activity: DateTime::parse_from_rfc3339(&row.4)?.with_timezone(&Utc),
            is_active: row.5,
        })
    }

    pub async fn get_balance(&self, wallet_id: &str) -> Result<Decimal> {
        let row = sqlx::query_as::<_, (f64,)>(
            "SELECT balance FROM wallets WHERE wallet_id = ?"
        )
        .bind(wallet_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|_| anyhow!("Wallet not found"))?;

        Ok(Decimal::from_f64_retain(row.0).unwrap_or(Decimal::ZERO))
    }

    pub async fn update_balance(&self, wallet_id: &str, new_balance: Decimal) -> Result<()> {
        sqlx::query(
            "UPDATE wallets SET balance = ?, last_activity = ? WHERE wallet_id = ?"
        )
        .bind(new_balance.to_string())
        .bind(Utc::now().to_rfc3339())
        .bind(wallet_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // Transaction operations

    pub async fn create_transaction(&self, transaction: Transaction) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO transactions (
                transaction_id, from_wallet, to_wallet, amount,
                transaction_type, status, fee, description, timestamp
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&transaction.transaction_id)
        .bind(&transaction.from_wallet)
        .bind(&transaction.to_wallet)
        .bind(transaction.amount.to_string())
        .bind(serde_json::to_string(&transaction.transaction_type)?)
        .bind(serde_json::to_string(&transaction.status)?)
        .bind(transaction.fee.to_string())
        .bind(&transaction.description)
        .bind(transaction.timestamp.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_transaction(&self, tx_id: &str) -> Result<TransactionResponse> {
        let row = sqlx::query_as::<_, (String, String, String, f64, String, String, f64, Option<String>, String, Option<i64>)>(
            r#"
            SELECT transaction_id, from_wallet, to_wallet, amount,
                   transaction_type, status, fee, description, timestamp, block_height
            FROM transactions WHERE transaction_id = ?
            "#
        )
        .bind(tx_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|_| anyhow!("Transaction not found"))?;

        Ok(TransactionResponse {
            transaction_id: row.0,
            from_wallet: row.1,
            to_wallet: row.2,
            amount: Decimal::from_f64_retain(row.3).unwrap_or(Decimal::ZERO),
            transaction_type: serde_json::from_str(&row.4)?,
            status: serde_json::from_str(&row.5)?,
            fee: Decimal::from_f64_retain(row.6).unwrap_or(Decimal::ZERO),
            description: row.7.unwrap_or_default(),
            timestamp: DateTime::parse_from_rfc3339(&row.8)?.with_timezone(&Utc),
            block_height: row.9.map(|h| h as u64),
            confirmation_count: 6, // Default confirmations
        })
    }

    pub async fn get_transactions(&self, wallet_id: &str, limit: usize) -> Result<Vec<Transaction>> {
        let rows = sqlx::query_as::<_, (String, String, String, f64, String, String, f64, Option<String>, String)>(
            r#"
            SELECT transaction_id, from_wallet, to_wallet, amount,
                   transaction_type, status, fee, description, timestamp
            FROM transactions
            WHERE from_wallet = ? OR to_wallet = ?
            ORDER BY timestamp DESC
            LIMIT ?
            "#
        )
        .bind(wallet_id)
        .bind(wallet_id)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await?;

        let mut transactions = Vec::new();
        for row in rows {
            transactions.push(Transaction {
                transaction_id: row.0,
                from_wallet: row.1,
                to_wallet: row.2,
                amount: Decimal::from_f64_retain(row.3).unwrap_or(Decimal::ZERO),
                transaction_type: serde_json::from_str(&row.4)?,
                status: serde_json::from_str(&row.5)?,
                fee: Decimal::from_f64_retain(row.6).unwrap_or(Decimal::ZERO),
                description: row.7.unwrap_or_default(),
                timestamp: DateTime::parse_from_rfc3339(&row.8)?.with_timezone(&Utc),
            });
        }

        Ok(transactions)
    }

    // Reward operations

    pub async fn create_reward(&self, reward: RewardEntry) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO rewards (reward_id, wallet_id, amount, reward_type, source, timestamp, claimed)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&reward.reward_id)
        .bind(&reward.wallet_id)
        .bind(reward.amount.to_string())
        .bind(serde_json::to_string(&reward.reward_type)?)
        .bind(serde_json::to_string(&reward.source)?)
        .bind(reward.timestamp.to_rfc3339())
        .bind(reward.claimed)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_pending_rewards(&self, wallet_id: &str) -> Result<Decimal> {
        let row = sqlx::query_as::<_, (f64,)>(
            "SELECT SUM(amount) FROM rewards WHERE wallet_id = ? AND claimed = 0"
        )
        .bind(wallet_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(Decimal::from_f64_retain(row.0).unwrap_or(Decimal::ZERO))
    }

    pub async fn get_reward_history(&self, wallet_id: &str, days: u32) -> Result<Vec<RewardEntry>> {
        let since = Utc::now() - chrono::Duration::days(days as i64);

        let rows = sqlx::query_as::<_, (String, String, f64, String, String, String, bool)>(
            r#"
            SELECT reward_id, wallet_id, amount, reward_type, source, timestamp, claimed
            FROM rewards
            WHERE wallet_id = ? AND timestamp > ?
            ORDER BY timestamp DESC
            "#
        )
        .bind(wallet_id)
        .bind(since.to_rfc3339())
        .fetch_all(&self.pool)
        .await?;

        let mut rewards = Vec::new();
        for row in rows {
            rewards.push(RewardEntry {
                reward_id: row.0,
                wallet_id: row.1,
                amount: Decimal::from_f64_retain(row.2).unwrap_or(Decimal::ZERO),
                reward_type: serde_json::from_str(&row.3)?,
                source: serde_json::from_str(&row.4)?,
                timestamp: DateTime::parse_from_rfc3339(&row.5)?.with_timezone(&Utc),
                claimed: row.6,
            });
        }

        Ok(rewards)
    }

    pub async fn claim_rewards(&self, wallet_id: &str) -> Result<Decimal> {
        let pending = self.get_pending_rewards(wallet_id).await?;

        sqlx::query(
            "UPDATE rewards SET claimed = 1 WHERE wallet_id = ? AND claimed = 0"
        )
        .bind(wallet_id)
        .execute(&self.pool)
        .await?;

        // Update wallet balance
        let current_balance = self.get_balance(wallet_id).await?;
        self.update_balance(wallet_id, current_balance + pending).await?;

        Ok(pending)
    }

    // Staking operations

    pub async fn create_stake(&self, stake: StakeInfo) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO stakes (
                stake_id, wallet_id, amount, start_date,
                lock_period_days, apy, accumulated_rewards, is_active
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&stake.stake_id)
        .bind(&stake.wallet_id)
        .bind(stake.amount.to_string())
        .bind(stake.start_date.to_rfc3339())
        .bind(stake.lock_period_days)
        .bind(stake.apy.to_string())
        .bind(stake.accumulated_rewards.to_string())
        .bind(stake.is_active)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_stakes(&self, wallet_id: &str) -> Result<Vec<StakeInfo>> {
        let rows = sqlx::query_as::<_, (String, String, f64, String, Option<i64>, f64, f64, bool)>(
            r#"
            SELECT stake_id, wallet_id, amount, start_date,
                   lock_period_days, apy, accumulated_rewards, is_active
            FROM stakes
            WHERE wallet_id = ? AND is_active = 1
            "#
        )
        .bind(wallet_id)
        .fetch_all(&self.pool)
        .await?;

        let mut stakes = Vec::new();
        for row in rows {
            stakes.push(StakeInfo {
                stake_id: row.0,
                wallet_id: row.1,
                amount: Decimal::from_f64_retain(row.2).unwrap_or(Decimal::ZERO),
                start_date: DateTime::parse_from_rfc3339(&row.3)?.with_timezone(&Utc),
                lock_period_days: row.4.map(|d| d as u32),
                apy: Decimal::from_f64_retain(row.5).unwrap_or(Decimal::ZERO),
                accumulated_rewards: Decimal::from_f64_retain(row.6).unwrap_or(Decimal::ZERO),
                is_active: row.7,
            });
        }

        Ok(stakes)
    }

    pub async fn get_total_staked(&self, wallet_id: &str) -> Result<Decimal> {
        let row = sqlx::query_as::<_, (f64,)>(
            "SELECT SUM(amount) FROM stakes WHERE wallet_id = ? AND is_active = 1"
        )
        .bind(wallet_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(Decimal::from_f64_retain(row.0).unwrap_or(Decimal::ZERO))
    }

    pub async fn update_stake_rewards(&self, stake_id: &str, rewards: Decimal) -> Result<()> {
        sqlx::query(
            "UPDATE stakes SET accumulated_rewards = accumulated_rewards + ? WHERE stake_id = ?"
        )
        .bind(rewards.to_string())
        .bind(stake_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn deactivate_stake(&self, stake_id: &str) -> Result<()> {
        sqlx::query(
            "UPDATE stakes SET is_active = 0 WHERE stake_id = ?"
        )
        .bind(stake_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}