//!
//! Weight Strategy Implementations
//!
//! 本模块实现多种权重分配算法，适用于指数篮子、资产配置等场景，确保权重计算合规、可扩展、可插拔。

// 引入核心模块、适配器 trait、错误类型和策略模块。
use crate::core::*;
use crate::core::adapter::AdapterTrait;
use crate::error::StrategyError;
use crate::strategies::*;
use anchor_lang::prelude::*;

/// 权重策略执行器结构体，支持多种权重分配算法。
pub struct WeightStrategyExecutor;

impl WeightStrategyExecutor {
    /// 执行等权重策略。
    ///
    /// # 参数
    /// * `token_count` - 资产数量。
    /// # 返回
    /// * 权重向量或错误。
    pub fn execute_equal_weight(token_count: usize) -> StrategyResult<Vec<u64>> {
        // 校验资产数量边界。
        if token_count == 0 || token_count > MAX_TOKENS {
            return Err(StrategyError::InvalidTokenCount.into());
        }
        // 计算等权重。
        let equal_weight = BASIS_POINTS_MAX / token_count as u64;
        let mut weights = vec![equal_weight; token_count];
        // 处理整除误差，调整第一个权重。
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
        // 计算市值（简化，实际应用真实供应量）。
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
        // 按市值比例分配权重。
        let mut weights: Vec<u64> = market_caps
            .iter()
            .map(|&cap| (cap * BASIS_POINTS_MAX) / total_market_cap)
            .collect();
        // 应用最小/最大权重约束。
        for weight in &mut weights {
            *weight = (*weight).max(params.min_weight).min(params.max_weight);
        }
        // 归一化。
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
        for (i, token) in token_info.iter().enumerate() {
            let momentum_score = Self::calculate_momentum_score(
                &price_history[i],
                params.lookback_period,
                params.momentum_factor,
            )?;
            momentum_scores.push(momentum_score);
            total_score += momentum_score;
        }
        // 分数转权重。
        let mut weights = Vec::new();
        if total_score > 0 {
            for score in momentum_scores {
                let base_weight = (score * BASIS_POINTS_MAX) / total_score;
                let adjusted_weight = base_weight.max(params.base_weight);
                weights.push(adjusted_weight);
            }
        } else {
            // 无动量信号时等权重。
            return Self::execute_equal_weight(token_info.len());
        }
        // 归一化。
        Self::normalize_weights(&mut weights)?;
        Ok(weights)
    }
    /// 执行波动率调整策略。
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
        let mut risk_adjusted_scores = Vec::new();
        let mut total_score = 0u64;
        for (i, _token) in token_info.iter().enumerate() {
            let volatility = volatility_data[i];
            let risk_score = Self::calculate_risk_adjusted_score(
                volatility,
                params.target_volatility,
                params.risk_aversion,
            )?;
            risk_adjusted_scores.push(risk_score);
            total_score += risk_score;
        }
        // 分数转权重。
        let mut weights = Vec::new();
        if total_score > 0 {
            for score in risk_adjusted_scores {
                let weight = (score * BASIS_POINTS_MAX) / total_score;
                weights.push(weight);
            }
        } else {
            // 无风险分数时等权重。
            return Self::execute_equal_weight(token_info.len());
        }
        // 归一化。
        Self::normalize_weights(&mut weights)?;
        Ok(weights)
    }
    /// 执行固定权重策略。
    pub fn execute_fixed_weight(fixed_weights: &[u64]) -> StrategyResult<Vec<u64>> {
        if fixed_weights.is_empty() || fixed_weights.len() > MAX_TOKENS {
            return Err(StrategyError::InvalidTokenCount.into());
        }
        // 校验权重总和为 10000。
        let total: u64 = fixed_weights.iter().sum();
        if total != BASIS_POINTS_MAX {
            return Err(StrategyError::InvalidWeightSum.into());
        }
        // 校验单个权重不超限。
        for &weight in fixed_weights {
            if weight > MAX_TOKEN_WEIGHT_BPS {
                return Err(StrategyError::InvalidStrategyParameters.into());
            }
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
        let mut technical_scores = Vec::new();
        let mut total_score = 0u64;
        for (i, _token) in token_info.iter().enumerate() {
            let technical_score = Self::calculate_technical_score(&technical_data[i], &params)?;
            technical_scores.push(technical_score);
            total_score += technical_score;
        }
        // 分数转权重。
        let mut weights = Vec::new();
        if total_score > 0 {
            for score in technical_scores {
                let weight = (score * BASIS_POINTS_MAX) / total_score;
                weights.push(weight);
            }
        } else {
            // 无技术分数时等权重。
            return Self::execute_equal_weight(token_info.len());
        }
        // 归一化。
        Self::normalize_weights(&mut weights)?;
        Ok(weights)
    }
    /// 计算动量分数。
    fn calculate_momentum_score(
        price_history: &[u64],
        lookback_period: u32,
        momentum_factor: u64,
    ) -> StrategyResult<u64> {
        if price_history.is_empty() {
            return Ok(5000); // 中性分数
        }
        let lookback = (lookback_period as usize).min(price_history.len());
        if lookback < 2 {
            return Ok(5000); // 中性分数
        }
        let recent_prices = &price_history[price_history.len() - lookback..];
        let start_price = recent_prices[0];
        let end_price = recent_prices[recent_prices.len() - 1];
        if start_price == 0 {
            return Ok(5000); // 中性分数
        }
        // 计算动量百分比变化。
        let momentum = if end_price >= start_price {
            ((end_price - start_price) * 10000) / start_price
        } else {
            0 // 简化：不考虑负动量
        };
        // 应用动量因子。
        let score = 5000 + ((momentum * momentum_factor) / 10000).min(4000);
        Ok(score)
    }
    /// 计算基于波动率的风险调整分数。
    fn calculate_risk_adjusted_score(
        volatility: u64,
        target_volatility: u64,
        risk_aversion: u64,
    ) -> StrategyResult<u64> {
        let base_score = 5000u64;
        if volatility <= target_volatility {
            // 奖励低波动率。
            let reward = ((target_volatility - volatility) * 2000) / target_volatility;
            Ok(base_score + reward.min(3000))
        } else {
            // 惩罚高波动率。
            let excess_vol = volatility - target_volatility;
            let penalty = (excess_vol * risk_aversion) / 10000;
            Ok(base_score.saturating_sub(penalty.min(4000)))
        }
    }
    /// 计算技术指标分数。
    fn calculate_technical_score(
        indicators: &TechnicalIndicators,
        params: &TechnicalIndicatorParams,
    ) -> StrategyResult<u64> {
        let mut total_score = 0u64;
        let mut weight_sum = 0u64;
        // RSI 分量。
        if let Some(rsi) = indicators.rsi {
            let rsi_score =
                Self::calculate_rsi_score(rsi, params.rsi_oversold, params.rsi_overbought);
            total_score += rsi_score * params.rsi_weight;
            weight_sum += params.rsi_weight;
        }
        // MACD 分量。
        if let Some(macd) = indicators.macd_signal {
            let macd_score = Self::calculate_macd_score(macd);
            total_score += macd_score * params.macd_weight;
            weight_sum += params.macd_weight;
        }
        // 布林带分量。
        if let Some(bb_position) = indicators.bollinger_position {
            let bb_score = Self::calculate_bollinger_score(bb_position);
            total_score += bb_score * params.bollinger_weight;
            weight_sum += params.bollinger_weight;
        }
        // 均线分量。
        if let Some(ma_convergence) = indicators.ma_convergence {
            let ma_score = Self::calculate_ma_score(ma_convergence);
            total_score += ma_score * params.ma_weight;
            weight_sum += params.ma_weight;
        }
        if weight_sum > 0 {
            Ok(total_score / weight_sum)
        } else {
            Ok(5000) // 中性分数
        }
    }
    /// 计算 RSI 分数。
    fn calculate_rsi_score(rsi: u64, oversold: u64, overbought: u64) -> u64 {
        if rsi <= oversold {
            8000 // 强买信号
        } else if rsi >= overbought {
            2000 // 强卖信号
        } else {
            // 线性插值。
            let mid_point = (oversold + overbought) / 2;
            if rsi <= mid_point {
                5000 + ((mid_point - rsi) * 3000) / (mid_point - oversold)
            } else {
                5000 - ((rsi - mid_point) * 3000) / (overbought - mid_point)
            }
        }
    }
    /// 计算 MACD 分数。
    fn calculate_macd_score(macd_signal: i64) -> u64 {
        if macd_signal > 100 {
            7000 // 多头信号
        } else if macd_signal < -100 {
            3000 // 空头信号
        } else {
            5000 + (macd_signal * 20) as u64
        }
    }
    /// 计算布林带分数。
    fn calculate_bollinger_score(bb_position: u64) -> u64 {
        if bb_position <= 2000 {
            7000 // 下轨，超卖
        } else if bb_position >= 8000 {
            3000 // 上轨，超买
        } else {
            5000 + ((5000 - bb_position as i64) / 2) as u64
        }
    }
    /// 计算均线分数。
    fn calculate_ma_score(ma_convergence: i64) -> u64 {
        if ma_convergence > 500 {
            7000 // 强上升趋势
        } else if ma_convergence < -500 {
            3000 // 强下降趋势
        } else {
            5000 + (ma_convergence * 4) as u64
        }
    }
    /// 归一化权重，确保总和为 10000。
    fn normalize_weights(weights: &mut [u64]) -> StrategyResult<()> {
        let total: u64 = weights.iter().sum();
        if total == 0 {
            return Err(StrategyError::InvalidWeightSum.into());
        }
        if total != BASIS_POINTS_MAX {
            for weight in weights.iter_mut() {
                *weight = (*weight * BASIS_POINTS_MAX) / total;
            }
            // 处理舍入误差。
            let new_total: u64 = weights.iter().sum();
            if new_total != BASIS_POINTS_MAX && !weights.is_empty() {
                let diff = if new_total > BASIS_POINTS_MAX {
                    new_total - BASIS_POINTS_MAX
                } else {
                    BASIS_POINTS_MAX - new_total
                };
                if new_total > BASIS_POINTS_MAX {
                    weights[0] = weights[0].saturating_sub(diff);
                } else {
                    weights[0] += diff;
                }
            }
        }
        Ok(())
    }
}

