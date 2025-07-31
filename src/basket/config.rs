//!
//! config.rs - 篮子配置相关类型定义
//!
//! 本文件定义所有与篮子相关的配置结构体，严格遵循Rust、Solana、Anchor、SOLID最佳实践，
//! 并逐行专业注释，便于审计、维护、扩展。

use anchor_lang::prelude::*;
use crate::algorithms::execution_optimizer::types::ExecutionMethod;

/// 优化配置
/// - 控制篮子操作的优化算法、并行度等
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct OptimizationConfig {
    /// 启用优化特性
    pub enabled: bool,
    /// 是否使用遗传算法
    pub use_genetic_algorithm: bool,
    /// 最大优化迭代次数
    pub max_iterations: u32,
    /// 收敛阈值
    pub convergence_threshold: u64,
    /// 启用并行处理
    pub enable_parallel: bool,
}

/// 套利配置
/// - 控制套利操作的最小利润、最大仓位、超时等
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct ArbitrageConfig {
    /// 最小利润（基点）
    pub min_profit_bps: u16,
    /// 最大仓位
    pub max_position_size: u64,
    /// 执行超时时间（秒）
    pub execution_timeout: u32,
    /// 启用跨协议套利
    pub enable_cross_protocol: bool,
}

/// 再平衡配置
/// - 控制再平衡操作的执行方式、风险限制等
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct RebalancingConfig {
    /// 再平衡执行方法
    pub execution_method: crate::algorithms::execution_optimizer::types::ExecutionMethod,
    /// 风险限制
    pub risk_limits: crate::core::types::RiskLimits,
    /// 启用渐进再平衡
    pub enable_gradual: bool,
    /// 再平衡频率限制
    pub frequency_limit: u32,
}

/// 风险监控配置
#[derive(Debug, Clone)]
pub struct RiskMonitoringConfig {
    /// 启用实时监控
    pub enable_realtime: bool,
    /// 监控频率（秒）
    pub frequency: u32,
    /// 告警阈值
    pub alert_thresholds: Vec<AlertThreshold>,
    /// 启用自动风险缓解
    pub enable_auto_mitigation: bool,
}

/// 告警阈值
#[derive(Debug, Clone)]
pub struct AlertThreshold {
    /// 指标名称
    pub metric: String,
    /// 阈值
    pub threshold: u64,
    /// 告警级别
    pub severity: AlertSeverity,
    /// 采取的动作
    pub action: String,
}

/// 告警级别
#[derive(Debug, Clone)]
pub enum AlertSeverity {
    /// 信息
    Info,
    /// 警告
    Warning,
    /// 错误
    Error,
    /// 严重
    Critical,
} 