//!
//! Strategies Module
//!
//! 本模块定义所有策略类型、参数结构体、配置结构体、优化与风险设置、版本管理 trait 及相关 trait，确保策略类型、参数、配置的合规性、可扩展性和可维护性。

// 引入版本管理、Anchor 依赖、核心 trait 和类型。
use crate::version::{ProgramVersion, Versioned, CURRENT_VERSION};
use anchor_lang::prelude::*;
use crate::core::adapter::AdapterTrait;
use crate::core::types::StrategyParams;

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

// u8 到 WeightStrategyType 的转换实现，便于序列化和兼容性。
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

// WeightStrategyType 到 u8 的转换实现。
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

// u8 到 RebalancingStrategyType 的转换实现。
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

// RebalancingStrategyType 到 u8 的转换实现。
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
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct EqualWeightParams {
    /// 指数包含的资产数量
    pub token_count: u32,
}

/// 市值加权策略参数结构体。
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct MarketCapWeightedParams {
    /// 最小权重（基点）
    pub min_weight: u64,
    /// 最大权重（基点）
    pub max_weight: u64,
    /// 再平衡频率（天）
    pub rebalance_frequency: u32,
}

/// 动量加权策略参数结构体。
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct MomentumWeightedParams {
    /// 动量回溯周期（天）
    pub lookback_period: u32,
    /// 动量因子权重
    pub momentum_factor: u64,
    /// 每个资产的基础权重（基点）
    pub base_weight: u64,
}

/// 波动率调整策略参数结构体。
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct VolatilityAdjustedParams {
    /// 波动率回溯周期（天）
    pub volatility_period: u32,
    /// 风险厌恶参数
    pub risk_aversion: u64,
    /// 目标波动率（基点）
    pub target_volatility: u64,
}

/// 固定权重策略参数结构体。
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct FixedWeightParams {
    /// 每个资产的固定权重（基点，需总和为 10000）
    pub weights: Vec<u64>,
}

/// 阈值再平衡参数结构体。
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct ThresholdRebalanceParams {
    /// 偏离目标权重的阈值（基点）
    pub threshold: u64,
    /// 最小再平衡间隔（秒）
    pub min_interval: u64,
}

/// 定时再平衡参数结构体。
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct TimeBasedRebalanceParams {
    /// 再平衡间隔（秒）
    pub interval: u64,
    /// 是否允许提前再平衡
    pub allow_early_rebalance: bool,
    /// 提前再平衡阈值（基点）
    pub early_threshold: u64,
}

/// 波动率触发再平衡参数结构体。
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct VolatilityTriggeredParams {
    /// 触发再平衡的波动率阈值（基点）
    pub volatility_threshold: u64,
    /// 波动率计算回溯周期（秒）
    pub volatility_period: u64,
    /// 最小再平衡间隔（秒）
    pub min_interval: u64,
}

/// 漂移再平衡参数结构体。
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct DriftBasedParams {
    /// 累计漂移阈值（基点）
    pub drift_threshold: u64,
    /// 最小再平衡间隔（秒）
    pub min_interval: u64,
    /// 漂移计算窗口（秒）
    pub drift_window: u64,
}

/// 混合再平衡参数结构体。
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
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
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
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

// 为策略类型实现版本管理 trait。
impl Versioned for WeightStrategyType {
    fn version(&self) -> ProgramVersion {
        CURRENT_VERSION
    }
    fn set_version(&mut self, _version: ProgramVersion) {
        // 策略类型不可变，版本始终为当前
    }
}

impl Versioned for RebalancingStrategyType {
    fn version(&self) -> ProgramVersion {
        CURRENT_VERSION
    }
    fn set_version(&mut self, _version: ProgramVersion) {
        // 策略类型不可变，版本始终为当前
    }
}

/// 策略配置结构体，支持版本管理。
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct StrategyConfig {
    /// 策略版本
    pub version: ProgramVersion,
    /// 配置 ID
    pub config_id: u64,
    /// 策略权限
    pub authority: Pubkey,
    /// 权重策略配置
    pub weight_config: WeightStrategyConfig,
    /// 再平衡策略配置
    pub rebalancing_config: RebalancingStrategyConfig,
    /// 性能优化设置
    pub optimization_settings: OptimizationSettings,
    /// 风险管理设置
    pub risk_settings: RiskSettings,
    /// 创建时间戳
    pub created_at: i64,
    /// 最后更新时间戳
    pub updated_at: i64,
}

/// 权重策略配置结构体。
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct WeightStrategyConfig {
    pub strategy_type: WeightStrategyType,
    pub parameters: Vec<u8>,
    pub token_mints: Vec<Pubkey>,
    pub is_active: bool,
    pub last_calculation: i64,
}

