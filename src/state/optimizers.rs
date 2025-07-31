/*!
 * Optimizer State Structures
 *
 * State definitions for execution optimization and risk management.
 */

use anchor_lang::prelude::*;
use crate::state::common::*;
use crate::core::traits::*;
use anchor_lang::prelude::ProgramError;
use crate::version::{ProgramVersion, Versioned};

// ========================= 优化器与风险管理器状态实现 =========================
// 本模块为执行优化、风险评估、AI 信号、外部信号等提供统一状态结构体和逻辑，
// 每个 struct、trait、impl、方法、参数、用途、边界、Anchor 相关点、事件、错误、测试等均有详细注释。

/// 执行优化器账户
#[account]
#[derive(Debug, InitSpace, PartialEq, Eq)]
pub struct ExecutionOptimizer {
    /// 通用账户基础信息
    pub base: BaseAccount,
    /// 优化器配置
    pub config: OptimizerConfig,
    /// 性能指标
    pub performance_metrics: OptimizerPerformanceMetrics,
    /// 上次优化时间
    pub last_optimized: i64,
    /// 执行统计
    pub execution_stats: ExecutionStats,
    /// AI/ML模型预测分数
    pub ai_score: Option<f64>,
    /// 外部信号
    #[max_len(16)]
    pub external_signals: Option<Vec<u64>>,
}

impl ExecutionOptimizer {
    /// 初始化优化器
    pub fn initialize(&mut self, authority: Pubkey, config: OptimizerConfig, bump: u8) -> anchor_lang::Result<()> {
        self.base = BaseAccount::new(authority, bump)?;
        self.config = config;
        self.performance_metrics = OptimizerPerformanceMetrics::default();
        self.last_optimized = 0;
        self.execution_stats = ExecutionStats::default();
        Ok(())
    }
    /// 更新配置
    pub fn update_config(&mut self, new_config: OptimizerConfig) -> anchor_lang::Result<()> {
        new_config.validate()?;
        self.config = new_config;
        self.base.touch()?;
        Ok(())
    }
    /// 记录优化结果
    pub fn record_optimization(&mut self, gas_saved: u64, slippage_reduced: u64, execution_time_ms: u64, ai_score: Option<f64>, external_signals: Option<Vec<u64>>) -> anchor_lang::Result<()> {
        self.performance_metrics.total_optimizations += 1;
        self.performance_metrics.total_gas_saved += gas_saved;
        self.performance_metrics.total_slippage_reduced += slippage_reduced;
        let total = self.performance_metrics.total_optimizations;
        self.performance_metrics.avg_gas_saved = (self.performance_metrics.avg_gas_saved * (total - 1) + gas_saved) / total;
        self.performance_metrics.avg_slippage_reduced = (self.performance_metrics.avg_slippage_reduced * (total - 1) + slippage_reduced) / total;
        self.last_optimized = Clock::get()?.unix_timestamp;
        self.base.touch()?;
        self.ai_score = ai_score;
        self.external_signals = external_signals;
        Ok(())
    }
}

/// 校验 trait 实现
impl Validatable for ExecutionOptimizer {
    fn validate(&self) -> anchor_lang::Result<()> {
        self.base.validate()?;
        self.config.validate()?;
        Ok(())
    }
}

/// 权限 trait 实现
impl Authorizable for ExecutionOptimizer {
    fn authority(&self) -> Pubkey { self.base.authority }
    fn transfer_authority(&mut self, new_authority: Pubkey) -> anchor_lang::Result<()> {
        self.base.authority = new_authority;
        self.base.touch()?;
        Ok(())
    }
}

/// 暂停 trait 实现
impl Pausable for ExecutionOptimizer {
    fn is_paused(&self) -> bool { self.base.is_paused }
    fn pause(&mut self) -> anchor_lang::Result<()> { self.base.pause() }
    fn unpause(&mut self) -> anchor_lang::Result<()> { self.base.unpause() }
    fn resume(&mut self) -> anchor_lang::Result<()> { self.unpause() }
}

/// 激活 trait 实现
impl Activatable for ExecutionOptimizer {
    fn is_active(&self) -> bool { self.base.is_active }
    fn activate(&mut self) -> anchor_lang::Result<()> { self.base.activate() }
    fn deactivate(&mut self) -> anchor_lang::Result<()> { self.base.deactivate() }
}

/// 版本 trait 实现
impl Versioned for ExecutionOptimizer {
    fn version(&self) -> ProgramVersion { self.base.version }
    fn set_version(&mut self, version: ProgramVersion) { self.base.set_version(version); }
}

// ========================= 风险管理器 =========================

