/*!
 * Constants Module - System-wide Configuration Values
 *
 * This module defines all system-wide constants and configuration values
 * used throughout the Solana AMM Index Token Strategy Engine v3.0.0.
 *
 * ## Organization
 *
 * Constants are organized into logical categories:
 * - **System Limits**: Maximum values for various system parameters
 * - **Performance Constants**: Performance-related configuration values
 * - **Financial Constants**: Financial and trading-related constants
 * - **Security Constants**: Security and risk management values
 * - **Time Constants**: Time-based configuration values
 * - **Validation Constants**: Input validation thresholds
 * - **Optimization Constants**: Performance optimization parameters
 *
 * ## Key Features
 *
 * - **Comprehensive Coverage**: All system constants in one place
 * - **Well-Documented**: Each constant includes detailed documentation
 * - **Type Safety**: Proper type definitions for all constants
 * - **Performance Optimized**: Constants optimized for Solana compute limits
 * - **Security Focused**: Security-first constant values
 */

use anchor_lang::prelude::*;

// ============================================================================
// SYSTEM LIMITS
// ============================================================================

/// Maximum number of tokens in a basket (gas optimization)
pub const MAX_TOKENS: usize = 50;

/// Maximum number of strategies per factory
pub const MAX_STRATEGIES_PER_FACTORY: usize = 100;

/// Maximum number of factories per program
pub const MAX_FACTORIES_PER_PROGRAM: usize = 1000;

/// Maximum strategy parameter size in bytes
pub const MAX_STRATEGY_PARAM_SIZE: usize = 1024;

/// Maximum basket composition size in bytes
pub const MAX_BASKET_COMPOSITION_SIZE: usize = 2048;

/// Maximum execution parameter size in bytes
pub const MAX_EXECUTION_PARAM_SIZE: usize = 512;

/// Maximum market data size in bytes
pub const MAX_MARKET_DATA_SIZE: usize = 4096;

/// Maximum cache entry size in bytes
pub const MAX_CACHE_ENTRY_SIZE: usize = 1024;

/// Maximum batch size for operations
pub const MAX_BATCH_SIZE: u32 = 50;

/// Maximum memory usage in bytes (optimized for Solana)
pub const MAX_MEMORY_BYTES: u64 = 1_000_000; // 1MB

/// Maximum compute units per transaction
pub const MAX_COMPUTE_UNITS: u64 = 200_000;

/// Maximum instruction data size
pub const MAX_INSTRUCTION_DATA_SIZE: usize = 10_240; // 10KB

// ============================================================================
// PERFORMANCE CONSTANTS
// ============================================================================

/// Default batch size for operations
pub const DEFAULT_BATCH_SIZE: u32 = 10;

/// Default cache TTL in seconds
pub const DEFAULT_CACHE_TTL: u64 = 300; // 5 minutes

/// Maximum cache TTL in seconds
pub const MAX_CACHE_TTL: u64 = 3600; // 1 hour

/// Minimum cache TTL in seconds
pub const MIN_CACHE_TTL: u64 = 60; // 1 minute

/// Default execution timeout in milliseconds
pub const DEFAULT_EXECUTION_TIMEOUT_MS: u64 = 5000; // 5 seconds

/// Maximum execution timeout in milliseconds
pub const MAX_EXECUTION_TIMEOUT_MS: u64 = 30000; // 30 seconds

/// Minimum execution timeout in milliseconds
pub const MIN_EXECUTION_TIMEOUT_MS: u64 = 1000; // 1 second

/// Default gas limit for operations
pub const DEFAULT_GAS_LIMIT: u64 = 100_000;

/// Maximum gas limit for operations
pub const MAX_GAS_LIMIT: u64 = 200_000;

/// Minimum gas limit for operations
pub const MIN_GAS_LIMIT: u64 = 10_000;

/// Performance monitoring interval in milliseconds
pub const PERFORMANCE_MONITORING_INTERVAL_MS: u64 = 100;

/// Memory monitoring threshold in bytes
pub const MEMORY_MONITORING_THRESHOLD: u64 = 500_000; // 500KB

/// Cache performance threshold (hit rate in basis points)
pub const CACHE_PERFORMANCE_THRESHOLD: u16 = 8000; // 80%

// ============================================================================
// FINANCIAL CONSTANTS
// ============================================================================

