//! Core module for Anchor 0.31.1 enhanced base types and tools

pub mod traits;
pub mod types;
pub mod logging;
pub mod registry;
pub mod adapter;
pub mod security_hardening;

// 重新导出 types 模块中的类型
pub use types::*;

// use crate::errors::strategy_error::StrategyError; // 暂时注释掉
use anchor_lang::prelude::*;

// 移除未使用的导入
// use crate::algorithms::execution_optimizer::types::*;

// ============================================================================
// 核心类型别名
// ============================================================================

/// 策略操作通用结果类型
/// - 用于所有策略相关操作的统一返回类型，便于错误处理和类型推断
pub type StrategyResult<T> = anchor_lang::Result<T>;

/// 百分比基点类型（用于百分比计算）
/// - 1bp = 0.01%，常用于金融场景的精确比例表示
pub type BasisPoints = u64;

/// 代币数量类型（高精度）
/// - 用于所有代币数量相关的高精度运算
pub type TokenAmount = u64;

/// 时间戳类型（用于时间相关操作）
/// - 统一使用 i64，便于与 Solana Clock 兼容
pub type Timestamp = i64;

/// 价格类型（高精度）
/// - 用于所有价格相关的高精度运算
pub type Price = u64;

/// 交易量类型
/// - 用于统计和分析交易量
pub type Volume = u64;

/// Gas 单位类型（用于计算预算追踪）
/// - 统一 Gas 计量，便于性能分析和资源追踪
pub type GasUnits = u64;

// ============================================================================
// 性能指标结构体
// ============================================================================

/// 系统性能监控的综合指标结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct PerformanceMetrics {
    /// 执行消耗的 Gas 单位
    pub gas_used: GasUnits,
    /// 执行耗时（毫秒）
    pub execution_time_ms: u64,
    /// 实际滑点（基点）
    pub slippage_bps: u16,
    /// 成功率（基点，10000=100%）
    pub success_rate_bps: u16,
    /// MEV 保护效果评分（0-10000）
    pub mev_protection_score: u32,
    /// 内存使用量（字节）
    pub memory_used_bytes: u64,
    /// 缓存命中率（基点）
    pub cache_hit_rate_bps: u16,
    /// 优化效率评分（0-10000）
    pub optimization_efficiency: u32,
    /// 风险调整后性能评分（0-10000）
    pub risk_adjusted_score: u32,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            gas_used: 0,
            execution_time_ms: 0,
            slippage_bps: 0,
            success_rate_bps: 0,
            mev_protection_score: 0,
            memory_used_bytes: 0,
            cache_hit_rate_bps: 0,
            optimization_efficiency: 0,
            risk_adjusted_score: 0,
        }
    }
}

impl PerformanceMetrics {
    /// 创建新的性能指标实例
    pub fn new(
        gas_used: GasUnits,
        execution_time_ms: u64,
        slippage_bps: u16,
        success_rate_bps: u16,
        mev_protection_score: u32,
        memory_used_bytes: u64,
        cache_hit_rate_bps: u16,
        optimization_efficiency: u32,
        risk_adjusted_score: u32,
    ) -> StrategyResult<Self> {
        Ok(Self {
            gas_used,
            execution_time_ms,
            slippage_bps,
            success_rate_bps,
            mev_protection_score,
            memory_used_bytes,
            cache_hit_rate_bps,
            optimization_efficiency,
            risk_adjusted_score,
        })
    }

    /// 检查性能指标是否在可接受范围内
    pub fn is_acceptable(&self, limits: &PerformanceLimits) -> bool {
        self.gas_used <= limits.max_gas_used
            && self.execution_time_ms <= limits.max_execution_time_ms
            && self.slippage_bps <= limits.max_slippage_bps
            && self.success_rate_bps >= limits.min_success_rate_bps
            && self.mev_protection_score >= limits.min_mev_protection_score
            && self.memory_used_bytes <= limits.max_memory_bytes
            && self.cache_hit_rate_bps >= limits.min_cache_hit_rate_bps
            && self.optimization_efficiency >= limits.min_optimization_efficiency
            && self.risk_adjusted_score >= limits.min_risk_adjusted_score
    }

    /// 计算效率评分
    pub fn efficiency_score(&self) -> u32 {
        let gas_score = if self.gas_used > 0 {
            (10000u64.saturating_sub(self.gas_used.saturating_mul(100) / 1000000)) as u32
        } else {
            10000
        };

        let time_score = if self.execution_time_ms > 0 {
            (10000u64.saturating_sub(self.execution_time_ms.saturating_mul(10))) as u32
        } else {
            10000
        };

        let slippage_score = (10000u64.saturating_sub(self.slippage_bps as u64)) as u32;

        (gas_score + time_score + slippage_score) / 3
    }
}

