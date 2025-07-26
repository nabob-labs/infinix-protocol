//!
//! metrics.rs - 篮子性能与风险度量类型定义
//!
//! 本文件定义所有与篮子相关的性能与风险度量结构体，严格遵循Rust、Solana、Anchor、SOLID最佳实践，
//! 并逐行专业注释，便于审计、维护、扩展。

/// 风险评估结构体
/// - 记录各类风险分数与建议
#[derive(Debug, Clone)]
pub struct RiskAssessment {
    /// 总体风险分数（0-10000）
    pub overall_risk_score: u32,
    /// 流动性风险
    pub liquidity_risk: u32,
    /// 集中度风险
    pub concentration_risk: u32,
    /// 执行风险
    pub execution_risk: u32,
    /// 市场风险
    pub market_risk: u32,
    /// 风险建议
    pub recommendations: Vec<String>,
}

/// 性能指标
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    /// 创建总数
    pub total_creations: u64,
    /// 赎回总数
    pub total_redemptions: u64,
    /// 再平衡总数
    pub total_rebalances: u64,
    /// 套利总数
    pub total_arbitrages: u64,
    /// 总处理量
    pub total_volume: u64,
    /// 总利润
    pub total_profit: u64,
    /// 平均执行时间
    pub average_execution_time: u64,
    /// 成功率
    pub success_rate: u16,
}

/// 性能快照
#[derive(Debug, Clone)]
pub struct PerformanceSnapshot {
    /// 时间戳
    pub timestamp: i64,
    /// 当前指标
    pub metrics: PerformanceMetrics,
    /// 附加上下文
    pub context: String,
}

/// 优化指标
#[derive(Debug, Clone)]
pub struct OptimizationMetrics {
    /// Gas节省（基点）
    pub gas_savings_bps: u32,
    /// 滑点降低（基点）
    pub slippage_reduction_bps: u32,
    /// 执行改进（基点）
    pub execution_improvement_bps: u32,
    /// MEV保护分数
    pub mev_protection_score: u32,
} 