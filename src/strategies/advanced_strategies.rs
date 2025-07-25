//! Advanced Strategy Implementations
//!
//! 本模块包含高级交易策略的实现，集成了 AI 和 ML 组件，适用于复杂的资产配置与动态权重调整。

// 引入核心模块、适配器 trait、错误类型和策略模块。
use crate::core::*;
use crate::core::adapter::AdapterTrait;
use crate::error::StrategyError;
use crate::strategies::*;
use anchor_lang::prelude::*;

/// 高级策略执行器结构体，具备 AI/ML 能力。
pub struct AdvancedStrategyExecutor;

// 为高级策略执行器实现适配器 trait，便于统一注册和管理。
impl AdapterTrait for AdvancedStrategyExecutor {
    /// 返回适配器名称。
    fn name(&self) -> &'static str { "advanced_strategy_executor" }
    /// 返回适配器版本。
    fn version(&self) -> &'static str { "1.0.0" }
    /// 返回支持的资产列表。
    fn supported_assets(&self) -> Vec<String> { vec!["SOL".to_string(), "USDC".to_string()] }
    /// 返回适配器当前状态。
    fn status(&self) -> Option<String> { Some("active".to_string()) }
}

// 使用构造器自动注册高级策略执行器到全局工厂。
#[ctor::ctor]
fn auto_register_advanced_strategy_executor() {
    // 实例化高级策略执行器。
    let adapter = AdvancedStrategyExecutor;
    // 获取全局适配器工厂的可变引用。
    let mut factory = crate::core::registry::ADAPTER_FACTORY.lock().unwrap();
    // 注册适配器。
    factory.register(adapter);
}