/// Basis points maximum (100%)
pub const BASIS_POINTS_MAX: u64 = 10_000;

/// Maximum slippage tolerance in basis points (10%)
pub const MAX_SLIPPAGE_BPS: u64 = 1_000;

/// Default slippage tolerance in basis points (0.5%)
pub const DEFAULT_SLIPPAGE_BPS: u64 = 50;

/// Minimum slippage tolerance in basis points (0.01%)
pub const MIN_SLIPPAGE_BPS: u64 = 1;

/// Maximum price impact in basis points (5%)
pub const MAX_PRICE_IMPACT_BPS: u64 = 500;

/// Default price impact tolerance in basis points (0.1%)
pub const DEFAULT_PRICE_IMPACT_BPS: u64 = 10;

/// Minimum price impact tolerance in basis points (0.001%)
pub const MIN_PRICE_IMPACT_BPS: u64 = 1;

/// Maximum fee in basis points (1%)
pub const MAX_FEE_BPS: u64 = 100;

/// Default fee in basis points (0.3%)
pub const DEFAULT_FEE_BPS: u64 = 30;

/// Minimum fee in basis points (0.01%)
pub const MIN_FEE_BPS: u64 = 1;

/// Maximum concentration in basis points (30%)
pub const MAX_CONCENTRATION_BPS: u64 = 3_000;

/// Default concentration limit in basis points (20%)
pub const DEFAULT_CONCENTRATION_BPS: u64 = 2_000;

/// Minimum concentration in basis points (1%)
pub const MIN_CONCENTRATION_BPS: u64 = 100;

/// Maximum volatility threshold in basis points (20%)
pub const MAX_VOLATILITY_BPS: u64 = 2_000;

/// Default volatility threshold in basis points (10%)
pub const DEFAULT_VOLATILITY_BPS: u64 = 1_000;

/// Minimum volatility threshold in basis points (1%)
pub const MIN_VOLATILITY_BPS: u64 = 100;

/// Maximum risk tolerance in basis points (100%)
pub const MAX_RISK_TOLERANCE_BPS: u64 = 10_000;

/// Default risk tolerance in basis points (50%)
pub const DEFAULT_RISK_TOLERANCE_BPS: u64 = 5_000;

/// Minimum risk tolerance in basis points (1%)
pub const MIN_RISK_TOLERANCE_BPS: u64 = 100;

/// Maximum profit threshold in basis points (100%)
pub const MAX_PROFIT_THRESHOLD_BPS: u64 = 10_000;

/// Default profit threshold in basis points (10%)
pub const DEFAULT_PROFIT_THRESHOLD_BPS: u64 = 1_000;

/// Minimum profit threshold in basis points (0.1%)
pub const MIN_PROFIT_THRESHOLD_BPS: u64 = 10;

// ============================================================================
// SECURITY CONSTANTS
// ============================================================================

/// Maximum authorization attempts
pub const MAX_AUTH_ATTEMPTS: u32 = 3;

/// Authorization timeout in seconds
pub const AUTH_TIMEOUT_SECONDS: u64 = 300; // 5 minutes

/// Maximum failed attempts before lockout
pub const MAX_FAILED_ATTEMPTS: u32 = 5;

/// Lockout duration in seconds
pub const LOCKOUT_DURATION_SECONDS: u64 = 1800; // 30 minutes

/// Maximum session duration in seconds
pub const MAX_SESSION_DURATION_SECONDS: u64 = 3600; // 1 hour

/// Minimum session duration in seconds
pub const MIN_SESSION_DURATION_SECONDS: u64 = 300; // 5 minutes

/// Security token expiry in seconds
pub const SECURITY_TOKEN_EXPIRY_SECONDS: u64 = 7200; // 2 hours

/// Maximum security token attempts
pub const MAX_SECURITY_TOKEN_ATTEMPTS: u32 = 3;

/// Circuit breaker activation threshold
pub const CIRCUIT_BREAKER_THRESHOLD: u32 = 5;

/// Circuit breaker reset timeout in seconds
pub const CIRCUIT_BREAKER_RESET_TIMEOUT: u64 = 300; // 5 minutes

/// Maximum concurrent operations per account
pub const MAX_CONCURRENT_OPERATIONS: u32 = 10;

/// Rate limiting window in seconds
pub const RATE_LIMIT_WINDOW_SECONDS: u64 = 60; // 1 minute

