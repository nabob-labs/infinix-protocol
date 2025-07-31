//!
//! data.rs - 数据校验器实现
//!
//! 本文件实现DataValidator结构体及其所有数据校验方法，严格遵循Rust、Anchor、SOLID最佳实践，
//! 并逐行专业注释，便于审计、维护、扩展。

use anchor_lang::prelude::*;
use crate::errors::strategy_error::StrategyError;

/// 数据校验器结构体
pub struct DataValidator;

impl DataValidator {
    /// 校验两个数组长度是否一致
    pub fn validate_array_lengths_match<T, U>(arr1: &[T], arr2: &[U]) -> anchor_lang::Result<()> {
        if arr1.len() != arr2.len() {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(())
    }

    /// 校验数组非空
    pub fn validate_not_empty<T>(arr: &[T]) -> anchor_lang::Result<()> {
        if arr.is_empty() {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(())
    }

    /// 校验所有数值为正
    pub fn validate_all_positive(values: &[u64]) -> anchor_lang::Result<()> {
        if values.iter().any(|&v| v == 0) {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(())
    }

    /// 校验数值在范围内
    pub fn validate_range(value: u64, min: u64, max: u64) -> anchor_lang::Result<()> {
        if value < min || value > max {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(())
    }

    /// 校验百分比基点
    pub fn validate_percentage(percentage_bps: u64) -> anchor_lang::Result<()> {
        if percentage_bps > 10_000 {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(())
    }

    /// 校验Pubkey非默认值
    pub fn validate_pubkey_not_default(pubkey: &Pubkey) -> anchor_lang::Result<()> {
        if *pubkey == Pubkey::default() {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(())
    }

    /// 校验账户discriminator
    pub fn validate_account_discriminator(
        account_data: &[u8],
        expected_discriminator: &[u8; 8],
    ) -> anchor_lang::Result<()> {
        if account_data.len() < 8 || &account_data[..8] != expected_discriminator {
            return Err(StrategyError::InvalidAccountDiscriminator.into());
        }
        Ok(())
    }
} 