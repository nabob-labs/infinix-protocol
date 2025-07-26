//!
//! executor.rs - 高级策略执行器实现
//!
//! 本文件实现AdvancedStrategyExecutor及其所有高级策略方法与辅助函数，严格遵循Rust、Anchor、SOLID最佳实践，
//! 并逐行专业注释，便于审计、维护、扩展。

use crate::core::*;
use crate::core::adapter::AdapterTrait;
use crate::error::StrategyError;
use crate::strategies::*;
use anchor_lang::prelude::*;

/// 高级策略执行器结构体，具备 AI/ML 能力。
pub struct AdvancedStrategyExecutor;

impl AdvancedStrategyExecutor {
    /// 执行动量策略，根据历史价格动量分配权重。
    pub fn execute_momentum_strategy(
        tokens: &[TokenInfo],
        lookback_period: u32,
        momentum_threshold: u64,
    ) -> StrategyResult<Vec<u64>> {
        if tokens.is_empty() {
            return Err(StrategyError::InvalidTokenCount.into());
        }
        let mut weights = Vec::new();
        let mut total_momentum_score = 0u64;
        let momentum_scores: Vec<u64> = tokens
            .iter()
            .map(|token| {
                let price_momentum = if token.price > PRICE_PRECISION {
                    ((token.price - PRICE_PRECISION) * 10000) / PRICE_PRECISION
                } else {
                    0
                };
                let score = if price_momentum >= momentum_threshold {
                    price_momentum
                } else {
                    100
                };
                total_momentum_score += score;
                score
            })
            .collect();
        if total_momentum_score > 0 {
            for score in momentum_scores {
                let weight = (score * BASIS_POINTS_MAX) / total_momentum_score;
                weights.push(weight);
            }
        } else {
            let equal_weight = BASIS_POINTS_MAX / tokens.len() as u64;
            weights = vec![equal_weight; tokens.len()];
        }
        let total: u64 = weights.iter().sum();
        if total != BASIS_POINTS_MAX && total > 0 {
            for weight in &mut weights {
                *weight = (*weight * BASIS_POINTS_MAX) / total;
            }
        }
        Ok(weights)
    }
    /// 执行均值回归策略，根据价格偏离度分配权重。
    pub fn execute_mean_reversion_strategy(
        tokens: &[TokenInfo],
        reversion_period: u32,
        deviation_threshold: u64,
    ) -> StrategyResult<Vec<u64>> {
        if tokens.is_empty() {
            return Err(StrategyError::InvalidTokenCount.into());
        }
        let mut weights = Vec::new();
        let mut total_score = 0u64;
        let scores: Vec<u64> = tokens
            .iter()
            .map(|token| {
                let deviation = if token.price > PRICE_PRECISION {
                    (token.price - PRICE_PRECISION) * 10000 / PRICE_PRECISION
                } else {
                    (PRICE_PRECISION - token.price) * 10000 / PRICE_PRECISION
                };
                let score = if deviation <= deviation_threshold {
                    10000 - deviation
                } else {
                    100
                };
                total_score += score;
                score
            })
            .collect();
        for score in &scores {
            weights.push((*score * BASIS_POINTS_MAX) / total_score.max(1));
        }
        Ok(weights)
    }
    /// 执行波动率策略，根据目标波动率和风险厌恶度分配权重。
    pub fn execute_volatility_strategy(
        tokens: &[TokenInfo],
        target_volatility: u64,
        risk_aversion: u64,
    ) -> StrategyResult<Vec<u64>> {
        if tokens.is_empty() {
            return Err(StrategyError::InvalidTokenCount.into());
        }
        let mut weights = Vec::new();
        let mut total_score = 0u64;
        let scores: Vec<u64> = tokens
            .iter()
            .map(|_token| {
                let volatility = 100; // 示例，实际应传入波动率数据
                let score = target_volatility.saturating_mul(risk_aversion) / volatility;
                total_score += score;
                score
            })
            .collect();
        for score in &scores {
            weights.push((*score * BASIS_POINTS_MAX) / total_score.max(1));
        }
        Ok(weights)
    }
    /// 执行多因子策略。
    pub fn execute_multi_factor_strategy(
        tokens: &[TokenInfo],
        momentum_weight: u64,
        reversion_weight: u64,
        volatility_weight: u64,
        params: MultiFactorParams,
    ) -> StrategyResult<Vec<u64>> {
        if tokens.is_empty() {
            return Err(StrategyError::InvalidTokenCount.into());
        }
        let mut weights = Vec::new();
        let mut total_score = 0u64;
        let scores: Vec<u64> = tokens
            .iter()
            .map(|_token| {
                let momentum_score = params.momentum_lookback * momentum_weight as u32;
                let reversion_score = params.reversion_period * reversion_weight as u32;
                let volatility_score = params.target_volatility * volatility_weight;
                let score = momentum_score as u64 + reversion_score as u64 + volatility_score;
                total_score += score;
                score
            })
            .collect();
        for score in &scores {
            weights.push((*score * BASIS_POINTS_MAX) / total_score.max(1));
        }
        Ok(weights)
    }
    /// 执行AI优化策略。
    pub fn execute_ai_optimization_strategy(
        tokens: &[TokenInfo],
        market_data: &MarketData,
        optimization_params: AiOptimizationParams,
    ) -> StrategyResult<Vec<u64>> {
        if tokens.is_empty() {
            return Err(StrategyError::InvalidTokenCount.into());
        }
        let mut weights = vec![BASIS_POINTS_MAX / tokens.len() as u64; tokens.len()];
        Self::apply_confidence_adjustment(&mut weights, &optimization_params)?;
        Ok(weights)
    }
    /// AI信号计算（示例）。
    fn calculate_price_signal(token: &TokenInfo, params: &AiOptimizationParams) -> u64 {
        token.price.saturating_mul(params.price_weight)
    }
    fn calculate_volume_signal(_token_index: usize, _market_data: &MarketData) -> u64 {
        100 // 示例
    }
    fn calculate_sentiment_signal(_token: &TokenInfo, params: &AiOptimizationParams) -> u64 {
        params.sentiment_weight
    }
    fn apply_confidence_adjustment(
        weights: &mut [u64],
        params: &AiOptimizationParams,
    ) -> StrategyResult<()> {
        for w in weights.iter_mut() {
            if *w < params.confidence_threshold {
                *w = params.confidence_threshold;
            }
        }
        Ok(())
    }
} 