/// Maximum operations per rate limit window
pub const MAX_OPERATIONS_PER_WINDOW: u32 = 100;

// ============================================================================
// TIME CONSTANTS
// ============================================================================

/// Seconds in a minute
pub const SECONDS_PER_MINUTE: u64 = 60;

/// Seconds in an hour
pub const SECONDS_PER_HOUR: u64 = 3600;

/// Seconds in a day
pub const SECONDS_PER_DAY: u64 = 86400;

/// Seconds in a week
pub const SECONDS_PER_WEEK: u64 = 604800;

/// Seconds in a month (30 days)
pub const SECONDS_PER_MONTH: u64 = 2592000;

/// Seconds in a year (365 days)
pub const SECONDS_PER_YEAR: u64 = 31536000;

/// Minimum rebalancing interval in seconds
pub const MIN_REBALANCING_INTERVAL_SECONDS: u64 = 300; // 5 minutes

/// Default rebalancing interval in seconds
pub const DEFAULT_REBALANCING_INTERVAL_SECONDS: u64 = 3600; // 1 hour

/// Maximum rebalancing interval in seconds
pub const MAX_REBALANCING_INTERVAL_SECONDS: u64 = 86400; // 1 day

/// Minimum strategy update interval in seconds
pub const MIN_STRATEGY_UPDATE_INTERVAL_SECONDS: u64 = 60; // 1 minute

/// Default strategy update interval in seconds
pub const DEFAULT_STRATEGY_UPDATE_INTERVAL_SECONDS: u64 = 300; // 5 minutes

/// Maximum strategy update interval in seconds
pub const MAX_STRATEGY_UPDATE_INTERVAL_SECONDS: u64 = 3600; // 1 hour

/// Price feed staleness threshold in seconds
pub const PRICE_FEED_STALENESS_THRESHOLD_SECONDS: u64 = 300; // 5 minutes

/// Market data staleness threshold in seconds
pub const MARKET_DATA_STALENESS_THRESHOLD_SECONDS: u64 = 60; // 1 minute

/// Cache staleness threshold in seconds
pub const CACHE_STALENESS_THRESHOLD_SECONDS: u64 = 30; // 30 seconds

// ============================================================================
// VALIDATION CONSTANTS
// ============================================================================

/// Minimum token amount for operations
pub const MIN_TOKEN_AMOUNT: u64 = 1;

/// Maximum token amount for operations
pub const MAX_TOKEN_AMOUNT: u64 = 1_000_000_000_000_000; // 1 quadrillion

/// Minimum basket creation amount
pub const MIN_BASKET_CREATION_AMOUNT: u64 = 1_000_000; // 1 token with 6 decimals

/// Maximum basket creation amount
pub const MAX_BASKET_CREATION_AMOUNT: u64 = 1_000_000_000_000; // 1M tokens with 6 decimals

/// Minimum basket redemption amount
pub const MIN_BASKET_REDEMPTION_AMOUNT: u64 = 1_000_000; // 1 token with 6 decimals

/// Maximum basket redemption amount
pub const MAX_BASKET_REDEMPTION_AMOUNT: u64 = 1_000_000_000_000; // 1M tokens with 6 decimals

/// Minimum arbitrage amount
pub const MIN_ARBITRAGE_AMOUNT: u64 = 100_000; // 0.1 token with 6 decimals

/// Maximum arbitrage amount
pub const MAX_ARBITRAGE_AMOUNT: u64 = 100_000_000_000; // 100K tokens with 6 decimals

/// Minimum strategy parameter length
pub const MIN_STRATEGY_PARAM_LENGTH: usize = 1;

/// Maximum strategy parameter length
pub const MAX_STRATEGY_PARAM_LENGTH: usize = 1024;

/// Minimum token mint length
pub const MIN_TOKEN_MINT_LENGTH: usize = 32;

/// Maximum token mint length
pub const MAX_TOKEN_MINT_LENGTH: usize = 32;

/// Minimum weight value in basis points
pub const MIN_WEIGHT_BPS: u64 = 1;

/// Maximum weight value in basis points
pub const MAX_WEIGHT_BPS: u64 = 10_000;

// ============================================================================
// OPTIMIZATION CONSTANTS
// ============================================================================

/// Default optimization iterations
pub const DEFAULT_OPTIMIZATION_ITERATIONS: u32 = 100;