/// 权重策略适配器结构体。
pub struct WeightStrategyAdapter;

impl AdapterTrait for WeightStrategyAdapter {
    fn name(&self) -> &'static str { "weight_strategy" }
    fn version(&self) -> &'static str { "1.0.0" }
    fn supported_assets(&self) -> Vec<String> { vec!["SOL".to_string(), "USDC".to_string()] }
    fn status(&self) -> Option<String> { Some("active".to_string()) }
}

// 使用构造器自动注册权重策略适配器到全局工厂。
#[ctor::ctor]
fn auto_register_weight_strategy_adapter() {
    let adapter = WeightStrategyAdapter;
    let mut factory = crate::core::registry::ADAPTER_FACTORY.lock().unwrap();
    factory.register(adapter);
}

/// 技术指标结构体。
#[derive(Debug, Clone)]
pub struct TechnicalIndicators {
    pub rsi: Option<u64>,                 // 相对强弱指标
    pub macd_signal: Option<i64>,         // MACD 信号
    pub bollinger_position: Option<u64>,  // 布林带位置
    pub ma_convergence: Option<i64>,      // 均线收敛
    pub volume_indicators: Option<VolumeIndicators>, // 成交量指标
}

/// 成交量指标结构体。
#[derive(Debug, Clone)]
pub struct VolumeIndicators {
    pub vwap: u64,        // 加权平均价
    pub obv: i64,         // 能量潮指标
    pub volume_roc: i64,  // 成交量变化率
}

/// 技术指标参数结构体。
#[derive(Debug, Clone)]
pub struct TechnicalIndicatorParams {
    pub rsi_weight: u64,       // RSI 权重
    pub macd_weight: u64,      // MACD 权重
    pub bollinger_weight: u64, // 布林带权重
    pub ma_weight: u64,        // 均线权重
    pub rsi_oversold: u64,     // RSI 超卖阈值
    pub rsi_overbought: u64,   // RSI 超买阈值
}

impl Default for TechnicalIndicatorParams {
    fn default() -> Self {
        Self {
            rsi_weight: 3000,       // 30%
            macd_weight: 2500,      // 25%
            bollinger_weight: 2500, // 25%
            ma_weight: 2000,        // 20%
            rsi_oversold: 3000,     // 30
            rsi_overbought: 7000,   // 70
        }
    }
}
