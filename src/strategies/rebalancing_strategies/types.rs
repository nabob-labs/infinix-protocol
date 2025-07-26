//!
//! types.rs - 再平衡相关类型定义
//!
//! 本文件定义了再平衡相关类型，严格遵循Rust、Anchor、SOLID最佳实践，
//! 并逐行专业注释，便于审计、维护、扩展。

use anchor_lang::prelude::*;

/// 再平衡操作类型枚举
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, PartialEq, Eq)]
pub enum RebalancingActionType {
    Buy,  // 买入
    Sell, // 卖出
}

/// 再平衡操作结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct RebalancingAction {
    pub token_index: usize,           // 资产索引
    pub action_type: RebalancingActionType, // 操作类型
    pub amount: u64,                  // 操作金额
    pub priority: u64,                // 优先级
}

/// 权重漂移结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct WeightDrift {
    pub token_index: usize,           // 资产索引
    pub magnitude: u64,               // 漂移幅度
    pub direction: DriftDirection,    // 漂移方向
    pub timestamp: i64,               // 时间戳
}

/// 漂移方向枚举
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, PartialEq, Eq)]
pub enum DriftDirection {
    Positive, // 权重增加
    Negative, // 权重减少
}

/// 混合再平衡参数结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct HybridRebalancingParams {
    pub enable_threshold: bool,       // 启用阈值分量
    pub enable_time: bool,            // 启用定时分量
    pub enable_volatility: bool,      // 启用波动率分量
    pub threshold_bps: u64,           // 阈值（基点）
    pub time_interval: u64,           // 定时间隔（秒）
    pub volatility_threshold: u64,    // 波动率阈值（基点）
    pub threshold_weight: u64,        // 阈值分量权重
    pub time_weight: u64,             // 定时分量权重
    pub volatility_weight: u64,       // 波动率分量权重
}

impl Default for HybridRebalancingParams {
    fn default() -> Self {
        Self {
            enable_threshold: true,
            enable_time: false,
            enable_volatility: false,
            threshold_bps: 100,
            time_interval: 86400,
            volatility_threshold: 200,
            threshold_weight: 1,
            time_weight: 1,
            volatility_weight: 1,
        }
    }
}

/// 市场上下文结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct MarketContext {
    pub last_rebalance: i64,              // 上次再平衡时间
    pub volatility_data: Vec<u64>,        // 波动率数据
    pub market_trend: MarketTrend,        // 市场趋势
    pub liquidity_conditions: LiquidityCondition, // 流动性状况
}

/// 市场趋势枚举
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, PartialEq, Eq)]
pub enum MarketTrend {
    Bullish,   // 多头
    Bearish,   // 空头
    Sideways,  // 震荡
}

/// 流动性状况枚举
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, PartialEq, Eq)]
pub enum LiquidityCondition {
    High,   // 高流动性
    Medium, // 中等流动性
    Low,    // 低流动性
} 