/// 再平衡策略配置结构体。
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct RebalancingStrategyConfig {
    pub strategy_type: RebalancingStrategyType,
    pub parameters: Vec<u8>,
    pub is_active: bool,
    pub last_rebalance: i64,
    pub next_rebalance: i64,
}

/// 性能优化设置结构体。
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct OptimizationSettings {
    /// 启用缓存
    pub enable_caching: bool,
    /// 启用并行处理
    pub enable_parallel: bool,
    /// 批处理大小
    pub batch_size: u32,
    /// 缓存过期时间（秒）
    pub cache_expiry: u64,
}

/// 风险管理设置结构体。
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct RiskSettings {
    /// 最大权重集中度（基点）
    pub max_concentration: u64,
    /// 每日最大再平衡次数
    pub max_daily_rebalances: u32,
    /// 启用断路器
    pub enable_circuit_breakers: bool,
    /// 断路器波动率阈值
    pub volatility_circuit_breaker: u64,
}

impl StrategyConfig {
    /// 创建新策略配置。
    pub fn new(
        config_id: u64,
        authority: Pubkey,
        weight_config: WeightStrategyConfig,
        rebalancing_config: RebalancingStrategyConfig,
    ) -> Result<Self> {
        let current_time = Clock::get()?.unix_timestamp;
        Ok(Self {
            version: CURRENT_VERSION,
            config_id,
            authority,
            weight_config,
            rebalancing_config,
            optimization_settings: OptimizationSettings {
                enable_caching: true,
                enable_parallel: false,
                batch_size: 10,
                cache_expiry: 300, // 5 分钟
            },
            risk_settings: RiskSettings {
                max_concentration: 5000, // 50%
                max_daily_rebalances: 4,
                enable_circuit_breakers: true,
                volatility_circuit_breaker: 2000, // 20%
            },
            created_at: current_time,
            updated_at: current_time,
        })
    }
    /// 更新权重策略配置。
    pub fn update_weight_config(&mut self, new_config: WeightStrategyConfig) -> Result<()> {
        self.weight_config = new_config;
        self.updated_at = Clock::get()?.unix_timestamp;
        Ok(())
    }
    /// 更新再平衡策略配置。
    pub fn update_rebalancing_config(
        &mut self,
        new_config: RebalancingStrategyConfig,
    ) -> Result<()> {
        self.rebalancing_config = new_config;
        self.updated_at = Clock::get()?.unix_timestamp;
        Ok(())
    }
    /// 判断是否需要再平衡。
    pub fn needs_rebalancing(&self) -> bool {
        let current_time = Clock::get().unwrap().unix_timestamp;
        self.rebalancing_config.is_active && current_time >= self.rebalancing_config.next_rebalance
    }
    /// 获取策略性能指标。
    pub fn get_performance_metrics(&self) -> StrategyPerformanceMetrics {
        StrategyPerformanceMetrics {
            config_id: self.config_id,
            last_weight_calculation: self.weight_config.last_calculation,
            last_rebalance: self.rebalancing_config.last_rebalance,
            next_rebalance: self.rebalancing_config.next_rebalance,
            uptime: Clock::get().unwrap().unix_timestamp - self.created_at,
            is_active: self.weight_config.is_active && self.rebalancing_config.is_active,
        }
    }
}

// 为策略配置实现版本管理 trait。
impl Versioned for StrategyConfig {
    fn version(&self) -> ProgramVersion {
        self.version
    }
    fn set_version(&mut self, version: ProgramVersion) {
        self.version = version;
        self.updated_at = Clock::get().unwrap().unix_timestamp;
    }
    fn migrate_to_1_0_1(&mut self) -> Result<()> {
        msg!(
            "Migrating StrategyConfig {} to version 1.0.1",
            self.config_id
        );
        // 版本迁移逻辑
        Ok(())
    }
    fn migrate_to_1_1_0(&mut self) -> Result<()> {
        msg!(
            "Migrating StrategyConfig {} to version 1.1.0",
            self.config_id
        );
        // 版本迁移逻辑
        Ok(())
    }
}

/// 策略性能指标结构体。
#[derive(Debug, Clone)]
pub struct StrategyPerformanceMetrics {
    pub config_id: u64,              // 配置 ID
    pub last_weight_calculation: i64,// 上次权重计算时间
    pub last_rebalance: i64,         // 上次再平衡时间
    pub next_rebalance: i64,         // 下次再平衡时间
    pub uptime: i64,                 // 运行时长
    pub is_active: bool,             // 是否激活
}

/// 策略 trait，所有策略需实现。
pub trait Strategy: AdapterTrait {
    fn execute(&self, params: &StrategyParams) -> anchor_lang::Result<StrategyResult>;
}

/// 策略参数结构体（占位，具体定义见实现）。
pub struct StrategyParams {
    // ...参数定义...
}

/// 策略结果结构体（占位，具体定义见实现）。
pub struct StrategyResult {
    // ...结果定义...
}
