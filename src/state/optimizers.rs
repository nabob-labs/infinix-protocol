/*!
 * Optimizer State Structures
 *
 * State definitions for execution optimization and risk management.
 */

use crate::core::*;
use crate::error::StrategyError;
use crate::core::traits::{Activatable, Pausable, Validatable};
use crate::state::common::*;
use crate::version::{ProgramVersion, Versioned};
use anchor_lang::prelude::*;
// Removed conflicting borsh import

/// Execution optimizer account
#[account]
#[derive(InitSpace)]
pub struct ExecutionOptimizer {
    /// Base account fields
    pub base: BaseAccount,

    /// Optimizer configuration
    pub config: OptimizerConfig,

    /// Performance metrics
    pub performance_metrics: OptimizerPerformanceMetrics,

    /// Last optimization timestamp
    pub last_optimized: i64,

    /// Execution statistics
    #[max_len(0)]
    pub execution_stats: ExecutionStats,

    /// AI/ML模型预测分数
    pub ai_score: Option<f64>,

    /// 外部信号（如链下回测、市场情绪等）
    pub external_signals: Option<Vec<u64>>,
}

impl ExecutionOptimizer {
    pub const INIT_SPACE: usize = 8 + // discriminator
        std::mem::size_of::<BaseAccount>() +
        std::mem::size_of::<OptimizerConfig>() +
        std::mem::size_of::<OptimizerPerformanceMetrics>() +
        8 + // last_optimized
        std::mem::size_of::<ExecutionStats>();

    /// Initialize the optimizer
    pub fn initialize(
        &mut self,
        authority: Pubkey,
        config: OptimizerConfig,
        bump: u8,
    ) -> Result<()> {
        self.base = BaseAccount::new(authority, bump)?;
        self.config = config;
        self.performance_metrics = OptimizerPerformanceMetrics::default();
        self.last_optimized = 0;
        self.execution_stats = ExecutionStats::default();

        Ok(())
    }

    /// Update configuration
    pub fn update_config(&mut self, new_config: OptimizerConfig) -> Result<()> {
        new_config.validate()?;
        self.config = new_config;
        self.base.touch()?;
        Ok(())
    }

    /// Record optimization result (多因子聚合)
    pub fn record_optimization(
        &mut self,
        gas_saved: u64,
        slippage_reduced: u64,
        execution_time_ms: u64,
        ai_score: Option<f64>,
        external_signals: Option<Vec<u64>>,
    ) -> Result<()> {
        self.performance_metrics.total_optimizations += 1;
        self.performance_metrics.total_gas_saved += gas_saved;
        self.performance_metrics.total_slippage_reduced += slippage_reduced;

        // Update averages
        let total = self.performance_metrics.total_optimizations;
        self.performance_metrics.avg_gas_saved =
            (self.performance_metrics.avg_gas_saved * (total - 1) + gas_saved) / total;
        self.performance_metrics.avg_slippage_reduced =
            (self.performance_metrics.avg_slippage_reduced * (total - 1) + slippage_reduced)
                / total;

        self.last_optimized = Clock::get()?.unix_timestamp;
        self.base.touch()?;

        // 智能化聚合
        self.ai_score = ai_score;
        self.external_signals = external_signals;

        Ok(())
    }
}

impl Validatable for ExecutionOptimizer {
    fn validate(&self) -> Result<()> {
        self.base.validate()?;
        self.config.validate()?;
        Ok(())
    }
}

impl Authorizable for ExecutionOptimizer {
    fn authority(&self) -> Pubkey {
        self.base.authority
    }

    fn transfer_authority(&mut self, new_authority: Pubkey) -> StrategyResult<()> {
        self.base.authority = new_authority;
        self.base.touch()?;
        Ok(())
    }
}

impl Pausable for ExecutionOptimizer {
    fn is_paused(&self) -> bool {
        self.base.is_paused
    }

    fn pause(&mut self) -> Result<()> {
        self.base.pause()
    }

    fn unpause(&mut self) -> Result<()> {
        self.base.unpause()
    }

    fn resume(&mut self) -> StrategyResult<()> {
        self.unpause()
    }
}

impl Activatable for ExecutionOptimizer {
    fn is_active(&self) -> bool {
        self.base.is_active
    }

    fn activate(&mut self) -> Result<()> {
        self.base.activate()
    }

    fn deactivate(&mut self) -> Result<()> {
        self.base.deactivate()
    }
}

impl Versioned for ExecutionOptimizer {
    fn version(&self) -> ProgramVersion {
        self.base.version
    }

    fn set_version(&mut self, version: ProgramVersion) {
        self.base.set_version(version);
    }
}

