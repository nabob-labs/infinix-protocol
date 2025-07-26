//!
//! business.rs - 业务校验器实现
//!
//! 本文件实现BusinessValidator结构体及其所有业务校验方法，严格遵循Rust、Anchor、SOLID最佳实践，
//! 并逐行专业注释，便于审计、维护、扩展。

use anchor_lang::prelude::*;
use crate::error::StrategyError;

/// 业务校验器结构体
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
        let mut need_rebalance = false;
        for (c, t) in current_weights.iter().zip(target_weights.iter()) {
            let diff = if c > t { c - t } else { t - c };
            if diff > threshold_bps {
                need_rebalance = true;
                break;
            }
        }
        if !need_rebalance {
            return Err(StrategyError::NoRebalanceNeeded.into());
        }
        if current_timestamp - last_rebalanced < min_interval as i64 {
            return Err(StrategyError::RebalanceIntervalTooShort.into());
        }
        Ok(())
    }

    /// 校验套利机会
    pub fn validate_arbitrage_opportunity(
        price_difference: u64,
        transaction_costs: u64,
        min_profit_bps: u64,
    ) -> Result<()> {
        if price_difference <= transaction_costs + min_profit_bps {
            return Err(StrategyError::NoArbitrageOpportunity.into());
        }
        Ok(())
    }

    /// 校验流动性是否充足
    pub fn validate_liquidity_sufficient(
        trade_amount: u64,
        available_liquidity: u64,
        min_liquidity_ratio_bps: u64,
    ) -> Result<()> {
        if available_liquidity == 0 || trade_amount * 10_000 / available_liquidity > min_liquidity_ratio_bps {
            return Err(StrategyError::InsufficientLiquidity.into());
        }
        Ok(())
    }

    /// 校验集中度限制
    pub fn validate_concentration_limits(
        weights: &[u64],
        max_concentration_bps: u64,
    ) -> Result<()> {
        for &w in weights {
            if w > max_concentration_bps {
                return Err(StrategyError::ConcentrationLimitExceeded.into());
            }
        }
        Ok(())
    }

    /// 校验熔断器
    pub fn validate_circuit_breaker(
        price_change_bps: u64,
        volume_change_bps: u64,
        circuit_breaker_threshold_bps: u64,
    ) -> Result<()> {
        if price_change_bps > circuit_breaker_threshold_bps || volume_change_bps > circuit_breaker_threshold_bps {
            return Err(StrategyError::CircuitBreakerTriggered.into());
        }
        Ok(())
    }
} 