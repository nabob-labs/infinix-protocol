/*!
 * 校验模块
 *
 * 本模块提供全面的校验函数，涵盖：
 * - 输入参数校验
 * - 账户状态校验
 * - 业务逻辑约束
 * - 安全检查
 * - 数据完整性验证
 */

use crate::core::constants::*;
use crate::core::types::*;
use crate::error::StrategyError;
use anchor_lang::prelude::*;

/// 通用校验 trait
/// - 支持参数/状态/业务等多场景可插拔校验器
pub trait Validator<T>: Send + Sync {
    /// 校验 value 是否满足约束，失败返回 Err
    fn validate(&self, value: &T) -> Result<()>;
}

/// 校验器注册表
/// - 支持批量注册和统一校验
pub struct ValidatorRegistry<T> {
    validators: Vec<Box<dyn Validator<T>>>,
}

impl<T> ValidatorRegistry<T> {
    /// 创建新注册表
    pub fn new() -> Self {
        Self {
            validators: Vec::new(),
        }
    }
    /// 注册校验器
    pub fn register(&mut self, validator: Box<dyn Validator<T>>) {
        self.validators.push(validator);
    }
    /// 依次校验所有注册校验器
    pub fn validate_all(&self, value: &T) -> Result<()> {
        for v in &self.validators {
            v.validate(value)?;
        }
        Ok(())
    }
}

/// 非空校验器
pub struct NotEmptyValidator;
impl<T: AsRef<[U]>, U> Validator<T> for NotEmptyValidator {
    fn validate(&self, value: &T) -> Result<()> {
        if value.as_ref().is_empty() {
            return Err(crate::error::StrategyError::InvalidStrategyParameters.into());
        }
        Ok(())
    }
}

/// 数值范围校验器
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

/// 权重和校验器（业务约束）
pub struct WeightsSumValidator;
impl Validator<Vec<u64>> for WeightsSumValidator {
    fn validate(&self, value: &Vec<u64>) -> Result<()> {
        if value.iter().sum::<u64>() != 10_000 {
            return Err(crate::error::StrategyError::InvalidWeightSum.into());
        }
        Ok(())
    }
}

/// 综合校验工具
pub struct Validator;

