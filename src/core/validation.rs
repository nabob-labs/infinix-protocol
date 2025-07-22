/*!
 * Validation Module
 *
 * This module provides comprehensive validation functions for:
 * - Input parameter validation
 * - Account state validation
 * - Business logic constraints
 * - Security checks
 * - Data integrity verification
 */

use crate::core::constants::*;
use crate::core::types::*;
use crate::error::StrategyError;
use anchor_lang::prelude::*;

/// Validator trait - 可插拔参数/状态/业务校验器
pub trait Validator<T>: Send + Sync {
    fn validate(&self, value: &T) -> Result<()>;
}

/// 工厂方法注册表
pub struct ValidatorRegistry<T> {
    validators: Vec<Box<dyn Validator<T>>>,
}

impl<T> ValidatorRegistry<T> {
    pub fn new() -> Self {
        Self {
            validators: Vec::new(),
        }
    }
    pub fn register(&mut self, validator: Box<dyn Validator<T>>) {
        self.validators.push(validator);
    }
    pub fn validate_all(&self, value: &T) -> Result<()> {
        for v in &self.validators {
            v.validate(value)?;
        }
        Ok(())
    }
}

/// 典型校验器实现
pub struct NotEmptyValidator;
impl<T: AsRef<[U]>> Validator<T> for NotEmptyValidator {
    fn validate(&self, value: &T) -> Result<()> {
        if value.as_ref().is_empty() {
            return Err(crate::error::StrategyError::InvalidStrategyParameters.into());
        }
        Ok(())
    }
}

pub struct RangeValidator {
    pub min: u64,
    pub max: u64,
}
impl Validator<u64> for RangeValidator {
    fn validate(&self, value: &u64) -> Result<()> {
        if *value < self.min || *value > self.max {
            return Err(crate::error::StrategyError::InvalidStrategyParameters.into());
        }
        Ok(())
    }
}

/// 业务校验器示例
pub struct WeightsSumValidator;
impl Validator<Vec<u64>> for WeightsSumValidator {
    fn validate(&self, value: &Vec<u64>) -> Result<()> {
        if value.iter().sum::<u64>() != 10_000 {
            return Err(crate::error::StrategyError::InvalidWeightSum.into());
        }
        Ok(())
    }
}

/// Comprehensive validation utilities
pub struct Validator;

impl Validator {
    /// Validate token weights sum to 100%
    pub fn validate_weights(weights: &[u64]) -> Result<()> {
        if weights.is_empty() {
            return Err(StrategyError::InvalidTokenCount.into());
        }

        if weights.len() > MAX_TOKENS {
            return Err(StrategyError::InvalidTokenCount.into());
        }

        let total: u64 = weights.iter().sum();
        if total != BASIS_POINTS_MAX {
            return Err(StrategyError::InvalidWeightSum.into());
        }

        // Validate individual weight constraints
        for &weight in weights {
            if weight < MIN_TOKEN_WEIGHT_BPS || weight > MAX_TOKEN_WEIGHT_BPS {
                return Err(StrategyError::InvalidStrategyParameters.into());
            }
        }

        Ok(())
    }

    /// Validate token count is within acceptable limits
    pub fn validate_token_count(count: usize) -> Result<()> {
        if count < MIN_TOKENS || count > MAX_TOKENS {
            return Err(StrategyError::InvalidTokenCount.into());
        }
        Ok(())
    }

    /// Validate slippage tolerance
    pub fn validate_slippage(slippage_bps: u64) -> Result<()> {
        if slippage_bps > MAX_SLIPPAGE_BPS {
            return Err(StrategyError::SlippageExceeded.into());
        }
        Ok(())
    }

    /// Validate price impact
    pub fn validate_price_impact(impact_bps: u64) -> Result<()> {
        if impact_bps > MAX_PRICE_IMPACT_BPS {
            return Err(StrategyError::SlippageExceeded.into());
        }
        Ok(())
    }

