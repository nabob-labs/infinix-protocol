/*!
 * Core Types Module - Optimized for Anchor 0.31.1
 *
 * This module defines fundamental types used throughout the system with:
 * - Comprehensive validation and error handling
 * - Type safety and bounds checking
 * - Performance-optimized data structures
 * - Solana-specific optimizations
 * - Clear documentation and examples
 */

use anchor_lang::prelude::*;

// ============================================================================
// RISK MANAGEMENT TYPES
// ============================================================================

/// Comprehensive risk metrics for portfolio assessment
/// All values are stored in basis points (10000 = 100%) for precision
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub struct RiskMetrics {
    /// Value at Risk at 95% confidence level in basis points
    pub var_95: u64,
    /// Value at Risk at 99% confidence level in basis points
    pub var_99: u64,
    /// Maximum drawdown in basis points
    pub max_drawdown: u64,
    /// Portfolio volatility in basis points
    pub volatility: u64,
    /// Sharpe ratio * 10000 for precision
    pub sharpe_ratio: i64,
    /// Beta * 10000 for precision (1.0 = 10000)
    pub beta: i64,
    /// VaR in basis points (current period)
    pub var_bps: u64,
    /// Concentration risk in basis points
    pub concentration_risk: u64,
    /// Overall risk score (0-10000, higher = more risky)
    pub overall_risk_score: u32,
    /// Maximum drawdown in basis points (historical)
    pub max_drawdown_bps: u64,
}

impl Default for RiskMetrics {
    fn default() -> Self {
        Self {
            var_95: 0,
            var_99: 0,
            max_drawdown: 0,
            volatility: 0,
            sharpe_ratio: 0,
            beta: 10_000, // Beta of 1.0
            var_bps: 0,
            concentration_risk: 0,
            overall_risk_score: 0,
            max_drawdown_bps: 0,
        }
    }
}

impl RiskMetrics {
    /// Create new risk metrics with validation
    pub fn new(
        var_95: u64,
        var_99: u64,
        max_drawdown: u64,
        volatility: u64,
        sharpe_ratio: i64,
        beta: i64,
        var_bps: u64,
        concentration_risk: u64,
        overall_risk_score: u32,
        max_drawdown_bps: u64,
    ) -> Result<Self> {
        // Validate all inputs are within reasonable bounds
        require!(
            var_95 <= BASIS_POINTS_MAX,
            crate::error::StrategyError::InvalidStrategyParameters
        );
        require!(
            var_99 <= BASIS_POINTS_MAX,
            crate::error::StrategyError::InvalidStrategyParameters
        );
        require!(
            max_drawdown <= BASIS_POINTS_MAX,
            crate::error::StrategyError::InvalidStrategyParameters
        );
        require!(
            volatility <= BASIS_POINTS_MAX,
            crate::error::StrategyError::InvalidStrategyParameters
        );
        require!(
            var_bps <= BASIS_POINTS_MAX,
            crate::error::StrategyError::InvalidStrategyParameters
        );
        require!(
            concentration_risk <= BASIS_POINTS_MAX,
            crate::error::StrategyError::InvalidStrategyParameters
        );
        require!(
            overall_risk_score <= 10_000,
            crate::error::StrategyError::InvalidStrategyParameters
        );
        require!(
            max_drawdown_bps <= BASIS_POINTS_MAX,
            crate::error::StrategyError::InvalidStrategyParameters
        );

        // Validate logical relationships
        require!(
            var_99 >= var_95,
            crate::error::StrategyError::InvalidStrategyParameters
        );
        require!(
            max_drawdown >= max_drawdown_bps,
            crate::error::StrategyError::InvalidStrategyParameters
        );

        Ok(Self {
            var_95,
            var_99,
            max_drawdown,
            volatility,
            sharpe_ratio,
            beta,
            var_bps,
            concentration_risk,
            overall_risk_score,
            max_drawdown_bps,
        })
    }

