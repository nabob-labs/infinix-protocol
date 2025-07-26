//!
//! types.rs - 高级策略参数与类型定义
//!
//! 本文件定义了MultiFactorParams、AiOptimizationParams、MarketData及其Default实现，严格遵循Rust、Anchor、SOLID最佳实践，
//! 并逐行专业注释，便于审计、维护、扩展。

use anchor_lang::prelude::*;

/// 多因子策略参数结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct MultiFactorParams {
    pub momentum_lookback: u32,      // 动量回溯周期
    pub momentum_threshold: u64,     // 动量阈值
    pub reversion_period: u32,       // 均值回归周期
    pub deviation_threshold: u64,    // 偏离阈值
    pub target_volatility: u64,      // 目标波动率
    pub risk_aversion: u64,          // 风险厌恶度
}

impl Default for MultiFactorParams {
    fn default() -> Self {
        Self {
            momentum_lookback: 30,
            momentum_threshold: 100,
            reversion_period: 30,
            deviation_threshold: 100,
            target_volatility: 200,
            risk_aversion: 1,
        }
    }
}

/// AI优化参数结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct AiOptimizationParams {
    pub price_weight: u64,           // 价格信号权重
    pub volume_weight: u64,          // 成交量信号权重
    pub sentiment_weight: u64,       // 情绪信号权重
    pub confidence_threshold: u64,   // 置信度阈值
    pub max_signal_strength: u64,    // 最大信号强度
}

impl Default for AiOptimizationParams {
    fn default() -> Self {
        Self {
            price_weight: 1,
            volume_weight: 1,
            sentiment_weight: 1,
            confidence_threshold: 100,
            max_signal_strength: 10000,
        }
    }
}

/// 市场数据结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct MarketData {
    pub token_supplies: Vec<u64>,        // 各 token 供应量
    pub historical_prices: Vec<u64>,     // 历史价格
    pub volatilities: Vec<u64>,          // 波动率数据
    pub volumes: Vec<u64>,               // 成交量数据
    pub timestamp: i64,                  // 数据时间戳
}

impl Default for MarketData {
    fn default() -> Self {
        Self {
            token_supplies: Vec::new(),
            historical_prices: Vec::new(),
            volatilities: Vec::new(),
            volumes: Vec::new(),
            timestamp: 0,
        }
    }
} 