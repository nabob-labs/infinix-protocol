use crate::core::constants::MAX_TOKENS;
use anchor_lang::prelude::*;
// Removed conflicting borsh import

use crate::state::baskets::{BasketConstituent, BasketIndexState, BasketStatus};

// 引入统一的策略/参数类型定义
use crate::algorithms::execution_optimizer::types::*;

// === 统一账户结构，Basket结构直接引用BasketIndexState ===
#[account]
/// 篮子账户结构体，封装所有篮子相关状态
/// - 统一管理篮子资产、状态、策略等信息
/// - 复用BasketIndexState，便于跨模块集成
pub struct Basket {
    /// 统一篮子状态，复用BasketIndexState
    pub state: BasketIndexState,
}

// BasketComposition、BasketConstituent等类型已统一到state/baskets.rs，去除本地重复定义
// 相关方法、事件、配置等全部依赖BasketIndexState及其相关类型

// 其余策略、配置、执行等类型如需保留，建议迁移到统一的algorithms/traits.rs或state/baskets.rs中

/// 篮子交易策略类型
/// - 定义篮子相关的主要交易场景
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, PartialEq)]
pub enum BasketTradingStrategy {
    /// 通过买入成分代币创建篮子
    Creation,
    /// 通过卖出成分代币赎回篮子
    Redemption,
    /// 利用篮子与成分价格套利
    Arbitrage,
    /// 对现有篮子持仓再平衡
    Rebalancing,
}

/// 交易执行参数
/// - 控制篮子交易的滑点、价格冲击、成交方式等
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct ExecutionParams {
    /// 最大滑点容忍度（基点）
    pub max_slippage: u16,
    /// 最大价格冲击（基点）
    pub max_price_impact: u16,
    /// 执行截止时间（unix时间戳）
    pub deadline: i64,
    /// 是否允许部分成交
    pub allow_partial_fill: bool,
    /// 最小成交百分比（基点）
    pub min_fill_percentage: u16,
}

/// 篮子交易结果
/// - 记录一次篮子交易的执行明细
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct BasketTradeResult {
    /// 交易执行ID
    pub execution_id: u64,
    /// 实际收到/发出的代币数量
    pub token_amounts: Vec<TokenAmount>,
    /// 总执行成本
    pub total_cost: u64,
    /// 平均滑点
    pub avg_slippage: u16,
    /// 执行时间戳
    pub executed_at: i64,
    /// 是否完全成交
    pub fully_executed: bool,
}

/// 代币交易结果明细
/// - 记录每个成分代币的成交数量、价格、滑点
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct TokenAmount {
    /// 代币mint
    pub mint: Pubkey,
    /// 交易数量
    pub amount: u64,
    /// 执行价格
    pub execution_price: u64,
    /// 实际滑点
    pub slippage: u16,
}

// Additional type definitions for basket operations

/// Optimization configuration for basket operations
/// - 控制篮子操作的优化算法、并行度等
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct OptimizationConfig {
    /// Enable optimization features
    pub enabled: bool,
    /// Use genetic algorithm for optimization
    pub use_genetic_algorithm: bool,
    /// Maximum optimization iterations
    pub max_iterations: u32,
    /// Convergence threshold
    pub convergence_threshold: u64,
    /// Enable parallel processing
    pub enable_parallel: bool,
}

/// Arbitrage configuration
/// - 控制套利操作的最小利润、最大仓位、超时等
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct ArbitrageConfig {
    /// Minimum profit in basis points
    pub min_profit_bps: u16,
    /// Maximum position size
    pub max_position_size: u64,
    /// Execution timeout in seconds
    pub execution_timeout: u32,
    /// Enable cross-protocol arbitrage
    pub enable_cross_protocol: bool,
}

/// Rebalancing configuration
/// - 控制再平衡操作的执行方式、风险限制等
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct RebalancingConfig {
    /// Execution method for rebalancing
    pub execution_method: ExecutionMethod,
    /// Risk limits for rebalancing
    pub risk_limits: RiskLimits,
    /// Enable gradual rebalancing
    pub enable_gradual: bool,
    /// Rebalancing frequency limit
    pub frequency_limit: u32,
}

/// Execution strategy for basket operations
#[derive(Debug, Clone)]
pub struct ExecutionStrategy {
    /// Type of execution
    pub execution_type: ExecutionType,
    /// Batch size for execution
    pub batch_size: u64,
    /// Time horizon for execution
    pub time_horizon: i64,
    /// Slippage tolerance
    pub slippage_tolerance: u16,
    /// Individual constituent orders
    pub constituent_orders: Vec<ConstituentOrder>,
}

