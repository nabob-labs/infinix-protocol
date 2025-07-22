/*!
 * Validation Utilities Module
 *
 * Comprehensive validation functions for input parameters and state.
 */

use crate::core::*;
use crate::core::constants::BASIS_POINTS_MAX;
use crate::error::StrategyError;
use anchor_lang::prelude::*;

/// Validation utilities
pub struct ValidationUtils;

impl ValidationUtils {
    /// Validate token count is within limits
    pub fn validate_token_count(count: usize) -> Result<()> {
        if count == 0 {
            return Err(StrategyError::InvalidTokenCount.into());
        }
        if count > MAX_TOKENS {
            return Err(StrategyError::InvalidTokenCount.into());
        }
        Ok(())
    }

    /// Validate weights sum to BASIS_POINTS_MAX (100%)
    pub fn validate_weights(weights: &[u64]) -> Result<()> {
        if weights.is_empty() {
            return Err(StrategyError::InvalidTokenCount.into());
        }

        let total: u64 = weights.iter().sum();
        if total != BASIS_POINTS_MAX {
            return Err(StrategyError::InvalidWeightSum.into());
        }

        // Check individual weight limits
        for &weight in weights {
            if weight > MAX_TOKEN_WEIGHT_BPS {
                return Err(StrategyError::InvalidStrategyParameters.into());
            }
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

    /// Validate no duplicate pubkeys
    pub fn validate_no_duplicates(pubkeys: &[Pubkey]) -> Result<()> {
        let mut seen = std::collections::HashSet::new();
        for pubkey in pubkeys {
            if !seen.insert(*pubkey) {
                return Err(StrategyError::InvalidStrategyParameters.into());
            }
        }
        Ok(())
    }

    /// Validate pubkey is not default
    pub fn validate_pubkey(pubkey: &Pubkey, field_name: &str) -> Result<()> {
        if *pubkey == Pubkey::default() {
            msg!("Invalid pubkey for field: {}", field_name);
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(())
    }

    /// Validate time interval
    pub fn validate_time_interval(interval: u64) -> Result<()> {
        if interval < MIN_REBALANCE_INTERVAL {
            return Err(StrategyError::InvalidTimeWindow.into());
        }
        if interval > MAX_REBALANCE_INTERVAL {
            return Err(StrategyError::InvalidTimeWindow.into());
        }
        Ok(())
    }

    /// Validate slippage tolerance
    pub fn validate_slippage(slippage: u64) -> Result<()> {
        if slippage > MAX_SLIPPAGE_BPS {
            return Err(StrategyError::SlippageExceeded.into());
        }
        Ok(())
    }

    /// Validate fee is within limits
    pub fn validate_fee(fee_bps: u64) -> Result<()> {
        if fee_bps > MAX_FEE_BPS as u64 {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(())
    }

    /// Validate amount is above minimum
    pub fn validate_minimum_amount(amount: u64, minimum: u64) -> Result<()> {
        if amount < minimum {
            return Err(StrategyError::BasketAmountTooSmall.into());
        }
        Ok(())
    }

    /// Validate array lengths match
    pub fn validate_array_lengths<T, U>(arr1: &[T], arr2: &[U]) -> Result<()> {
        if arr1.len() != arr2.len() {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(())
    }

    /// Validate timestamp is not too old
    pub fn validate_timestamp_freshness(timestamp: i64, max_age_seconds: i64) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;
        if current_time - timestamp > max_age_seconds {
            return Err(StrategyError::PriceFeedUnavailable.into());
        }
        Ok(())
    }

    /// Validate authority matches expected
    pub fn validate_authority(expected: &Pubkey, actual: &Pubkey) -> Result<()> {
        if *expected != *actual {
            return Err(StrategyError::Unauthorized.into());
        }
        Ok(())
    }
}