impl AdvancedStrategyExecutor {
    /// 执行动量策略，根据历史价格动量分配权重。
    ///
    /// # 参数
    /// * `tokens` - 资产信息数组。
    /// * `lookback_period` - 回溯周期。
    /// * `momentum_threshold` - 动量阈值。
    /// # 返回
    /// * 权重向量或错误。
    pub fn execute_momentum_strategy(
        tokens: &[TokenInfo],
        lookback_period: u32,
        momentum_threshold: u64,
    ) -> StrategyResult<Vec<u64>> {
        // 边界检查：资产数组不能为空。
        if tokens.is_empty() {
            return Err(StrategyError::InvalidTokenCount.into());
        }
        // 初始化权重和总动量分数。
        let mut weights = Vec::new();
        let mut total_momentum_score = 0u64;
        // 计算每个 token 的动量分数。
        let momentum_scores: Vec<u64> = tokens
            .iter()
            .map(|token| {
                // 简化动量计算，实际应基于历史价格。
                let price_momentum = if token.price > PRICE_PRECISION {
                    ((token.price - PRICE_PRECISION) * 10000) / PRICE_PRECISION
                } else {
                    0
                };
                // 若动量高于阈值则采用，否则最低分。
                let score = if price_momentum >= momentum_threshold {
                    price_momentum
                } else {
                    100 // 最低分
                };
                // 累加总分。
                total_momentum_score += score;
                score
            })
            .collect();
        // 分数转权重。
        if total_momentum_score > 0 {
            for score in momentum_scores {
                let weight = (score * BASIS_POINTS_MAX) / total_momentum_score;
                weights.push(weight);
            }
        } else {
            // 若无动量信号则均分权重。
            let equal_weight = BASIS_POINTS_MAX / tokens.len() as u64;
            weights = vec![equal_weight; tokens.len()];
        }
        // 归一化权重，确保总和为 10000。
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
        let mut total_reversion_score = 0u64;
        let reversion_scores: Vec<u64> = tokens
            .iter()
            .map(|token| {
                // 简化均值回归计算。
                let price_deviation = if token.price > PRICE_PRECISION {
                    ((token.price - PRICE_PRECISION) * 10000) / PRICE_PRECISION
                } else {
                    ((PRICE_PRECISION - token.price) * 10000) / PRICE_PRECISION
                };
                // 偏离度高于阈值则高分，否则中性分。
                let score = if price_deviation >= deviation_threshold {
                    10000 - price_deviation.min(9900)
                } else {
                    5000
                };
                total_reversion_score += score;
                score
            })
            .collect();
        if total_reversion_score > 0 {
            for score in reversion_scores {
                let weight = (score * BASIS_POINTS_MAX) / total_reversion_score;
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
    /// 执行波动率策略，根据波动率和风险厌恶度分配权重。
    pub fn execute_volatility_strategy(
        tokens: &[TokenInfo],
        target_volatility: u64,
        risk_aversion: u64,
    ) -> StrategyResult<Vec<u64>> {
        if tokens.is_empty() {
            return Err(StrategyError::InvalidTokenCount.into());
        }
        let mut weights = Vec::new();
        let mut total_vol_score = 0u64;
        let vol_scores: Vec<u64> = tokens
            .iter()
            .map(|token| {
                // 简化波动率估算。
                let estimated_vol = if token.price > PRICE_PRECISION {
                    2000
                } else {
                    3000
                };
                // 超过目标波动率则惩罚，否则奖励。
                let vol_adjustment = if estimated_vol > target_volatility {
                    let penalty = ((estimated_vol - target_volatility) * risk_aversion) / 10000;
                    10000u64.saturating_sub(penalty)
                } else {
                    10000 + ((target_volatility - estimated_vol) / 2)
                };
                total_vol_score += vol_adjustment;
                vol_adjustment
            })
            .collect();
        if total_vol_score > 0 {
            for score in vol_scores {
                let weight = (score * BASIS_POINTS_MAX) / total_vol_score;
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
    /// 执行多因子策略，结合动量、均值回归和波动率信号。
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
        // 校验因子权重之和为 10000。
        let total_factor_weight = momentum_weight + reversion_weight + volatility_weight;
        if total_factor_weight != BASIS_POINTS_MAX {
            return Err(StrategyError::InvalidWeightSum.into());
        }
        // 计算各因子权重。
        let momentum_weights = Self::execute_momentum_strategy(
            tokens,
            params.momentum_lookback,
            params.momentum_threshold,
        )?;
        let reversion_weights = Self::execute_mean_reversion_strategy(
            tokens,
            params.reversion_period,
            params.deviation_threshold,
        )?;
        let volatility_weights = Self::execute_volatility_strategy(
            tokens,
            params.target_volatility,
            params.risk_aversion,
        )?;
        // 按因子权重加权合成。
        let mut combined_weights = Vec::new();
        for i in 0..tokens.len() {
            let momentum_contrib = (momentum_weights[i] * momentum_weight) / BASIS_POINTS_MAX;
            let reversion_contrib = (reversion_weights[i] * reversion_weight) / BASIS_POINTS_MAX;
            let volatility_contrib = (volatility_weights[i] * volatility_weight) / BASIS_POINTS_MAX;
            let combined_weight = momentum_contrib + reversion_contrib + volatility_contrib;
            combined_weights.push(combined_weight);
        }
        // 归一化。
        let total: u64 = combined_weights.iter().sum();
        if total != BASIS_POINTS_MAX && total > 0 {
            for weight in &mut combined_weights {
                *weight = (*weight * BASIS_POINTS_MAX) / total;
            }
        }
        Ok(combined_weights)
    }
    /// 执行 AI 优化策略，集成多种 AI 信号。
    pub fn execute_ai_optimization_strategy(
        tokens: &[TokenInfo],
        market_data: &MarketData,
        optimization_params: AiOptimizationParams,
    ) -> StrategyResult<Vec<u64>> {
        if tokens.is_empty() {
            return Err(StrategyError::InvalidTokenCount.into());
        }
        // 简化 AI 优化，实际应集成 ML 模型。
        let mut ai_scores = Vec::new();
        let mut total_score = 0u64;
        for (i, token) in tokens.iter().enumerate() {
            // 组合多种 AI 因子。
            let price_signal = Self::calculate_price_signal(token, &optimization_params);
            let volume_signal = Self::calculate_volume_signal(i, market_data);
            let sentiment_signal = Self::calculate_sentiment_signal(token, &optimization_params);
            // 按权重加权。
            let ai_score = (price_signal * optimization_params.price_weight
                + volume_signal * optimization_params.volume_weight
                + sentiment_signal * optimization_params.sentiment_weight)
                / BASIS_POINTS_MAX;
            ai_scores.push(ai_score);
            total_score += ai_score;
        }
        // 分数转权重。
        let mut weights = Vec::new();
        if total_score > 0 {
            for score in ai_scores {
                let weight = (score * BASIS_POINTS_MAX) / total_score;
                weights.push(weight);
            }
        } else {
            let equal_weight = BASIS_POINTS_MAX / tokens.len() as u64;
            weights = vec![equal_weight; tokens.len()];
        }
        // 应用 AI 置信度调整。
        if optimization_params.confidence_threshold > 0 {
            Self::apply_confidence_adjustment(&mut weights, &optimization_params)?;
        }
        Ok(weights)
    }
    /// 计算 AI 优化的价格信号。
    fn calculate_price_signal(token: &TokenInfo, params: &AiOptimizationParams) -> u64 {
        let price_momentum = if token.price > PRICE_PRECISION {
            ((token.price - PRICE_PRECISION) * 10000) / PRICE_PRECISION
        } else {
            0
        };
        let signal_strength = price_momentum.min(params.max_signal_strength);
        signal_strength
    }
    /// 计算 AI 优化的成交量信号。
    fn calculate_volume_signal(token_index: usize, market_data: &MarketData) -> u64 {
        let base_volume_signal = 5000u64;
        let volume_adjustment = (token_index as u64 * 500).min(2000);
        base_volume_signal + volume_adjustment
    }
    /// 计算 AI 优化的情绪信号。
    fn calculate_sentiment_signal(token: &TokenInfo, params: &AiOptimizationParams) -> u64 {
        let base_sentiment = 5000u64;
        let sentiment_adjustment = if token.price > PRICE_PRECISION {
            1000
        } else {
            0
        };
        (base_sentiment + sentiment_adjustment).min(params.max_signal_strength)
    }
    /// 应用置信度调整。
    fn apply_confidence_adjustment(
        weights: &mut [u64],
        params: &AiOptimizationParams,
    ) -> StrategyResult<()> {
        let confidence_factor = params.confidence_threshold;
        for weight in weights.iter_mut() {
            if confidence_factor < 7000 {
                let adjustment = (*weight * confidence_factor) / BASIS_POINTS_MAX;
                *weight = adjustment;
            }
        }
        let total: u64 = weights.iter().sum();
        if total > 0 && total != BASIS_POINTS_MAX {
            for weight in weights.iter_mut() {
                *weight = (*weight * BASIS_POINTS_MAX) / total;
            }
        }
        Ok(())
    }
}

/// 多因子策略参数结构体。
#[derive(Debug, Clone)]
pub struct MultiFactorParams {
    pub momentum_lookback: u32,      // 动量回溯周期
    pub momentum_threshold: u64,     // 动量阈值
    pub reversion_period: u32,       // 均值回归周期
    pub deviation_threshold: u64,    // 偏离阈值
    pub target_volatility: u64,      // 目标波动率
    pub risk_aversion: u64,          // 风险厌恶度
}

impl Default for MultiFactorParams {
    fn default() -> Self {
        Self {
            momentum_lookback: 30,
            momentum_threshold: 500, // 5%
            reversion_period: 14,
            deviation_threshold: 1000, // 10%
            target_volatility: 2000,   // 20%
            risk_aversion: 5000,       // 50%
        }
    }
}

/// AI 优化策略参数结构体。
#[derive(Debug, Clone)]
pub struct AiOptimizationParams {
    pub price_weight: u64,           // 价格信号权重
    pub volume_weight: u64,          // 成交量信号权重
    pub sentiment_weight: u64,       // 情绪信号权重
    pub confidence_threshold: u64,   // 置信度阈值
    pub max_signal_strength: u64,    // 最大信号强度
}

impl Default for AiOptimizationParams {
    fn default() -> Self {
        Self {
            price_weight: 4000,         // 40%
            volume_weight: 3000,        // 30%
            sentiment_weight: 3000,     // 30%
            confidence_threshold: 7000, // 70%
            max_signal_strength: 8000,  // 80%
        }
    }
}

/// 高级策略市场数据结构体。
#[derive(Debug, Clone)]
pub struct MarketData {
    pub token_supplies: Vec<u64>,        // 各 token 供应量
    pub historical_prices: Vec<u64>,     // 历史价格
    pub volatilities: Vec<u64>,          // 波动率数据
    pub volumes: Vec<u64>,               // 成交量数据
    pub timestamp: i64,                  // 数据时间戳
}

impl Default for MarketData {
    fn default() -> Self {
        Self {
            token_supplies: Vec::new(),
            historical_prices: Vec::new(),
            volatilities: Vec::new(),
            volumes: Vec::new(),
            timestamp: 0,
        }
    }
}
