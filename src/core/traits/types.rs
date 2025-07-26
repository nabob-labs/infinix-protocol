//!
//! types.rs - 核心业务结构体与枚举定义
//!
//! 本文件定义了系统核心业务结构体与枚举，严格遵循Rust、Anchor、SOLID最佳实践，
//! 并逐行专业注释，便于审计、维护、扩展。

use anchor_lang::prelude::*;
use std::collections::HashMap;

/// 性能指标结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, Default)]
pub struct Metrics {
    /// 执行次数
    pub execution_count: u64,
    /// 错误次数
    pub error_count: u64,
    /// 平均执行耗时（毫秒）
    pub avg_execution_time_ms: u64,
    /// 最后更新时间戳
    pub last_updated: i64,
}

/// 限流配置结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, Default)]
pub struct RateLimitConfig {
    /// 最大操作次数
    pub max_operations: u32,
    /// 时间窗口（秒）
    pub time_window_seconds: u64,
    /// 突发允许次数
    pub burst_allowance: u32,
}

/// 熔断器配置结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, Default)]
pub struct CircuitBreakerConfig {
    /// 失败阈值
    pub failure_threshold: u32,
    /// 恢复超时时间（秒）
    pub recovery_timeout_seconds: u64,
}

/// 初始化参数结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct InitializationParams {
    /// 权限公钥
    pub authority: Pubkey,
    /// bump 值
    pub bump: u8,
}

/// 健康状态枚举
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
    Offline,
}

/// 版本结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, PartialEq, Eq)]
pub struct Version {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

impl Version {
    /// 构造函数
    pub fn new(major: u16, minor: u16, patch: u16) -> Self {
        Self { major, minor, patch }
    }
    /// 判断是否为破坏性变更
    pub fn is_breaking_change(&self, other: &Version) -> bool {
        self.major != other.major
    }
}

/// 费用配置结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, Default)]
pub struct FeeConfig {
    pub management_fee_bps: u16,
    pub performance_fee_bps: u16,
    pub creation_fee_bps: u16,
    pub redemption_fee_bps: u16,
    pub fee_collector: Pubkey,
}

/// 策略性能指标结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, Default)]
pub struct StrategyPerformanceMetrics {
    pub total_executions: u64,
    pub successful_executions: u64,
    pub average_execution_time_ms: u64,
    pub total_value_processed: u64,
    pub total_fees_collected: u64,
    pub success_rate_bps: u64,
    pub average_slippage_bps: u64,
    pub risk_adjusted_return_bps: i64,
}

/// 市场数据类型枚举
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, PartialEq, Eq)]
pub enum MarketDataType {
    Price,
    Volume,
    Liquidity,
    Volatility,
    TechnicalIndicators,
    OrderBook,
    Trades,
}

/// DEX买卖方向枚举
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, PartialEq, Eq)]
pub enum DexSide {
    Buy,
    Sell,
}

/// DEX交易结果结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, Default)]
pub struct DexTradeResult {
    pub success: bool,
    pub executed_amount: u64,
    pub avg_price: u64,
    pub fee_paid: u64,
}

/// 资源用量结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, Default)]
pub struct ResourceUsage {
    pub memory_bytes: u64,
    pub compute_units: u32,
    pub storage_bytes: u64,
    pub network_bandwidth: u64,
}

/// 资源需求结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, Default)]
pub struct ResourceRequirement {
    pub memory_bytes: u64,
    pub compute_units: u32,
    pub storage_bytes: u64,
    pub network_bandwidth: u64,
    pub duration_seconds: u64,
}

/// 预定ID类型
pub type ReservationId = u64;

/// 熔断器状态枚举
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, PartialEq, Eq)]
pub enum CircuitState {
    Closed,   // 正常
    Open,     // 熔断
    HalfOpen, // 测试恢复
}

/// 健康检查结果结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, Default)]
pub struct HealthCheckResult {
    pub status: HealthStatus,
    pub message: String,
    pub timestamp: i64,
    pub metrics: HashMap<String, f64>,
}

/// 健康指标结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, Default)]
pub struct HealthMetric {
    pub value: f64,
    pub threshold: f64,
    pub status: HealthStatus,
    pub description: String,
} 