/// Risk manager account
#[account]
#[derive(InitSpace)]
pub struct RiskManager {
    /// Base account fields
    #[max_len(0)]
    pub base: BaseAccount,

    /// Risk limits configuration
    #[max_len(0)]
    pub risk_limits: RiskLimits,

    /// Current risk metrics
    #[max_len(0)]
    pub current_metrics: RiskMetrics,

    /// Circuit breaker status
    pub circuit_breaker_active: bool,

    /// Last risk assessment timestamp
    pub last_assessment: i64,

    /// Execution statistics
    #[max_len(0)]
    pub execution_stats: ExecutionStats,

    /// AI/ML风险预测分数
    pub ai_risk_score: Option<f64>,

    /// 外部风险信号
    pub external_risk_signals: Option<Vec<u64>>,
}

impl RiskManager {
    pub const INIT_SPACE: usize = 8 + // discriminator
        std::mem::size_of::<BaseAccount>() +
        std::mem::size_of::<RiskLimits>() +
        std::mem::size_of::<RiskMetrics>() +
        1 + // circuit_breaker_active
        8 + // last_assessment
        std::mem::size_of::<ExecutionStats>();

    /// Initialize the risk manager
    pub fn initialize(
        &mut self,
        authority: Pubkey,
        risk_limits: RiskLimits,
        bump: u8,
    ) -> Result<()> {
        self.base = BaseAccount::new(authority, bump)?;
        self.risk_limits = risk_limits;
        self.current_metrics = RiskMetrics::default();
        self.circuit_breaker_active = false;
        self.last_assessment = 0;
        self.execution_stats = ExecutionStats::default();

        Ok(())
    }

    /// Update risk limits
    pub fn update_risk_limits(&mut self, new_limits: RiskLimits) -> Result<()> {
        new_limits.validate()?;
        self.risk_limits = new_limits;
        self.base.touch()?;
        Ok(())
    }

    /// Assess risk and update metrics (多因子聚合)
    pub fn assess_risk(
        &mut self,
        portfolio_value: u64,
        weights: &[u64],
        ai_risk_score: Option<f64>,
        external_risk_signals: Option<Vec<u64>>,
    ) -> Result<()> {
        // Calculate concentration risk
        let concentration_risk = self.calculate_concentration_risk(weights);

        // Calculate VaR (simplified)
        let var_bps = self.calculate_var(portfolio_value, weights)?;

        // 多因子聚合
        let ai_score = ai_risk_score.unwrap_or(0.0);
        let ext_score = external_risk_signals
            .as_ref()
            .and_then(|v| v.first().cloned())
            .unwrap_or(0);
        let overall = ((concentration_risk as f64 * 0.4
            + var_bps as f64 * 0.4
            + ai_score * 0.1
            + ext_score as f64 * 0.1)
            .min(10000.0)) as u32;

        // Update current metrics
        self.current_metrics.var_bps = var_bps;
        self.current_metrics.concentration_risk = concentration_risk as u64;
        self.current_metrics.overall_risk_score = overall;

        // Check if circuit breaker should be activated
        if self.should_activate_circuit_breaker() {
            self.activate_circuit_breaker()?;
        }

        self.last_assessment = Clock::get()?.unix_timestamp;
        self.base.touch()?;

        self.ai_risk_score = ai_risk_score;
        self.external_risk_signals = external_risk_signals;

        Ok(())
    }

    /// Calculate concentration risk
    fn calculate_concentration_risk(&self, weights: &[u64]) -> u32 {
        if weights.is_empty() {
            return 0;
        }

        // Calculate Herfindahl-Hirschman Index
        let hhi: u64 = weights.iter().map(|&w| (w * w) / BASIS_POINTS_MAX).sum();

        // Convert to risk score (0-10000)
        (hhi / 100).min(10000) as u32
    }

    /// Calculate Value at Risk (simplified)
    fn calculate_var(&self, portfolio_value: u64, weights: &[u64]) -> Result<u64> {
        if portfolio_value == 0 || weights.is_empty() {
            return Ok(0);
        }

        // Simplified VaR calculation
        // In production, this would use historical data and sophisticated models
        let volatility_estimate = 2000u64; // 20% annual volatility estimate
        let confidence_factor = 1960u64; // 95% confidence ≈ 1.96 * 1000

        let var =
            (portfolio_value * volatility_estimate * confidence_factor) / (BASIS_POINTS_MAX * 1000);

        Ok(var.min(self.risk_limits.max_var_bps))
    }