    /// Check if risk metrics indicate high risk
    pub fn is_high_risk(&self) -> bool {
        self.overall_risk_score > 7_000 || // 70% risk score
        self.var_95 > 2_000 || // 20% VaR
        self.concentration_risk > 3_000 || // 30% concentration
        self.max_drawdown > 1_500 // 15% drawdown
    }

    /// Check if risk metrics are within acceptable limits
    pub fn is_within_limits(&self, limits: &RiskLimits) -> bool {
        self.var_95 <= limits.max_var_bps
            && self.concentration_risk <= limits.max_concentration_bps
            && self.max_drawdown <= limits.max_drawdown_bps
            && self.overall_risk_score <= limits.max_risk_score
    }

    /// Calculate risk-adjusted return
    pub fn risk_adjusted_return(&self, return_bps: i64) -> i64 {
        if self.volatility == 0 {
            return 0;
        }
        // Sharpe ratio calculation: (return - risk_free_rate) / volatility
        // Assuming 0% risk-free rate for simplicity
        (return_bps * 10_000) / self.volatility as i64
    }
}

// ============================================================================
// MARKET DATA TYPES
// ============================================================================

/// Comprehensive market data structure for trading decisions
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct MarketData {
    /// Current price in smallest unit (e.g., lamports for SOL)
    pub price: u64,
    /// 24-hour trading volume in smallest unit
    pub volume_24h: u64,
    /// Market capitalization in smallest unit
    pub market_cap: u64,
    /// Available liquidity in smallest unit
    pub liquidity: u64,
    /// Timestamp when data was collected
    pub timestamp: i64,
}

impl Default for MarketData {
    fn default() -> Self {
        Self {
            price: 0,
            volume_24h: 0,
            market_cap: 0,
            liquidity: 0,
            timestamp: 0,
        }
    }
}

impl MarketData {
    /// Create new market data with validation
    pub fn new(
        price: u64,
        volume_24h: u64,
        market_cap: u64,
        liquidity: u64,
        timestamp: i64,
    ) -> Result<Self> {
        require!(price > 0, crate::error::StrategyError::InvalidMarketData);
        require!(
            timestamp > 0,
            crate::error::StrategyError::InvalidMarketData
        );

        Ok(Self {
            price,
            volume_24h,
            market_cap,
            liquidity,
            timestamp,
        })
    }

    /// Check if market data is stale (older than 5 minutes)
    pub fn is_stale(&self, current_time: i64) -> bool {
        current_time - self.timestamp > 300 // 5 minutes
    }

    /// Calculate price volatility from historical data
    pub fn calculate_volatility(&self, historical_prices: &[u64]) -> u64 {
        if historical_prices.len() < 2 {
            return 0;
        }

        let mut variance = 0u64;
        let mean = historical_prices.iter().sum::<u64>() / historical_prices.len() as u64;

        for &price in historical_prices {
            let diff = if price > mean {
                price - mean
            } else {
                mean - price
            };
            variance += diff * diff;
        }

        variance / historical_prices.len() as u64
    }

    /// Calculate bid-ask spread estimate
    pub fn estimate_spread(&self) -> u64 {
        // Simple spread estimation based on liquidity
        if self.liquidity == 0 {
            return 100; // 1% default spread
        }

        let spread_bps = (self.volume_24h * 100) / self.liquidity;
        spread_bps.min(500) // Cap at 5%
    }
}

// ============================================================================
// TOKEN INFORMATION TYPES
// ============================================================================

/// Comprehensive token information for portfolio management
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct TokenInfo {
    /// Token mint address
    pub mint: Pubkey,
    /// Token symbol (e.g., "SOL", "USDC")
    pub symbol: String,
    /// Token decimals (e.g., 9 for SOL, 6 for USDC)
    pub decimals: u8,
    /// Current price in smallest unit
    pub price: u64,
    /// Whether token is active for trading
    pub is_active: bool,
}

impl Default for TokenInfo {
    fn default() -> Self {
        Self {
            mint: Pubkey::default(),
            symbol: String::new(),
            decimals: 6,
            price: 0,
            is_active: true,
        }
    }
}

