/*!
 * Advanced Trading Algorithms Module
 *
 * This module contains mature, production-ready implementations of sophisticated
 * trading algorithms used throughout the index token system.
 */

pub mod execution_optimizer;
pub mod market_impact;
pub mod risk_assessment;
pub mod smart_routing;
pub mod twap;
pub mod vwap;

// Re-export key types and functions
pub use execution_optimizer::*;
pub use market_impact::*;
pub use risk_assessment::*;
pub use smart_routing::*;
pub use twap::*;
pub use vwap::*;

use crate::core::*;
use crate::error::StrategyError;
use anchor_lang::prelude::*;

/// Algorithm execution result with comprehensive metrics
#[derive(Debug, Clone)]
pub struct AlgorithmResult {
    /// Algorithm type that was executed
    pub algorithm_type: AlgorithmType,
    /// Total volume processed
    pub volume_processed: u64,
    /// Execution efficiency score (0-10000)
    pub efficiency_score: u32,
    /// Gas consumption
    pub gas_used: u64,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
    /// Success status
    pub success: bool,
    /// Additional metrics
    pub metrics: AlgorithmMetrics,
}

/// Types of algorithms available
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AlgorithmType {
    TWAP,
    VWAP,
    SmartRouting,
    MarketMaking,
    Arbitrage,
    RiskAssessment,
}

/// Comprehensive algorithm metrics
#[derive(Debug, Clone, Default)]
pub struct AlgorithmMetrics {
    /// Slippage experienced in basis points
    pub slippage_bps: u64,
    /// Price improvement achieved in basis points
    pub price_improvement_bps: u64,
    /// MEV protection effectiveness (0-10000)
    pub mev_protection_score: u32,
    /// Liquidity utilization efficiency (0-10000)
    pub liquidity_efficiency: u32,
    /// Risk-adjusted return
    pub risk_adjusted_return: i64,
    /// Total operations performed
    pub total_operations: u64,
    /// Successful operations
    pub successful_operations: u64,
    /// Failed operations
    pub failed_operations: u64,
    /// Average execution time in milliseconds
    pub avg_execution_time_ms: u64,
    /// Total execution time in milliseconds
    pub total_execution_time_ms: u64,
    /// Last operation timestamp
    pub last_operation_timestamp: i64,
}

impl AlgorithmMetrics {
    /// Update metrics with operation result
    pub fn update_with_operation(&mut self, success: bool, execution_time_ms: u64) {
        self.total_operations += 1;
        if success {
            self.successful_operations += 1;
        } else {
            self.failed_operations += 1;
        }

        self.total_execution_time_ms += execution_time_ms;
        self.avg_execution_time_ms = self.total_execution_time_ms / self.total_operations;

        self.last_operation_timestamp = Clock::get().unwrap().unix_timestamp;
    }

    /// Get success rate in basis points
    pub fn success_rate_bps(&self) -> u16 {
        if self.total_operations > 0 {
            (self.successful_operations * 10_000) / self.total_operations
        } else {
            10_000
        }
    }

    /// Get error rate in basis points
    pub fn error_rate_bps(&self) -> u16 {
        if self.total_operations > 0 {
            (self.failed_operations * 10_000) / self.total_operations
        } else {
            0
        }
    }
}

/// Base trait for all trading algorithms
pub trait TradingAlgorithm {
    type Input;
    type Output;
    type Config;

    /// Execute the algorithm with given parameters
    fn execute(
        &mut self,
        input: Self::Input,
        config: &Self::Config,
        market_data: &EnhancedMarketData,
    ) -> StrategyResult<Self::Output>;

    /// Validate algorithm parameters before execution
    fn validate_parameters(&self, input: &Self::Input, config: &Self::Config)
        -> StrategyResult<()>;

    /// Get algorithm performance metrics
    fn get_metrics(&self) -> AlgorithmMetrics;

    /// Reset algorithm state
    fn reset(&mut self);
}

/// Algorithm factory for creating algorithm instances
pub struct AlgorithmFactory;

impl AlgorithmFactory {
    /// Create a new TWAP algorithm instance
    pub fn create_twap() -> TwapAlgorithm {
        TwapAlgorithm::new()
    }

    /// Create a new VWAP algorithm instance
    pub fn create_vwap() -> VwapAlgorithm {
        VwapAlgorithm::new()
    }

    /// Create a new smart routing algorithm instance
    pub fn create_smart_routing() -> SmartRoutingAlgorithm {
        SmartRoutingAlgorithm::new()
    }

    /// Create a new market impact calculator
    pub fn create_market_impact_calculator() -> MarketImpactCalculator {
        MarketImpactCalculator::new()
    }

    /// Create a new risk assessment engine
    pub fn create_risk_assessor() -> RiskAssessmentEngine {
        RiskAssessmentEngine::new()
    }

    /// Create an execution optimizer
    pub fn create_execution_optimizer() -> ExecutionOptimizer {
        ExecutionOptimizer::new()
    }
}
