/*!
 * Price Utilities Module
 *
 * Price calculation and analysis utilities for trading strategies.
 */

use crate::core::*;
use crate::error::StrategyError;
use crate::utils::math::MathOps;
use anchor_lang::prelude::*;

/// Price calculation utilities
pub struct PriceUtils;

impl PriceUtils {
    /// Calculate total portfolio value
    pub fn calculate_total_value(tokens: &[TokenWeight], price_feeds: &[PriceFeed]) -> Result<u64> {
        if tokens.len() != price_feeds.len() {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        let mut total_value = 0u64;

        for (token, price_feed) in tokens.iter().zip(price_feeds.iter()) {
            // Validate price feed
            price_feed.validate()?;

            // Calculate token value
            let token_value = MathOps::div(
                MathOps::mul(token.balance, price_feed.price)?,
                PRICE_PRECISION,
            )?;

            total_value = MathOps::add(total_value, token_value)?;
        }

        Ok(total_value)
    }

    /// Calculate price impact for a trade
    pub fn calculate_price_impact(
        trade_amount: u64,
        liquidity: u64,
        current_price: u64,
    ) -> Result<u64> {
        if liquidity == 0 {
            return Ok(MAX_SLIPPAGE_BPS); // Maximum impact if no liquidity
        }

        // Simplified price impact calculation
        // Impact = (trade_amount / liquidity) * impact_factor
        let impact_factor = 5000u64; // 50% base impact factor
        let raw_impact = MathOps::div(MathOps::mul(trade_amount, impact_factor)?, liquidity)?;

        // Cap at maximum slippage
        Ok(raw_impact.min(MAX_SLIPPAGE_BPS))
    }

    /// Calculate TWAP (Time Weighted Average Price)
    pub fn calculate_twap(prices: &[u64], time_weights: &[u64]) -> Result<u64> {
        if prices.is_empty() || prices.len() != time_weights.len() {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        MathOps::weighted_average(prices, time_weights)
    }

    /// Calculate VWAP (Volume Weighted Average Price)
    pub fn calculate_vwap(prices: &[u64], volumes: &[u64]) -> Result<u64> {
        if prices.is_empty() || prices.len() != volumes.len() {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        MathOps::weighted_average(prices, volumes)
    }

    /// Calculate price volatility
    pub fn calculate_volatility(prices: &[u64]) -> Result<u64> {
        if prices.len() < 2 {
            return Ok(0);
        }

        // Calculate mean
        let sum: u64 = prices.iter().sum();
        let mean = sum / prices.len() as u64;

        // Calculate variance
        let variance_sum: u64 = prices
            .iter()
            .map(|&price| {
                let diff = if price > mean {
                    price - mean
                } else {
                    mean - price
                };
                diff * diff
            })
            .sum();

        let variance = variance_sum / prices.len() as u64;

        // Return standard deviation (square root of variance)
        Ok(MathOps::sqrt(variance))
    }

    /// Calculate price change percentage
    pub fn calculate_price_change(old_price: u64, new_price: u64) -> Result<i64> {
        if old_price == 0 {
            return Ok(0);
        }

        let change = if new_price >= old_price {
            MathOps::mul(new_price - old_price, BASIS_POINTS_MAX)?
        } else {
            MathOps::mul(old_price - new_price, BASIS_POINTS_MAX)?
        };

        let percentage = MathOps::div(change, old_price)? as i64;

        Ok(if new_price >= old_price {
            percentage
        } else {
            -percentage
        })
    }

    /// Calculate moving average
    pub fn calculate_moving_average(prices: &[u64], window: usize) -> Result<u64> {
        if prices.is_empty() || window == 0 || window > prices.len() {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        let sum: u64 = prices.iter().rev().take(window).sum();
        Ok(sum / window as u64)
    }

    /// Validate price feed data
    pub fn validate_price_feed(price_feed: &PriceFeed) -> Result<()> {
        // Check if price is valid
        if price_feed.price == 0 {
            return Err(StrategyError::PriceFeedUnavailable.into());
        }

        // Check if price feed is not too old
        let current_time = Clock::get()?.unix_timestamp;
        if current_time - price_feed.last_updated > PRICE_FEED_STALENESS_THRESHOLD {
            return Err(StrategyError::PriceFeedUnavailable.into());
        }

        // Check if price feed is marked as valid
        if !price_feed.is_valid {
            return Err(StrategyError::PriceFeedUnavailable.into());
        }

        Ok(())
    }
}

/// Token weight structure for calculations
#[derive(Debug, Clone)]
pub struct TokenWeight {
    pub mint: Pubkey,
    pub current_weight: u64,
    pub target_weight: u64,
    pub balance: u64,
    pub price: u64,
}

/// Rebalancing action structure
#[derive(Debug, Clone)]
pub struct RebalanceAction {
    pub token_mint: Pubkey,
    pub action_type: u8, // 0 = buy, 1 = sell
    pub amount: u64,
    pub price_impact: u64,
}

/// Price feed validation trait
impl PriceFeed {
    pub fn validate(&self) -> Result<()> {
        PriceUtils::validate_price_feed(self)
    }
}