/// Maximum optimization iterations
pub const MAX_OPTIMIZATION_ITERATIONS: u32 = 1000;

/// Minimum optimization iterations
pub const MIN_OPTIMIZATION_ITERATIONS: u32 = 10;

/// Default convergence threshold
pub const DEFAULT_CONVERGENCE_THRESHOLD: u64 = 100; // 1 basis point

/// Maximum convergence threshold
pub const MAX_CONVERGENCE_THRESHOLD: u64 = 1000; // 10 basis points

/// Minimum convergence threshold
pub const MIN_CONVERGENCE_THRESHOLD: u64 = 1; // 0.01 basis points

/// Default genetic algorithm population size
pub const DEFAULT_GA_POPULATION_SIZE: u32 = 50;

/// Maximum genetic algorithm population size
pub const MAX_GA_POPULATION_SIZE: u32 = 200;

/// Minimum genetic algorithm population size
pub const MIN_GA_POPULATION_SIZE: u32 = 10;

/// Default genetic algorithm mutation rate (basis points)
pub const DEFAULT_GA_MUTATION_RATE_BPS: u64 = 100; // 1%

/// Maximum genetic algorithm mutation rate (basis points)
pub const MAX_GA_MUTATION_RATE_BPS: u64 = 1000; // 10%

/// Minimum genetic algorithm mutation rate (basis points)
pub const MIN_GA_MUTATION_RATE_BPS: u64 = 10; // 0.1%

/// Default genetic algorithm crossover rate (basis points)
pub const DEFAULT_GA_CROSSOVER_RATE_BPS: u64 = 7000; // 70%

/// Maximum genetic algorithm crossover rate (basis points)
pub const MAX_GA_CROSSOVER_RATE_BPS: u64 = 9500; // 95%

/// Minimum genetic algorithm crossover rate (basis points)
pub const MIN_GA_CROSSOVER_RATE_BPS: u64 = 5000; // 50%

/// Default parallel processing threads
pub const DEFAULT_PARALLEL_THREADS: u32 = 4;

/// Maximum parallel processing threads
pub const MAX_PARALLEL_THREADS: u32 = 16;

/// Minimum parallel processing threads
pub const MIN_PARALLEL_THREADS: u32 = 1;

// ============================================================================
// ALGORITHM CONSTANTS
// ============================================================================

/// Default TWAP duration in seconds
pub const DEFAULT_TWAP_DURATION_SECONDS: u64 = 3600; // 1 hour

/// Maximum TWAP duration in seconds
pub const MAX_TWAP_DURATION_SECONDS: u64 = 86400; // 1 day

/// Minimum TWAP duration in seconds
pub const MIN_TWAP_DURATION_SECONDS: u64 = 300; // 5 minutes

/// Default TWAP interval in seconds
pub const DEFAULT_TWAP_INTERVAL_SECONDS: u64 = 300; // 5 minutes

/// Maximum TWAP interval in seconds
pub const MAX_TWAP_INTERVAL_SECONDS: u64 = 3600; // 1 hour

/// Minimum TWAP interval in seconds
pub const MIN_TWAP_INTERVAL_SECONDS: u64 = 60; // 1 minute

/// Default VWAP lookback period in seconds
pub const DEFAULT_VWAP_LOOKBACK_SECONDS: u64 = 3600; // 1 hour

/// Maximum VWAP lookback period in seconds
pub const MAX_VWAP_LOOKBACK_SECONDS: u64 = 86400; // 1 day

/// Minimum VWAP lookback period in seconds
pub const MIN_VWAP_LOOKBACK_SECONDS: u64 = 300; // 5 minutes

/// Default market impact calculation window
pub const DEFAULT_MARKET_IMPACT_WINDOW: u32 = 100;

/// Maximum market impact calculation window
pub const MAX_MARKET_IMPACT_WINDOW: u32 = 1000;

/// Minimum market impact calculation window
pub const MIN_MARKET_IMPACT_WINDOW: u32 = 10;

/// Default risk assessment lookback period
pub const DEFAULT_RISK_ASSESSMENT_LOOKBACK: u32 = 100;

/// Maximum risk assessment lookback period
pub const MAX_RISK_ASSESSMENT_LOOKBACK: u32 = 1000;

