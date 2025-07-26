//!
//! config.rs - 策略配置与管理结构体实现
//!
//! 本文件实现所有策略配置与管理结构体及其方法，严格遵循Rust、Anchor、SOLID最佳实践，
//! 并逐行专业注释，便于审计、维护、扩展。

use anchor_lang::prelude::*;
use crate::version::{ProgramVersion, Versioned, CURRENT_VERSION};
use crate::core::types::StrategyParams;
use crate::strategies::types::*;

/// 策略配置结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
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

impl StrategyConfig {
    /// 构造函数
    pub fn new(
        config_id: u64,
        authority: Pubkey,
        weight_config: WeightStrategyConfig,
        rebalancing_config: RebalancingStrategyConfig,
    ) -> Result<Self> {
        Ok(Self {
            version: CURRENT_VERSION,
            config_id,
            authority,
            weight_config,
            rebalancing_config,
            optimization_settings: OptimizationSettings::default(),
            risk_settings: RiskSettings::default(),
            created_at: Clock::get()?.unix_timestamp,
            updated_at: Clock::get()?.unix_timestamp,
        })
    }
    /// 更新权重策略配置
    pub fn update_weight_config(&mut self, new_config: WeightStrategyConfig) -> Result<()> {
        self.weight_config = new_config;
        self.updated_at = Clock::get()?.unix_timestamp;
        Ok(())
    }
    /// 更新再平衡策略配置
    pub fn update_rebalancing_config(&mut self, new_config: RebalancingStrategyConfig) -> Result<()> {
        self.rebalancing_config = new_config;
        self.updated_at = Clock::get()?.unix_timestamp;
        Ok(())
    }
    /// 判断是否需要再平衡
    pub fn needs_rebalancing(&self) -> bool {
        self.rebalancing_config.is_active
    }
    /// 获取性能指标
    pub fn get_performance_metrics(&self) -> StrategyPerformanceMetrics {
        StrategyPerformanceMetrics {
            config_id: self.config_id,
            last_weight_calculation: self.weight_config.last_calculation,
            last_rebalance: self.rebalancing_config.last_rebalance,
            next_rebalance: self.rebalancing_config.next_rebalance,
            uptime: Clock::get().map(|c| c.unix_timestamp - self.created_at).unwrap_or(0),
            is_active: self.weight_config.is_active && self.rebalancing_config.is_active,
        }
    }
}

impl Versioned for StrategyConfig {
    fn version(&self) -> ProgramVersion {
        self.version
    }
    fn set_version(&mut self, version: ProgramVersion) {
        self.version = version;
    }
}

/// 权重策略配置结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct WeightStrategyConfig {
    pub strategy_type: WeightStrategyType,
    pub parameters: Vec<u8>,
    pub token_mints: Vec<Pubkey>,
    pub is_active: bool,
    pub last_calculation: i64,
}

/// 再平衡策略配置结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct RebalancingStrategyConfig {
    pub strategy_type: RebalancingStrategyType,
    pub parameters: Vec<u8>,
    pub is_active: bool,
    pub last_rebalance: i64,
    pub next_rebalance: i64,
}

/// 性能优化设置结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, Default)]
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

/// 风险管理设置结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, Default)]
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

/// 策略性能指标结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, Default)]
pub struct StrategyPerformanceMetrics {
    pub config_id: u64,              // 配置 ID
    pub last_weight_calculation: i64,// 上次权重计算时间
    pub last_rebalance: i64,         // 上次再平衡时间
    pub next_rebalance: i64,         // 下次再平衡时间
    pub uptime: i64,                 // 运行时长
    pub is_active: bool,             // 是否激活
} 