/*!
 * Error Handling Module for Solana AMM Index Token Strategies
 *
 * This module defines comprehensive error types for the entire strategy system.
 * Each error is carefully categorized and provides clear, actionable error messages
 * to help developers and users understand what went wrong and how to fix it.
 *
 * ## Error Categories
 *
 * 1. **Strategy Configuration Errors**: Invalid strategy types or parameters
 * 2. **Mathematical Errors**: Overflow, division by zero, invalid calculations
 * 3. **Market Data Errors**: Price feed issues, liquidity problems
 * 4. **Execution Errors**: Trading failures, slippage violations
 * 5. **Authorization Errors**: Access control violations
 * 6. **State Errors**: Invalid strategy states or timing issues
 *
 * ## Error Handling Best Practices
 *
 * - All errors include descriptive messages for debugging
 * - Error codes are designed to be machine-readable for automated handling
 * - Critical errors (like math overflow) are distinguished from recoverable errors
 * - Error messages provide context about what operation failed and why
 */

use anchor_lang::prelude::*;

/// Comprehensive error codes for the index token strategy system
///
/// This enum defines all possible error conditions that can occur during
/// strategy creation, execution, and management. Each error includes a
/// descriptive message to aid in debugging and user experience.
///
/// # Error Code Ranges
/// - 6000-6099: Strategy configuration and validation errors
/// - 6100-6199: Mathematical and calculation errors  
/// - 6200-6299: Market data and liquidity errors
/// - 6300-6399: Execution and trading errors
/// - 6400-6499: Authorization and access control errors
/// - 6500-6599: State management and timing errors
#[error_code]
pub enum StrategyError {
    // Strategy Configuration Errors (6000-6099)
    /// Invalid weight strategy type specified
    ///
    /// This error occurs when an unsupported or invalid weight strategy type
    /// is provided during strategy creation. Valid types include EqualWeight,
    /// MarketCapWeighted, MomentumWeighted, VolatilityAdjusted, FixedWeight,
    /// and TechnicalIndicator.
    #[msg("Invalid weight strategy type - must be one of: EqualWeight, MarketCapWeighted, MomentumWeighted, VolatilityAdjusted, FixedWeight, TechnicalIndicator")]
    InvalidWeightStrategyType,

    /// Invalid rebalancing strategy type specified
    ///
    /// This error occurs when an unsupported rebalancing strategy type is
    /// provided. Valid types include ThresholdBased, TimeBased, VolatilityTriggered,
    /// DriftBased, and Hybrid.
    #[msg("Invalid rebalancing strategy type - must be one of: ThresholdBased, TimeBased, VolatilityTriggered, DriftBased, Hybrid")]
    InvalidRebalancingStrategyType,

    /// Strategy parameters are invalid or malformed
    ///
    /// This error occurs when strategy-specific parameters fail validation.
    /// Common causes include out-of-range values, missing required parameters,
    /// or parameters that conflict with each other.
    #[msg("Invalid strategy parameters - check parameter ranges and required fields")]
    InvalidStrategyParameters,

    /// Token weights do not sum to 100% (10000 basis points)
    ///
    /// All weight allocations must sum to exactly 10000 basis points (100%).
    /// This ensures complete portfolio allocation without over or under-allocation.
    #[msg("Weights do not sum to 100% (10000 basis points) - total weight allocation must equal 10000")]
    InvalidWeightSum,

    // Mathematical and Calculation Errors (6100-6199)
    /// Mathematical operation resulted in overflow
    ///
    /// This critical error occurs when arithmetic operations exceed the maximum
    /// value for the data type. This is a safety mechanism to prevent undefined
    /// behavior and potential security vulnerabilities.
    #[msg("Mathematical overflow detected - operation result exceeds maximum value")]
    MathOverflow,

    /// Division by zero attempted
    ///
    /// This error occurs when a calculation attempts to divide by zero, which
    /// would result in undefined behavior. Common causes include empty portfolios
    /// or zero liquidity conditions.
    #[msg("Division by zero attempted - check for zero values in calculations")]
    DivisionByZero,

