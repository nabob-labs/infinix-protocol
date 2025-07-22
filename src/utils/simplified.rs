/*!
 * Simplified Utilities Module
 *
 * This module provides simplified, maintainable utility functions that reduce
 * code complexity while maintaining functionality. All operations are designed
 * for clarity, safety, and performance.
 */

use crate::core::*;
use crate::error::StrategyError;
use anchor_lang::prelude::*;

/// Simplified mathematical operations with built-in safety checks
pub struct SimplifiedMath;

impl SimplifiedMath {
    /// Safe multiplication with overflow protection
    pub fn safe_mul(a: u64, b: u64) -> Result<u64> {
        a.checked_mul(b).ok_or(StrategyError::MathOverflow.into())
    }

    /// Safe division with zero-check protection
    pub fn safe_div(a: u64, b: u64) -> Result<u64> {
        if b == 0 {
            return Err(StrategyError::DivisionByZero.into());
        }
        Ok(a / b)
    }

    /// Safe addition with overflow protection
    pub fn safe_add(a: u64, b: u64) -> Result<u64> {
        a.checked_add(b).ok_or(StrategyError::MathOverflow.into())
    }

    /// Safe subtraction with underflow protection
    pub fn safe_sub(a: u64, b: u64) -> Result<u64> {
        a.checked_sub(b).ok_or(StrategyError::MathOverflow.into())
    }

    /// Calculate percentage with basis points precision
    pub fn calculate_percentage(value: u64, percentage_bps: u64) -> Result<u64> {
        Self::safe_div(Self::safe_mul(value, percentage_bps)?, BASIS_POINTS_MAX)
    }

    /// Normalize weights to sum to 100% (10000 basis points)
    pub fn normalize_weights(weights: &mut [u64]) -> Result<()> {
        if weights.is_empty() {
            return Ok(());
        }

        let total: u64 = weights.iter().sum();
        if total == 0 {
            // Equal distribution if all weights are zero
            let equal_weight = BASIS_POINTS_MAX / weights.len() as u64;
            weights.fill(equal_weight);
            return Ok(());
        }

        // Normalize to 100%
        for weight in weights.iter_mut() {
            *weight = Self::safe_div(Self::safe_mul(*weight, BASIS_POINTS_MAX)?, total)?;
        }

        Ok(())
    }

    /// Calculate weighted average
    pub fn weighted_average(values: &[u64], weights: &[u64]) -> Result<u64> {
        if values.len() != weights.len() || values.is_empty() {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        let mut weighted_sum = 0u64;
        let mut total_weight = 0u64;

        for (value, weight) in values.iter().zip(weights.iter()) {
            weighted_sum = Self::safe_add(weighted_sum, Self::safe_mul(*value, *weight)?)?;
            total_weight = Self::safe_add(total_weight, *weight)?;
        }

        if total_weight == 0 {
            return Ok(0);
        }

        Self::safe_div(weighted_sum, total_weight)
    }

    /// Calculate compound interest
    pub fn compound_interest(principal: u64, rate_bps: u64, periods: u64) -> Result<u64> {
        if periods == 0 {
            return Ok(principal);
        }

        let rate_factor = Self::safe_add(BASIS_POINTS_MAX, rate_bps)?;
        let mut result = principal;

        for _ in 0..periods {
            result = Self::safe_div(Self::safe_mul(result, rate_factor)?, BASIS_POINTS_MAX)?;
        }

        Ok(result)
    }

    /// Alias for safe_mul for compatibility
    pub fn mul(a: u64, b: u64) -> Result<u64> {
        Self::safe_mul(a, b)
    }

    /// Alias for safe_add for compatibility
    pub fn add(a: u64, b: u64) -> Result<u64> {
        Self::safe_add(a, b)
    }

    /// Alias for safe_div for compatibility
    pub fn div(a: u64, b: u64) -> Result<u64> {
        Self::safe_div(a, b)
    }
}

/// Simplified validation utilities
pub struct SimplifiedValidation;

impl SimplifiedValidation {
    /// Validate token count is within acceptable limits
    pub fn validate_token_count(count: usize) -> Result<()> {
        if count == 0 || count > MAX_TOKENS {
            return Err(StrategyError::InvalidTokenCount.into());
        }
        Ok(())
    }

