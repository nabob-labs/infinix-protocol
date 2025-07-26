//!
//! optimization.rs - 数值优化函数实现
//!
//! 本文件实现Optimization结构体及其所有数值优化方法，严格遵循Rust、Anchor、SOLID最佳实践，
//! 并逐行专业注释，便于审计、维护、扩展。

use crate::error::StrategyError;
use anchor_lang::prelude::*;

/// 数值优化工具结构体
/// - 提供常用数值优化函数实现
pub struct Optimization;

impl Optimization {
    /// 黄金分割法一维极值搜索
    pub fn golden_section_search<F>(f: F, a: Decimal, b: Decimal, tolerance: Decimal) -> Result<Decimal>
    where
        F: Fn(Decimal) -> Result<Decimal>,
    {
        let phi = (1.0 + 5.0_f64.sqrt()) / 2.0;
        let mut a = a;
        let mut b = b;
        let mut c = b - (b - a) / Decimal::from(phi);
        let mut d = a + (b - a) / Decimal::from(phi);
        while (b - a).abs() > tolerance {
            let fc = f(c)?;
            let fd = f(d)?;
            if fc < fd {
                b = d;
            } else {
                a = c;
            }
            c = b - (b - a) / Decimal::from(phi);
            d = a + (b - a) / Decimal::from(phi);
        }
        Ok((b + a) / Decimal::from(2u64))
    }

    /// 牛顿法一维极值搜索
    pub fn newton_method<F, G>(f: F, df: G, initial_guess: Decimal, tolerance: Decimal, max_iterations: u32) -> Result<Decimal>
    where
        F: Fn(Decimal) -> Result<Decimal>,
        G: Fn(Decimal) -> Result<Decimal>,
    {
        let mut x = initial_guess;
        for _ in 0..max_iterations {
            let fx = f(x)?;
            let dfx = df(x)?;
            if dfx == Decimal::ZERO {
                return Err(StrategyError::MathOverflow);
            }
            let x_new = x - fx / dfx;
            if (x_new - x).abs() < tolerance {
                return Ok(x_new);
            }
            x = x_new;
        }
        Err(StrategyError::NotConverged)
    }

    /// 二分法一维极值搜索
    pub fn bisection_method<F>(f: F, a: Decimal, b: Decimal, tolerance: Decimal) -> Result<Decimal>
    where
        F: Fn(Decimal) -> Result<Decimal>,
    {
        let mut a = a;
        let mut b = b;
        let mut fa = f(a)?;
        let mut fb = f(b)?;
        if fa * fb > Decimal::ZERO {
            return Err(StrategyError::InvalidStrategyParameters);
        }
        while (b - a).abs() > tolerance {
            let c = (a + b) / Decimal::from(2u64);
            let fc = f(c)?;
            if fc == Decimal::ZERO {
                return Ok(c);
            } else if fa * fc < Decimal::ZERO {
                b = c;
                fb = fc;
            } else {
                a = c;
                fa = fc;
            }
        }
        Ok((a + b) / Decimal::from(2u64))
    }
} 