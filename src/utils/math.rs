//!
//! Mathematical Utilities Module
//!
//! 本模块实现核心数学运算，所有操作均带有溢出/下溢/除零等安全检查，适用于金融与策略场景下的高安全性数值处理。

// 引入核心模块、错误类型和 Anchor 依赖。
use crate::core::*;
use crate::error::StrategyError;
use anchor_lang::prelude::*;

/// 数学运算工具结构体，所有方法均带安全检查。
pub struct MathOps;

impl MathOps {
    /// 安全乘法，带溢出检查。
    pub fn mul(a: u64, b: u64) -> Result<u64> {
        a.checked_mul(b).ok_or(StrategyError::MathOverflow.into())
    }
    /// 安全除法，带除零检查。
    pub fn div(a: u64, b: u64) -> Result<u64> {
        if b == 0 {
            return Err(StrategyError::DivisionByZero.into());
        }
        Ok(a / b)
    }
    /// 安全加法，带溢出检查。
    pub fn add(a: u64, b: u64) -> Result<u64> {
        a.checked_add(b).ok_or(StrategyError::MathOverflow.into())
    }
    /// 安全减法，带下溢检查。
    pub fn sub(a: u64, b: u64) -> Result<u64> {
        a.checked_sub(b).ok_or(StrategyError::MathOverflow.into())
    }
    /// 归一化权重数组，使其和为 BASIS_POINTS_MAX。
    pub fn normalize_weights(weights: &mut [u64]) -> Result<()> {
        if weights.is_empty() {
            return Ok(());
        }
        let total: u64 = weights.iter().sum();
        if total == 0 {
            let equal_weight = BASIS_POINTS_MAX / weights.len() as u64;
            weights.fill(equal_weight);
            // 处理余数，补到第一个权重。
            let remainder = BASIS_POINTS_MAX % weights.len() as u64;
            if remainder > 0 && !weights.is_empty() {
                weights[0] += remainder;
            }
            return Ok(());
        }
        // 归一化到 BASIS_POINTS_MAX。
        for weight in weights.iter_mut() {
            *weight = Self::div(Self::mul(*weight, BASIS_POINTS_MAX)?, total)?;
        }
        // 处理舍入误差。
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
    /// 计算 value 的百分比（基点）。
    pub fn percentage(value: u64, percentage_bps: u64) -> Result<u64> {
        Self::div(Self::mul(value, percentage_bps)?, BASIS_POINTS_MAX)
    }
    /// 计算加权平均。
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
    /// 牛顿法计算整数平方根。
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
    /// 幂运算，重复乘法实现。
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
