pub mod basket_manager;
pub mod enhanced_trading_engine;
pub mod execution_optimizer;
pub mod liquidity_aggregator;
pub mod risk_manager;
pub mod trading_engine;

// Re-export key types from sub-modules
pub use basket_manager::*;
pub use enhanced_trading_engine::*;
pub use execution_optimizer::*;
pub use liquidity_aggregator::*;
pub use risk_manager::*;
pub use trading_engine::*;

use crate::core::constants::MAX_TOKENS;
use anchor_lang::prelude::*;
// Removed conflicting borsh import

/// Core basket trading strategy types and utilities
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, PartialEq)]
pub enum BasketTradingStrategy {
    /// Create basket by buying constituent tokens
    Creation,
    /// Redeem basket by selling constituent tokens
    Redemption,
    /// Arbitrage between basket and constituent prices
    Arbitrage,
    /// Rebalance existing basket holdings
    Rebalancing,
}

/// Basket composition definition
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, InitSpace)]
pub struct BasketComposition {
    /// Basket identifier
    pub basket_id: u64,
    /// Token mints and their target weights
    #[max_len(MAX_TOKENS)]
    pub constituents: Vec<BasketConstituent>,
    /// Total basket supply
    pub total_supply: u64,
    /// Minimum creation/redemption amount
    pub min_amount: u64,
    /// Creation/redemption fee (basis points)
    pub fee_bps: u16,
    /// Last update timestamp
    pub last_updated: i64,
}

/// Individual basket constituent
/// Updated for Anchor 0.32 with InitSpace trait implementation
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, InitSpace)]
pub struct BasketConstituent {
    /// Token mint address
    pub mint: Pubkey,
    /// Target weight in basis points (10000 = 100%)
    pub weight: u64,
    /// Current balance in the basket
    pub balance: u64,
    /// Minimum trade size for this token
    pub min_trade_size: u64,
    /// Liquidity pool address for trading
    pub pool_address: Pubkey,
}

/// Trading execution parameters
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct ExecutionParams {
    /// Maximum slippage tolerance (basis points)
    pub max_slippage: u16,
    /// Maximum price impact (basis points)
    pub max_price_impact: u16,
    /// Execution deadline (unix timestamp)
    pub deadline: i64,
    /// Whether to use partial fills
    pub allow_partial_fill: bool,
    /// Minimum fill percentage (basis points)
    pub min_fill_percentage: u16,
}

/// Basket trading result
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct BasketTradeResult {
    /// Trade execution ID
    pub execution_id: u64,
    /// Actual tokens received/sent
    pub token_amounts: Vec<TokenAmount>,
    /// Total execution cost
    pub total_cost: u64,
    /// Average slippage experienced
    pub avg_slippage: u16,
    /// Execution timestamp
    pub executed_at: i64,
    /// Whether trade was fully executed
    pub fully_executed: bool,
}

/// Token amount for trade results
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct TokenAmount {
    /// Token mint
    pub mint: Pubkey,
    /// Amount traded
    pub amount: u64,
    /// Price at execution
    pub execution_price: u64,
    /// Slippage experienced
    pub slippage: u16,
}

// Additional type definitions for basket operations

/// Optimization configuration for basket operations
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

/// Execution methods
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub enum ExecutionMethod {
    /// Immediate execution
    Immediate,
    /// Gradual execution over time
    Gradual,
    /// Optimal execution algorithm
    Optimal,
    /// Custom execution strategy
    Custom,
}

/// Risk limits configuration
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct RiskLimits {
    /// Maximum slippage tolerance
    pub max_slippage_bps: u16,
    /// Maximum price impact
    pub max_price_impact_bps: u16,
    /// Maximum position concentration
    pub max_concentration_bps: u16,
    /// VaR limit
    pub var_limit: u64,
}

/// Market conditions snapshot
#[derive(Debug, Clone)]
pub struct MarketConditions {
    /// Token liquidity mapping
    pub token_liquidity: std::collections::HashMap<Pubkey, u64>,
    /// Market volatility index
    pub volatility_index: u32,
    /// Current gas price
    pub gas_price: u64,
    /// Market timestamp
    pub timestamp: i64,
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
    pub composition: BasketComposition,
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
