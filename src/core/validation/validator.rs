//!
//! validator.rs - 综合校验工具实现
//!
//! 本文件实现Validator结构体及其所有静态方法，严格遵循Rust、Anchor、SOLID最佳实践，
//! 并逐行专业注释，便于审计、维护、扩展。

use anchor_lang::prelude::*;
use crate::core::constants::*;
use crate::core::types::*;
use crate::errors::strategy_error::StrategyError;

/// 综合校验工具结构体
pub struct Validator;

impl Validator {
    /// 校验权重和为 100%
    pub fn validate_weights(weights: &[u64]) -> anchor_lang::Result<()> {
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
        Ok(())
    }

    /// 校验token数量
    pub fn validate_token_count(count: usize) -> anchor_lang::Result<()> {
        if count == 0 || count > MAX_TOKENS {
            return Err(StrategyError::InvalidTokenCount.into());
        }
        Ok(())
    }

    /// 校验滑点范围
    pub fn validate_slippage(slippage_bps: u64) -> anchor_lang::Result<()> {
        if slippage_bps > MAX_SLIPPAGE_BPS {
            return Err(StrategyError::InvalidSlippage.into());
        }
        Ok(())
    }

    /// 校验价格冲击范围
    pub fn validate_price_impact(impact_bps: u64) -> anchor_lang::Result<()> {
        if impact_bps > MAX_PRICE_IMPACT_BPS {
            return Err(StrategyError::InvalidPriceImpact.into());
        }
        Ok(())
    }

    /// 校验再平衡阈值
    pub fn validate_rebalancing_threshold(threshold_bps: u64) -> anchor_lang::Result<()> {
        if threshold_bps > MAX_REBALANCE_THRESHOLD_BPS {
            return Err(StrategyError::InvalidRebalanceThreshold.into());
        }
        Ok(())
    }

    /// 校验时间间隔
    pub fn validate_time_interval(interval_seconds: u64) -> anchor_lang::Result<()> {
        if interval_seconds < MIN_TIME_INTERVAL_SECONDS {
            return Err(StrategyError::InvalidTimeInterval.into());
        }
        Ok(())
    }

    /// 校验费用参数
    pub fn validate_fees(
        management_fee_bps: u16,
        performance_fee_bps: u16,
        creation_fee_bps: u16,
        redemption_fee_bps: u16,
    ) -> anchor_lang::Result<()> {
        if management_fee_bps > MAX_MANAGEMENT_FEE_BPS
            || performance_fee_bps > MAX_PERFORMANCE_FEE_BPS
            || creation_fee_bps > MAX_CREATION_FEE_BPS
            || redemption_fee_bps > MAX_REDEMPTION_FEE_BPS
        {
            return Err(StrategyError::InvalidFee.into());
        }
        Ok(())
    }

    /// 校验篮子金额
    pub fn validate_basket_amount(amount: u64) -> anchor_lang::Result<()> {
        if amount == 0 || amount > MAX_BASKET_AMOUNT {
            return Err(StrategyError::InvalidBasketAmount.into());
        }
        Ok(())
    }

    /// 校验执行参数
    pub fn validate_execution_params(params: &ExecutionParams) -> anchor_lang::Result<()> {
        // 生产环境应详细校验各参数
        if params.amount == 0 {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(())
    }

    /// 校验价格源
    pub fn validate_price_feed(price_feed: &PriceFeed, current_timestamp: i64) -> anchor_lang::Result<()> {
        if price_feed.price == 0 {
            return Err(StrategyError::InvalidMarketData.into());
        }
        if current_timestamp - price_feed.timestamp > MAX_PRICE_FEED_AGE_SECONDS as i64 {
            return Err(StrategyError::StaleMarketData.into());
        }
        Ok(())
    }

    /// 校验市场数据
    pub fn validate_market_data(market_data: &MarketData) -> anchor_lang::Result<()> {
        if market_data.price == 0 || market_data.liquidity == 0 {
            return Err(StrategyError::InvalidMarketData.into());
        }
        Ok(())
    }

    /// 校验风险指标
    pub fn validate_risk_metrics(risk_metrics: &RiskMetrics) -> anchor_lang::Result<()> {
        if risk_metrics.var_95 > 10000 || risk_metrics.var_99 > 10000 {
            return Err(StrategyError::InvalidRiskMetrics.into());
        }
        Ok(())
    }

    /// 校验优化配置
    pub fn validate_optimization_config(config: &OptimizationConfig) -> anchor_lang::Result<()> {
        if config.max_iterations == 0 {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(())
    }

    /// 校验权限
    pub fn validate_authority(
        expected_authority: &Pubkey,
        provided_authority: &Pubkey,
    ) -> anchor_lang::Result<()> {
        if expected_authority != provided_authority {
            return Err(StrategyError::NotAllowed.into());
        }
        Ok(())
    }

    /// 校验未暂停
    pub fn validate_not_paused(is_paused: bool) -> anchor_lang::Result<()> {
        if is_paused {
            return Err(StrategyError::Paused.into());
        }
        Ok(())
    }

    /// 校验时间戳
    pub fn validate_timestamp(timestamp: i64, current_timestamp: i64) -> anchor_lang::Result<()> {
        if timestamp > current_timestamp {
            return Err(StrategyError::InvalidTimestamp.into());
        }
        Ok(())
    }

    /// 校验截止时间
    pub fn validate_deadline(deadline: i64, current_timestamp: i64) -> anchor_lang::Result<()> {
        if current_timestamp > deadline {
            return Err(StrategyError::DeadlinePassed.into());
        }
        Ok(())
    }
} 