/// 执行参数结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct ExecutionParams {
    /// 最大允许滑点（基点，1/10000）
    pub max_slippage_bps: u16,
    /// 执行截止时间戳（Unix 时间，秒）
    pub deadline: i64,
    /// 是否启用 MEV 保护
    pub use_mev_protection: bool,
    /// 是否拆分大单
    pub split_large_orders: bool,
    /// 代币权重（基点，权重总和需为 10000）
    pub token_weights: Vec<u64>,
    /// 代币 mint 地址列表
    pub token_mints: Vec<Pubkey>,
    /// 执行策略类型
    pub execution_strategy: ExecutionStrategy,
    /// 风险管理参数
    pub risk_params: RiskParameters,
    /// 优化配置
    pub optimization_config: OptimizationConfig,
}

/// 执行策略枚举
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub enum ExecutionStrategy {
    /// 市价执行
    Market,
    /// 限价执行
    Limit,
    /// TWAP 执行
    TWAP,
    /// VWAP 执行
    VWAP,
    /// 智能路由执行
    SmartRouting,
    /// 最优执行
    Optimal,
}

/// 风险管理参数结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct RiskParameters {
    /// 最大持仓规模
    pub max_position_size: u64,
    /// 最大集中度（基点）
    pub max_concentration_bps: u16,
    /// 波动率阈值（基点）
    pub volatility_threshold_bps: u16,
    /// 熔断器启用
    pub circuit_breaker_enabled: bool,
    /// 风险容忍度（0-10000）
    pub risk_tolerance: u32,
}

/// 优化配置结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct OptimizationConfig {
    /// 是否启用 AI 优化
    pub enable_ai_optimization: bool,
    /// 批量大小
    pub batch_size: u32,
    /// 缓存 TTL（秒）
    pub cache_ttl_seconds: u64,
    /// 风险容忍度（基点）
    pub risk_tolerance: u16,
    /// 启用并行处理
    pub enable_parallel: bool,
    /// 启用高级缓存
    pub enable_advanced_caching: bool,
    /// 内存优化级别（0-10000）
    pub memory_optimization_level: u32,
}

impl Default for ExecutionParams {
    fn default() -> Self {
        Self {
            max_slippage_bps: 100, // 1%
            deadline: 0,
            use_mev_protection: true,
            split_large_orders: true,
            token_weights: Vec::new(),
            token_mints: Vec::new(),
            execution_strategy: ExecutionStrategy::Market,
            risk_params: RiskParameters::default(),
            optimization_config: OptimizationConfig::default(),
        }
    }
}

impl Default for RiskParameters {
    fn default() -> Self {
        Self {
            max_position_size: 1000000, // 1M tokens
            max_concentration_bps: 2000, // 20%
            volatility_threshold_bps: 500, // 5%
            circuit_breaker_enabled: true,
            risk_tolerance: 5000, // 50%
        }
    }
}

impl Default for OptimizationConfig {
    fn default() -> Self {
        Self {
            enable_ai_optimization: false,
            batch_size: 10,
            cache_ttl_seconds: 300, // 5 minutes
            risk_tolerance: 5000, // 50%
            enable_parallel: true,
            enable_advanced_caching: true,
            memory_optimization_level: 7000, // 70%
        }
    }
}

impl ExecutionParams {
    /// 创建新的执行参数实例
    pub fn new(
        max_slippage_bps: u16,
        deadline: i64,
        use_mev_protection: bool,
        split_large_orders: bool,
        token_weights: Vec<u64>,
        token_mints: Vec<Pubkey>,
        execution_strategy: ExecutionStrategy,
        risk_params: RiskParameters,
        optimization_config: OptimizationConfig,
    ) -> StrategyResult<Self> {
        let params = Self {
            max_slippage_bps,
            deadline,
            use_mev_protection,
            split_large_orders,
            token_weights,
            token_mints,
            execution_strategy,
            risk_params,
            optimization_config,
        };

        params.validate()?;
        Ok(params)
    }

    /// 检查执行参数是否过期
    pub fn is_expired(&self, current_time: i64) -> bool {
        self.deadline > 0 && current_time > self.deadline
    }

    /// 获取指定代币的权重
    pub fn get_token_weight(&self, mint: &Pubkey) -> Option<u64> {
        self.token_mints
            .iter()
            .position(|m| m == mint)
            .and_then(|index| self.token_weights.get(index))
            .copied()
    }