/// 风险管理器账户
#[account]
#[derive(Debug, InitSpace, PartialEq, Eq)]
pub struct RiskManager {
    /// 通用账户基础信息
    pub base: BaseAccount,
    /// 风险限额配置
    pub risk_limits: RiskLimits,
    /// 当前风险指标
    pub current_metrics: RiskMetrics,
    /// 熔断器状态
    pub circuit_breaker_active: bool,
    /// 上次风险评估时间
    pub last_assessment: i64,
    /// 执行统计
    pub execution_stats: ExecutionStats,
    /// AI/ML风险预测分数
    pub ai_risk_score: Option<f64>,
    /// 外部风险信号
    #[max_len(16)]
    pub external_risk_signals: Option<Vec<u64>>,
}

impl RiskManager {
    /// 初始化风险管理器
    pub fn initialize(&mut self, authority: Pubkey, risk_limits: RiskLimits, bump: u8) -> anchor_lang::Result<()> {
        self.base = BaseAccount::new(authority, bump)?;
        self.risk_limits = risk_limits;
        self.current_metrics = RiskMetrics::default();
        self.circuit_breaker_active = false;
        self.last_assessment = 0;
        self.execution_stats = ExecutionStats::default();
        Ok(())
    }
    /// 更新风险限额
    pub fn update_risk_limits(&mut self, new_limits: RiskLimits) -> anchor_lang::Result<()> {
        new_limits.validate()?;
        self.risk_limits = new_limits;
        self.base.touch()?;
        Ok(())
    }
    /// 风险评估
    pub fn assess_risk(&mut self, portfolio_value: u64, weights: &[u64], ai_risk_score: Option<f64>, external_risk_signals: Option<Vec<u64>>) -> anchor_lang::Result<()> {
        let concentration_risk = self.calculate_concentration_risk(weights);
        let var_bps = self.calculate_var(portfolio_value, weights)?;
        let ai_score = ai_risk_score.unwrap_or(0.0);
        let ext_score = external_risk_signals.as_ref().and_then(|v| v.first().cloned()).unwrap_or(0);
        let overall = ((concentration_risk as f64 * 0.4 + var_bps as f64 * 0.4 + ai_score * 0.1 + ext_score as f64 * 0.1).min(10000.0)) as u32;
        self.current_metrics.var_bps = var_bps;
        self.current_metrics.concentration_risk = concentration_risk as u64;
        self.current_metrics.overall_risk_score = overall;
        if self.should_activate_circuit_breaker() {
            self.activate_circuit_breaker()?;
        }
        self.last_assessment = Clock::get()?.unix_timestamp;
        self.base.touch()?;
        self.ai_risk_score = ai_risk_score;
        self.external_risk_signals = external_risk_signals;
        Ok(())
    }
    /// 计算集中度风险
    fn calculate_concentration_risk(&self, weights: &[u64]) -> u32 {
        if weights.is_empty() { return 0; }
        let hhi: u64 = weights.iter().map(|&w| (w * w) / 10_000).sum();
        (hhi / 100).min(10_000) as u32
    }
    /// 计算VaR
    fn calculate_var(&self, portfolio_value: u64, weights: &[u64]) -> anchor_lang::Result<u64> {
        if portfolio_value == 0 || weights.is_empty() { return Ok(0); }
        let volatility_estimate = 2000u64;
        let confidence_factor = 1960u64;
        let var = (portfolio_value * volatility_estimate * confidence_factor) / (10_000 * 1000);
        Ok(var.min(self.risk_limits.max_var_bps as u64))
    }
    /// 是否应激活熔断器
    fn should_activate_circuit_breaker(&self) -> bool {
        if !self.risk_limits.enable_circuit_breakers { return false; }
        self.current_metrics.concentration_risk > self.risk_limits.max_concentration_bps as u64
            || self.current_metrics.var_bps > self.risk_limits.max_var_bps as u64
            || self.current_metrics.max_drawdown_bps > self.risk_limits.max_drawdown_bps as u64
    }
    /// 激活熔断器
    pub fn activate_circuit_breaker(&mut self) -> anchor_lang::Result<()> {
        self.circuit_breaker_active = true;
        self.base.touch()?;
        msg!("Circuit breaker activated due to risk limit breach");
        Ok(())
    }
    /// 关闭熔断器
    pub fn deactivate_circuit_breaker(&mut self) -> anchor_lang::Result<()> {
        self.circuit_breaker_active = false;
        self.base.touch()?;
        msg!("Circuit breaker deactivated");
        Ok(())
    }
    /// 当前风险条件下操作是否允许
    pub fn is_operation_allowed(&self, operation_risk_score: u32) -> bool {
        if self.circuit_breaker_active { return false; }
        let total_risk = self.current_metrics.overall_risk_score + operation_risk_score;
        total_risk <= 8000
    }
}

/// 校验 trait 实现
impl Validatable for RiskManager {
    fn validate(&self) -> anchor_lang::Result<()> {
        self.base.validate()?;
        self.risk_limits.validate()?;
        Ok(())
    }
}