impl Validator {
    /// 校验权重和为 100%
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
        // 校验单个权重边界
        for &weight in weights {
            if weight < MIN_TOKEN_WEIGHT_BPS || weight > MAX_TOKEN_WEIGHT_BPS {
                return Err(StrategyError::InvalidStrategyParameters.into());
            }
        }
        Ok(())
    }
    /// 校验代币数量在合法范围
    pub fn validate_token_count(count: usize) -> Result<()> {
        if count < MIN_TOKENS || count > MAX_TOKENS {
            return Err(StrategyError::InvalidTokenCount.into());
        }
        Ok(())
    }
    /// 校验滑点容忍度
    pub fn validate_slippage(slippage_bps: u64) -> Result<()> {
        if slippage_bps > MAX_SLIPPAGE_BPS {
            return Err(StrategyError::SlippageExceeded.into());
        }
        Ok(())
    }
    /// 校验价格冲击
    pub fn validate_price_impact(impact_bps: u64) -> Result<()> {
        if impact_bps > MAX_PRICE_IMPACT_BPS {
            return Err(StrategyError::SlippageExceeded.into());
        }
        Ok(())
    }
    /// 校验再平衡阈值
    pub fn validate_rebalancing_threshold(threshold_bps: u64) -> Result<()> {
        if threshold_bps > MAX_REBALANCE_THRESHOLD_BPS {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(())
    }
    /// 校验时间区间
    pub fn validate_time_interval(interval_seconds: u64) -> Result<()> {
        if interval_seconds < MIN_REBALANCE_INTERVAL || interval_seconds > MAX_REBALANCE_INTERVAL {
            return Err(StrategyError::InvalidTimeWindow.into());
        }
        Ok(())
    }
    /// 校验各类费用参数
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
    /// 校验篮子创建数量
    pub fn validate_basket_amount(amount: u64) -> Result<()> {
        if amount < MIN_BASKET_CREATION_AMOUNT {
            return Err(StrategyError::BasketAmountTooSmall.into());
        }
        if amount > MAX_BASKET_CREATION_AMOUNT {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(())
    }
    /// 校验执行参数
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
    /// 校验价格喂价数据
    pub fn validate_price_feed(price_feed: &PriceFeed, current_timestamp: i64) -> Result<()> {
        if !price_feed.is_valid {
            return Err(StrategyError::PriceFeedUnavailable.into());
        }
        if price_feed.price == 0 {
            return Err(StrategyError::InvalidMarketData.into());
        }
        // 校验喂价是否过期
        if current_timestamp - price_feed.last_updated > PRICE_FEED_STALENESS_THRESHOLD {
            return Err(StrategyError::PriceFeedUnavailable.into());
        }
        Ok(())
    }
    /// 校验市场数据一致性
    pub fn validate_market_data(market_data: &MarketData) -> Result<()> {
        let token_count = market_data.token_supplies.len();
        if token_count != market_data.historical_prices.len()
            || token_count != market_data.volatilities.len()
            || token_count != market_data.technical_indicators.len()
        {
            return Err(StrategyError::InvalidMarketData.into());
        }
        Self::validate_token_count(token_count)?;
        // 校验供应和价格均非零
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
    /// 校验风险指标
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
    /// 校验优化配置
    pub fn validate_optimization_config(config: &OptimizationConfig) -> Result<()> {
        if config.max_batch_size > MAX_BATCH_SIZE {
            return Err(StrategyError::BatchSizeExceeded.into());
        }
        if config.optimization_timeout_seconds > MAX_EXECUTION_TIMEOUT {
            return Err(StrategyError::InvalidTimeWindow.into());
        }
        Ok(())
    }
    /// 校验账户权限
    pub fn validate_authority(
        expected_authority: &Pubkey,
        provided_authority: &Pubkey,
    ) -> Result<()> {
        if expected_authority != provided_authority {
            return Err(StrategyError::Unauthorized.into());
        }
        Ok(())
    }
    /// 校验账户未暂停
    pub fn validate_not_paused(is_paused: bool) -> Result<()> {
        if is_paused {
            return Err(StrategyError::StrategyPaused.into());
        }
        Ok(())
    }
    /// 校验时间戳不在未来
    pub fn validate_timestamp(timestamp: i64, current_timestamp: i64) -> Result<()> {
        if timestamp > current_timestamp {
            return Err(StrategyError::InvalidTimeWindow.into());
        }
        Ok(())
    }
    /// 校验截止时间未过
    pub fn validate_deadline(deadline: i64, current_timestamp: i64) -> Result<()> {
        if deadline != 0 && current_timestamp > deadline {
            return Err(StrategyError::InvalidTimeWindow.into());
        }
        Ok(())
    }
}

/// 通用数值区间校验
pub fn validate_amount(amount: u64, min: u64, max: u64) -> Result<()> {
    require!(amount >= min && amount <= max, crate::error::ErrorCode::InvalidAlgorithmParameters);
    Ok(())
}

/// 校验 pubkey 非默认
pub fn validate_pubkey(key: &Pubkey) -> Result<()> {
    require!(*key != Pubkey::default(), crate::error::ErrorCode::InvalidAlgorithmParameters);
    Ok(())
}

/// 校验权重和
pub fn validate_weights(weights: &[u64], expected_sum: u64) -> Result<()> {
    let sum: u64 = weights.iter().sum();
    require!(sum == expected_sum, crate::error::ErrorCode::InvalidWeightSum);
    Ok(())
}

/// 业务逻辑校验器
pub struct BusinessValidator;

impl BusinessValidator {
    /// 校验是否需要再平衡
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
        // 校验最小再平衡间隔
        let time_since_last = (current_timestamp - last_rebalanced) as u64;
        if time_since_last < min_interval {
            return Err(StrategyError::RebalancingThresholdNotMet.into());
        }
        // 校验权重偏差
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
    /// 校验套利机会是否有利可图
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
    /// 校验流动性是否充足
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
    /// 校验投资组合集中度限制
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
    /// 校验熔断器条件
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

/// 数据完整性校验器
pub struct DataValidator;

impl DataValidator {
    /// 校验数组长度一致
    pub fn validate_array_lengths_match<T, U>(arr1: &[T], arr2: &[U]) -> Result<()> {
        if arr1.len() != arr2.len() {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(())
    }
    /// 校验数组非空
    pub fn validate_not_empty<T>(arr: &[T]) -> Result<()> {
        if arr.is_empty() {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(())
    }
    /// 校验所有值为正
    pub fn validate_all_positive(values: &[u64]) -> Result<()> {
        for &value in values {
            if value == 0 {
                return Err(StrategyError::InvalidStrategyParameters.into());
            }
        }
        Ok(())
    }
    /// 校验值在区间内
    pub fn validate_range(value: u64, min: u64, max: u64) -> Result<()> {
        if value < min || value > max {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(())
    }
    /// 校验百分比值
    pub fn validate_percentage(percentage_bps: u64) -> Result<()> {
        if percentage_bps > BASIS_POINTS_MAX {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(())
    }
    /// 校验 pubkey 非默认
    pub fn validate_pubkey_not_default(pubkey: &Pubkey) -> Result<()> {
        if *pubkey == Pubkey::default() {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(())
    }
    /// 校验账户 discriminator
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

/// 性能校验器
pub struct PerformanceValidator;

impl PerformanceValidator {
    /// 校验计算预算充足
    pub fn validate_compute_budget(required_units: u32, available_units: u32) -> Result<()> {
        if required_units > available_units {
            return Err(StrategyError::ComputeBudgetExceeded.into());
        }
        Ok(())
    }
    /// 校验内存使用量在限额内
    pub fn validate_memory_usage(used_memory: usize, max_memory: usize) -> Result<()> {
        if used_memory > max_memory {
            return Err(StrategyError::MemoryOptimizationFailed.into());
        }
        Ok(())
    }
    /// 校验缓存性能
    pub fn validate_cache_performance(hit_rate_bps: u32, min_hit_rate_bps: u32) -> Result<()> {
        if hit_rate_bps < min_hit_rate_bps {
            return Err(StrategyError::CachePerformanceDegraded.into());
        }
        Ok(())
    }
    /// 校验执行时间在限额内
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