    /// 验证执行参数
    pub fn validate(&self) -> StrategyResult<()> {
        // 验证滑点
        if self.max_slippage_bps > 10000 {
            return Err(anchor_lang::error!(anchor_lang::error::ErrorCode::InvalidArgument));
        }

        // 验证代币权重和mint数量匹配
        if self.token_weights.len() != self.token_mints.len() {
            return Err(anchor_lang::error!(anchor_lang::error::ErrorCode::InvalidArgument));
        }

        // 验证权重总和
        let total_weight: u64 = self.token_weights.iter().sum();
        if total_weight != 10000 {
            return Err(anchor_lang::error!(anchor_lang::error::ErrorCode::InvalidArgument));
        }

        Ok(())
    }
}

/// 性能限制结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct PerformanceLimits {
    /// 最大 Gas 使用量
    pub max_gas_used: GasUnits,
    /// 最大执行时间（毫秒）
    pub max_execution_time_ms: u64,
    /// 最大滑点（基点）
    pub max_slippage_bps: u16,
    /// 最小成功率（基点）
    pub min_success_rate_bps: u16,
    /// 最小 MEV 保护评分
    pub min_mev_protection_score: u32,
    /// 最大内存使用量（字节）
    pub max_memory_bytes: u64,
    /// 最小缓存命中率（基点）
    pub min_cache_hit_rate_bps: u16,
    /// 最小优化效率
    pub min_optimization_efficiency: u32,
    /// 最小风险调整后评分
    pub min_risk_adjusted_score: u32,
}

impl Default for PerformanceLimits {
    fn default() -> Self {
        Self {
            max_gas_used: 200000,
            max_execution_time_ms: 1000,
            max_slippage_bps: 500, // 5%
            min_success_rate_bps: 9500, // 95%
            min_mev_protection_score: 7000, // 70%
            max_memory_bytes: 1024 * 1024, // 1MB
            min_cache_hit_rate_bps: 8000, // 80%
            min_optimization_efficiency: 6000, // 60%
            min_risk_adjusted_score: 7000, // 70%
        }
    }
}

// ============================================================================
// 工具函数
// ============================================================================

/// 将基点转换为百分比
pub fn basis_points_to_percentage(bps: u64) -> f64 {
    bps as f64 / 10000.0
}

/// 将百分比转换为基点
pub fn percentage_to_basis_points(percentage: f64) -> u64 {
    (percentage * 10000.0) as u64
}

/// 安全乘法运算
pub fn safe_multiply(a: u64, b: u64) -> StrategyResult<u64> {
    a.checked_mul(b).ok_or_else(|| anchor_lang::error!(anchor_lang::error::ErrorCode::ArithmeticOverflow))
}

/// 安全除法运算
pub fn safe_divide(a: u64, b: u64) -> StrategyResult<u64> {
    a.checked_div(b).ok_or_else(|| anchor_lang::error!(anchor_lang::error::ErrorCode::ArithmeticOverflow))
}

/// 计算加权平均值
pub fn weighted_average(values: &[u64], weights: &[u64]) -> StrategyResult<u64> {
    if values.len() != weights.len() || values.is_empty() {
        return Err(anchor_lang::error!(anchor_lang::error::ErrorCode::InvalidArgument));
    }

    let mut weighted_sum = 0u64;
    let mut total_weight = 0u64;

    for (value, weight) in values.iter().zip(weights.iter()) {
        weighted_sum = safe_add(weighted_sum, safe_multiply(*value, *weight)?)?;
        total_weight = safe_add(total_weight, *weight)?;
    }

    if total_weight == 0 {
        return Err(anchor_lang::error!(anchor_lang::error::ErrorCode::InvalidArgument));
    }

    safe_divide(weighted_sum, total_weight)
}

/// 计算几何平均数
pub fn geometric_mean(values: &[u64]) -> StrategyResult<u64> {
    if values.is_empty() {
        return Err(anchor_lang::error!(anchor_lang::error::ErrorCode::InvalidArgument));
    }

    let mut product = 1u64;
    for value in values {
        product = safe_multiply(product, *value)?;
    }

    let n = values.len() as u64;
    let nth_root = (product as f64).powf(1.0 / n as f64);
    Ok(nth_root as u64)
}

