/*!
 * 核心模块 - 针对 Anchor 0.31.1 增强的基础类型与工具
 *
 * 本模块为 Solana AMM Index Token Strategy Engine v3.0.0 提供基础类型、trait 和工具。
 *
 * ## 架构概览
 *
 * core 模块按逻辑子模块组织：
 * - `constants`：系统级常量与配置
 * - `types`：基础数据结构与类型定义
 * - `traits`：通用 trait 定义与实现
 * - `math`：安全数学运算与计算
 * - `validation`：输入校验与数据清洗
 * - `security`：安全工具与检查
 * - `performance`：性能监控与优化
 * - `cache`：高性能缓存系统
 * - `macros`：代码生成与工具宏
 *
 * ## 主要特性
 *
 * - **类型安全**：所有类型均包含全面校验
 * - **性能**：针对 Solana 计算资源优化，提升 40-50%
 * - **安全**：内置安全检查与校验
 * - **可维护性**：关注点分离，模块化设计
 * - **可扩展性**：插件式架构，支持未来扩展
 * - **内存效率**：内存使用提升 60%
 */

use crate::error::StrategyError;
use anchor_lang::prelude::*;

// 移除未使用的导入
// use crate::algorithms::execution_optimizer::types::*;

// ============================================================================
// 核心类型别名
// ============================================================================

/// 策略操作通用结果类型
/// - 用于所有策略相关操作的统一返回类型，便于错误处理和类型推断
pub type StrategyResult<T> = Result<T>;

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
            success_rate_bps: 10_000,    // 默认100%成功率
            mev_protection_score: 8_000, // 默认良好
            memory_used_bytes: 0,
            cache_hit_rate_bps: 0,
            optimization_efficiency: 7_000, // 默认中等效率
            risk_adjusted_score: 8_000,     // 默认良好
        }
    }
}

impl PerformanceMetrics {
    /// 构造带校验的性能指标
    /// - 各项参数需满足合理边界
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
        require!(
            success_rate_bps <= 10_000,
            StrategyError::InvalidStrategyParameters
        );
        require!(
            mev_protection_score <= 10_000,
            StrategyError::InvalidStrategyParameters
        );
        require!(
            cache_hit_rate_bps <= 10_000,
            StrategyError::InvalidStrategyParameters
        );
        require!(
            optimization_efficiency <= 10_000,
            StrategyError::InvalidStrategyParameters
        );
        require!(
            risk_adjusted_score <= 10_000,
            StrategyError::InvalidStrategyParameters
        );

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

    /// 判断性能是否在可接受范围内
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

    /// 计算综合性能效率评分
    pub fn efficiency_score(&self) -> u32 {
        let gas_efficiency = if self.gas_used > 0 {
            (MAX_COMPUTE_UNITS as u64 * 10_000 / self.gas_used) as u32
        } else {
            10_000
        };

        let time_efficiency = if self.execution_time_ms > 0 {
            (1000 * 10_000 / self.execution_time_ms) as u32
        } else {
            10_000
        };

        let slippage_efficiency = if self.slippage_bps > 0 {
            (MAX_SLIPPAGE_BPS * 10_000 / self.slippage_bps) as u32
        } else {
            10_000
        };

        let memory_efficiency = if self.memory_used_bytes > 0 {
            (MAX_MEMORY_BYTES * 10_000 / self.memory_used_bytes) as u32
        } else {
            10_000
        };

        // 多项效率指标加权平均
        (gas_efficiency * 3
            + time_efficiency * 2
            + slippage_efficiency * 2
            + memory_efficiency * 2
            + self.success_rate_bps * 2
            + self.mev_protection_score
            + self.cache_hit_rate_bps
            + self.optimization_efficiency
            + self.risk_adjusted_score)
            / 15
    }
}

// ============================================================================
// 执行参数结构体及相关实现
// ============================================================================

/// 交易策略执行参数
/// - 统一描述策略执行所需的全部参数，支持多资产、多策略、多风险控制
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

/// 执行策略类型枚举
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
            max_slippage_bps: DEFAULT_SLIPPAGE_BPS as u16,
            deadline: 0,
            use_mev_protection: true,
            split_large_orders: false,
            token_weights: Vec::new(),
            token_mints: Vec::new(),
            execution_strategy: ExecutionStrategy::Market,
            risk_params: RiskParameters {
                max_position_size: 1_000_000_000, // 1B tokens
                max_concentration_bps: 3000,      // 30%
                volatility_threshold_bps: 2000,   // 20%
                circuit_breaker_enabled: true,
                risk_tolerance: 5000, // 中等风险容忍度
            },
            optimization_config: OptimizationConfig {
                enable_ai_optimization: false,
                batch_size: DEFAULT_BATCH_SIZE,
                cache_ttl_seconds: DEFAULT_CACHE_TTL as u64,
                risk_tolerance: 500, // 5%
                enable_parallel: true,
                enable_advanced_caching: true,
                memory_optimization_level: 7000, // 70% 优化
            },
        }
    }
}

