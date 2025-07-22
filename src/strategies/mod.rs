pub mod advanced_strategies;
pub mod rebalancing_strategies;
pub mod strategy_factory;
pub mod strategy_registry;
pub mod strategy_validator;
pub mod weight_strategies;

use crate::version::{ProgramVersion, Versioned, CURRENT_VERSION};
use anchor_lang::prelude::*;
// Removed conflicting borsh import

/// Enumeration of available weight strategy types
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, PartialEq, InitSpace)]
pub enum WeightStrategyType {
    /// Equal weight distribution across all tokens
    EqualWeight,
    /// Market capitalization weighted
    MarketCapWeighted,
    /// Price momentum based weighting
    MomentumWeighted,
    /// Volatility adjusted weighting
    VolatilityAdjusted,
    /// Custom fixed weights
    FixedWeight,
    /// Dynamic rebalancing based on technical indicators
    TechnicalIndicator,
}

impl From<u8> for WeightStrategyType {
    fn from(value: u8) -> Self {
        match value {
            0 => WeightStrategyType::EqualWeight,
            1 => WeightStrategyType::MarketCapWeighted,
            2 => WeightStrategyType::MomentumWeighted,
            3 => WeightStrategyType::VolatilityAdjusted,
            4 => WeightStrategyType::FixedWeight,
            5 => WeightStrategyType::TechnicalIndicator,
            _ => WeightStrategyType::EqualWeight,
        }
    }
}

impl From<WeightStrategyType> for u8 {
    fn from(strategy_type: WeightStrategyType) -> Self {
        match strategy_type {
            WeightStrategyType::EqualWeight => 0,
            WeightStrategyType::MarketCapWeighted => 1,
            WeightStrategyType::MomentumWeighted => 2,
            WeightStrategyType::VolatilityAdjusted => 3,
            WeightStrategyType::FixedWeight => 4,
            WeightStrategyType::TechnicalIndicator => 5,
        }
    }
}

/// Enumeration of available rebalancing strategy types
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, PartialEq, InitSpace)]
pub enum RebalancingStrategyType {
    /// Threshold-based rebalancing
    ThresholdBased,
    /// Time-based periodic rebalancing
    TimeBased,
    /// Volatility-triggered rebalancing
    VolatilityTriggered,
    /// Drift-based rebalancing
    DriftBased,
    /// Hybrid approach combining multiple triggers
    Hybrid,
}

impl From<u8> for RebalancingStrategyType {
    fn from(value: u8) -> Self {
        match value {
            0 => RebalancingStrategyType::ThresholdBased,
            1 => RebalancingStrategyType::TimeBased,
            2 => RebalancingStrategyType::VolatilityTriggered,
            3 => RebalancingStrategyType::DriftBased,
            4 => RebalancingStrategyType::Hybrid,
            _ => RebalancingStrategyType::ThresholdBased,
        }
    }
}

impl From<RebalancingStrategyType> for u8 {
    fn from(strategy_type: RebalancingStrategyType) -> Self {
        match strategy_type {
            RebalancingStrategyType::ThresholdBased => 0,
            RebalancingStrategyType::TimeBased => 1,
            RebalancingStrategyType::VolatilityTriggered => 2,
            RebalancingStrategyType::DriftBased => 3,
            RebalancingStrategyType::Hybrid => 4,
        }
    }
}

/// Parameters for equal weight strategy
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct EqualWeightParams {
    /// Number of tokens in the index
    pub token_count: u32,
}

/// Parameters for market cap weighted strategy
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct MarketCapWeightedParams {
    /// Minimum weight for any token (basis points)
    pub min_weight: u64,
    /// Maximum weight for any token (basis points)
    pub max_weight: u64,
    /// Rebalancing frequency in days
    pub rebalance_frequency: u32,
}

/// Parameters for momentum weighted strategy
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct MomentumWeightedParams {
    /// Lookback period for momentum calculation (days)
    pub lookback_period: u32,
    /// Momentum factor weight
    pub momentum_factor: u64,
    /// Base weight for each token (basis points)
    pub base_weight: u64,
}

/// Parameters for volatility adjusted strategy
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct VolatilityAdjustedParams {
    /// Volatility lookback period (days)
    pub volatility_period: u32,
    /// Risk aversion parameter
    pub risk_aversion: u64,
    /// Target volatility (basis points)
    pub target_volatility: u64,
}

