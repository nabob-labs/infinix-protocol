//!
//! safe_math.rs - 安全数学运算函数实现
//!
//! 本文件实现SafeMath结构体及其所有安全数学运算方法，严格遵循Rust、Anchor、SOLID最佳实践，
//! 并逐行专业注释，便于审计、维护、扩展。

use crate::errors::strategy_error::StrategyError;
use anchor_lang::prelude::*;

/// 安全数学运算工具结构体
/// - 提供常用安全数学运算函数实现
pub struct SafeMath;

impl SafeMath {
    /// 安全加法
    pub fn add(a: u64, b: u64) -> anchor_lang::Result<u64> {
        a.checked_add(b).ok_or(StrategyError::MathOverflow)
    }
    /// 安全减法
    pub fn sub(a: u64, b: u64) -> anchor_lang::Result<u64> {
        a.checked_sub(b).ok_or(StrategyError::MathOverflow)
    }
    /// 安全乘法
    pub fn mul(a: u64, b: u64) -> anchor_lang::Result<u64> {
        a.checked_mul(b).ok_or(StrategyError::MathOverflow)
    }
    /// 安全除法
    pub fn div(a: u64, b: u64) -> anchor_lang::Result<u64> {
        if b == 0 {
            return Err(StrategyError::MathOverflow);
        }
        a.checked_div(b).ok_or(StrategyError::MathOverflow)
    }
    /// 精确除法（带小数）
    pub fn div_precise(a: u64, b: u64, precision: u8) -> anchor_lang::Result<u64> {
        if b == 0 {
            return Err(StrategyError::MathOverflow);
        }
        let factor = 10u64.checked_pow(precision as u32).ok_or(StrategyError::MathOverflow)?;
        a.checked_mul(factor).ok_or(StrategyError::MathOverflow)?.checked_div(b).ok_or(StrategyError::MathOverflow)
    }
    /// 百分比基点计算
    pub fn percentage_bps(amount: u64, percentage_bps: u64) -> anchor_lang::Result<u64> {
        amount.checked_mul(percentage_bps).ok_or(StrategyError::MathOverflow)?.checked_div(10_000).ok_or(StrategyError::MathOverflow)
    }
    /// 复利计算
    pub fn compound_interest(principal: u64, rate_bps: u64, periods: u32) -> anchor_lang::Result<u64> {
        let mut amount = principal;
        for _ in 0..periods {
            amount = amount.checked_add(amount.checked_mul(rate_bps).ok_or(StrategyError::MathOverflow)?.checked_div(10_000).ok_or(StrategyError::MathOverflow)?).ok_or(StrategyError::MathOverflow)?;
        }
        Ok(amount)
    }
} 