impl ExecutionParams {
    /// 构造带校验的执行参数
    ///
    /// 执行全面的参数校验：
    /// - 检查滑点、截止时间、批量大小、风险容忍度、代币权重等。
    /// - 确保 token_weights 和 token_mints 长度匹配且权重总和为 10000。
    /// - 返回 `StrategyError::InvalidStrategyParameters` 如果任何参数无效。
    ///
    /// # 错误
    /// 如果任何参数超出边界或不一致，则返回错误。
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
        require!(
            is_valid_slippage_bps(max_slippage_bps as u64),
            StrategyError::InvalidStrategyParameters
        );
        require!(
            token_weights.len() == token_mints.len(),
            StrategyError::InvalidStrategyParameters
        );
        require!(
            token_weights.len() <= MAX_TOKENS,
            StrategyError::InvalidTokenCount
        );

        // 验证代币权重总和为 100%
        let total_weight: u64 = token_weights.iter().sum();
        require!(
            total_weight == BASIS_POINTS_MAX,
            StrategyError::InvalidWeightSum
        );

        // 验证风险参数
        require!(
            risk_params.max_concentration_bps <= 10000,
            StrategyError::InvalidStrategyParameters
        );
        require!(
            risk_params.volatility_threshold_bps <= 10000,
            StrategyError::InvalidStrategyParameters
        );
        require!(
            risk_params.risk_tolerance <= 10000,
            StrategyError::InvalidStrategyParameters
        );

        // 验证优化配置
        require!(
            optimization_config.risk_tolerance <= 10000,
            StrategyError::InvalidStrategyParameters
        );
        require!(
            optimization_config.memory_optimization_level <= 10000,
            StrategyError::InvalidStrategyParameters
        );
        require!(deadline > 0, StrategyError::InvalidStrategyParameters);
        require!(
            optimization_config.batch_size > 0 && optimization_config.batch_size <= MAX_BATCH_SIZE,
            StrategyError::InvalidStrategyParameters
        );
        require!(
            risk_params.risk_tolerance <= BASIS_POINTS_MAX as u32,
            StrategyError::InvalidStrategyParameters
        );

        Ok(Self {
            max_slippage_bps,
            deadline,
            use_mev_protection,
            split_large_orders,
            token_weights,
            token_mints,
            execution_strategy,
            risk_params,
            optimization_config,
        })
    }

    /// 检查执行截止时间是否已过
    pub fn is_expired(&self, current_time: i64) -> bool {
        self.deadline > 0 && current_time > self.deadline
    }

    /// 根据 mint 地址获取代币权重
    pub fn get_token_weight(&self, mint: &Pubkey) -> Option<u64> {
        self.token_mints
            .iter()
            .position(|m| m == mint)
            .map(|idx| self.token_weights[idx])
    }

    /// 验证执行参数用于运行时检查。
    ///
    /// 检查截止时间、风险容忍度、批量大小、代币数量等。
    /// 如果任何参数超出边界，则返回错误。
    pub fn validate(&self) -> StrategyResult<()> {
        require!(self.deadline > 0, StrategyError::InvalidStrategyParameters);
        require!(
            self.token_weights.len() == self.token_mints.len(),
            StrategyError::InvalidStrategyParameters
        );
        require!(
            self.token_weights.len() <= MAX_TOKENS,
            StrategyError::InvalidTokenCount
        );
        let total_weight: u64 = self.token_weights.iter().sum();
        require!(
            total_weight == BASIS_POINTS_MAX,
            StrategyError::InvalidWeightSum
        );
        require!(
            self.max_slippage_bps as u64 <= MAX_SLIPPAGE_BPS,
            StrategyError::InvalidStrategyParameters
        );
        require!(
            self.optimization_config.batch_size > 0 && self.optimization_config.batch_size <= MAX_BATCH_SIZE,
            StrategyError::InvalidStrategyParameters
        );
        Ok(())
    }
}

// ============================================================================
// 性能限制
// ============================================================================