impl TokenInfo {
    /// Create new token info with validation
    pub fn new(
        mint: Pubkey,
        symbol: String,
        decimals: u8,
        price: u64,
        is_active: bool,
    ) -> Result<Self> {
        require!(
            !symbol.is_empty(),
            crate::error::StrategyError::InvalidStrategyParameters
        );
        require!(
            decimals <= 18,
            crate::error::StrategyError::InvalidStrategyParameters
        );
        require!(price > 0, crate::error::StrategyError::InvalidMarketData);

        Ok(Self {
            mint,
            symbol,
            decimals,
            price,
            is_active,
        })
    }

    /// Convert amount from smallest unit to human readable
    pub fn to_human_readable(&self, amount: u64) -> f64 {
        amount as f64 / (10_u64.pow(self.decimals as u32) as f64)
    }

    /// Convert human readable amount to smallest unit
    pub fn from_human_readable(&self, amount: f64) -> u64 {
        (amount * 10_u64.pow(self.decimals as u32) as f64) as u64
    }

    /// Calculate market value of token amount
    pub fn market_value(&self, amount: u64) -> u64 {
        amount * self.price / 10_u64.pow(self.decimals as u32)
    }
}

// ============================================================================
// WEIGHT ALLOCATION TYPES
// ============================================================================

/// Weight allocation for portfolio construction
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct WeightAllocation {
    /// Token mint address
    pub token_mint: Pubkey,
    /// Weight in basis points (10000 = 100%)
    pub weight_bps: u64,
}

impl Default for WeightAllocation {
    fn default() -> Self {
        Self {
            token_mint: Pubkey::default(),
            weight_bps: 0,
        }
    }
}

impl WeightAllocation {
    /// Create new weight allocation with validation
    pub fn new(token_mint: Pubkey, weight_bps: u64) -> Result<Self> {
        require!(
            weight_bps <= BASIS_POINTS_MAX,
            crate::error::StrategyError::InvalidWeightSum
        );

        Ok(Self {
            token_mint,
            weight_bps,
        })
    }

    /// Get weight as percentage (0.0 to 1.0)
    pub fn weight_percentage(&self) -> f64 {
        self.weight_bps as f64 / BASIS_POINTS_MAX as f64
    }

    /// Set weight from percentage (0.0 to 1.0)
    pub fn set_weight_percentage(&mut self, percentage: f64) -> Result<()> {
        require!(
            percentage >= 0.0 && percentage <= 1.0,
            crate::error::StrategyError::InvalidStrategyParameters
        );
        self.weight_bps = (percentage * BASIS_POINTS_MAX as f64) as u64;
        Ok(())
    }
}

// ============================================================================
// RISK LIMITS TYPES
// ============================================================================

/// Risk limits for portfolio management
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct RiskLimits {
    /// Maximum value at risk in basis points
    pub max_var_bps: u32,
    /// Maximum concentration in basis points
    pub max_concentration_bps: u32,
    /// Maximum drawdown in basis points
    pub max_drawdown_bps: u32,
    /// Maximum risk score
    pub max_risk_score: u32,
    /// Enable circuit breakers
    pub enable_circuit_breakers: bool,
}

impl Default for RiskLimits {
    fn default() -> Self {
        Self {
            max_var_bps: 500,            // 5%
            max_concentration_bps: 3000, // 30%
            max_drawdown_bps: 1000,      // 10%
            max_risk_score: 7500,        // 75%
            enable_circuit_breakers: true,
        }
    }
}

