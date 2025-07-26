//!
//! types.rs - 篮子相关类型定义
//!
//! 本文件定义所有与篮子相关的核心类型，严格遵循Rust、Solana、Anchor、SOLID最佳实践，
//! 并逐行专业注释，便于审计、维护、扩展。

use anchor_lang::prelude::*;
use crate::state::baskets::{BasketConstituent, BasketIndexState};

/// 篮子账户结构体，封装所有篮子相关状态
/// - 统一管理篮子资产、状态、策略等信息
/// - 复用BasketIndexState，便于跨模块集成
#[account]
pub struct Basket {
    /// 统一篮子状态，复用BasketIndexState
    pub state: BasketIndexState,
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