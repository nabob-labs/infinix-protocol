//!
//! types.rs - 策略类型与参数结构体定义
//!
//! 本文件定义了所有策略类型枚举、参数结构体及其实现，严格遵循Rust、Anchor、SOLID最佳实践，
//! 并逐行专业注释，便于审计、维护、扩展。

use anchor_lang::prelude::*;

/// 权重策略类型枚举，定义所有支持的权重分配算法。
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, PartialEq, InitSpace)]
pub enum WeightStrategyType {
    /// 等权重分配
    EqualWeight,
    /// 市值加权分配
    MarketCapWeighted,
    /// 动量加权分配
    MomentumWeighted,
    /// 波动率调整分配
    VolatilityAdjusted,
    /// 固定权重分配
    FixedWeight,
    /// 技术指标动态分配
    TechnicalIndicator,
}

impl From<u8> for WeightStrategyType {
    fn from(value: u8) -> Self {
        match value {
            0 => WeightStrategyType::EqualWeight,
            1 => WeightStrategyType::MarketCapWeighted,
            2 => WeightStrategyType::MomentumWeighted,
            3 => WeightStrategyType::VolatilityAdjusted,
            4 => WeightStrategyType::FixedWeight,
            5 => WeightStrategyType::TechnicalIndicator,
            _ => WeightStrategyType::EqualWeight,
        }
    }
}
impl From<WeightStrategyType> for u8 {
    fn from(strategy_type: WeightStrategyType) -> Self {
        match strategy_type {
            WeightStrategyType::EqualWeight => 0,
            WeightStrategyType::MarketCapWeighted => 1,
            WeightStrategyType::MomentumWeighted => 2,
            WeightStrategyType::VolatilityAdjusted => 3,
            WeightStrategyType::FixedWeight => 4,
            WeightStrategyType::TechnicalIndicator => 5,
        }
    }
}

/// 再平衡策略类型枚举，定义所有支持的再平衡算法。
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, PartialEq, InitSpace)]
pub enum RebalancingStrategyType {
    /// 阈值触发再平衡
    ThresholdBased,
    /// 定时周期再平衡
    TimeBased,
    /// 波动率触发再平衡
    VolatilityTriggered,
    /// 漂移触发再平衡
    DriftBased,
    /// 混合多因子再平衡
    Hybrid,
}

impl From<u8> for RebalancingStrategyType {
    fn from(value: u8) -> Self {
        match value {
            0 => RebalancingStrategyType::ThresholdBased,
            1 => RebalancingStrategyType::TimeBased,
            2 => RebalancingStrategyType::VolatilityTriggered,
            3 => RebalancingStrategyType::DriftBased,
            4 => RebalancingStrategyType::Hybrid,
            _ => RebalancingStrategyType::ThresholdBased,
        }
    }
}
impl From<RebalancingStrategyType> for u8 {
    fn from(strategy_type: RebalancingStrategyType) -> Self {
        match strategy_type {
            RebalancingStrategyType::ThresholdBased => 0,
            RebalancingStrategyType::TimeBased => 1,
            RebalancingStrategyType::VolatilityTriggered => 2,
            RebalancingStrategyType::DriftBased => 3,
            RebalancingStrategyType::Hybrid => 4,
        }
    }
}

/// 等权重策略参数结构体。
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct EqualWeightParams {
    /// 指数包含的资产数量
    pub token_count: u32,
}

/// 市值加权策略参数结构体。
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct MarketCapWeightedParams {
    /// 最小权重（基点）
    pub min_weight: u64,
    /// 最大权重（基点）
    pub max_weight: u64,
    /// 再平衡频率（天）
    pub rebalance_frequency: u32,
}

/// 动量加权策略参数结构体。
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct MomentumWeightedParams {
    /// 动量回溯周期（天）
    pub lookback_period: u32,
    /// 动量因子权重
    pub momentum_factor: u64,
    /// 每个资产的基础权重（基点）
    pub base_weight: u64,
}

/// 波动率调整策略参数结构体。
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct VolatilityAdjustedParams {
    /// 波动率回溯周期（天）
    pub volatility_period: u32,
    /// 风险厌恶参数
    pub risk_aversion: u64,
    /// 目标波动率（基点）
    pub target_volatility: u64,
}

/// 固定权重策略参数结构体。
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct FixedWeightParams {
    /// 每个资产的固定权重（基点，需总和为 10000）
    pub weights: Vec<u64>,
}

/// 阈值再平衡参数结构体。
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct ThresholdRebalanceParams {
    /// 偏离目标权重的阈值（基点）
    pub threshold: u64,
    /// 最小再平衡间隔（秒）
    pub min_interval: u64,
}

/// 定时再平衡参数结构体。
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct TimeBasedRebalanceParams {
    /// 再平衡间隔（秒）
    pub interval: u64,
    /// 是否允许提前再平衡
    pub allow_early_rebalance: bool,
    /// 提前再平衡阈值（基点）
    pub early_threshold: u64,
}

/// 波动率触发再平衡参数结构体。
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct VolatilityTriggeredParams {
    /// 触发再平衡的波动率阈值（基点）
    pub volatility_threshold: u64,
    /// 波动率计算回溯周期（秒）
    pub volatility_period: u64,
    /// 最小再平衡间隔（秒）
    pub min_interval: u64,
}

/// 漂移触发再平衡参数结构体。
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct DriftBasedParams {
    /// 累计漂移阈值（基点）
    pub drift_threshold: u64,
    /// 最小再平衡间隔（秒）
    pub min_interval: u64,
    /// 漂移计算窗口（秒）
    pub drift_window: u64,
}

/// 混合再平衡参数结构体。
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct HybridRebalanceParams {
    /// 启用阈值分量
    pub enable_threshold: bool,
    /// 启用定时分量
    pub enable_time: bool,
    /// 启用波动率分量
    pub enable_volatility: bool,
    /// 启用漂移分量
    pub enable_drift: bool,
    /// 组合策略类型
    pub combination_strategy: HybridCombinationStrategy,
    /// 阈值分量权重
    pub threshold_weight: u32,
    /// 定时分量权重
    pub time_weight: u32,
    /// 波动率分量权重
    pub volatility_weight: u32,
    /// 漂移分量权重
    pub drift_weight: u32,
    /// 组合触发阈值
    pub trigger_threshold: u32,
    /// 阈值策略参数
    pub threshold_params: Vec<u8>,
    /// 定时策略参数
    pub time_params: Vec<u8>,
    /// 波动率策略参数
    pub volatility_params: Vec<u8>,
    /// 漂移策略参数
    pub drift_params: Vec<u8>,
}

/// 混合再平衡组合策略类型枚举。
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, PartialEq, Eq)]
pub enum HybridCombinationStrategy {
    /// 任一分量触发即触发
    Any,
    /// 多数分量触发才触发
    Majority,
    /// 所有分量均触发才触发
    All,
    /// 按权重加权组合信号
    Weighted,
} 