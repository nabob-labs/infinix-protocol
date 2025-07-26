//!
//! strategy.rs - 篮子策略相关类型定义
//!
//! 本文件定义所有与篮子相关的策略结构体与枚举，严格遵循Rust、Solana、Anchor、SOLID最佳实践，
//! 并逐行专业注释，便于审计、维护、扩展。

use anchor_lang::prelude::*;

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

/// 执行策略
#[derive(Debug, Clone)]
pub struct ExecutionStrategy {
    /// 执行类型
    pub execution_type: ExecutionType,
    /// 批量执行大小
    pub batch_size: u64,
    /// 执行时间范围
    pub time_horizon: i64,
    /// 滑点容忍度
    pub slippage_tolerance: u16,
    /// 成分订单
    pub constituent_orders: Vec<ConstituentOrder>,
}

/// 执行类型
#[derive(Debug, Clone)]
pub enum ExecutionType {
    /// 市价执行
    Market,
    /// 限价执行
    Limit,
    /// TWAP执行
    TWAP,
    /// VWAP执行
    VWAP,
    /// 最优执行
    Optimal,
}

/// 成分订单
#[derive(Debug, Clone)]
pub struct ConstituentOrder {
    /// 代币mint
    pub mint: Pubkey,
    /// 订单数量
    pub amount: u64,
    /// 订单类型
    pub order_type: OrderType,
    /// 限价（可选）
    pub price_limit: Option<u64>,
    /// 有效时间类型
    pub time_in_force: TimeInForce,
}

/// 订单类型
#[derive(Debug, Clone)]
pub enum OrderType {
    /// 市价单
    Market,
    /// 限价单
    Limit,
    /// 止损单
    Stop,
    /// 止损限价单
    StopLimit,
}

/// 有效时间类型
#[derive(Debug, Clone)]
pub enum TimeInForce {
    /// GTC - 撤销前有效
    GTC,
    /// IOC - 立即成交或取消
    IOC,
    /// FOK - 全部成交否则取消
    FOK,
    /// GTD - 指定日期前有效
    GTD(i64),
}

/// 赎回策略
#[derive(Debug, Clone)]
pub struct RedemptionStrategy {
    /// 赎回类型
    pub redemption_type: RedemptionType,
    /// 执行策略
    pub execution_strategy: ExecutionStrategy,
    /// 启用优化
    pub optimization_enabled: bool,
}

/// 赎回类型
#[derive(Debug, Clone)]
pub enum RedemptionType {
    /// 按比例赎回
    ProRata,
    /// 优化赎回
    Optimized,
    /// 自定义赎回
    Custom,
}

/// 再平衡策略
#[derive(Debug, Clone)]
pub struct RebalancingStrategy {
    /// 需要的交易
    pub trades: Vec<RebalancingTrade>,
    /// 执行方法
    pub execution_method: ExecutionMethod,
    /// 风险限制
    pub risk_limits: RiskLimits,
}

/// 再平衡交易
#[derive(Debug, Clone)]
pub struct RebalancingTrade {
    /// 代币mint
    pub mint: Pubkey,
    /// 交易类型
    pub trade_type: TradeType,
    /// 交易数量
    pub amount: u64,
    /// 紧急程度
    pub urgency: TradeUrgency,
}

/// 交易类型
#[derive(Debug, Clone)]
pub enum TradeType {
    /// 买入
    Buy,
    /// 卖出
    Sell,
}

/// 交易紧急程度
#[derive(Debug, Clone)]
pub enum TradeUrgency {
    /// 低
    Low,
    /// 中
    Medium,
    /// 高
    High,
    /// 紧急
    Critical,
}

/// 套利机会
#[derive(Debug, Clone)]
pub struct ArbitrageOpportunity {
    /// 套利类型
    pub opportunity_type: ArbitrageType,
    /// 当前净值
    pub nav: u64,
    /// 市场价格
    pub market_price: u64,
    /// 利润（基点）
    pub profit_bps: u64,
    /// 最大可用规模
    pub size_limit: u64,
    /// 置信度
    pub confidence: u32,
}

/// 套利类型
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub enum ArbitrageType {
    /// 创建套利
    Creation,
    /// 赎回套利
    Redemption,
    /// 跨协议套利
    CrossProtocol,
    /// 统计套利
    Statistical,
} 