/// Minimum risk assessment lookback period
pub const MIN_RISK_ASSESSMENT_LOOKBACK: u32 = 10;

// ============================================================================
// ERROR CONSTANTS
// ============================================================================

/// Maximum error message length
pub const MAX_ERROR_MESSAGE_LENGTH: usize = 256;

/// Maximum error context length
pub const MAX_ERROR_CONTEXT_LENGTH: usize = 512;

/// Maximum error stack trace length
pub const MAX_ERROR_STACK_TRACE_LENGTH: usize = 1024;

/// Default error retry attempts
pub const DEFAULT_ERROR_RETRY_ATTEMPTS: u32 = 3;

/// Maximum error retry attempts
pub const MAX_ERROR_RETRY_ATTEMPTS: u32 = 10;

/// Default error retry delay in milliseconds
pub const DEFAULT_ERROR_RETRY_DELAY_MS: u64 = 1000; // 1 second

/// Maximum error retry delay in milliseconds
pub const MAX_ERROR_RETRY_DELAY_MS: u64 = 10000; // 10 seconds

/// Minimum error retry delay in milliseconds
pub const MIN_ERROR_RETRY_DELAY_MS: u64 = 100; // 0.1 seconds

// ============================================================================
// LOGGING CONSTANTS
// ============================================================================

/// Maximum log message length
pub const MAX_LOG_MESSAGE_LENGTH: usize = 512;

/// Maximum log level length
pub const MAX_LOG_LEVEL_LENGTH: usize = 16;

/// Maximum log timestamp length
pub const MAX_LOG_TIMESTAMP_LENGTH: usize = 32;

/// Default log buffer size
pub const DEFAULT_LOG_BUFFER_SIZE: usize = 1000;

/// Maximum log buffer size
pub const MAX_LOG_BUFFER_SIZE: usize = 10000;

/// Minimum log buffer size
pub const MIN_LOG_BUFFER_SIZE: usize = 100;

/// Default log flush interval in milliseconds
pub const DEFAULT_LOG_FLUSH_INTERVAL_MS: u64 = 1000; // 1 second

/// Maximum log flush interval in milliseconds
pub const MAX_LOG_FLUSH_INTERVAL_MS: u64 = 10000; // 10 seconds

/// Minimum log flush interval in milliseconds
pub const MIN_LOG_FLUSH_INTERVAL_MS: u64 = 100; // 0.1 seconds

// ============================================================================
// VALIDATION FUNCTIONS
// ============================================================================

/// Validate batch size
pub fn is_valid_batch_size(batch_size: u32) -> bool {
    batch_size > 0 && batch_size <= MAX_BATCH_SIZE
}

/// Validate cache TTL
pub fn is_valid_cache_ttl(ttl: u64) -> bool {
    ttl >= MIN_CACHE_TTL && ttl <= MAX_CACHE_TTL
}

/// Validate slippage basis points
pub fn is_valid_slippage_bps(slippage_bps: u64) -> bool {
    slippage_bps >= MIN_SLIPPAGE_BPS && slippage_bps <= MAX_SLIPPAGE_BPS
}

/// Validate fee basis points
pub fn is_valid_fee_bps(fee_bps: u64) -> bool {
    fee_bps >= MIN_FEE_BPS && fee_bps <= MAX_FEE_BPS
}

/// Validate concentration basis points
pub fn is_valid_concentration_bps(concentration_bps: u64) -> bool {
    concentration_bps >= MIN_CONCENTRATION_BPS && concentration_bps <= MAX_CONCENTRATION_BPS
}

/// Validate volatility basis points
pub fn is_valid_volatility_bps(volatility_bps: u64) -> bool {
    volatility_bps >= MIN_VOLATILITY_BPS && volatility_bps <= MAX_VOLATILITY_BPS
}

/// Validate risk tolerance basis points
pub fn is_valid_risk_tolerance_bps(risk_tolerance_bps: u64) -> bool {
    risk_tolerance_bps >= MIN_RISK_TOLERANCE_BPS && risk_tolerance_bps <= MAX_RISK_TOLERANCE_BPS
}

/// Validate token amount
pub fn is_valid_token_amount(amount: u64) -> bool {
    amount >= MIN_TOKEN_AMOUNT && amount <= MAX_TOKEN_AMOUNT
}

