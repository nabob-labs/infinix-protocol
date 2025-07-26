//!
//! types.rs - 技术指标类型定义
//!
//! 本文件定义了TechnicalIndicators、VolumeIndicators、TechnicalIndicatorParams及其Default实现，严格遵循Rust、Anchor、SOLID最佳实践，
//! 并逐行专业注释，便于审计、维护、扩展。

use anchor_lang::prelude::*;

/// 技术指标结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, Default)]
pub struct TechnicalIndicators {
    pub rsi: Option<u64>,                 // 相对强弱指标
    pub macd_signal: Option<i64>,         // MACD 信号
    pub bollinger_position: Option<u64>,  // 布林带位置
    pub ma_convergence: Option<i64>,      // 均线收敛
    pub volume_indicators: Option<VolumeIndicators>, // 成交量指标
}

/// 成交量指标结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, Default)]
pub struct VolumeIndicators {
    pub vwap: u64,        // 加权平均价
    pub obv: i64,         // 能量潮指标
    pub volume_roc: i64,  // 成交量变化率
}

/// 技术指标参数结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct TechnicalIndicatorParams {
    pub rsi_weight: u64,       // RSI 权重
    pub macd_weight: u64,      // MACD 权重
    pub bollinger_weight: u64, // 布林带权重
    pub ma_weight: u64,        // 均线权重
    pub rsi_oversold: u64,     // RSI 超卖阈值
    pub rsi_overbought: u64,   // RSI 超买阈值
}

impl Default for TechnicalIndicatorParams {
    fn default() -> Self {
        Self {
            rsi_weight: 1,
            macd_weight: 1,
            bollinger_weight: 1,
            ma_weight: 1,
            rsi_oversold: 30,
            rsi_overbought: 70,
        }
    }
} 