//!
//! risk.rs - 风险指标与风险限制类型定义
//!
//! 本文件定义了RiskMetrics、RiskLimits等风险相关结构体及其实现，严格遵循Rust、Anchor、SOLID最佳实践，
//! 并逐行专业注释，便于审计、维护、扩展。

use anchor_lang::prelude::*;

/// 投资组合风险指标结构体
/// - 所有数值均以基点（10000=100%）存储，保证精度
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub struct RiskMetrics {
    /// 95%置信区间VaR（基点）
    pub var_95: u64,
    /// 99%置信区间VaR（基点）
    pub var_99: u64,
    /// 最大回撤（基点）
    pub max_drawdown: u64,
    /// 投资组合波动率（基点）
    pub volatility: u64,
    /// Sharpe比率*10000
    pub sharpe_ratio: i64,
    /// Beta*10000（1.0=10000）
    pub beta: i64,
    /// 当前周期VaR（基点）
    pub var_bps: u64,
    /// 集中度风险（基点）
    pub concentration_risk: u64,
    /// 总体风险评分（0-10000，越高越风险大）
    pub overall_risk_score: u32,
    /// 历史最大回撤（基点）
    pub max_drawdown_bps: u64,
}

impl Default for RiskMetrics {
    fn default() -> Self {
        Self {
            var_95: 0,
            var_99: 0,
            max_drawdown: 0,
            volatility: 0,
            sharpe_ratio: 0,
            beta: 0,
            var_bps: 0,
            concentration_risk: 0,
            overall_risk_score: 0,
            max_drawdown_bps: 0,
        }
    }
}

impl RiskMetrics {
    /// 构造函数，类型安全
    pub fn new(
        var_95: u64,
        var_99: u64,
        max_drawdown: u64,
        volatility: u64,
        sharpe_ratio: i64,
        beta: i64,
        var_bps: u64,
        concentration_risk: u64,
        overall_risk_score: u32,
        max_drawdown_bps: u64,
    ) -> Result<Self> {
        Ok(Self {
            var_95,
            var_99,
            max_drawdown,
            volatility,
            sharpe_ratio,
            beta,
            var_bps,
            concentration_risk,
            overall_risk_score,
            max_drawdown_bps,
        })
    }
    /// 判断是否高风险
    pub fn is_high_risk(&self) -> bool {
        self.overall_risk_score > 8000
    }
    /// 判断是否在风险限制内
    pub fn is_within_limits(&self, limits: &RiskLimits) -> bool {
        !limits.is_violated(self)
    }
    /// 计算风险调整后收益
    pub fn risk_adjusted_return(&self, return_bps: i64) -> i64 {
        if self.volatility == 0 { return return_bps; }
        return_bps * 10000 / self.volatility as i64
    }
}

/// 风险限制结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub struct RiskLimits {
    /// 最大VaR（基点）
    pub max_var_bps: u32,
    /// 最大集中度（基点）
    pub max_concentration_bps: u32,
    /// 最大回撤（基点）
    pub max_drawdown_bps: u32,
    /// 最大风险评分
    pub max_risk_score: u32,
    /// 启用熔断器
    pub enable_circuit_breakers: bool,
}

impl Default for RiskLimits {
    fn default() -> Self {
        Self {
            max_var_bps: 0,
            max_concentration_bps: 0,
            max_drawdown_bps: 0,
            max_risk_score: 0,
            enable_circuit_breakers: false,
        }
    }
}

impl RiskLimits {
    /// 构造函数
    pub fn new(
        max_var_bps: u64,
        max_concentration_bps: u64,
        max_drawdown_bps: u64,
        max_risk_score: u32,
    ) -> Result<Self> {
        Ok(Self {
            max_var_bps: max_var_bps as u32,
            max_concentration_bps: max_concentration_bps as u32,
            max_drawdown_bps: max_drawdown_bps as u32,
            max_risk_score,
            enable_circuit_breakers: false,
        })
    }
    /// 判断风险指标是否违反限制
    pub fn is_violated(&self, metrics: &RiskMetrics) -> bool {
        metrics.var_bps > self.max_var_bps as u64
            || metrics.concentration_risk > self.max_concentration_bps as u64
            || metrics.max_drawdown_bps > self.max_drawdown_bps as u64
            || metrics.overall_risk_score > self.max_risk_score
    }
} 