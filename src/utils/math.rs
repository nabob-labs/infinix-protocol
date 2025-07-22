/*!
 * Mathematical Utilities Module
 *
 * Core mathematical operations with safety checks and overflow protection.
 */

use crate::core::*;
use crate::error::StrategyError;
use anchor_lang::prelude::*;

/// Mathematical operations with safety checks
pub struct MathOps;

impl MathOps {
    /// Safe multiplication with overflow check
    pub fn mul(a: u64, b: u64) -> Result<u64> {
        a.checked_mul(b).ok_or(StrategyError::MathOverflow.into())
    }

    /// Safe division with zero check
    pub fn div(a: u64, b: u64) -> Result<u64> {
        if b == 0 {
            return Err(StrategyError::DivisionByZero.into());
        }
        Ok(a / b)
    }

    /// Safe addition with overflow check
    pub fn add(a: u64, b: u64) -> Result<u64> {
        a.checked_add(b).ok_or(StrategyError::MathOverflow.into())
    }

    /// Safe subtraction with underflow check
    pub fn sub(a: u64, b: u64) -> Result<u64> {
        a.checked_sub(b).ok_or(StrategyError::MathOverflow.into())
    }

    /// Normalize weights to sum to BASIS_POINTS_MAX
    pub fn normalize_weights(weights: &mut [u64]) -> Result<()> {
        if weights.is_empty() {
            return Ok(());
        }

        let total: u64 = weights.iter().sum();
        if total == 0 {
            let equal_weight = BASIS_POINTS_MAX / weights.len() as u64;
            weights.fill(equal_weight);

            // Handle remainder
            let remainder = BASIS_POINTS_MAX % weights.len() as u64;
            if remainder > 0 && !weights.is_empty() {
                weights[0] += remainder;
            }
            return Ok(());
        }

        // Normalize to BASIS_POINTS_MAX
        for weight in weights.iter_mut() {
            *weight = Self::div(Self::mul(*weight, BASIS_POINTS_MAX)?, total)?;
        }

        // Adjust for rounding errors
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
                weights[0] = Self::add(weights[0], diff)?;
            }
        }

        Ok(())
    }

    /// Calculate percentage of a value
    pub fn percentage(value: u64, percentage_bps: u64) -> Result<u64> {
        Self::div(Self::mul(value, percentage_bps)?, BASIS_POINTS_MAX)
    }

    /// Calculate weighted average
    pub fn weighted_average(values: &[u64], weights: &[u64]) -> Result<u64> {
        if values.len() != weights.len() || values.is_empty() {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        let mut weighted_sum = 0u64;
        let mut total_weight = 0u64;

        for (value, weight) in values.iter().zip(weights.iter()) {
            weighted_sum = Self::add(weighted_sum, Self::mul(*value, *weight)?)?;
            total_weight = Self::add(total_weight, *weight)?;
        }

        if total_weight == 0 {
            return Ok(0);
        }

        Self::div(weighted_sum, total_weight)
    }

    /// Calculate square root using Newton's method
    pub fn sqrt(n: u64) -> u64 {
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

    /// Calculate power using repeated multiplication
    pub fn pow(base: u64, exp: u32) -> Result<u64> {
        if exp == 0 {
            return Ok(1);
        }

        let mut result = 1u64;
        let mut base = base;
        let mut exp = exp;

        while exp > 0 {
            if exp % 2 == 1 {
                result = Self::mul(result, base)?;
            }
            base = Self::mul(base, base)?;
            exp /= 2;
        }

        Ok(result)
    }
}
