/*!
 * Weight Strategy Implementations
 *
 * Contains various weight calculation algorithms for portfolio construction.
 */

use crate::core::*;
use crate::error::StrategyError;
use crate::strategies::*;
use anchor_lang::prelude::*;

/// Weight strategy executor for different weighting algorithms
pub struct WeightStrategyExecutor;

impl WeightStrategyExecutor {
    /// Execute equal weight strategy
    pub fn execute_equal_weight(token_count: usize) -> StrategyResult<Vec<u64>> {
        if token_count == 0 || token_count > MAX_TOKENS {
            return Err(StrategyError::InvalidTokenCount.into());
        }

        let equal_weight = BASIS_POINTS_MAX / token_count as u64;
        let mut weights = vec![equal_weight; token_count];

        // Handle rounding by adjusting the first weight
        let total: u64 = weights.iter().sum();
        if total != BASIS_POINTS_MAX {
            weights[0] += BASIS_POINTS_MAX - total;
        }

        Ok(weights)
    }

    /// Execute market cap weighted strategy
    pub fn execute_market_cap_weighted(
        token_info: &[TokenInfo],
        params: MarketCapWeightedParams,
    ) -> StrategyResult<Vec<u64>> {
        if token_info.is_empty() || token_info.len() > MAX_TOKENS {
            return Err(StrategyError::InvalidTokenCount.into());
        }

        // Calculate market caps (simplified - in production would use actual supply data)
        let market_caps: Vec<u64> = token_info
            .iter()
            .map(|token| {
                // Estimate market cap based on price (simplified)
                let estimated_supply = 1_000_000_000u64; // 1B tokens estimate
                token.price.saturating_mul(estimated_supply)
            })
            .collect();

        let total_market_cap: u64 = market_caps.iter().sum();
        if total_market_cap == 0 {
            return Err(StrategyError::InvalidMarketData.into());
        }

        // Calculate raw weights based on market cap
        let mut weights: Vec<u64> = market_caps
            .iter()
            .map(|&cap| (cap * BASIS_POINTS_MAX) / total_market_cap)
            .collect();

        // Apply min/max constraints
        for weight in &mut weights {
            *weight = (*weight).max(params.min_weight).min(params.max_weight);
        }

        // Renormalize after applying constraints
        let total_constrained: u64 = weights.iter().sum();
        if total_constrained > 0 {
            for weight in &mut weights {
                *weight = (*weight * BASIS_POINTS_MAX) / total_constrained;
            }
        }

        Ok(weights)
    }

    /// Execute momentum weighted strategy
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

        // Convert momentum scores to weights
        let mut weights = Vec::new();
        if total_score > 0 {
            for score in momentum_scores {
                let base_weight = (score * BASIS_POINTS_MAX) / total_score;
                let adjusted_weight = base_weight.max(params.base_weight);
                weights.push(adjusted_weight);
            }
        } else {
            // Fallback to equal weights
            return Self::execute_equal_weight(token_info.len());
        }

        // Normalize weights
        Self::normalize_weights(&mut weights)?;

