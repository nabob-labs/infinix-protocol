//!
//! Validation Utilities Module
//!
//! 本模块实现输入参数与状态的全量合规性校验，涵盖 token 数量、权重、参数、去重、时间、滑点、费用、权限等，确保所有输入与状态均满足业务和安全要求。

// 引入核心模块、常量、错误类型和 Anchor 依赖。
use crate::core::*;
use crate::core::constants::BASIS_POINTS_MAX;
use crate::error::StrategyError;
use anchor_lang::prelude::*;

/// 校验工具结构体，提供各类输入与状态的合规性校验。
pub struct ValidationUtils;

impl ValidationUtils {
    /// 校验 token 数量在合法范围内。
    pub fn validate_token_count(count: usize) -> Result<()> {
        if count == 0 {
            return Err(StrategyError::InvalidTokenCount.into());
        }
        if count > MAX_TOKENS {
            return Err(StrategyError::InvalidTokenCount.into());
        }
        Ok(())
    }
    /// 校验权重数组总和为 BASIS_POINTS_MAX，且单个权重不超限。
    pub fn validate_weights(weights: &[u64]) -> Result<()> {
        if weights.is_empty() {
            return Err(StrategyError::InvalidTokenCount.into());
        }
        let total: u64 = weights.iter().sum();
        if total != BASIS_POINTS_MAX {
            return Err(StrategyError::InvalidWeightSum.into());
        }
        // 校验单个权重不超限。
        for &weight in weights {
            if weight > MAX_TOKEN_WEIGHT_BPS {
                return Err(StrategyError::InvalidStrategyParameters.into());
            }
        }
        Ok(())
    }
    /// 校验参数字节数组长度不超限。
    pub fn validate_parameters_size(params: &[u8], max_size: usize) -> Result<()> {
        if params.len() > max_size {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(())
    }
    /// 校验 pubkey 数组无重复。
    pub fn validate_no_duplicates(pubkeys: &[Pubkey]) -> Result<()> {
        let mut seen = std::collections::HashSet::new();
        for pubkey in pubkeys {
            if !seen.insert(*pubkey) {
                return Err(StrategyError::InvalidStrategyParameters.into());
            }
        }
        Ok(())
    }
    /// 校验 pubkey 非默认值。
    pub fn validate_pubkey(pubkey: &Pubkey, field_name: &str) -> Result<()> {
        if *pubkey == Pubkey::default() {
            msg!("Invalid pubkey for field: {}", field_name);
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(())
    }
    /// 校验时间区间在合法范围内。
    pub fn validate_time_interval(interval: u64) -> Result<()> {
        if interval < MIN_REBALANCE_INTERVAL {
            return Err(StrategyError::InvalidTimeWindow.into());
        }
        if interval > MAX_REBALANCE_INTERVAL {
            return Err(StrategyError::InvalidTimeWindow.into());
        }
        Ok(())
    }
    /// 校验滑点容忍度不超限。
    pub fn validate_slippage(slippage: u64) -> Result<()> {
        if slippage > MAX_SLIPPAGE_BPS {
            return Err(StrategyError::SlippageExceeded.into());
        }
        Ok(())
    }
    /// 校验费用在合法范围内。
    pub fn validate_fee(fee_bps: u64) -> Result<()> {
        if fee_bps > MAX_FEE_BPS as u64 {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(())
    }
    /// 校验金额大于最小值。
    pub fn validate_minimum_amount(amount: u64, minimum: u64) -> Result<()> {
        if amount < minimum {
            return Err(StrategyError::BasketAmountTooSmall.into());
        }
        Ok(())
    }
    /// 校验两个数组长度一致。
    pub fn validate_array_lengths<T, U>(arr1: &[T], arr2: &[U]) -> Result<()> {
        if arr1.len() != arr2.len() {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(())
    }
    /// 校验时间戳未过期。
    pub fn validate_timestamp_freshness(timestamp: i64, max_age_seconds: i64) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;
        if current_time - timestamp > max_age_seconds {
            return Err(StrategyError::PriceFeedUnavailable.into());
        }
        Ok(())
    }
    /// 校验权限 pubkey 是否匹配。
    pub fn validate_authority(expected: &Pubkey, actual: &Pubkey) -> Result<()> {
        if *expected != *actual {
            return Err(StrategyError::Unauthorized.into());
        }
        Ok(())
    }
}