/// Parameters for fixed weight strategy
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct FixedWeightParams {
    /// Fixed weights for each token (basis points, must sum to 10000)
    pub weights: Vec<u64>,
}

/// Parameters for threshold-based rebalancing
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct ThresholdRebalanceParams {
    /// Threshold deviation from target weight (basis points)
    pub threshold: u64,
    /// Minimum time between rebalances (seconds)
    pub min_interval: u64,
}

/// Parameters for time-based rebalancing
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct TimeBasedRebalanceParams {
    /// Rebalancing interval in seconds
    pub interval: u64,
    /// Whether to allow early rebalancing if threshold is exceeded
    pub allow_early_rebalance: bool,
    /// Early rebalance threshold (basis points)
    pub early_threshold: u64,
}

/// Parameters for volatility-triggered rebalancing
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct VolatilityTriggeredParams {
    /// Volatility threshold for triggering rebalance (basis points)
    pub volatility_threshold: u64,
    /// Lookback period for volatility calculation (seconds)
    pub volatility_period: u64,
    /// Minimum time between rebalances (seconds)
    pub min_interval: u64,
}

/// Parameters for drift-based rebalancing
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct DriftBasedParams {
    /// Cumulative drift threshold (basis points)
    pub drift_threshold: u64,
    /// Minimum time between rebalances (seconds)
    pub min_interval: u64,
    /// Drift calculation window (seconds)
    pub drift_window: u64,
}

/// Parameters for hybrid rebalancing strategy
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct HybridRebalanceParams {
    /// Enable threshold-based component
    pub enable_threshold: bool,
    /// Enable time-based component
    pub enable_time: bool,
    /// Enable volatility-triggered component
    pub enable_volatility: bool,
    /// Enable drift-based component
    pub enable_drift: bool,
    /// Combination strategy for triggers
    pub combination_strategy: HybridCombinationStrategy,
    /// Weight for threshold component
    pub threshold_weight: u32,
    /// Weight for time component
    pub time_weight: u32,
    /// Weight for volatility component
    pub volatility_weight: u32,
    /// Weight for drift component
    pub drift_weight: u32,
    /// Trigger threshold for weighted combination
    pub trigger_threshold: u32,
    /// Parameters for threshold strategy
    pub threshold_params: Vec<u8>,
    /// Parameters for time strategy
    pub time_params: Vec<u8>,
    /// Parameters for volatility strategy
    pub volatility_params: Vec<u8>,
    /// Parameters for drift strategy
    pub drift_params: Vec<u8>,
}

/// Combination strategies for hybrid rebalancing
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub enum HybridCombinationStrategy {
    /// Trigger if any component strategy triggers
    Any,
    /// Trigger if majority of enabled strategies trigger
    Majority,
    /// Trigger only if all enabled strategies trigger
    All,
    /// Use weighted combination of strategy signals
    Weighted,
}

impl Versioned for WeightStrategyType {
    fn version(&self) -> ProgramVersion {
        CURRENT_VERSION
    }

    fn set_version(&mut self, _version: ProgramVersion) {
        // Strategy types are immutable, version is always current
    }
}

impl Versioned for RebalancingStrategyType {
    fn version(&self) -> ProgramVersion {
        CURRENT_VERSION
    }

    fn set_version(&mut self, _version: ProgramVersion) {
        // Strategy types are immutable, version is always current
    }
}

/// Enhanced strategy configuration with version support
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct StrategyConfig {
    /// Strategy version
    pub version: ProgramVersion,
    /// Configuration ID
    pub config_id: u64,
    /// Strategy authority
    pub authority: Pubkey,
    /// Weight strategy configuration
    pub weight_config: WeightStrategyConfig,
    /// Rebalancing strategy configuration
    pub rebalancing_config: RebalancingStrategyConfig,
    /// Performance optimization settings
    pub optimization_settings: OptimizationSettings,
    /// Risk management settings
    pub risk_settings: RiskSettings,
    /// Creation timestamp
    pub created_at: i64,
    /// Last update timestamp
    pub updated_at: i64,
}

/// Weight strategy configuration
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct WeightStrategyConfig {
    pub strategy_type: WeightStrategyType,
    pub parameters: Vec<u8>,
    pub token_mints: Vec<Pubkey>,
    pub is_active: bool,
    pub last_calculation: i64,
}

