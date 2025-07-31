//! 执行优化器参数与类型定义模块
//! 包含高级策略参数、风险参数、优化配置、市场建模、工厂等类型定义。

use crate::algorithms::traits::ExecutionStrategy; // 引入执行算法 trait
use std::collections::HashMap; // HashMap 用于算法注册表
use std::sync::{Arc, RwLock}; // Arc+RwLock 用于线程安全的算法工厂
use anchor_lang::prelude::*; // Anchor 预导入，包含序列化、账户等

// === 指数代币高级策略与参数类型（迁移自 index_tokens/mod.rs） ===

/// 指数代币业绩指标结构体
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, Default, InitSpace)]
pub struct IndexTokenPerformanceMetrics {
    /// 自成立以来总收益（基点）
    pub total_return_bps: i64, // 总收益
    /// 年化收益（基点）
    pub annualized_return_bps: i64, // 年化收益
    /// 最大回撤（基点）
    pub max_drawdown_bps: u64, // 最大回撤
    /// 波动率（基点）
    pub volatility_bps: u64, // 波动率
    /// Sharpe 比率（放大1000倍）
    pub sharpe_ratio: i64, // Sharpe 比率
    /// 再平衡次数
    pub rebalance_count: u64, // 再平衡次数
    /// 平均再平衡成本（基点）
    pub avg_rebalancing_cost_bps: u64, // 平均再平衡成本
}

/// 高级交易参数结构体
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct AdvancedTradingParams {
    pub strategy_type: AdvancedTradingStrategy,           // 策略类型
    pub execution_method: ExecutionMethod,                // 执行方式
    pub risk_parameters: RiskParameters,                  // 风控参数
    pub optimization_settings: OptimizationSettings,      // 优化配置
}

/// 高级交易策略枚举
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub enum AdvancedTradingStrategy {
    TWAP, // 时间加权平均价格
    VWAP, // 成交量加权平均价格
    Implementation, // 实施短线
    Momentum, // 动量
    MeanReversion, // 均值回归
}

/// 执行方式枚举
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub enum ExecutionMethod {
    Immediate, // 立即执行
    Gradual,   // 渐进执行
    Optimal,   // 最优执行
    Adaptive,  // 自适应
}

/// 风控参数结构体
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct RiskParameters {
    pub max_slippage_bps: u64,         // 最大滑点
    pub max_position_size_bps: u64,    // 最大持仓
    pub stop_loss_bps: u64,            // 止损
    pub take_profit_bps: u64,          // 止盈
}

/// 优化配置结构体
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct OptimizationSettings {
    pub enable_mev_protection: bool,       // 是否启用 MEV 保护
    pub enable_gas_optimization: bool,     // 是否启用 Gas 优化
    pub target_execution_time: u64,        // 目标执行时间
    pub priority_fee_multiplier: u64,      // 优先费倍率
}

/// 做市参数结构体
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct MarketMakingParams {
    pub spread_bps: u64,                   // 买卖价差
    pub depth_levels: u32,                 // 深度档位
    pub inventory_target_bps: u64,         // 库存目标
    pub rebalance_threshold_bps: u64,      // 再平衡阈值
}

/// 套利参数结构体
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct ArbitrageParams {
    pub min_profit_bps: u64,               // 最小套利利润
    pub max_position_size: u64,            // 最大持仓
    pub execution_timeout: u64,            // 执行超时
    pub cross_protocol_enabled: bool,      // 是否跨协议
}

/// AMM 路由结构体
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct AMMRoute {
    pub protocol: String,                  // 协议名
    pub pool_address: Pubkey,              // 池地址
    pub fee_bps: u64,                      // 手续费
    pub liquidity: u64,                    // 流动性
}

/// 流动性提供参数结构体
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct LiquidityProvisionParams {
    pub target_utilization_bps: u64,       // 目标利用率
    pub fee_tier: u32,                     // 费率档位
    pub range_width_bps: u64,              // 区间宽度
    pub rebalance_frequency: u64,          // 再平衡频率
}

/// 算法交易参数结构体
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct AlgorithmicTradingParams {
    pub algorithm_type: AlgorithmType,                 // 算法类型
    pub signal_threshold: u64,                         // 信号阈值
    pub position_sizing_method: PositionSizingMethod,  // 仓位分配方式
    pub risk_management: RiskManagementSettings,       // 风控设置
}