impl RiskLimits {
    /// Create new risk limits with validation
    pub fn new(
        max_var_bps: u64,
        max_concentration_bps: u64,
        max_drawdown_bps: u64,
        max_risk_score: u32,
    ) -> Result<Self> {
        require!(
            max_var_bps <= BASIS_POINTS_MAX,
            crate::error::StrategyError::InvalidStrategyParameters
        );
        require!(
            max_concentration_bps <= BASIS_POINTS_MAX,
            crate::error::StrategyError::InvalidStrategyParameters
        );
        require!(
            max_drawdown_bps <= BASIS_POINTS_MAX,
            crate::error::StrategyError::InvalidStrategyParameters
        );
        require!(
            max_risk_score <= 10_000,
            crate::error::StrategyError::InvalidStrategyParameters
        );

        Ok(Self {
            max_var_bps,
            max_concentration_bps,
            max_drawdown_bps,
            max_risk_score,
        })
    }

    /// Check if risk metrics violate limits
    pub fn is_violated(&self, metrics: &RiskMetrics) -> bool {
        metrics.var_95 > self.max_var_bps
            || metrics.concentration_risk > self.max_concentration_bps
            || metrics.max_drawdown > self.max_drawdown_bps
            || metrics.overall_risk_score > self.max_risk_score
    }
}

impl Validatable for RiskLimits {
    fn validate(&self) -> Result<()> {
        require!(
            self.max_var_bps <= 10000,
            crate::error::StrategyError::InvalidStrategyParameters
        );
        require!(
            self.max_concentration_bps <= 10000,
            crate::error::StrategyError::InvalidStrategyParameters
        );
        require!(
            self.max_drawdown_bps <= 10000,
            crate::error::StrategyError::InvalidStrategyParameters
        );
        require!(
            self.max_risk_score <= 10000,
            crate::error::StrategyError::InvalidStrategyParameters
        );
        Ok(())
    }
}

// ============================================================================
// VALIDATION TRAITS
// ============================================================================

/// Trait for types that can be validated
pub trait Validatable {
    /// Validate the type and return Result
    fn validate(&self) -> Result<()>;
}

impl Validatable for RiskMetrics {
    fn validate(&self) -> Result<()> {
        require!(
            self.var_95 <= BASIS_POINTS_MAX,
            crate::error::StrategyError::InvalidStrategyParameters
        );
        require!(
            self.var_99 <= BASIS_POINTS_MAX,
            crate::error::StrategyError::InvalidStrategyParameters
        );
        require!(
            self.max_drawdown <= BASIS_POINTS_MAX,
            crate::error::StrategyError::InvalidStrategyParameters
        );
        require!(
            self.volatility <= BASIS_POINTS_MAX,
            crate::error::StrategyError::InvalidStrategyParameters
        );
        require!(
            self.var_bps <= BASIS_POINTS_MAX,
            crate::error::StrategyError::InvalidStrategyParameters
        );
        require!(
            self.concentration_risk <= BASIS_POINTS_MAX,
            crate::error::StrategyError::InvalidStrategyParameters
        );
        require!(
            self.overall_risk_score <= 10_000,
            crate::error::StrategyError::InvalidStrategyParameters
        );
        require!(
            self.max_drawdown_bps <= BASIS_POINTS_MAX,
            crate::error::StrategyError::InvalidStrategyParameters
        );
        Ok(())
    }
}

impl Validatable for MarketData {
    fn validate(&self) -> Result<()> {
        require!(
            self.price > 0,
            crate::error::StrategyError::InvalidMarketData
        );
        require!(
            self.timestamp > 0,
            crate::error::StrategyError::InvalidMarketData
        );
        Ok(())
    }
}

impl Validatable for TokenInfo {
    fn validate(&self) -> Result<()> {
        require!(
            !self.symbol.is_empty(),
            crate::error::StrategyError::InvalidStrategyParameters
        );
        require!(
            self.decimals <= 18,
            crate::error::StrategyError::InvalidStrategyParameters
        );
        require!(
            self.price > 0,
            crate::error::StrategyError::InvalidMarketData
        );
        Ok(())
    }
}

impl Validatable for WeightAllocation {
    fn validate(&self) -> Result<()> {
        require!(
            self.weight_bps <= BASIS_POINTS_MAX,
            crate::error::StrategyError::InvalidWeightSum
        );
        Ok(())
    }
}