/// Execution types
#[derive(Debug, Clone)]
pub enum ExecutionType {
    /// Market execution
    Market,
    /// Limit execution
    Limit,
    /// TWAP execution
    TWAP,
    /// VWAP execution
    VWAP,
    /// Optimal execution
    Optimal,
}

/// Individual constituent order
#[derive(Debug, Clone)]
pub struct ConstituentOrder {
    /// Token mint
    pub mint: Pubkey,
    /// Order amount
    pub amount: u64,
    /// Order type
    pub order_type: OrderType,
    /// Price limit (if applicable)
    pub price_limit: Option<u64>,
    /// Time in force
    pub time_in_force: TimeInForce,
}

/// Order types
#[derive(Debug, Clone)]
pub enum OrderType {
    /// Market order
    Market,
    /// Limit order
    Limit,
    /// Stop order
    Stop,
    /// Stop-limit order
    StopLimit,
}

/// Time in force options
#[derive(Debug, Clone)]
pub enum TimeInForce {
    /// Good till cancelled
    GTC,
    /// Immediate or cancel
    IOC,
    /// Fill or kill
    FOK,
    /// Good till date
    GTD(i64),
}

/// Redemption strategy
#[derive(Debug, Clone)]
pub struct RedemptionStrategy {
    /// Type of redemption
    pub redemption_type: RedemptionType,
    /// Execution strategy
    pub execution_strategy: ExecutionStrategy,
    /// Enable optimization
    pub optimization_enabled: bool,
}

/// Redemption types
#[derive(Debug, Clone)]
pub enum RedemptionType {
    /// Pro-rata redemption
    ProRata,
    /// Optimized redemption
    Optimized,
    /// Custom redemption
    Custom,
}

/// Rebalancing strategy
#[derive(Debug, Clone)]
pub struct RebalancingStrategy {
    /// Required trades
    pub trades: Vec<RebalancingTrade>,
    /// Execution method
    pub execution_method: ExecutionMethod,
    /// Risk limits
    pub risk_limits: RiskLimits,
}

/// Individual rebalancing trade
#[derive(Debug, Clone)]
pub struct RebalancingTrade {
    /// Token mint
    pub mint: Pubkey,
    /// Trade type
    pub trade_type: TradeType,
    /// Trade amount
    pub amount: u64,
    /// Trade urgency
    pub urgency: TradeUrgency,
}

/// Trade types
#[derive(Debug, Clone)]
pub enum TradeType {
    /// Buy order
    Buy,
    /// Sell order
    Sell,
}

/// Trade urgency levels
#[derive(Debug, Clone)]
pub enum TradeUrgency {
    /// Low urgency
    Low,
    /// Medium urgency
    Medium,
    /// High urgency
    High,
    /// Critical urgency
    Critical,
}

/// Arbitrage opportunity
#[derive(Debug, Clone)]
pub struct ArbitrageOpportunity {
    /// Type of arbitrage
    pub opportunity_type: ArbitrageType,
    /// Current NAV
    pub nav: u64,
    /// Market price
    pub market_price: u64,
    /// Profit in basis points
    pub profit_bps: u64,
    /// Maximum size for this opportunity
    pub size_limit: u64,
    /// Confidence level
    pub confidence: u32,
}

/// Arbitrage types
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub enum ArbitrageType {
    /// Creation arbitrage
    Creation,
    /// Redemption arbitrage
    Redemption,
    /// Cross-protocol arbitrage
    CrossProtocol,
    /// Statistical arbitrage
    Statistical,
}

/// Risk assessment results
#[derive(Debug, Clone)]
pub struct RiskAssessment {
    /// Overall risk score (0-10000)
    pub overall_risk_score: u32,
    /// Liquidity risk
    pub liquidity_risk: u32,
    /// Concentration risk
    pub concentration_risk: u32,
    /// Execution risk
    pub execution_risk: u32,
    /// Market risk
    pub market_risk: u32,
    /// Risk recommendations
    pub recommendations: Vec<String>,
}

/// Basket creation result
#[derive(Debug, Clone)]
pub struct BasketCreationResult {
    /// Created basket ID
    pub basket_id: u64,
    /// Number of tokens created
    pub tokens_created: u64,
    /// Execution cost
    pub execution_cost: u64,
    /// Slippage experienced
    pub slippage_experienced: u16,
    /// Gas used
    pub gas_used: u64,
    /// Execution time in milliseconds
    pub execution_time: u64,
    /// Success flag
    pub success: bool,
}