/// 算法类型枚举
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub enum AlgorithmType {
    MeanReversion, // 均值回归
    Momentum,      // 动量
    Arbitrage,     // 套利
    MarketMaking,  // 做市
    TrendFollowing,// 趋势跟踪
}

/// 仓位分配方式枚举
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub enum PositionSizingMethod {
    Fixed,         // 固定仓位
    Proportional,  // 比例仓位
    KellyOptimal,  // 凯利最优
    RiskParity,    // 风险平价
}

/// 风控设置结构体
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct RiskManagementSettings {
    pub max_drawdown_bps: u64,         // 最大回撤
    pub position_limit_bps: u64,       // 持仓上限
    pub correlation_limit: u64,        // 相关性上限
    pub var_limit_bps: u64,            // VaR 上限
}

/// 组合优化参数结构体
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct PortfolioOptimizationParams {
    pub optimization_objective: OptimizationObjective,     // 优化目标
    pub constraints: OptimizationConstraints,              // 约束条件
    pub rebalancing_frequency: u64,                       // 再平衡频率
    pub transaction_cost_model: TransactionCostModel,      // 交易成本模型
}

/// 优化目标枚举
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub enum OptimizationObjective {
    MaximizeReturn,        // 最大化收益
    MinimizeRisk,          // 最小化风险
    MaximizeSharpe,        // 最大化 Sharpe 比率
    MinimizeTrackingError, // 最小化跟踪误差
}

/// 优化约束结构体
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct OptimizationConstraints {
    pub min_weight_bps: u64,              // 最小权重
    pub max_weight_bps: u64,              // 最大权重
    pub sector_limits: Vec<SectorLimit>,  // 行业限制
    pub turnover_limit_bps: u64,          // 换手率上限
}

/// 行业限制结构体
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct SectorLimit {
    pub sector_id: u32,                   // 行业ID
    pub max_weight_bps: u64,              // 最大权重
}

/// 交易成本模型结构体
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct TransactionCostModel {
    pub fixed_cost: u64,                  // 固定成本
    pub proportional_cost_bps: u64,       // 比例成本
    pub market_impact_model: MarketImpactModel, // 市场冲击模型
}

/// 市场冲击模型枚举
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub enum MarketImpactModel {
    Linear,       // 线性模型
    SquareRoot,   // 平方根模型
    Logarithmic,  // 对数模型
}

/// 算法工厂结构体，支持算法注册、获取、默认算法管理
pub struct AlgorithmFactory {
    algorithms: RwLock<HashMap<String, Arc<dyn ExecutionStrategy + Send + Sync>>>, // 算法注册表
    default: RwLock<Option<String>>, // 默认算法名称
}

impl AlgorithmFactory {
    /// 创建新工厂实例
    pub fn new() -> Self {
        Self {
            algorithms: RwLock::new(HashMap::new()), // 初始化注册表
            default: RwLock::new(None), // 默认算法为空
        }
    }
    /// 注册算法
    pub fn register(&self, name: &str, algo: Arc<dyn ExecutionStrategy + Send + Sync>) {
        self.algorithms.write().unwrap().insert(name.to_string(), algo); // 注册算法实例
    }
    /// 获取算法
    pub fn get(&self, name: &str) -> Option<Arc<dyn ExecutionStrategy + Send + Sync>> {
        self.algorithms.read().unwrap().get(name).cloned() // 获取算法实例
    }
    /// 列出所有已注册算法名称
    pub fn list(&self) -> Vec<String> {
        self.algorithms.read().unwrap().keys().cloned().collect() // 返回所有算法名称
    }
    /// 设置默认算法
    pub fn set_default(&self, name: &str) {
        *self.default.write().unwrap() = Some(name.to_string()); // 设置默认算法名称
    }
    /// 获取默认算法
    pub fn get_default(&self) -> Option<Arc<dyn ExecutionStrategy + Send + Sync>> {
        let name = self.default.read().unwrap();
        name.as_ref().and_then(|n| self.get(n)) // 获取默认算法实例
    }
} 