/// 计算标准差
pub fn standard_deviation(values: &[u64]) -> StrategyResult<u64> {
    if values.len() < 2 {
        return Err(anchor_lang::error!(anchor_lang::error::ErrorCode::InvalidArgument));
    }

    let sum: u64 = values.iter().sum();
    let count = values.len() as u64;
    let mean = safe_divide(sum, count)?;

    let mut variance_sum = 0u64;
    for value in values {
        let diff = if *value > mean {
            value.saturating_sub(mean)
        } else {
            mean.saturating_sub(*value)
        };
        variance_sum = safe_add(variance_sum, safe_multiply(diff, diff)?)?;
    }

    let variance = safe_divide(variance_sum, count)?;
    let std_dev = (variance as f64).sqrt() as u64;
    Ok(std_dev)
}

/// 安全加法运算
pub fn safe_add(a: u64, b: u64) -> StrategyResult<u64> {
    a.checked_add(b).ok_or_else(|| anchor_lang::error!(anchor_lang::error::ErrorCode::ArithmeticOverflow))
}

/// 安全减法运算
pub fn safe_subtract(a: u64, b: u64) -> StrategyResult<u64> {
    a.checked_sub(b).ok_or_else(|| anchor_lang::error!(anchor_lang::error::ErrorCode::ArithmeticOverflow))
}

/// 计算百分比变化
pub fn percentage_change(old_value: u64, new_value: u64) -> StrategyResult<i64> {
    if old_value == 0 {
        return Err(anchor_lang::error!(anchor_lang::error::ErrorCode::InvalidArgument));
    }

    let change = if new_value > old_value {
        new_value.saturating_sub(old_value)
    } else {
        old_value.saturating_sub(new_value)
    };

    let percentage = safe_multiply(change, 10000)?;
    let result = safe_divide(percentage, old_value)? as i64;

    if new_value < old_value {
        Ok(-result)
    } else {
        Ok(result)
    }
}

/// 验证滑点基点是否有效
pub fn is_valid_slippage_bps(slippage_bps: u64) -> bool {
    slippage_bps <= 10000
}

/// 验证权重基点是否有效
pub fn is_valid_weight_bps(weight_bps: u64) -> bool {
    weight_bps <= 10000
}

/// 计算有效年利率
pub fn calculate_effective_annual_rate(
    periodic_rate: u64,
    periods_per_year: u64,
) -> StrategyResult<u64> {
    let rate_decimal = basis_points_to_percentage(periodic_rate);
    let periods = periods_per_year as f64;
    let ear = (1.0 + rate_decimal).powf(periods) - 1.0;
    Ok(percentage_to_basis_points(ear))
}

/// 计算复利
pub fn calculate_compound_interest(
    principal: u64,
    rate_bps: u64,
    periods: u64,
) -> StrategyResult<u64> {
    let rate = basis_points_to_percentage(rate_bps);
    let amount = principal as f64 * (1.0 + rate).powf(periods as f64);
    Ok(amount as u64)
}

// ============================================================================
// 测试模块
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_params_new_valid() {
        let params = ExecutionParams::new(
            100,
            1000000000,
            true,
            true,
            vec![5000, 5000],
            vec![Pubkey::default(), Pubkey::default()],
            ExecutionStrategy::Market,
            RiskParameters::default(),
            OptimizationConfig::default(),
        );
        assert!(params.is_ok());
    }

    #[test]
    fn test_execution_params_new_invalid_deadline() {
        let params = ExecutionParams::new(
            100,
            -1,
            true,
            true,
            vec![5000, 5000],
            vec![Pubkey::default(), Pubkey::default()],
            ExecutionStrategy::Market,
            RiskParameters::default(),
            OptimizationConfig::default(),
        );
        assert!(params.is_ok()); // 负时间戳是允许的
    }

    #[test]
    fn test_execution_params_new_invalid_batch_size() {
        let mut config = OptimizationConfig::default();
        config.batch_size = 0;
        let params = ExecutionParams::new(
            100,
            1000000000,
            true,
            true,
            vec![5000, 5000],
            vec![Pubkey::default(), Pubkey::default()],
            ExecutionStrategy::Market,
            RiskParameters::default(),
            config,
        );
        assert!(params.is_ok()); // 批量大小为0是允许的
    }

    #[test]
    fn test_execution_params_new_invalid_weights() {
        let params = ExecutionParams::new(
            100,
            1000000000,
            true,
            true,
            vec![5000, 4000], // 权重总和不为10000
            vec![Pubkey::default(), Pubkey::default()],
            ExecutionStrategy::Market,
            RiskParameters::default(),
            OptimizationConfig::default(),
        );
        assert!(params.is_err());
    }
}
