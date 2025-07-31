//!
//! statistics.rs - 统计分析函数实现
//!
//! 本文件实现Statistics结构体及其所有统计分析方法，严格遵循Rust、Anchor、SOLID最佳实践，
//! 并逐行专业注释，便于审计、维护、扩展。

use crate::errors::strategy_error::StrategyError;
use anchor_lang::prelude::*;

/// 统计分析工具结构体
/// - 提供常用统计分析函数实现
pub struct Statistics;

impl Statistics {
    /// 计算均值
    pub fn mean(data: &[Decimal]) -> anchor_lang::Result<Decimal> {
        if data.is_empty() {
            return Err(StrategyError::InvalidStrategyParameters);
        }
        let sum: Decimal = data.iter().copied().sum();
        Ok(sum / Decimal::from(data.len() as u64))
    }

    /// 计算方差
    pub fn variance(data: &[Decimal]) -> anchor_lang::Result<Decimal> {
        let mean = Self::mean(data)?;
        let var = data.iter().map(|&x| (x - mean).powi(2)).sum::<Decimal>() / Decimal::from(data.len() as u64);
        Ok(var)
    }

    /// 计算标准差
    pub fn std_dev(data: &[Decimal]) -> anchor_lang::Result<Decimal> {
        let var = Self::variance(data)?;
        Ok(var.sqrt())
    }

    /// 计算偏度
    pub fn skewness(data: &[Decimal]) -> anchor_lang::Result<Decimal> {
        let mean = Self::mean(data)?;
        let std = Self::std_dev(data)?;
        let n = data.len() as f64;
        if std == Decimal::ZERO {
            return Ok(Decimal::ZERO);
        }
        let skew = data.iter().map(|&x| ((x - mean) / std).powi(3)).sum::<Decimal>() / Decimal::from(n);
        Ok(skew)
    }

    /// 计算峰度
    pub fn kurtosis(data: &[Decimal]) -> anchor_lang::Result<Decimal> {
        let mean = Self::mean(data)?;
        let std = Self::std_dev(data)?;
        let n = data.len() as f64;
        if std == Decimal::ZERO {
            return Ok(Decimal::ZERO);
        }
        let kurt = data.iter().map(|&x| ((x - mean) / std).powi(4)).sum::<Decimal>() / Decimal::from(n);
        Ok(kurt)
    }

    /// 计算相关系数
    pub fn correlation(x: &[Decimal], y: &[Decimal]) -> anchor_lang::Result<Decimal> {
        if x.len() != y.len() || x.is_empty() {
            return Err(StrategyError::InvalidStrategyParameters);
        }
        let mean_x = Self::mean(x)?;
        let mean_y = Self::mean(y)?;
        let std_x = Self::std_dev(x)?;
        let std_y = Self::std_dev(y)?;
        if std_x == Decimal::ZERO || std_y == Decimal::ZERO {
            return Ok(Decimal::ZERO);
        }
        let cov = x.iter().zip(y.iter()).map(|(&xi, &yi)| (xi - mean_x) * (yi - mean_y)).sum::<Decimal>() / Decimal::from(x.len() as u64);
        Ok(cov / (std_x * std_y))
    }

    /// 历史模拟法VaR
    pub fn var_historical(returns: &[Decimal], confidence_level: Decimal) -> anchor_lang::Result<Decimal> {
        if returns.is_empty() {
            return Err(StrategyError::InvalidStrategyParameters);
        }
        let mut sorted = returns.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let idx = ((1.0 - confidence_level.to_f64().unwrap_or(0.95)) * sorted.len() as f64).ceil() as usize;
        Ok(sorted.get(idx).copied().unwrap_or(Decimal::ZERO))
    }

    /// 历史模拟法CVaR
    pub fn cvar_historical(returns: &[Decimal], confidence_level: Decimal) -> anchor_lang::Result<Decimal> {
        if returns.is_empty() {
            return Err(StrategyError::InvalidStrategyParameters);
        }
        let mut sorted = returns.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let idx = ((1.0 - confidence_level.to_f64().unwrap_or(0.95)) * sorted.len() as f64).ceil() as usize;
        let tail = &sorted[..idx];
        if tail.is_empty() {
            return Ok(Decimal::ZERO);
        }
        Ok(tail.iter().copied().sum::<Decimal>() / Decimal::from(tail.len() as u64))
    }

    /// 计算Sharpe比率
    pub fn sharpe_ratio(returns: &[Decimal], risk_free_rate: Decimal) -> anchor_lang::Result<Decimal> {
        let mean = Self::mean(returns)?;
        let std = Self::std_dev(returns)?;
        if std == Decimal::ZERO {
            return Ok(Decimal::ZERO);
        }
        Ok((mean - risk_free_rate) / std)
    }

    /// 计算最大回撤
    pub fn max_drawdown(prices: &[Decimal]) -> anchor_lang::Result<Decimal> {
        if prices.is_empty() {
            return Err(StrategyError::InvalidStrategyParameters);
        }
        let mut max_dd = Decimal::ZERO;
        let mut peak = prices[0];
        for &price in prices {
            if price > peak {
                peak = price;
            }
            let dd = (peak - price) / peak;
            if dd > max_dd {
                max_dd = dd;
            }
        }
        Ok(max_dd)
    }
} 