    /// Validate fee is within acceptable range
    pub fn validate_fee_bps(fee_bps: u64) -> Result<()> {
        if fee_bps > MAX_FEE_BPS as u64 {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(())
    }

    /// Validate slippage tolerance
    pub fn validate_slippage_bps(slippage_bps: u64) -> Result<()> {
        if slippage_bps > MAX_SLIPPAGE_BPS {
            return Err(StrategyError::SlippageExceeded.into());
        }
        Ok(())
    }

    /// Validate weights sum to 100%
    pub fn validate_weights_sum(weights: &[u64]) -> Result<()> {
        let total: u64 = weights.iter().sum();
        if total != BASIS_POINTS_MAX {
            return Err(StrategyError::InvalidWeightSum.into());
        }
        Ok(())
    }

    /// Validate time interval
    pub fn validate_time_interval(interval: u64, min: u64, max: u64) -> Result<()> {
        if interval < min || interval > max {
            return Err(StrategyError::InvalidTimeWindow.into());
        }
        Ok(())
    }

    /// Validate amount is above minimum threshold
    pub fn validate_minimum_amount(amount: u64, minimum: u64) -> Result<()> {
        if amount < minimum {
            return Err(StrategyError::BasketAmountTooSmall.into());
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

    /// Validate array lengths match
    pub fn validate_array_lengths_match<T, U>(arr1: &[T], arr2: &[U]) -> Result<()> {
        if arr1.len() != arr2.len() {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(())
    }

    /// Validate parameters size
    pub fn validate_parameters_size(params: &[u8], max_size: usize) -> Result<()> {
        if params.len() > max_size {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(())
    }

    /// Validate no duplicate pubkeys in array
    pub fn validate_no_duplicates(pubkeys: &[Pubkey]) -> Result<()> {
        let mut seen = std::collections::HashSet::new();
        for pubkey in pubkeys {
            if !seen.insert(*pubkey) {
                return Err(StrategyError::InvalidStrategyParameters.into());
            }
        }
        Ok(())
    }

    /// Validate weights (alias for validate_weights_sum)
    pub fn validate_weights(weights: &[u64]) -> Result<()> {
        Self::validate_weights_sum(weights)
    }

    /// Validate pubkey (alias for validate_pubkey_not_default)
    pub fn validate_pubkey(pubkey: &Pubkey, _name: &str) -> Result<()> {
        Self::validate_pubkey_not_default(pubkey)
    }

    /// Validate slippage (alias for validate_slippage_bps)
    pub fn validate_slippage(slippage_bps: u64) -> Result<()> {
        Self::validate_slippage_bps(slippage_bps)
    }
}

/// Simplified risk assessment utilities
pub struct SimplifiedRisk;

impl SimplifiedRisk {
    /// Calculate concentration risk based on weight distribution
    pub fn calculate_concentration_risk(weights: &[u64]) -> u32 {
        if weights.is_empty() {
            return 0;
        }

        // Calculate Herfindahl-Hirschman Index (HHI) simplified
        let hhi: u64 = weights.iter().map(|&w| (w * w) / BASIS_POINTS_MAX).sum();

        // Convert to risk score (0-10000)
        // Higher HHI = higher concentration = higher risk
        (hhi / 100).min(10000) as u32
    }

    /// Check if position size exceeds risk limits
    pub fn check_risk_limits(
        amount: u64,
        portfolio_value: u64,
        max_position_bps: u64,
    ) -> Result<()> {
        if portfolio_value == 0 {
            return Ok(());
        }

        let position_percentage = SimplifiedMath::safe_div(
            SimplifiedMath::safe_mul(amount, BASIS_POINTS_MAX)?,
            portfolio_value,
        )?;

        if position_percentage > max_position_bps {
            return Err(StrategyError::RiskLimitsExceeded.into());
        }

        Ok(())
    }

    /// Calculate simple VaR (Value at Risk) estimate
    pub fn calculate_simple_var(
        portfolio_value: u64,
        volatility_bps: u64,
        confidence_bps: u64,
    ) -> Result<u64> {
        // Simplified VaR calculation using normal distribution approximation
        let z_score = match confidence_bps {
            9500 => 1960, // 95% confidence ≈ 1.96 * 1000
            9900 => 2330, // 99% confidence ≈ 2.33 * 1000
            _ => 1960,    // Default to 95%
        };

        let var = SimplifiedMath::safe_div(
            SimplifiedMath::safe_mul(
                SimplifiedMath::safe_mul(portfolio_value, volatility_bps)?,
                z_score,
            )?,
            SimplifiedMath::safe_mul(BASIS_POINTS_MAX, 1000)?,
        )?;

        Ok(var)
    }

    /// Assess liquidity risk based on token distribution
    pub fn assess_liquidity_risk(token_counts: &[u64], total_liquidity: u64) -> u32 {
        if token_counts.is_empty() || total_liquidity == 0 {
            return 10000; // Maximum risk for no liquidity
        }

        // Calculate average liquidity per token
        let avg_liquidity = total_liquidity / token_counts.len() as u64;

        // Risk increases as liquidity decreases
        let risk_score = if avg_liquidity > 1_000_000 {
            1000 // Low risk
        } else if avg_liquidity > 100_000 {
            3000 // Medium risk
        } else if avg_liquidity > 10_000 {
            6000 // High risk
        } else {
            9000 // Very high risk
        };

        risk_score
    }
}

/// Simplified price calculation utilities
pub struct SimplifiedPrice;

impl SimplifiedPrice {
    /// Calculate simple moving average
    pub fn simple_moving_average(prices: &[u64], window: usize) -> Result<u64> {
        if prices.is_empty() || window == 0 || window > prices.len() {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        let sum: u64 = prices.iter().rev().take(window).sum();
        Ok(sum / window as u64)
    }

    /// Calculate price change percentage
    pub fn price_change_percentage(old_price: u64, new_price: u64) -> Result<i64> {
        if old_price == 0 {
            return Ok(0);
        }

        let change = if new_price >= old_price {
            SimplifiedMath::safe_mul(new_price - old_price, BASIS_POINTS_MAX)?
        } else {
            SimplifiedMath::safe_mul(old_price - new_price, BASIS_POINTS_MAX)?
        };

        let percentage = SimplifiedMath::safe_div(change, old_price)? as i64;

        Ok(if new_price >= old_price {
            percentage
        } else {
            -percentage
        })
    }

    /// Calculate volatility (simplified standard deviation)
    pub fn calculate_volatility(prices: &[u64]) -> Result<u64> {
        if prices.len() < 2 {
            return Ok(0);
        }

        let mean = prices.iter().sum::<u64>() / prices.len() as u64;
        let variance: u64 = prices
            .iter()
            .map(|&price| {
                let diff = if price > mean {
                    price - mean
                } else {
                    mean - price
                };
                diff * diff
            })
            .sum::<u64>()
            / prices.len() as u64;

        // Simplified square root approximation
        Ok(Self::sqrt_approximation(variance))
    }

    /// Simple square root approximation using Newton's method
    fn sqrt_approximation(n: u64) -> u64 {
        if n == 0 {
            return 0;
        }

        let mut x = n;
        let mut y = (x + 1) / 2;

        while y < x {
            x = y;
            y = (x + n / x) / 2;
        }

        x
    }

    /// Calculate TWAP (Time Weighted Average Price) simplified
    pub fn calculate_twap(prices: &[u64], weights: &[u64]) -> Result<u64> {
        if prices.is_empty() || prices.len() != weights.len() {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        SimplifiedMath::weighted_average(prices, weights)
    }

    /// Calculate price impact for trading
    pub fn calculate_price_impact(
        trade_amount: u64,
        available_liquidity: u64,
        base_impact_bps: u64,
    ) -> Result<u64> {
        if available_liquidity == 0 {
            return Ok(10000); // 100% impact if no liquidity
        }

        // Simple linear price impact model
        let impact_ratio = SimplifiedMath::safe_div(
            SimplifiedMath::safe_mul(trade_amount, BASIS_POINTS_MAX)?,
            available_liquidity,
        )?;

        let price_impact = SimplifiedMath::safe_mul(impact_ratio, base_impact_bps)?;
        Ok(price_impact.min(10000)) // Cap at 100%
    }

    /// Calculate total portfolio value
    pub fn calculate_total_value(
        tokens: &[crate::core::TokenWeight],
        price_feeds: &[crate::core::PriceFeed],
    ) -> Result<u64> {
        if tokens.len() != price_feeds.len() {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        let mut total_value = 0u64;

        for (token, price_feed) in tokens.iter().zip(price_feeds.iter()) {
            if token.mint != price_feed.mint {
                return Err(StrategyError::InvalidMarketData.into());
            }

            let token_value = SimplifiedMath::safe_div(
                SimplifiedMath::safe_mul(token.balance, price_feed.price)?,
                PRICE_PRECISION,
            )?;

            total_value = SimplifiedMath::safe_add(total_value, token_value)?;
        }

        Ok(total_value)
    }
}

/// Simplified portfolio utilities
pub struct SimplifiedPortfolio;

impl SimplifiedPortfolio {
    /// Calculate portfolio NAV (Net Asset Value)
    pub fn calculate_nav(
        token_balances: &[u64],
        token_prices: &[u64],
        total_shares: u64,
    ) -> Result<u64> {
        if token_balances.len() != token_prices.len() || total_shares == 0 {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        let mut total_value = 0u64;

        for (balance, price) in token_balances.iter().zip(token_prices.iter()) {
            let value = SimplifiedMath::safe_div(
                SimplifiedMath::safe_mul(*balance, *price)?,
                PRICE_PRECISION,
            )?;
            total_value = SimplifiedMath::safe_add(total_value, value)?;
        }

        SimplifiedMath::safe_div(
            SimplifiedMath::safe_mul(total_value, PRICE_PRECISION)?,
            total_shares,
        )
    }

    /// Calculate rebalancing requirements
    pub fn calculate_rebalancing_needs(
        current_weights: &[u64],
        target_weights: &[u64],
        threshold_bps: u64,
    ) -> Result<Vec<bool>> {
        if current_weights.len() != target_weights.len() {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        let needs_rebalancing: Vec<bool> = current_weights
            .iter()
            .zip(target_weights.iter())
            .map(|(current, target)| {
                let deviation = if current > target {
                    current - target
                } else {
                    target - current
                };
                deviation >= threshold_bps
            })
            .collect();

        Ok(needs_rebalancing)
    }

    /// Calculate optimal trade sizes for rebalancing
    pub fn calculate_trade_sizes(
        current_balances: &[u64],
        target_weights: &[u64],
        total_value: u64,
    ) -> Result<Vec<i64>> {
        if current_balances.len() != target_weights.len() {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        let mut trade_sizes = Vec::new();

        for (balance, weight) in current_balances.iter().zip(target_weights.iter()) {
            let target_value = SimplifiedMath::safe_div(
                SimplifiedMath::safe_mul(total_value, *weight)?,
                BASIS_POINTS_MAX,
            )?;

            let trade_size = if target_value >= *balance {
                (target_value - balance) as i64
            } else {
                -((balance - target_value) as i64)
            };

            trade_sizes.push(trade_size);
        }

        Ok(trade_sizes)
    }

    /// Calculate portfolio diversification score
    pub fn calculate_diversification_score(weights: &[u64]) -> u32 {
        if weights.is_empty() {
            return 0;
        }

        // Calculate effective number of assets (inverse of HHI)
        let hhi: u64 = weights.iter().map(|&w| (w * w) / BASIS_POINTS_MAX).sum();

        if hhi == 0 {
            return 0;
        }

        // Effective number of assets
        let effective_assets = BASIS_POINTS_MAX / hhi;

        // Convert to score (0-10000), higher is better
        (effective_assets * 1000).min(10000) as u32
    }
}

/// Trait for validatable types
pub trait Validatable {
    fn validate(&self) -> Result<()>;
}

/// Trait for authorizable types
pub trait Authorizable {
    fn authority(&self) -> Pubkey;
    fn is_authorized(&self, authority: &Pubkey) -> bool {
        self.authority() == *authority
    }
}

/// Trait for pausable types
pub trait Pausable {
    fn is_paused(&self) -> bool;
    fn pause(&mut self) -> Result<()>;
    fn unpause(&mut self) -> Result<()>;
}

/// Simplified cache implementation
pub struct SimplifiedCache<K, V> {
    entries: std::collections::HashMap<K, (V, i64)>,
    ttl_seconds: i64,
}

impl<K: std::hash::Hash + Eq + Clone, V: Clone> SimplifiedCache<K, V> {
    pub fn new(ttl_seconds: i64) -> Self {
        Self {
            entries: std::collections::HashMap::new(),
            ttl_seconds,
        }
    }

    pub fn get(&self, key: &K) -> Option<V> {
        if let Some((value, timestamp)) = self.entries.get(key) {
            let current_time = Clock::get().ok()?.unix_timestamp;
            if current_time - timestamp < self.ttl_seconds {
                return Some(value.clone());
            }
        }
        None
    }

    pub fn set(&mut self, key: K, value: V) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;
        self.entries.insert(key, (value, current_time));
        Ok(())
    }

    pub fn clear_expired(&mut self) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;
        self.entries
            .retain(|_, (_, timestamp)| current_time - *timestamp < self.ttl_seconds);
        Ok(())
    }
}

/// Simplified batch processor
pub struct SimplifiedBatchProcessor;

impl SimplifiedBatchProcessor {
    /// Process items in batches with a given batch size
    pub fn process_in_batches<T, F, R>(
        items: &[T],
        batch_size: usize,
        mut processor: F,
    ) -> Result<Vec<R>>
    where
        F: FnMut(&[T]) -> Result<Vec<R>>,
    {
        if batch_size == 0 {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        let mut results = Vec::new();

        for chunk in items.chunks(batch_size) {
            let mut batch_results = processor(chunk)?;
            results.append(&mut batch_results);
        }

        Ok(results)
    }

    /// Calculate optimal batch size based on compute constraints
    pub fn calculate_optimal_batch_size(
        item_compute_cost: u32,
        available_compute: u32,
        max_batch_size: u32,
    ) -> u32 {
        if item_compute_cost == 0 {
            return max_batch_size;
        }

        let computed_size = available_compute / item_compute_cost;
        computed_size.min(max_batch_size).max(1)
    }
}
