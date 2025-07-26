//!
//! result.rs - 篮子操作结果类型定义
//!
//! 本文件定义所有与篮子相关的操作结果结构体，严格遵循Rust、Solana、Anchor、SOLID最佳实践，
//! 并逐行专业注释，便于审计、维护、扩展。

use anchor_lang::prelude::*;
use crate::basket::strategy::ArbitrageType;
use crate::state::baskets::BasketConstituent;

/// 篮子交易结果
/// - 记录一次篮子交易的执行明细
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct BasketTradeResult {
    /// 交易执行ID
    pub execution_id: u64,
    /// 实际收到/发出的代币数量
    pub token_amounts: Vec<crate::basket::types::TokenAmount>,
    /// 总执行成本
    pub total_cost: u64,
    /// 平均滑点
    pub avg_slippage: u16,
    /// 执行时间戳
    pub executed_at: i64,
    /// 是否完全成交
    pub fully_executed: bool,
}

/// 篮子创建结果
#[derive(Debug, Clone)]
pub struct BasketCreationResult {
    /// 创建的篮子ID
    pub basket_id: u64,
    /// 创建的代币数量
    pub tokens_created: u64,
    /// 执行成本
    pub execution_cost: u64,
    /// 滑点
    pub slippage_experienced: u16,
    /// Gas消耗
    pub gas_used: u64,
    /// 执行时间（毫秒）
    pub execution_time: u64,
    /// 是否成功
    pub success: bool,
}

/// 篮子赎回结果
#[derive(Debug, Clone)]
pub struct BasketRedemptionResult {
    /// 篮子ID
    pub basket_id: u64,
    /// 赎回的代币数量
    pub tokens_redeemed: u64,
    /// 执行结果
    pub execution_result: BasketCreationResult,
}

/// 再平衡结果
#[derive(Debug, Clone)]
pub struct RebalancingResult {
    /// 篮子ID
    pub basket_id: u64,
    /// 执行的交易数
    pub trades_executed: u32,
    /// 总成本
    pub total_cost: u64,
    /// 平均滑点
    pub average_slippage: u16,
    /// 执行时间
    pub execution_time: u64,
    /// 成功率
    pub success_rate: u16,
}

/// 套利结果
#[derive(Debug, Clone)]
pub struct ArbitrageResult {
    /// 篮子ID
    pub basket_id: u64,
    /// 套利类型
    pub opportunity_type: ArbitrageType,
    /// 执行数量
    pub execution_amount: u64,
    /// 实现利润
    pub profit_realized: u64,
    /// 执行成本
    pub execution_cost: u64,
    /// 净利润
    pub net_profit: u64,
    /// 执行时间
    pub execution_time: u64,
}

/// 优化篮子结果
#[derive(Debug, Clone)]
pub struct OptimizedBasketResult {
    /// 篮子ID
    pub basket_id: u64,
    /// 优化后的成分
    pub composition: Vec<BasketConstituent>,
    /// 执行结果
    pub execution_result: BasketCreationResult,
    /// 风险指标
    pub risk_metrics: crate::basket::metrics::RiskAssessment,
    /// 优化指标
    pub optimization_metrics: crate::basket::metrics::OptimizationMetrics,
}

/// 优化赎回结果
#[derive(Debug, Clone)]
pub struct OptimizedRedemptionResult {
    /// 篮子ID
    pub basket_id: u64,
    /// 赎回结果
    pub redemption_result: BasketRedemptionResult,
    /// 风险指标
    pub risk_metrics: crate::basket::metrics::RiskAssessment,
    /// 优化指标
    pub optimization_metrics: crate::basket::metrics::OptimizationMetrics,
} 