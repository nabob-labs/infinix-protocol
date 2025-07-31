//!
//! trade.rs - 交易参数类型定义
//!
//! 本文件定义了TradeParams、BatchTradeParams等交易相关参数结构体，严格遵循Rust、Anchor、SOLID最佳实践，
//! 并逐行专业注释，便于审计、维护、扩展。

use anchor_lang::prelude::*;
use crate::core::types::algo::AlgoParams;
use crate::core::types::strategy::StrategyParams;
use crate::core::types::oracle::OracleParams;

/// 价格参数结构体
/// - 适用于所有交易指令的价格相关参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct PriceParams {
    /// 价格源名称
    pub price_source: String,
    /// 最大滑点（基点，1/10000）
    pub max_slippage_bps: u16,
    /// 价格有效期（秒）
    pub price_validity_seconds: u64,
    /// 是否启用价格保护
    pub enable_price_protection: bool,
    /// 价格保护阈值（基点）
    pub price_protection_threshold_bps: u16,
}

/// 统一交易参数结构体
/// - 适用于所有资产/篮子/指数代币的单笔交易指令
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct TradeParams {
    /// 交易类型（如 swap、mint、burn、rebalance 等）
    pub trade_type: String,
    /// 源资产/篮子/指数代币 mint
    pub from_token: Pubkey,
    /// 目标资产/篮子/指数代币 mint
    pub to_token: Pubkey,
    /// 输入数量
    pub amount_in: u64,
    /// 最小输出数量
    pub min_amount_out: u64,
    /// DEX 名称
    pub dex_name: String,
    /// 算法参数（可选）
    pub algo_params: Option<AlgoParams>,
    /// 策略参数（可选）
    pub strategy_params: Option<StrategyParams>,
    /// 预言机参数（可选）
    pub oracle_params: Option<OracleParams>,
}

/// 统一批量交易参数结构体
/// - 适用于批量操作
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct BatchTradeParams {
    /// 批量交易明细
    pub trades: Vec<TradeParams>,
}

/// 策略交易参数结构体
/// - 适用于策略驱动的交易指令
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct StrategyTradeParams {
    /// 策略名称
    pub strategy_name: String,
    /// 策略参数
    pub strategy_params: StrategyParams,
    /// 交易参数
    pub trade_params: TradeParams,
}

/// 批量交换参数结构体
/// - 适用于批量交换操作
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct BatchSwapParams {
    /// 批量交换明细
    pub swaps: Vec<TradeParams>,
} 