    /// Validate rebalancing threshold
    pub fn validate_rebalancing_threshold(threshold_bps: u64) -> Result<()> {
        if threshold_bps > MAX_REBALANCE_THRESHOLD_BPS {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(())
    }

    /// Validate time interval
    pub fn validate_time_interval(interval_seconds: u64) -> Result<()> {
        if interval_seconds < MIN_REBALANCE_INTERVAL || interval_seconds > MAX_REBALANCE_INTERVAL {
            return Err(StrategyError::InvalidTimeWindow.into());
        }
        Ok(())
    }

    /// Validate fee parameters
    pub fn validate_fees(
        management_fee_bps: u16,
        performance_fee_bps: u16,
        creation_fee_bps: u16,
        redemption_fee_bps: u16,
    ) -> Result<()> {
        if management_fee_bps > MAX_MANAGEMENT_FEE_BPS {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        if performance_fee_bps > MAX_PERFORMANCE_FEE_BPS {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        if creation_fee_bps > MAX_CREATION_FEE_BPS {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        if redemption_fee_bps > MAX_REDEMPTION_FEE_BPS {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(())
    }

    /// Validate basket creation amount
    pub fn validate_basket_amount(amount: u64) -> Result<()> {
        if amount < MIN_BASKET_CREATION_AMOUNT {
            return Err(StrategyError::BasketAmountTooSmall.into());
        }
        if amount > MAX_BASKET_CREATION_AMOUNT {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(())
    }

    /// Validate execution parameters
    pub fn validate_execution_params(params: &ExecutionParams) -> Result<()> {
        Self::validate_slippage(params.max_slippage_bps)?;
        Self::validate_price_impact(params.max_price_impact_bps)?;

        if params.min_fill_percentage_bps > BASIS_POINTS_MAX {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        if params.token_mints.len() != params.token_weights.len() {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        Self::validate_token_count(params.token_mints.len())?;
        Self::validate_weights(&params.token_weights)?;

        Ok(())
    }

    /// Validate price feed data
    pub fn validate_price_feed(price_feed: &PriceFeed, current_timestamp: i64) -> Result<()> {
        if !price_feed.is_valid {
            return Err(StrategyError::PriceFeedUnavailable.into());
        }

        if price_feed.price == 0 {
            return Err(StrategyError::InvalidMarketData.into());
        }

        // Check if price feed is stale
        if current_timestamp - price_feed.last_updated > PRICE_FEED_STALENESS_THRESHOLD {
            return Err(StrategyError::PriceFeedUnavailable.into());
        }

        Ok(())
    }

    /// Validate market data consistency
    pub fn validate_market_data(market_data: &MarketData) -> Result<()> {
        let token_count = market_data.token_supplies.len();

        if token_count != market_data.historical_prices.len()
            || token_count != market_data.volatilities.len()
            || token_count != market_data.technical_indicators.len()
        {
            return Err(StrategyError::InvalidMarketData.into());
        }

        Self::validate_token_count(token_count)?;

        // Validate that supplies and prices are non-zero
        for &supply in &market_data.token_supplies {
            if supply == 0 {
                return Err(StrategyError::InvalidMarketData.into());
            }
        }

        for &price in &market_data.historical_prices {
            if price == 0 {
                return Err(StrategyError::InvalidMarketData.into());
            }
        }

        Ok(())
    }

    /// Validate risk metrics
    pub fn validate_risk_metrics(risk_metrics: &RiskMetrics) -> Result<()> {
        if risk_metrics.var_bps > MAX_VAR_THRESHOLD_BPS {
            return Err(StrategyError::RiskLimitsExceeded.into());
        }

        if risk_metrics.max_drawdown_bps > MAX_DRAWDOWN_THRESHOLD_BPS {
            return Err(StrategyError::RiskLimitsExceeded.into());
        }

        if risk_metrics.concentration_risk > BASIS_POINTS_MAX as u32 {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        Ok(())
    }

    /// Validate optimization configuration
    pub fn validate_optimization_config(config: &OptimizationConfig) -> Result<()> {
        if config.max_batch_size > MAX_BATCH_SIZE {
            return Err(StrategyError::BatchSizeExceeded.into());
        }

        if config.optimization_timeout_seconds > MAX_EXECUTION_TIMEOUT {
            return Err(StrategyError::InvalidTimeWindow.into());
        }

        Ok(())
    }

    /// Validate account authority
    pub fn validate_authority(
        expected_authority: &Pubkey,
        provided_authority: &Pubkey,
    ) -> Result<()> {
        if expected_authority != provided_authority {
            return Err(StrategyError::Unauthorized.into());
        }
        Ok(())
    }

    /// Validate account is not paused
    pub fn validate_not_paused(is_paused: bool) -> Result<()> {
        if is_paused {
            return Err(StrategyError::StrategyPaused.into());
        }
        Ok(())
    }

    /// Validate timestamp is not in the future
    pub fn validate_timestamp(timestamp: i64, current_timestamp: i64) -> Result<()> {
        if timestamp > current_timestamp {
            return Err(StrategyError::InvalidTimeWindow.into());
        }
        Ok(())
    }

    /// Validate deadline has not passed
    pub fn validate_deadline(deadline: i64, current_timestamp: i64) -> Result<()> {
        if deadline != 0 && current_timestamp > deadline {
            return Err(StrategyError::InvalidTimeWindow.into());
        }
        Ok(())
    }
}

/// Business logic validation
pub struct BusinessValidator;

impl BusinessValidator {
    /// Validate rebalancing is needed
    pub fn validate_rebalancing_needed(
        current_weights: &[u64],
        target_weights: &[u64],
        threshold_bps: u64,
        last_rebalanced: i64,
        min_interval: u64,
        current_timestamp: i64,
    ) -> Result<()> {
        if current_weights.len() != target_weights.len() {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        // Check time interval
        let time_since_last = (current_timestamp - last_rebalanced) as u64;
        if time_since_last < min_interval {
            return Err(StrategyError::RebalancingThresholdNotMet.into());
        }

        // Check weight deviation
        let mut max_deviation = 0u64;
        for (current, target) in current_weights.iter().zip(target_weights.iter()) {
            let deviation = if current > target {
                current - target
            } else {
                target - current
            };
            max_deviation = max_deviation.max(deviation);
        }

        if max_deviation < threshold_bps {
            return Err(StrategyError::RebalancingThresholdNotMet.into());
        }

        Ok(())
    }

    /// Validate arbitrage opportunity is profitable
    pub fn validate_arbitrage_opportunity(
        price_difference: u64,
        transaction_costs: u64,
        min_profit_bps: u64,
    ) -> Result<()> {
        if price_difference <= transaction_costs {
            return Err(StrategyError::ArbitrageNotProfitable.into());
        }

        let profit = price_difference - transaction_costs;
        let profit_percentage = (profit * BASIS_POINTS_MAX) / price_difference;

        if profit_percentage < min_profit_bps {
            return Err(StrategyError::ArbitrageNotProfitable.into());
        }

        Ok(())
    }

    /// Validate liquidity is sufficient for trade
    pub fn validate_liquidity_sufficient(
        trade_amount: u64,
        available_liquidity: u64,
        min_liquidity_ratio_bps: u64,
    ) -> Result<()> {
        if available_liquidity == 0 {
            return Err(StrategyError::InsufficientLiquidity.into());
        }

        let liquidity_ratio = (trade_amount * BASIS_POINTS_MAX) / available_liquidity;
        if liquidity_ratio > min_liquidity_ratio_bps {
            return Err(StrategyError::InsufficientLiquidity.into());
        }

        Ok(())
    }

    /// Validate portfolio concentration limits
    pub fn validate_concentration_limits(
        weights: &[u64],
        max_concentration_bps: u64,
    ) -> Result<()> {
        for &weight in weights {
            if weight > max_concentration_bps {
                return Err(StrategyError::RiskLimitsExceeded.into());
            }
        }
        Ok(())
    }

    /// Validate circuit breaker conditions
    pub fn validate_circuit_breaker(
        price_change_bps: u64,
        volume_change_bps: u64,
        circuit_breaker_threshold_bps: u64,
    ) -> Result<()> {
        if price_change_bps > circuit_breaker_threshold_bps
            || volume_change_bps > circuit_breaker_threshold_bps
        {
            return Err(StrategyError::CircuitBreakerActivated.into());
        }
        Ok(())
    }
}

/// Data integrity validation
pub struct DataValidator;

impl DataValidator {
    /// Validate array lengths match
    pub fn validate_array_lengths_match<T, U>(arr1: &[T], arr2: &[U]) -> Result<()> {
        if arr1.len() != arr2.len() {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(())
    }

    /// Validate array is not empty
    pub fn validate_not_empty<T>(arr: &[T]) -> Result<()> {
        if arr.is_empty() {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(())
    }

    /// Validate all values in array are positive
    pub fn validate_all_positive(values: &[u64]) -> Result<()> {
        for &value in values {
            if value == 0 {
                return Err(StrategyError::InvalidStrategyParameters.into());
            }
        }
        Ok(())
    }

    /// Validate values are within range
    pub fn validate_range(value: u64, min: u64, max: u64) -> Result<()> {
        if value < min || value > max {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(())
    }

    /// Validate percentage values
    pub fn validate_percentage(percentage_bps: u64) -> Result<()> {
        if percentage_bps > BASIS_POINTS_MAX {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(())
    }

    /// Validate pubkey is not default
    pub fn validate_pubkey_not_default(pubkey: &Pubkey) -> Result<()> {
        if *pubkey == Pubkey::default() {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(())
    }

    /// Validate account discriminator
    pub fn validate_account_discriminator(
        account_data: &[u8],
        expected_discriminator: &[u8; 8],
    ) -> Result<()> {
        if account_data.len() < 8 {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        let discriminator = &account_data[0..8];
        if discriminator != expected_discriminator {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        Ok(())
    }
}

/// Performance validation
pub struct PerformanceValidator;

impl PerformanceValidator {
    /// Validate compute budget is sufficient
    pub fn validate_compute_budget(required_units: u32, available_units: u32) -> Result<()> {
        if required_units > available_units {
            return Err(StrategyError::ComputeBudgetExceeded.into());
        }
        Ok(())
    }

    /// Validate memory usage is within limits
    pub fn validate_memory_usage(used_memory: usize, max_memory: usize) -> Result<()> {
        if used_memory > max_memory {
            return Err(StrategyError::MemoryOptimizationFailed.into());
        }
        Ok(())
    }

    /// Validate cache performance
    pub fn validate_cache_performance(hit_rate_bps: u32, min_hit_rate_bps: u32) -> Result<()> {
        if hit_rate_bps < min_hit_rate_bps {
            return Err(StrategyError::CachePerformanceDegraded.into());
        }
        Ok(())
    }

    /// Validate execution time is within limits
    pub fn validate_execution_time(execution_time_ms: u64, max_time_ms: u64) -> Result<()> {
        if execution_time_ms > max_time_ms {
            return Err(StrategyError::InvalidTimeWindow.into());
        }
        Ok(())
    }
}

// 单元测试
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weight_validation() {
        // Valid weights that sum to 100%
        let valid_weights = vec![3000, 3000, 4000]; // 30%, 30%, 40%
        assert!(Validator::validate_weights(&valid_weights).is_ok());

        // Invalid weights that don't sum to 100%
        let invalid_weights = vec![3000, 3000, 3000]; // 90% total
        assert!(Validator::validate_weights(&invalid_weights).is_err());

        // Empty weights
        let empty_weights = vec![];
        assert!(Validator::validate_weights(&empty_weights).is_err());
    }

    #[test]
    fn test_slippage_validation() {
        assert!(Validator::validate_slippage(100).is_ok()); // 1%
        assert!(Validator::validate_slippage(MAX_SLIPPAGE_BPS).is_ok()); // Max allowed
        assert!(Validator::validate_slippage(MAX_SLIPPAGE_BPS + 1).is_err()); // Over limit
    }

    #[test]
    fn test_token_count_validation() {
        assert!(Validator::validate_token_count(5).is_ok());
        assert!(Validator::validate_token_count(MAX_TOKENS).is_ok());
        assert!(Validator::validate_token_count(0).is_err());
        assert!(Validator::validate_token_count(MAX_TOKENS + 1).is_err());
    }

    #[test]
    fn test_business_validation() {
        // Test rebalancing needed validation
        let current_weights = vec![3000, 3000, 4000];
        let target_weights = vec![3500, 2500, 4000]; // 5% deviation
        let threshold = 400; // 4% threshold
        let last_rebalanced = 0;
        let min_interval = 3600; // 1 hour
        let current_time = 7200; // 2 hours later

        assert!(BusinessValidator::validate_rebalancing_needed(
            &current_weights,
            &target_weights,
            threshold,
            last_rebalanced,
            min_interval,
            current_time
        )
        .is_ok());
    }

    #[test]
    fn test_data_validation() {
        let values = vec![100, 200, 300];
        assert!(DataValidator::validate_not_empty(&values).is_ok());
        assert!(DataValidator::validate_all_positive(&values).is_ok());

        let empty_values: Vec<u64> = vec![];
        assert!(DataValidator::validate_not_empty(&empty_values).is_err());

        let zero_values = vec![100, 0, 300];
        assert!(DataValidator::validate_all_positive(&zero_values).is_err());
    }

    #[test]
    fn test_not_empty_validator() {
        let v = NotEmptyValidator;
        assert!(v.validate(&vec![1, 2, 3]).is_ok());
        assert!(v.validate(&Vec::<u8>::new()).is_err());
    }
    #[test]
    fn test_range_validator() {
        let v = RangeValidator { min: 10, max: 20 };
        assert!(v.validate(&15).is_ok());
        assert!(v.validate(&5).is_err());
        assert!(v.validate(&25).is_err());
    }
    #[test]
    fn test_weights_sum_validator() {
        let v = WeightsSumValidator;
        assert!(v.validate(&vec![5000, 5000]).is_ok());
        assert!(v.validate(&vec![4000, 4000]).is_err());
    }
    #[test]
    fn test_registry() {
        let mut reg = ValidatorRegistry::<u64>::new();
        reg.register(Box::new(RangeValidator { min: 1, max: 10 }));
        assert!(reg.validate_all(&5).is_ok());
        assert!(reg.validate_all(&0).is_err());
    }
}