    /// Invalid number of tokens specified
    ///
    /// This error occurs when the token count is outside acceptable limits.
    /// Minimum is typically 1 token, maximum is usually 50 tokens for gas efficiency.
    #[msg("Invalid token count - must be between 1 and 50 tokens")]
    InvalidTokenCount,

    // Market Data and Liquidity Errors (6200-6299)
    /// Price feed data is unavailable or invalid
    ///
    /// This error occurs when required price data is missing, stale, or invalid.
    /// Price feeds must be recent (typically within 5 minutes) and have valid prices.
    #[msg("Price feed unavailable or invalid - check price feed freshness and validity")]
    PriceFeedUnavailable,

    /// Invalid market data provided
    ///
    /// This error occurs when market data is malformed, inconsistent, or contains
    /// invalid values that cannot be used for calculations.
    #[msg("Invalid market data - check data consistency and format")]
    InvalidMarketData,

    /// Insufficient liquidity for the requested operation
    ///
    /// This error occurs when there isn't enough liquidity in the market to
    /// execute the requested trade or rebalancing operation at acceptable prices.
    #[msg(
        "Insufficient liquidity for rebalancing - reduce trade size or increase slippage tolerance"
    )]
    InsufficientLiquidity,

    // Execution and Trading Errors (6300-6399)
    /// Rebalancing threshold requirements not met
    ///
    /// This error occurs when attempting to rebalance but the deviation from
    /// target weights hasn't reached the configured threshold, or insufficient
    /// time has passed since the last rebalancing.
    #[msg("Rebalancing threshold not met - weight deviation or time interval insufficient")]
    RebalancingThresholdNotMet,

    /// Slippage tolerance exceeded during execution
    ///
    /// This error occurs when the actual slippage during trade execution exceeds
    /// the maximum allowed slippage tolerance. This protects against excessive
    /// price impact and unfavorable execution.
    #[msg("Slippage tolerance exceeded - actual slippage higher than maximum allowed")]
    SlippageExceeded,

    /// Strategy execution failed due to market conditions or constraints
    ///
    /// This general execution error occurs when a strategy cannot be executed
    /// due to various market conditions, constraint violations, or system issues.
    #[msg("Strategy execution failed - check market conditions and constraints")]
    StrategyExecutionFailed,

    // Authorization and Access Control Errors (6400-6499)
    /// Unauthorized access attempt
    ///
    /// This error occurs when an account attempts to perform an operation
    /// without proper authorization. Only strategy owners and authorized
    /// accounts can modify strategies.
    #[msg("Unauthorized access - only strategy authority can perform this operation")]
    Unauthorized,

    // State Management and Timing Errors (6500-6599)
    /// Strategy is currently paused and cannot execute
    ///
    /// This error occurs when attempting to use a strategy that has been
    /// paused by its owner or due to risk management conditions.
    #[msg("Strategy is paused - unpause strategy before execution")]
    StrategyPaused,

    /// Invalid time window for the operation
    ///
    /// This error occurs when time-based parameters are invalid, such as
    /// minimum intervals that are too short or maximum intervals that are
    /// too long for practical use.
    #[msg("Invalid time window - check minimum and maximum time intervals")]
    InvalidTimeWindow,

    // Additional Advanced Errors for Basket Trading
    /// Basket creation amount below minimum threshold
    ///
    /// This error occurs when attempting to create a basket with an amount
    /// below the minimum required threshold for economic viability.
    #[msg("Basket creation amount below minimum threshold")]
    BasketAmountTooSmall,

    /// Basket redemption amount exceeds available supply
    ///
    /// This error occurs when attempting to redeem more basket tokens than
    /// are currently in circulation or available for redemption.
    #[msg("Basket redemption amount exceeds available supply")]
    BasketRedemptionExceedsSupply,

    /// Arbitrage opportunity insufficient for profitable execution
    ///
    /// This error occurs when an arbitrage opportunity exists but the profit
    /// margin is too small to cover transaction costs and provide meaningful returns.
    #[msg("Arbitrage opportunity insufficient for profitable execution")]
    ArbitrageNotProfitable,

    /// Risk limits exceeded - operation blocked by risk management
    ///
    /// This error occurs when an operation would violate configured risk limits
    /// such as maximum position size, concentration limits, or drawdown thresholds.
    #[msg("Risk limits exceeded - operation blocked by risk management system")]
    RiskLimitsExceeded,

    /// Circuit breaker activated due to extreme market conditions
    ///
    /// This error occurs when circuit breakers are triggered due to extreme
    /// market volatility, liquidity drain, or other exceptional conditions.
    #[msg("Circuit breaker activated due to extreme market conditions")]
    CircuitBreakerActivated,

    /// Execution optimizer not available or inactive
    ///
    /// This error occurs when advanced execution optimization is requested
    /// but the execution optimizer is not available or has been deactivated.
    #[msg("Execution optimizer not available or inactive")]
    ExecutionOptimizerUnavailable,

    // Performance and Optimization Errors (6600-6699)
    /// Batch operation size exceeds maximum limit
    ///
    /// This error occurs when attempting to batch more operations than
    /// the system can handle in a single transaction due to compute limits.
    #[msg("Batch operation size exceeds maximum limit - reduce batch size")]
    BatchSizeExceeded,

    /// Memory optimization failed due to data constraints
    ///
    /// This error occurs when memory optimization cannot be applied due to
    /// data structure constraints or size limitations.
    #[msg("Memory optimization failed - check data structure constraints")]
    MemoryOptimizationFailed,

    /// Compute budget exceeded during optimization
    ///
    /// This error occurs when optimization algorithms consume too much
    /// compute budget, preventing transaction completion.
    #[msg("Compute budget exceeded during optimization - simplify operation")]
    ComputeBudgetExceeded,

    /// Cache miss rate too high for optimal performance
    ///
    /// This error occurs when cache performance degrades below acceptable
    /// thresholds, indicating need for cache optimization or data restructuring.
    #[msg("Cache performance degraded - optimize data access patterns")]
    CachePerformanceDegraded,

    // Version Management Errors (6700-6799)
    /// Program version incompatibility detected
    ///
    /// This error occurs when trying to use an account or feature with
    /// an incompatible program version.
    #[msg("Program version incompatibility - upgrade required")]
    IncompatibleVersion,

    /// Feature not supported in current version
    ///
    /// This error occurs when trying to use a feature that is not
    /// available in the current program version.
    #[msg("Feature not supported in current program version")]
    FeatureNotSupported,

    /// Migration failed during version upgrade
    ///
    /// This error occurs when automatic migration of account data
    /// fails during a version upgrade process.
    #[msg("Account migration failed during version upgrade")]
    MigrationFailed,

    /// Invalid version format or specification
    ///
    /// This error occurs when version information is malformed
    /// or contains invalid version numbers.
    #[msg("Invalid version format or specification")]
    InvalidVersion,

    // Advanced Strategy Errors (6800-6899)
    /// AI optimization model not available
    ///
    /// This error occurs when AI-powered optimization is requested
    /// but the required models are not loaded or available.
    #[msg("AI optimization model not available")]
    AIModelUnavailable,

    /// Machine learning prediction failed
    ///
    /// This error occurs when ML-based predictions fail due to
    /// insufficient data or model errors.
    #[msg("Machine learning prediction failed")]
    MLPredictionFailed,

    /// Advanced strategy configuration invalid
    ///
    /// This error occurs when advanced strategy parameters are
    /// outside acceptable ranges or conflict with each other.
    #[msg("Advanced strategy configuration invalid")]
    AdvancedStrategyConfigInvalid,

    /// Cross-chain operation not supported
    ///
    /// This error occurs when attempting cross-chain operations
    /// that are not supported in the current configuration.
    #[msg("Cross-chain operation not supported")]
    CrossChainNotSupported,
}