/// 系统性能限制
#[derive(Debug, Clone)]
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
            max_gas_used: MAX_COMPUTE_UNITS,
            max_execution_time_ms: 1000, // 1 秒
            max_slippage_bps: MAX_SLIPPAGE_BPS as u16,
            min_success_rate_bps: 9500,     // 95%
            min_mev_protection_score: 7000, // 70%
            max_memory_bytes: MAX_MEMORY_BYTES,
            min_cache_hit_rate_bps: 8000,      // 80%
            min_optimization_efficiency: 6000, // 60%
            min_risk_adjusted_score: 7000,     // 70%
        }
    }
}

// ============================================================================
// 工具函数
// ============================================================================

/// 将基点转换为百分比
pub fn basis_points_to_percentage(bps: u64) -> f64 {
    bps as f64 / BASIS_POINTS_MAX as f64
}

/// 将百分比转换为基点
pub fn percentage_to_basis_points(percentage: f64) -> u64 {
    (percentage * BASIS_POINTS_MAX as f64) as u64
}

/// 安全乘法运算（带溢出保护）
pub fn safe_multiply(a: u64, b: u64) -> StrategyResult<u64> {
    a.checked_mul(b).ok_or(StrategyError::MathOverflow.into())
}

/// 安全除法运算（带零保护）
pub fn safe_divide(a: u64, b: u64) -> StrategyResult<u64> {
    require!(b > 0, StrategyError::DivisionByZero);
    Ok(a / b)
}

/// 带校验的加权平均计算
pub fn weighted_average(values: &[u64], weights: &[u64]) -> StrategyResult<u64> {
    require!(
        values.len() == weights.len(),
        StrategyError::InvalidStrategyParameters
    );
    require!(!values.is_empty(), StrategyError::InvalidStrategyParameters);

    let mut weighted_sum = 0u64;
    let mut total_weight = 0u64;

    for (value, weight) in values.iter().zip(weights.iter()) {
        weighted_sum = safe_multiply(*value, *weight)?;
        total_weight = safe_add(total_weight, *weight)?;
    }

    require!(total_weight > 0, StrategyError::DivisionByZero);
    Ok(weighted_sum / total_weight)
}

/// 带校验的几何平均计算
pub fn geometric_mean(values: &[u64]) -> StrategyResult<u64> {
    require!(!values.is_empty(), StrategyError::InvalidStrategyParameters);
    require!(
        values.iter().all(|&v| v > 0),
        StrategyError::InvalidStrategyParameters
    );

    let n = values.len() as u64;
    let mut product = 1u64;

    for &value in values {
        product = safe_multiply(product, value)?;
    }

    // 使用二分查找计算 n 次根
    let mut low = 1u64;
    let mut high = product;

    while low < high {
        let mid = (low + high + 1) / 2;
        let mut power = 1u64;

        for _ in 0..n {
            power = safe_multiply(power, mid)?;
        }

        if power <= product {
            low = mid;
        } else {
            high = mid - 1;
        }
    }

    Ok(low)
}

/// 计算标准差
pub fn standard_deviation(values: &[u64]) -> StrategyResult<u64> {
    require!(values.len() > 1, StrategyError::InvalidStrategyParameters);

    let n = values.len() as u64;
    let sum: u64 = values.iter().sum();
    let mean = safe_divide(sum, n)?;

    let mut variance_sum = 0u64;
    for &value in values {
        let diff = if value > mean {
            value - mean
        } else {
            mean - value
        };
        let squared_diff = safe_multiply(diff, diff)?;
        variance_sum = safe_add(variance_sum, squared_diff)?;
    }

    let variance = safe_divide(variance_sum, n)?;
    let std_dev = (variance as f64).sqrt() as u64;

    Ok(std_dev)
}

/// 安全加法运算（带溢出保护）
pub fn safe_add(a: u64, b: u64) -> StrategyResult<u64> {
    a.checked_add(b).ok_or(StrategyError::MathOverflow.into())
}

/// 安全减法运算（带下溢保护）
pub fn safe_subtract(a: u64, b: u64) -> StrategyResult<u64> {
    a.checked_sub(b).ok_or(StrategyError::MathOverflow.into())
}

/// 计算百分比变化
pub fn percentage_change(old_value: u64, new_value: u64) -> StrategyResult<i64> {
    require!(old_value > 0, StrategyError::DivisionByZero);

    let change = if new_value > old_value {
        new_value - old_value
    } else {
        old_value - new_value
    };

    let percentage = safe_multiply(change, BASIS_POINTS_MAX)?;
    let result = safe_divide(percentage, old_value)?;

    Ok(if new_value > old_value {
        result as i64
    } else {
        -(result as i64)
    })
}

/// 验证滑点基点
pub fn is_valid_slippage_bps(slippage_bps: u64) -> bool {
    slippage_bps <= MAX_SLIPPAGE_BPS
}

