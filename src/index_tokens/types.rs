/*!
 * Index Token Types
 * 
 * Type definitions for advanced index token trading functionality.
 */

use anchor_lang::prelude::*;
use ::borsh::{BorshDeserialize, BorshSerialize};

/// Advanced trading parameters
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct AdvancedTradingParams {
    /// Trading strategy type
    pub strategy_type: u8,
    /// Maximum slippage tolerance
    pub max_slippage_bps: u16,
    /// Execution timeout
    pub timeout_seconds: u32,
    /// Enable MEV protection
    pub enable_mev_protection: bool,
    /// Custom parameters
    pub custom_params: Vec<u8>,
}

/// Trading execution mode
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub enum TradingExecutionMode {
    /// Immediate execution
    Immediate,
    /// Gradual execution
    Gradual,
    /// Optimal timing
    Optimal,
    /// Custom mode
    Custom,
}

/// Market making parameters
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct MarketMakingParams {
    /// Base spread in basis points
    pub base_spread_bps: u16,
    /// Maximum position size
    pub max_position_size: u64,
    /// Inventory target
    pub inventory_target: u64,
    /// Risk adjustment factor
    pub risk_adjustment: u32,
}

/// Dynamic spread configuration
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct DynamicSpreadConfig {
    /// Minimum spread
    pub min_spread_bps: u16,
    /// Maximum spread
    pub max_spread_bps: u16,
    /// Volatility adjustment
    pub volatility_adjustment: u32,
    /// Liquidity adjustment
    pub liquidity_adjustment: u32,
}

/// Arbitrage parameters
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct ArbitrageParams {
    /// Minimum profit threshold
    pub min_profit_bps: u16,
    /// Maximum position size
    pub max_position_size: u64,
    /// Execution timeout
    pub timeout_seconds: u32,
    /// Enable cross-protocol arbitrage
    pub enable_cross_protocol: bool,
}

/// AMM route information
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct AMMRoute {
    /// AMM protocol identifier
    pub protocol_id: u8,
    /// Pool address
    pub pool_address: Pubkey,
    /// Expected price
    pub expected_price: u64,
    /// Liquidity depth
    pub liquidity_depth: u64,
}

/// Liquidity provision parameters
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct LiquidityProvisionParams {
    /// Target liquidity amount
    pub target_amount: u64,
    /// Price range width
    pub range_width_bps: u16,
    /// Rebalancing frequency
    pub rebalance_frequency: u32,
    /// Fee tier
    pub fee_tier: u16,
}

/// Dynamic range configuration
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct DynamicRangeConfig {
    /// Base range width
    pub base_range_bps: u16,
    /// Volatility multiplier
    pub volatility_multiplier: u32,
    /// Minimum range
    pub min_range_bps: u16,
    /// Maximum range
    pub max_range_bps: u16,
}

/// Algorithmic trading parameters
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct AlgorithmicTradingParams {
    /// Algorithm type
    pub algorithm_type: u8,
    /// Signal threshold
    pub signal_threshold: u32,
    /// Position sizing method
    pub position_sizing: u8,
    /// Risk management rules
    pub risk_rules: Vec<u8>,
}

/// Trading signal configuration
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct TradingSignalConfig {
    /// Signal sources
    pub signal_sources: Vec<u8>,
    /// Signal weights
    pub signal_weights: Vec<u32>,
    /// Confirmation threshold
    pub confirmation_threshold: u32,
    /// Signal timeout
    pub timeout_seconds: u32,
}

/// Portfolio optimization parameters
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct PortfolioOptimizationParams {
    /// Optimization objective
    pub objective: u8,
    /// Risk tolerance
    pub risk_tolerance: u32,
    /// Rebalancing threshold
    pub rebalance_threshold_bps: u16,
    /// Constraints
    pub constraints: Vec<u8>,
}

/// Risk model configuration
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct RiskModelConfig {
    /// Model type
    pub model_type: u8,
    /// Lookback period
    pub lookback_period: u32,
    /// Confidence level
    pub confidence_level: u16,
    /// Model parameters
    pub model_params: Vec<u8>,
}

impl Default for AdvancedTradingParams {
    fn default() -> Self {
        Self {
            strategy_type: 0,
            max_slippage_bps: 100,
            timeout_seconds: 300,
            enable_mev_protection: true,
            custom_params: Vec::new(),
        }
    }
}

impl Default for MarketMakingParams {
    fn default() -> Self {
        Self {
            base_spread_bps: 50,
            max_position_size: 1000000,
            inventory_target: 500000,
            risk_adjustment: 1000,
        }
    }
}

impl Default for ArbitrageParams {
    fn default() -> Self {
        Self {
            min_profit_bps: 10,
            max_position_size: 1000000,
            timeout_seconds: 60,
            enable_cross_protocol: true,
        }
    }
}

impl Default for LiquidityProvisionParams {
    fn default() -> Self {
        Self {
            target_amount: 1000000,
            range_width_bps: 200,
            rebalance_frequency: 3600,
            fee_tier: 30,
        }
    }
}

impl Default for AlgorithmicTradingParams {
    fn default() -> Self {
        Self {
            algorithm_type: 0,
            signal_threshold: 5000,
            position_sizing: 0,
            risk_rules: Vec::new(),
        }
    }
}

impl Default for PortfolioOptimizationParams {
    fn default() -> Self {
        Self {
            objective: 0,
            risk_tolerance: 5000,
            rebalance_threshold_bps: 500,
            constraints: Vec::new(),
        }
    }
}