/// Basket redemption result
#[derive(Debug, Clone)]
pub struct BasketRedemptionResult {
    /// Basket ID
    pub basket_id: u64,
    /// Number of tokens redeemed
    pub tokens_redeemed: u64,
    /// Execution result
    pub execution_result: BasketCreationResult,
}

/// Rebalancing result
#[derive(Debug, Clone)]
pub struct RebalancingResult {
    /// Basket ID
    pub basket_id: u64,
    /// Number of trades executed
    pub trades_executed: u32,
    /// Total cost
    pub total_cost: u64,
    /// Average slippage
    pub average_slippage: u16,
    /// Execution time
    pub execution_time: u64,
    /// Success rate
    pub success_rate: u16,
}

/// Arbitrage execution result
#[derive(Debug, Clone)]
pub struct ArbitrageResult {
    /// Basket ID
    pub basket_id: u64,
    /// Opportunity type
    pub opportunity_type: ArbitrageType,
    /// Execution amount
    pub execution_amount: u64,
    /// Profit realized
    pub profit_realized: u64,
    /// Execution cost
    pub execution_cost: u64,
    /// Net profit
    pub net_profit: u64,
    /// Execution time
    pub execution_time: u64,
}

/// Optimized basket result
#[derive(Debug, Clone)]
pub struct OptimizedBasketResult {
    /// Basket ID
    pub basket_id: u64,
    /// Optimized composition
    pub composition: Vec<BasketConstituent>, // 替换为 Vec<BasketConstituent>
    /// Execution result
    pub execution_result: BasketCreationResult,
    /// Risk metrics
    pub risk_metrics: RiskAssessment,
    /// Optimization metrics
    pub optimization_metrics: OptimizationMetrics,
}

/// Optimized redemption result
#[derive(Debug, Clone)]
pub struct OptimizedRedemptionResult {
    /// Basket ID
    pub basket_id: u64,
    /// Redemption result
    pub redemption_result: BasketRedemptionResult,
    /// Risk metrics
    pub risk_metrics: RiskAssessment,
    /// Optimization metrics
    pub optimization_metrics: OptimizationMetrics,
}

/// Optimization metrics
#[derive(Debug, Clone)]
pub struct OptimizationMetrics {
    /// Gas savings in basis points
    pub gas_savings_bps: u32,
    /// Slippage reduction in basis points
    pub slippage_reduction_bps: u32,
    /// Execution improvement in basis points
    pub execution_improvement_bps: u32,
    /// MEV protection score
    pub mev_protection_score: u32,
}

/// Performance metrics
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    /// Total number of creations
    pub total_creations: u64,
    /// Total number of redemptions
    pub total_redemptions: u64,
    /// Total number of rebalances
    pub total_rebalances: u64,
    /// Total number of arbitrages
    pub total_arbitrages: u64,
    /// Total volume processed
    pub total_volume: u64,
    /// Total profit generated
    pub total_profit: u64,
    /// Average execution time
    pub average_execution_time: u64,
    /// Success rate
    pub success_rate: u16,
}

/// Performance snapshot
#[derive(Debug, Clone)]
pub struct PerformanceSnapshot {
    /// Timestamp
    pub timestamp: i64,
    /// Metrics at this point
    pub metrics: PerformanceMetrics,
    /// Additional context
    pub context: String,
}

/// Risk monitoring configuration
#[derive(Debug, Clone)]
pub struct RiskMonitoringConfig {
    /// Enable real-time monitoring
    pub enable_realtime: bool,
    /// Monitoring frequency in seconds
    pub frequency: u32,
    /// Alert thresholds
    pub alert_thresholds: Vec<AlertThreshold>,
    /// Enable automatic risk mitigation
    pub enable_auto_mitigation: bool,
}

/// Alert threshold configuration
#[derive(Debug, Clone)]
pub struct AlertThreshold {
    /// Metric name
    pub metric: String,
    /// Threshold value
    pub threshold: u64,
    /// Alert severity
    pub severity: AlertSeverity,
    /// Action to take
    pub action: String,
}

/// Alert severity levels
#[derive(Debug, Clone)]
pub enum AlertSeverity {
    /// Information
    Info,
    /// Warning
    Warning,
    /// Error
    Error,
    /// Critical
    Critical,
}