    /// Check if circuit breaker should be activated
    fn should_activate_circuit_breaker(&self) -> bool {
        if !self.risk_limits.enable_circuit_breakers {
            return false;
        }

        self.current_metrics.concentration_risk > self.risk_limits.max_concentration_bps
            || self.current_metrics.var_bps > self.risk_limits.max_var_bps
            || self.current_metrics.max_drawdown_bps > self.risk_limits.max_drawdown_bps
    }

    /// Activate circuit breaker
    pub fn activate_circuit_breaker(&mut self) -> Result<()> {
        self.circuit_breaker_active = true;
        self.base.touch()?;

        msg!("Circuit breaker activated due to risk limit breach");
        Ok(())
    }

    /// Deactivate circuit breaker
    pub fn deactivate_circuit_breaker(&mut self) -> Result<()> {
        self.circuit_breaker_active = false;
        self.base.touch()?;

        msg!("Circuit breaker deactivated");
        Ok(())
    }

    /// Check if operation is allowed under current risk conditions
    pub fn is_operation_allowed(&self, operation_risk_score: u32) -> bool {
        if self.circuit_breaker_active {
            return false;
        }

        let total_risk = self.current_metrics.overall_risk_score + operation_risk_score;
        total_risk <= 8000 // Allow operations up to 80% risk score
    }
}

impl Validatable for RiskManager {
    fn validate(&self) -> Result<()> {
        self.base.validate()?;
        self.risk_limits.validate()?;
        Ok(())
    }
}

impl Authorizable for RiskManager {
    fn authority(&self) -> Pubkey {
        self.base.authority
    }

    fn transfer_authority(&mut self, new_authority: Pubkey) -> StrategyResult<()> {
        self.base.authority = new_authority;
        self.base.touch()?;
        Ok(())
    }
}

impl Pausable for RiskManager {
    fn is_paused(&self) -> bool {
        self.base.is_paused
    }

    fn pause(&mut self) -> Result<()> {
        self.base.pause()
    }

    fn unpause(&mut self) -> Result<()> {
        self.base.unpause()
    }

    fn resume(&mut self) -> StrategyResult<()> {
        self.unpause()
    }
}

impl Activatable for RiskManager {
    fn is_active(&self) -> bool {
        self.base.is_active
    }

    fn activate(&mut self) -> Result<()> {
        self.base.activate()
    }

    fn deactivate(&mut self) -> Result<()> {
        self.base.deactivate()
    }
}

impl Versioned for RiskManager {
    fn version(&self) -> ProgramVersion {
        self.base.version
    }

    fn set_version(&mut self, version: ProgramVersion) {
        self.base.set_version(version);
    }
}

/// Optimizer configuration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, InitSpace)]
pub struct OptimizerConfig {
    /// Enable gas optimization
    pub enable_gas_optimization: bool,

    /// Enable MEV protection
    pub enable_mev_protection: bool,

    /// Enable batch processing
    pub enable_batch_processing: bool,

    /// Maximum batch size
    pub max_batch_size: u32,

    /// Optimization timeout in seconds
    pub optimization_timeout_seconds: u32,

    /// Target gas savings in basis points
    pub target_gas_savings_bps: u64,

    /// Target slippage reduction in basis points
    pub target_slippage_reduction_bps: u64,
}

impl Default for OptimizerConfig {
    fn default() -> Self {
        Self {
            enable_gas_optimization: true,
            enable_mev_protection: true,
            enable_batch_processing: true,
            max_batch_size: DEFAULT_BATCH_SIZE,
            optimization_timeout_seconds: DEFAULT_OPTIMIZATION_TIMEOUT,
            target_gas_savings_bps: 1000,       // 10%
            target_slippage_reduction_bps: 500, // 5%
        }
    }
}

impl Validatable for OptimizerConfig {
    fn validate(&self) -> Result<()> {
        if self.max_batch_size == 0 || self.max_batch_size > MAX_BATCH_SIZE {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        if self.optimization_timeout_seconds == 0
            || self.optimization_timeout_seconds > MAX_EXECUTION_TIMEOUT
        {
            return Err(StrategyError::InvalidTimeWindow.into());
        }

        Ok(())
    }
}

/// Optimizer performance metrics
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Default, InitSpace)]
pub struct OptimizerPerformanceMetrics {
    /// Total number of optimizations performed
    pub total_optimizations: u64,

    /// Total gas saved across all optimizations
    pub total_gas_saved: u64,

    /// Total slippage reduced across all optimizations
    pub total_slippage_reduced: u64,

    /// Average gas saved per optimization
    pub avg_gas_saved: u64,

    /// Average slippage reduced per optimization
    pub avg_slippage_reduced: u64,

    /// Success rate in basis points
    pub success_rate_bps: u64,

    /// MEV protection effectiveness score
    pub mev_protection_score: u32,
}