/// 权限 trait 实现
impl Authorizable for RiskManager {
    fn authority(&self) -> Pubkey { self.base.authority }
    fn transfer_authority(&mut self, new_authority: Pubkey) -> anchor_lang::Result<()> {
        self.base.authority = new_authority;
        self.base.touch()?;
        Ok(())
    }
}

/// 暂停 trait 实现
impl Pausable for RiskManager {
    fn is_paused(&self) -> bool { self.base.is_paused }
    fn pause(&mut self) -> anchor_lang::Result<()> { self.base.pause() }
    fn unpause(&mut self) -> anchor_lang::Result<()> { self.base.unpause() }
    fn resume(&mut self) -> anchor_lang::Result<()> { self.unpause() }
}

/// 激活 trait 实现
impl Activatable for RiskManager {
    fn is_active(&self) -> bool { self.base.is_active }
    fn activate(&mut self) -> anchor_lang::Result<()> { self.base.activate() }
    fn deactivate(&mut self) -> anchor_lang::Result<()> { self.base.deactivate() }
}

/// 版本 trait 实现
impl Versioned for RiskManager {
    fn version(&self) -> ProgramVersion { self.base.version }
    fn set_version(&mut self, version: ProgramVersion) { self.base.set_version(version); }
}

// ========================= 优化器配置与性能指标 =========================

/// 优化器配置
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, InitSpace)]
pub struct OptimizerConfig {
    /// 是否启用gas优化
    pub enable_gas_optimization: bool,
    /// 是否启用MEV保护
    pub enable_mev_protection: bool,
    /// 是否启用批处理
    pub enable_batch_processing: bool,
    /// 最大批处理数
    pub max_batch_size: u32,
    /// 优化超时时间（秒）
    pub optimization_timeout_seconds: u32,
    /// 目标gas节省（bps）
    pub target_gas_savings_bps: u64,
    /// 目标滑点降低（bps）
    pub target_slippage_reduction_bps: u64,
}

impl Default for OptimizerConfig {
    fn default() -> Self {
        Self {
            enable_gas_optimization: true,
            enable_mev_protection: true,
            enable_batch_processing: true,
            max_batch_size: 16,
            optimization_timeout_seconds: 60,
            target_gas_savings_bps: 1000,
            target_slippage_reduction_bps: 500,
        }
    }
}

impl Validatable for OptimizerConfig {
    fn validate(&self) -> anchor_lang::Result<()> {
        if self.max_batch_size == 0 || self.max_batch_size > 128 {
            return Err(ProgramError::InvalidStrategyParameters.into());
        }
        if self.optimization_timeout_seconds == 0 || self.optimization_timeout_seconds > 600 {
            return Err(ProgramError::InvalidStrategyParameters.into());
        }
        Ok(())
    }
}

/// 优化器性能指标
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Default, InitSpace)]
pub struct OptimizerPerformanceMetrics {
    /// 优化总次数
    pub total_optimizations: u64,
    /// 累计gas节省
    pub total_gas_saved: u64,
    /// 累计滑点降低
    pub total_slippage_reduced: u64,
    /// 平均每次gas节省
    pub avg_gas_saved: u64,
    /// 平均每次滑点降低
    pub avg_slippage_reduced: u64,
    /// 成功率（bps）
    pub success_rate_bps: u64,
    /// MEV保护效果评分
    pub mev_protection_score: u32,
}

// ========================= 单元测试 =========================
#[cfg(test)]
mod tests {
    use super::*;
    use anchor_lang::prelude::Pubkey;

    #[test]
    fn test_optimizer_validate() {
        let mut optimizer = ExecutionOptimizer {
            base: BaseAccount::new(Pubkey::new_unique(), 1).unwrap(),
            config: OptimizerConfig::default(),
            performance_metrics: OptimizerPerformanceMetrics::default(),
            last_optimized: 0,
            execution_stats: ExecutionStats::default(),
            ai_score: None,
            external_signals: None,
        };
        assert!(optimizer.validate().is_ok());
        optimizer.config.max_batch_size = 0;
        assert!(optimizer.validate().is_err());
    }

    #[test]
    fn test_risk_manager_validate() {
        let mut risk_manager = RiskManager {
            base: BaseAccount::new(Pubkey::new_unique(), 1).unwrap(),
            risk_limits: RiskLimits::default(),
            current_metrics: RiskMetrics::default(),
            circuit_breaker_active: false,
            last_assessment: 0,
            execution_stats: ExecutionStats::default(),
            ai_risk_score: None,
            external_risk_signals: None,
        };
        assert!(risk_manager.validate().is_ok());
        risk_manager.risk_limits.max_var_bps = 0;
        assert!(risk_manager.validate().is_ok());
    }
}