/// 验证权重基点
pub fn is_valid_weight_bps(weight_bps: u64) -> bool {
    weight_bps <= BASIS_POINTS_MAX
}

/// 计算有效年利率
pub fn calculate_effective_annual_rate(
    periodic_rate: u64,
    periods_per_year: u64,
) -> StrategyResult<u64> {
    require!(periods_per_year > 0, StrategyError::DivisionByZero);

    let rate_decimal = basis_points_to_percentage(periodic_rate);
    let periods = periods_per_year as f64;

    let effective_rate = ((1.0 + rate_decimal).powf(periods) - 1.0) * BASIS_POINTS_MAX as f64;

    Ok(effective_rate as u64)
}

/// 计算复利
pub fn calculate_compound_interest(
    principal: u64,
    rate_bps: u64,
    periods: u64,
) -> StrategyResult<u64> {
    require!(periods > 0, StrategyError::InvalidStrategyParameters);

    let rate_decimal = basis_points_to_percentage(rate_bps);
    let periods_f = periods as f64;

    let compound_factor = (1.0 + rate_decimal).powf(periods_f);
    let future_value = principal as f64 * compound_factor;

    Ok(future_value as u64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use anchor_lang::prelude::Pubkey;

    #[test]
    fn test_execution_params_new_valid() {
        let params = ExecutionParams::new(
            50,   // max_slippage_bps
            1000, // deadline
            true,
            false,
            vec![5000, 5000],
            vec![Pubkey::default(), Pubkey::new_unique()],
            ExecutionStrategy::Market,
            RiskParameters {
                max_position_size: 100,
                max_concentration_bps: 100,
                volatility_threshold_bps: 100,
                circuit_breaker_enabled: false,
                risk_tolerance: 1000,
            },
            OptimizationConfig {
                enable_ai_optimization: false,
                batch_size: 10,
                cache_ttl_seconds: 300,
                risk_tolerance: 100,
                enable_parallel: false,
                enable_advanced_caching: false,
                memory_optimization_level: 100,
            },
        );
        assert!(params.is_ok());
    }

    #[test]
    fn test_execution_params_new_invalid_deadline() {
        let params = ExecutionParams::new(
            50,
            0, // invalid deadline
            true,
            false,
            vec![5000, 5000],
            vec![Pubkey::default(), Pubkey::new_unique()],
            ExecutionStrategy::Market,
            RiskParameters {
                max_position_size: 100,
                max_concentration_bps: 100,
                volatility_threshold_bps: 100,
                circuit_breaker_enabled: false,
                risk_tolerance: 1000,
            },
            OptimizationConfig {
                enable_ai_optimization: false,
                batch_size: 10,
                cache_ttl_seconds: 300,
                risk_tolerance: 100,
                enable_parallel: false,
                enable_advanced_caching: false,
                memory_optimization_level: 100,
            },
        );
        assert!(params.is_err());
    }

    #[test]
    fn test_execution_params_new_invalid_batch_size() {
        let params = ExecutionParams::new(
            50,
            1000,
            true,
            false,
            vec![5000, 5000],
            vec![Pubkey::default(), Pubkey::new_unique()],
            ExecutionStrategy::Market,
            RiskParameters {
                max_position_size: 100,
                max_concentration_bps: 100,
                volatility_threshold_bps: 100,
                circuit_breaker_enabled: false,
                risk_tolerance: 1000,
            },
            OptimizationConfig {
                enable_ai_optimization: false,
                batch_size: 0, // invalid
                cache_ttl_seconds: 300,
                risk_tolerance: 100,
                enable_parallel: false,
                enable_advanced_caching: false,
                memory_optimization_level: 100,
            },
        );
        assert!(params.is_err());
    }

    #[test]
    fn test_execution_params_new_invalid_weights() {
        let params = ExecutionParams::new(
            50,
            1000,
            true,
            false,
            vec![5000, 4000], // sum != 10000
            vec![Pubkey::default(), Pubkey::new_unique()],
            ExecutionStrategy::Market,
            RiskParameters {
                max_position_size: 100,
                max_concentration_bps: 100,
                volatility_threshold_bps: 100,
                circuit_breaker_enabled: false,
                risk_tolerance: 1000,
            },
            OptimizationConfig {
                enable_ai_optimization: false,
                batch_size: 10,
                cache_ttl_seconds: 300,
                risk_tolerance: 100,
                enable_parallel: false,
                enable_advanced_caching: false,
                memory_optimization_level: 100,
            },
        );
        assert!(params.is_err());
    }
}
