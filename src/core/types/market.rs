//!
//! market.rs - 市场数据类型定义
//!
//! 本文件定义了MarketData结构体及其实现，严格遵循Rust、Anchor、SOLID最佳实践，
//! 并逐行专业注释，便于审计、维护、扩展。

use anchor_lang::prelude::*;

/// 市场数据结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub struct MarketData {
    /// 当前价格（最小单位）
    pub price: u64,
    /// 24小时交易量（最小单位）
    pub volume_24h: u64,
    /// 市值（最小单位）
    pub market_cap: u64,
    /// 可用流动性（最小单位）
    pub liquidity: u64,
    /// 数据采集时间戳
    pub timestamp: i64,
}

impl Default for MarketData {
    fn default() -> Self {
        Self {
            price: 0,
            volume_24h: 0,
            market_cap: 0,
            liquidity: 0,
            timestamp: 0,
        }
    }
}

impl MarketData {
    /// 构造函数
    pub fn new(
        price: u64,
        volume_24h: u64,
        market_cap: u64,
        liquidity: u64,
        timestamp: i64,
    ) -> Result<Self> {
        Ok(Self {
            price,
            volume_24h,
            market_cap,
            liquidity,
            timestamp,
        })
    }
    /// 判断数据是否过期
    pub fn is_stale(&self, current_time: i64) -> bool {
        current_time - self.timestamp > 3600
    }
    /// 计算历史波动率
    pub fn calculate_volatility(&self, historical_prices: &[u64]) -> u64 {
        if historical_prices.is_empty() { return 0; }
        let mean = historical_prices.iter().sum::<u64>() as f64 / historical_prices.len() as f64;
        let var = historical_prices.iter().map(|&p| (p as f64 - mean).powi(2)).sum::<f64>() / historical_prices.len() as f64;
        var.sqrt() as u64
    }
    /// 估算买卖价差
    pub fn estimate_spread(&self) -> u64 {
        self.price / 1000
    }
} 