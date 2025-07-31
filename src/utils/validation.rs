//! Validation模块 - 验证工具
//! 
//! 本模块提供验证功能，包含：
//! - 数据验证
//! - 格式验证
//! - 业务规则验证
//! - 安全验证
//! 
//! 设计理念：
//! - 完整性：确保验证的完整性
//! - 性能：使用高效的验证算法
//! - 安全性：防止恶意输入
//! - 可扩展：支持自定义验证规则
//! - 设计意图：极致安全、高性能、可扩展

use anchor_lang::prelude::*;             // Anchor 预导入，包含Pubkey、Result等
// use crate::core::constants; // 暂时注释掉
// use crate::errors::strategy_error::StrategyError;

/// 校验工具结构体，提供各类输入与状态的合规性校验。
pub struct ValidationUtils;

impl ValidationUtils {
    /// 校验 token 数量在合法范围内。
    pub fn validate_token_count(count: usize) -> anchor_lang::Result<()> {
        if count == 0 {
            return Err(StrategyError::InvalidTokenCount.into());
        }
        if count > MAX_TOKENS {
            return Err(StrategyError::InvalidTokenCount.into());
        }
        Ok(())
    }
    /// 校验权重数组总和为 BASIS_POINTS_MAX，且单个权重不超限。
    pub fn validate_weights(weights: &[u64]) -> anchor_lang::Result<()> {
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
    pub fn validate_parameters_size(params: &[u8], max_size: usize) -> anchor_lang::Result<()> {
        if params.len() > max_size {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(())
    }
    /// 校验 pubkey 数组无重复。
    pub fn validate_no_duplicates(pubkeys: &[Pubkey]) -> anchor_lang::Result<()> {
        let mut seen = std::collections::HashSet::new();
        for pubkey in pubkeys {
            if !seen.insert(*pubkey) {
                return Err(StrategyError::InvalidStrategyParameters.into());
            }
        }
        Ok(())
    }
    /// 校验 pubkey 非默认值。
    pub fn validate_pubkey(pubkey: &Pubkey, field_name: &str) -> anchor_lang::Result<()> {
        if *pubkey == Pubkey::default() {
            msg!("Invalid pubkey for field: {}", field_name);
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(())
    }
    /// 校验时间区间在合法范围内。
    pub fn validate_time_interval(interval: u64) -> anchor_lang::Result<()> {
        if interval < MIN_REBALANCE_INTERVAL {
            return Err(StrategyError::InvalidTimeWindow.into());
        }
        if interval > MAX_REBALANCE_INTERVAL {
            return Err(StrategyError::InvalidTimeWindow.into());
        }
        Ok(())
    }
    /// 校验滑点容忍度不超限。
    pub fn validate_slippage(slippage: u64) -> anchor_lang::Result<()> {
        if slippage > MAX_SLIPPAGE_BPS {
            return Err(StrategyError::SlippageExceeded.into());
        }
        Ok(())
    }
    /// 校验费用在合法范围内。
    pub fn validate_fee(fee_bps: u64) -> anchor_lang::Result<()> {
        if fee_bps > MAX_FEE_BPS as u64 {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(())
    }
    /// 校验金额大于最小值。
    pub fn validate_minimum_amount(amount: u64, minimum: u64) -> anchor_lang::Result<()> {
        if amount < minimum {
            return Err(StrategyError::BasketAmountTooSmall.into());
        }
        Ok(())
    }
    /// 校验两个数组长度一致。
    pub fn validate_array_lengths<T, U>(arr1: &[T], arr2: &[U]) -> anchor_lang::Result<()> {
        if arr1.len() != arr2.len() {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(())
    }
    /// 校验时间戳未过期。
    pub fn validate_timestamp_freshness(timestamp: i64, max_age_seconds: i64) -> anchor_lang::Result<()> {
        let current_time = Clock::get()?.unix_timestamp;
        if current_time - timestamp > max_age_seconds {
            return Err(StrategyError::PriceFeedUnavailable.into());
        }
        Ok(())
    }
    /// 校验权限 pubkey 是否匹配。
    pub fn validate_authority(expected: &Pubkey, actual: &Pubkey) -> anchor_lang::Result<()> {
        if *expected != *actual {
            return Err(StrategyError::Unauthorized.into());
        }
        Ok(())
    }
}
