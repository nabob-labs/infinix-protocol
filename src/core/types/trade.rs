//!
//! trade.rs - 交易参数类型定义
//!
//! 本文件定义了TradeParams、BatchTradeParams等交易相关参数结构体，严格遵循Rust、Anchor、SOLID最佳实践，
//! 并逐行专业注释，便于审计、维护、扩展。

use anchor_lang::prelude::*;
use crate::core::types::algo::AlgoParams;
use crate::core::types::strategy::StrategyParams;
use crate::core::types::oracle::OracleParams;

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