/// Validate basket creation amount
pub fn is_valid_basket_creation_amount(amount: u64) -> bool {
    amount >= MIN_BASKET_CREATION_AMOUNT && amount <= MAX_BASKET_CREATION_AMOUNT
}

/// Validate basket redemption amount
pub fn is_valid_basket_redemption_amount(amount: u64) -> bool {
    amount >= MIN_BASKET_REDEMPTION_AMOUNT && amount <= MAX_BASKET_REDEMPTION_AMOUNT
}

/// Validate arbitrage amount
pub fn is_valid_arbitrage_amount(amount: u64) -> bool {
    amount >= MIN_ARBITRAGE_AMOUNT && amount <= MAX_ARBITRAGE_AMOUNT
}

/// Validate strategy parameter length
pub fn is_valid_strategy_param_length(length: usize) -> bool {
    length >= MIN_STRATEGY_PARAM_LENGTH && length <= MAX_STRATEGY_PARAM_LENGTH
}

/// Validate token count
pub fn is_valid_token_count(count: usize) -> bool {
    count > 0 && count <= MAX_TOKENS
}

/// Validate optimization iterations
pub fn is_valid_optimization_iterations(iterations: u32) -> bool {
    iterations >= MIN_OPTIMIZATION_ITERATIONS && iterations <= MAX_OPTIMIZATION_ITERATIONS
}

/// Validate genetic algorithm population size
pub fn is_valid_ga_population_size(size: u32) -> bool {
    size >= MIN_GA_POPULATION_SIZE && size <= MAX_GA_POPULATION_SIZE
}

/// Validate genetic algorithm mutation rate
pub fn is_valid_ga_mutation_rate(rate_bps: u64) -> bool {
    rate_bps >= MIN_GA_MUTATION_RATE_BPS && rate_bps <= MAX_GA_MUTATION_RATE_BPS
}

/// Validate genetic algorithm crossover rate
pub fn is_valid_ga_crossover_rate(rate_bps: u64) -> bool {
    rate_bps >= MIN_GA_CROSSOVER_RATE_BPS && rate_bps <= MAX_GA_CROSSOVER_RATE_BPS
}

/// Validate parallel processing threads
pub fn is_valid_parallel_threads(threads: u32) -> bool {
    threads >= MIN_PARALLEL_THREADS && threads <= MAX_PARALLEL_THREADS
}

/// Validate rebalancing interval
pub fn is_valid_rebalancing_interval(interval_seconds: u64) -> bool {
    interval_seconds >= MIN_REBALANCING_INTERVAL_SECONDS
        && interval_seconds <= MAX_REBALANCING_INTERVAL_SECONDS
}

/// Validate strategy update interval
pub fn is_valid_strategy_update_interval(interval_seconds: u64) -> bool {
    interval_seconds >= MIN_STRATEGY_UPDATE_INTERVAL_SECONDS
        && interval_seconds <= MAX_STRATEGY_UPDATE_INTERVAL_SECONDS
}

/// Validate TWAP duration
pub fn is_valid_twap_duration(duration_seconds: u64) -> bool {
    duration_seconds >= MIN_TWAP_DURATION_SECONDS && duration_seconds <= MAX_TWAP_DURATION_SECONDS
}

/// Validate TWAP interval
pub fn is_valid_twap_interval(interval_seconds: u64) -> bool {
    interval_seconds >= MIN_TWAP_INTERVAL_SECONDS && interval_seconds <= MAX_TWAP_INTERVAL_SECONDS
}

/// Validate VWAP lookback period
pub fn is_valid_vwap_lookback(lookback_seconds: u64) -> bool {
    lookback_seconds >= MIN_VWAP_LOOKBACK_SECONDS && lookback_seconds <= MAX_VWAP_LOOKBACK_SECONDS
}

/// Validate market impact window
pub fn is_valid_market_impact_window(window: u32) -> bool {
    window >= MIN_MARKET_IMPACT_WINDOW && window <= MAX_MARKET_IMPACT_WINDOW
}

/// Validate risk assessment lookback
pub fn is_valid_risk_assessment_lookback(lookback: u32) -> bool {
    lookback >= MIN_RISK_ASSESSMENT_LOOKBACK && lookback <= MAX_RISK_ASSESSMENT_LOOKBACK
}

/// Program ID for deployment (replace with actual ID in production)
pub const PROGRAM_ID: &str = "11111111111111111111111111111111";