/// Rebalancing strategy configuration
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct RebalancingStrategyConfig {
    pub strategy_type: RebalancingStrategyType,
    pub parameters: Vec<u8>,
    pub is_active: bool,
    pub last_rebalance: i64,
    pub next_rebalance: i64,
}

/// Performance optimization settings
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct OptimizationSettings {
    /// Enable calculation caching
    pub enable_caching: bool,
    /// Enable parallel processing
    pub enable_parallel: bool,
    /// Batch processing size
    pub batch_size: u32,
    /// Cache expiry time (seconds)
    pub cache_expiry: u64,
}

/// Risk management settings
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct RiskSettings {
    /// Maximum weight concentration (basis points)
    pub max_concentration: u64,
    /// Maximum daily rebalance frequency
    pub max_daily_rebalances: u32,
    /// Enable circuit breakers
    pub enable_circuit_breakers: bool,
    /// Volatility threshold for circuit breaker
    pub volatility_circuit_breaker: u64,
}

impl StrategyConfig {
    /// Create new strategy configuration
    pub fn new(
        config_id: u64,
        authority: Pubkey,
        weight_config: WeightStrategyConfig,
        rebalancing_config: RebalancingStrategyConfig,
    ) -> Result<Self> {
        let current_time = Clock::get()?.unix_timestamp;

        Ok(Self {
            version: CURRENT_VERSION,
            config_id,
            authority,
            weight_config,
            rebalancing_config,
            optimization_settings: OptimizationSettings {
                enable_caching: true,
                enable_parallel: false,
                batch_size: 10,
                cache_expiry: 300, // 5 minutes
            },
            risk_settings: RiskSettings {
                max_concentration: 5000, // 50%
                max_daily_rebalances: 4,
                enable_circuit_breakers: true,
                volatility_circuit_breaker: 2000, // 20%
            },
            created_at: current_time,
            updated_at: current_time,
        })
    }

    /// Update weight strategy configuration
    pub fn update_weight_config(&mut self, new_config: WeightStrategyConfig) -> Result<()> {
        self.weight_config = new_config;
        self.updated_at = Clock::get()?.unix_timestamp;
        Ok(())
    }

    /// Update rebalancing strategy configuration
    pub fn update_rebalancing_config(
        &mut self,
        new_config: RebalancingStrategyConfig,
    ) -> Result<()> {
        self.rebalancing_config = new_config;
        self.updated_at = Clock::get()?.unix_timestamp;
        Ok(())
    }

    /// Check if strategy needs rebalancing
    pub fn needs_rebalancing(&self) -> bool {
        let current_time = Clock::get().unwrap().unix_timestamp;
        self.rebalancing_config.is_active && current_time >= self.rebalancing_config.next_rebalance
    }

    /// Get strategy performance metrics
    pub fn get_performance_metrics(&self) -> StrategyPerformanceMetrics {
        StrategyPerformanceMetrics {
            config_id: self.config_id,
            last_weight_calculation: self.weight_config.last_calculation,
            last_rebalance: self.rebalancing_config.last_rebalance,
            next_rebalance: self.rebalancing_config.next_rebalance,
            uptime: Clock::get().unwrap().unix_timestamp - self.created_at,
            is_active: self.weight_config.is_active && self.rebalancing_config.is_active,
        }
    }
}

impl Versioned for StrategyConfig {
    fn version(&self) -> ProgramVersion {
        self.version
    }

    fn set_version(&mut self, version: ProgramVersion) {
        self.version = version;
        self.updated_at = Clock::get().unwrap().unix_timestamp;
    }

    fn migrate_to_1_0_1(&mut self) -> Result<()> {
        msg!(
            "Migrating StrategyConfig {} to version 1.0.1",
            self.config_id
        );
        // Add any specific migration logic
        Ok(())
    }

    fn migrate_to_1_1_0(&mut self) -> Result<()> {
        msg!(
            "Migrating StrategyConfig {} to version 1.1.0",
            self.config_id
        );
        // Add any specific migration logic
        Ok(())
    }
}

/// Strategy performance metrics
#[derive(Debug, Clone)]
pub struct StrategyPerformanceMetrics {
    pub config_id: u64,
    pub last_weight_calculation: i64,
    pub last_rebalance: i64,
    pub next_rebalance: i64,
    pub uptime: i64,
    pub is_active: bool,
}