        Ok(weights)
    }

    /// Execute volatility adjusted strategy
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

        // Convert risk scores to weights
        let mut weights = Vec::new();
        if total_score > 0 {
            for score in risk_adjusted_scores {
                let weight = (score * BASIS_POINTS_MAX) / total_score;
                weights.push(weight);
            }
        } else {
            // Fallback to equal weights
            return Self::execute_equal_weight(token_info.len());
        }

        // Normalize weights
        Self::normalize_weights(&mut weights)?;

        Ok(weights)
    }

    /// Execute fixed weight strategy
    pub fn execute_fixed_weight(fixed_weights: &[u64]) -> StrategyResult<Vec<u64>> {
        if fixed_weights.is_empty() || fixed_weights.len() > MAX_TOKENS {
            return Err(StrategyError::InvalidTokenCount.into());
        }

        // Validate weights sum to 100%
        let total: u64 = fixed_weights.iter().sum();
        if total != BASIS_POINTS_MAX {
            return Err(StrategyError::InvalidWeightSum.into());
        }

        // Validate individual weight limits
        for &weight in fixed_weights {
            if weight > MAX_TOKEN_WEIGHT_BPS {
                return Err(StrategyError::InvalidStrategyParameters.into());
            }
        }

        Ok(fixed_weights.to_vec())
    }

    /// Execute technical indicator based strategy
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

        // Convert technical scores to weights
        let mut weights = Vec::new();
        if total_score > 0 {
            for score in technical_scores {
                let weight = (score * BASIS_POINTS_MAX) / total_score;
                weights.push(weight);
            }
        } else {
            // Fallback to equal weights
            return Self::execute_equal_weight(token_info.len());
        }

        // Normalize weights
        Self::normalize_weights(&mut weights)?;

        Ok(weights)
    }

    /// Calculate momentum score for a token
    fn calculate_momentum_score(
        price_history: &[u64],
        lookback_period: u32,
        momentum_factor: u64,
    ) -> StrategyResult<u64> {
        if price_history.is_empty() {
            return Ok(5000); // Neutral score
        }

        let lookback = (lookback_period as usize).min(price_history.len());
        if lookback < 2 {
            return Ok(5000); // Neutral score
        }

        let recent_prices = &price_history[price_history.len() - lookback..];
        let start_price = recent_prices[0];
        let end_price = recent_prices[recent_prices.len() - 1];

        if start_price == 0 {
            return Ok(5000); // Neutral score
        }

        // Calculate momentum as percentage change
        let momentum = if end_price >= start_price {
            ((end_price - start_price) * 10000) / start_price
        } else {
            0 // No negative momentum in this simplified version
        };

        // Apply momentum factor
        let score = 5000 + ((momentum * momentum_factor) / 10000).min(4000);

        Ok(score)
    }

    /// Calculate risk-adjusted score based on volatility
    fn calculate_risk_adjusted_score(
        volatility: u64,
        target_volatility: u64,
        risk_aversion: u64,
    ) -> StrategyResult<u64> {
        let base_score = 5000u64; // Neutral score

        if volatility <= target_volatility {
            // Reward low volatility
            let reward = ((target_volatility - volatility) * 2000) / target_volatility;
            Ok(base_score + reward.min(3000))
        } else {
            // Penalize high volatility based on risk aversion
            let excess_vol = volatility - target_volatility;
            let penalty = (excess_vol * risk_aversion) / 10000;
            Ok(base_score.saturating_sub(penalty.min(4000)))
        }
    }

    /// Calculate technical indicator score
    fn calculate_technical_score(
        indicators: &TechnicalIndicators,
        params: &TechnicalIndicatorParams,
    ) -> StrategyResult<u64> {
        let mut total_score = 0u64;
        let mut weight_sum = 0u64;

        // RSI component
        if let Some(rsi) = indicators.rsi {
            let rsi_score =
                Self::calculate_rsi_score(rsi, params.rsi_oversold, params.rsi_overbought);
            total_score += rsi_score * params.rsi_weight;
            weight_sum += params.rsi_weight;
        }

        // MACD component
        if let Some(macd) = indicators.macd_signal {
            let macd_score = Self::calculate_macd_score(macd);
            total_score += macd_score * params.macd_weight;
            weight_sum += params.macd_weight;
        }

        // Bollinger Band component
        if let Some(bb_position) = indicators.bollinger_position {
            let bb_score = Self::calculate_bollinger_score(bb_position);
            total_score += bb_score * params.bollinger_weight;
            weight_sum += params.bollinger_weight;
        }

        // Moving Average component
        if let Some(ma_convergence) = indicators.ma_convergence {
            let ma_score = Self::calculate_ma_score(ma_convergence);
            total_score += ma_score * params.ma_weight;
            weight_sum += params.ma_weight;
        }

        if weight_sum > 0 {
            Ok(total_score / weight_sum)
        } else {
            Ok(5000) // Neutral score
        }
    }

    /// Calculate RSI-based score
    fn calculate_rsi_score(rsi: u64, oversold: u64, overbought: u64) -> u64 {
        if rsi <= oversold {
            8000 // Strong buy signal
        } else if rsi >= overbought {
            2000 // Strong sell signal
        } else {
            // Linear interpolation between oversold and overbought
            let mid_point = (oversold + overbought) / 2;
            if rsi <= mid_point {
                5000 + ((mid_point - rsi) * 3000) / (mid_point - oversold)
            } else {
                5000 - ((rsi - mid_point) * 3000) / (overbought - mid_point)
            }
        }
    }

    /// Calculate MACD-based score
    fn calculate_macd_score(macd_signal: i64) -> u64 {
        if macd_signal > 100 {
            7000 // Bullish signal
        } else if macd_signal < -100 {
            3000 // Bearish signal
        } else {
            // Linear scaling around neutral
            5000 + (macd_signal * 20) as u64
        }
    }

    /// Calculate Bollinger Band score
    fn calculate_bollinger_score(bb_position: u64) -> u64 {
        if bb_position <= 2000 {
            // Lower band
            7000 // Oversold, potential buy
        } else if bb_position >= 8000 {
            // Upper band
            3000 // Overbought, potential sell
        } else {
            // Linear scaling
            5000 + ((5000 - bb_position as i64) / 2) as u64
        }
    }

    /// Calculate Moving Average score
    fn calculate_ma_score(ma_convergence: i64) -> u64 {
        if ma_convergence > 500 {
            7000 // Strong uptrend
        } else if ma_convergence < -500 {
            3000 // Strong downtrend
        } else {
            // Linear scaling
            5000 + (ma_convergence * 4) as u64
        }
    }

    /// Normalize weights to sum to BASIS_POINTS_MAX
    fn normalize_weights(weights: &mut [u64]) -> StrategyResult<()> {
        let total: u64 = weights.iter().sum();
        if total == 0 {
            return Err(StrategyError::InvalidWeightSum.into());
        }

        if total != BASIS_POINTS_MAX {
            for weight in weights.iter_mut() {
                *weight = (*weight * BASIS_POINTS_MAX) / total;
            }

            // Handle rounding errors
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

/// Technical indicators structure
#[derive(Debug, Clone)]
pub struct TechnicalIndicators {
    pub rsi: Option<u64>,
    pub macd_signal: Option<i64>,
    pub bollinger_position: Option<u64>,
    pub ma_convergence: Option<i64>,
    pub volume_indicators: Option<VolumeIndicators>,
}

/// Volume indicators structure
#[derive(Debug, Clone)]
pub struct VolumeIndicators {
    pub vwap: u64,
    pub obv: i64,
    pub volume_roc: i64,
}

/// Technical indicator parameters
#[derive(Debug, Clone)]
pub struct TechnicalIndicatorParams {
    pub rsi_weight: u64,
    pub macd_weight: u64,
    pub bollinger_weight: u64,
    pub ma_weight: u64,
    pub rsi_oversold: u64,
    pub rsi_overbought: u64,
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
