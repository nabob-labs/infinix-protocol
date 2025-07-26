//!
//! functions.rs - 通用校验函数实现
//!
//! 本文件实现通用校验函数，严格遵循Rust、Anchor、SOLID最佳实践，
//! 并逐行专业注释，便于审计、维护、扩展。

use anchor_lang::prelude::*;
use crate::error::StrategyError;

/// 校验数值在指定范围内
pub fn validate_amount(amount: u64, min: u64, max: u64) -> Result<()> {
    if amount < min || amount > max {
        return Err(StrategyError::InvalidStrategyParameters.into());
    }
    Ok(())
}

/// 校验Pubkey非默认值
pub fn validate_pubkey(key: &Pubkey) -> Result<()> {
    if *key == Pubkey::default() {
        return Err(StrategyError::InvalidStrategyParameters.into());
    }
    Ok(())
}

/// 校验权重和为指定值
pub fn validate_weights(weights: &[u64], expected_sum: u64) -> Result<()> {
    let total: u64 = weights.iter().sum();
    if total != expected_sum {
        return Err(StrategyError::InvalidWeightSum.into());
    }
    Ok(())
} 