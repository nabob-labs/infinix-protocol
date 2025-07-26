//!
//! executor.rs - 权重策略执行器实现
//!
//! 本文件实现WeightStrategyExecutor及其所有权重分配算法与辅助函数，严格遵循Rust、Anchor、SOLID最佳实践，
//! 并逐行专业注释，便于审计、维护、扩展。

use crate::core::*;
use crate::core::adapter::AdapterTrait;
use crate::error::StrategyError;
use crate::strategies::*;
use anchor_lang::prelude::*;

/// 权重策略执行器结构体，支持多种权重分配算法。
pub struct WeightStrategyExecutor;

impl WeightStrategyExecutor {
    /// 执行等权重策略。
    pub fn execute_equal_weight(token_count: usize) -> StrategyResult<Vec<u64>> {
        if token_count == 0 || token_count > MAX_TOKENS {
            return Err(StrategyError::InvalidTokenCount.into());
        }
        let equal_weight = BASIS_POINTS_MAX / token_count as u64;
        let mut weights = vec![equal_weight; token_count];
        let total: u64 = weights.iter().sum();
        if total != BASIS_POINTS_MAX {
            weights[0] += BASIS_POINTS_MAX - total;
        }
        Ok(weights)
    }
    /// 执行市值加权策略。
    pub fn execute_market_cap_weighted(
        token_info: &[TokenInfo],
        params: MarketCapWeightedParams,
    ) -> StrategyResult<Vec<u64>> {
        if token_info.is_empty() || token_info.len() > MAX_TOKENS {
            return Err(StrategyError::InvalidTokenCount.into());
        }
        let market_caps: Vec<u64> = token_info
            .iter()
            .map(|token| {
                let estimated_supply = 1_000_000_000u64;
                token.price.saturating_mul(estimated_supply)
            })
            .collect();
        let total_market_cap: u64 = market_caps.iter().sum();
        if total_market_cap == 0 {
            return Err(StrategyError::InvalidMarketData.into());
        }
        let mut weights: Vec<u64> = market_caps
            .iter()
            .map(|&cap| (cap * BASIS_POINTS_MAX) / total_market_cap)
            .collect();
        for weight in &mut weights {
            *weight = (*weight).max(params.min_weight).min(params.max_weight);
        }
        let total_constrained: u64 = weights.iter().sum();
        if total_constrained > 0 {
            for weight in &mut weights {
                *weight = (*weight * BASIS_POINTS_MAX) / total_constrained;
            }
        }
        Ok(weights)
    }
    /// 执行动量加权策略。
    pub fn execute_momentum_weighted(
        token_info: &[TokenInfo],
        price_history: &[Vec<u64>],
        params: MomentumWeightedParams,
    ) -> StrategyResult<Vec<u64>> {
        if token_info.is_empty() || token_info.len() > MAX_TOKENS {
            return Err(StrategyError::InvalidTokenCount.into());
        }
        if price_history.len() != token_info.len() {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        let mut momentum_scores = Vec::new();
        let mut total_score = 0u64;
        for (i, _token) in token_info.iter().enumerate() {
            let momentum_score = Self::calculate_momentum_score(
                &price_history[i],
                params.lookback_period,
                params.momentum_factor,
            )?;
            momentum_scores.push(momentum_score);
            total_score += momentum_score;
        }
        let mut weights = Vec::new();
        for score in &momentum_scores {
            weights.push((*score * BASIS_POINTS_MAX) / total_score.max(1));
        }
        Ok(weights)
    }
    /// 执行波动率调整加权策略。
    pub fn execute_volatility_adjusted(
        token_info: &[TokenInfo],
        volatility_data: &[u64],
        params: VolatilityAdjustedParams,
    ) -> StrategyResult<Vec<u64>> {
        if token_info.is_empty() || token_info.len() > MAX_TOKENS {
            return Err(StrategyError::InvalidTokenCount.into());
        }
        if volatility_data.len() != token_info.len() {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        let mut risk_scores = Vec::new();
        let mut total_score = 0u64;
        for (i, _token) in token_info.iter().enumerate() {
            let risk_score = Self::calculate_risk_adjusted_score(
                volatility_data[i],
                params.target_volatility,
                params.risk_aversion,
            )?;
            risk_scores.push(risk_score);
            total_score += risk_score;
        }
        let mut weights = Vec::new();
        for score in &risk_scores {
            weights.push((*score * BASIS_POINTS_MAX) / total_score.max(1));
        }
        Ok(weights)
    }
    /// 执行固定权重策略。
    pub fn execute_fixed_weight(fixed_weights: &[u64]) -> StrategyResult<Vec<u64>> {
        if fixed_weights.is_empty() || fixed_weights.len() > MAX_TOKENS {
            return Err(StrategyError::InvalidTokenCount.into());
        }
        let total: u64 = fixed_weights.iter().sum();
        if total != BASIS_POINTS_MAX {
            return Err(StrategyError::InvalidWeightSum.into());
        }
        Ok(fixed_weights.to_vec())
    }
    /// 执行技术指标加权策略。
    pub fn execute_technical_indicator(
        token_info: &[TokenInfo],
        technical_data: &[TechnicalIndicators],
        params: TechnicalIndicatorParams,
    ) -> StrategyResult<Vec<u64>> {
        if token_info.is_empty() || token_info.len() > MAX_TOKENS {
            return Err(StrategyError::InvalidTokenCount.into());
        }
        if technical_data.len() != token_info.len() {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        let mut tech_scores = Vec::new();
        let mut total_score = 0u64;
        for (i, indicators) in technical_data.iter().enumerate() {
            let score = Self::calculate_technical_score(indicators, &params)?;
            tech_scores.push(score);
            total_score += score;
        }
        let mut weights = Vec::new();
        for score in &tech_scores {
            weights.push((*score * BASIS_POINTS_MAX) / total_score.max(1));
        }
        Ok(weights)
    }
    /// 计算动量分数。
    fn calculate_momentum_score(
        price_history: &[u64],
        lookback_period: u32,
        momentum_factor: u64,
    ) -> StrategyResult<u64> {
        if price_history.len() < lookback_period as usize + 1 {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        let start = price_history.len() - lookback_period as usize - 1;
        let end = price_history.len() - 1;
        let price_return = price_history[end].saturating_sub(price_history[start]);
        Ok(price_return.saturating_mul(momentum_factor))
    }
    /// 计算风险调整分数。
    fn calculate_risk_adjusted_score(
        volatility: u64,
        target_volatility: u64,
        risk_aversion: u64,
    ) -> StrategyResult<u64> {
        if volatility == 0 {
            return Ok(target_volatility);
        }
        Ok(target_volatility.saturating_mul(risk_aversion) / volatility)
    }
    /// 计算技术指标分数。
    fn calculate_technical_score(
        indicators: &TechnicalIndicators,
        params: &TechnicalIndicatorParams,
    ) -> StrategyResult<u64> {
        let mut score = 0u64;
        if let Some(rsi) = indicators.rsi {
            score += Self::calculate_rsi_score(rsi, params.rsi_oversold, params.rsi_overbought) * params.rsi_weight;
        }
        if let Some(macd_signal) = indicators.macd_signal {
            score += Self::calculate_macd_score(macd_signal) * params.macd_weight;
        }
        if let Some(bb_position) = indicators.bollinger_position {
            score += Self::calculate_bollinger_score(bb_position) * params.bollinger_weight;
        }
        if let Some(ma_convergence) = indicators.ma_convergence {
            score += Self::calculate_ma_score(ma_convergence) * params.ma_weight;
        }
        Ok(score)
    }
    /// 计算RSI分数。
    fn calculate_rsi_score(rsi: u64, oversold: u64, overbought: u64) -> u64 {
        if rsi < oversold {
            2
        } else if rsi > overbought {
            0
        } else {
            1
        }
    }
    /// 计算MACD分数。
    fn calculate_macd_score(macd_signal: i64) -> u64 {
        if macd_signal > 0 {
            2
        } else if macd_signal < 0 {
            0
        } else {
            1
        }
    }
    /// 计算布林带分数。
    fn calculate_bollinger_score(bb_position: u64) -> u64 {
        if bb_position < 30 {
            2
        } else if bb_position > 70 {
            0
        } else {
            1
        }
    }
    /// 计算均线分数。
    fn calculate_ma_score(ma_convergence: i64) -> u64 {
        if ma_convergence > 0 {
            2
        } else if ma_convergence < 0 {
            0
        } else {
            1
        }
    }
    /// 权重归一化。
    fn normalize_weights(weights: &mut [u64]) -> StrategyResult<()> {
        let total: u64 = weights.iter().sum();
        if total == 0 {
            return Err(StrategyError::InvalidWeightSum.into());
        }
        for w in weights.iter_mut() {
            *w = (*w * BASIS_POINTS_MAX) / total;
        }
        Ok(())
    }
} 