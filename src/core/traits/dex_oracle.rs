//!
//! dex_oracle.rs - DEX/Oracle/Liquidity/Slippage等接口Trait定义
//!
//! 本文件定义了DEX、Oracle、流动性、滑点等相关接口Trait，严格遵循Rust、Anchor、SOLID最佳实践，
//! 并逐行专业注释，便于审计、维护、扩展。

use anchor_lang::prelude::*;
use crate::core::traits::types::{DexTradeResult, DexSide, MarketDataType};
use std::fmt::Debug;

/// 价格预言机接口Trait
pub trait PriceOracle: Send + Sync {
    /// 获取指定token的最新价格（单位：最小计价单位）
    fn get_price(&self, token_mint: Pubkey) -> anchor_lang::Result<u64>;
}

/// DEX客户端接口Trait
pub trait DexClient: Send + Sync {
    /// 市价单撮合
    fn market_order(&self, token_mint: Pubkey, amount: u64, side: DexSide) -> anchor_lang::Result<DexTradeResult>;
    /// 限价单撮合
    fn limit_order(&self, token_mint: Pubkey, amount: u64, price: u64, side: DexSide) -> anchor_lang::Result<DexTradeResult>;
}

/// 流动性源接口Trait
pub trait LiquiditySource: Send + Sync {
    /// 查询指定token的可用流动性
    fn get_liquidity(&self, token_mint: Pubkey) -> anchor_lang::Result<u64>;
    /// 查询所有支持token的流动性
    fn get_all_liquidity(&self) -> anchor_lang::Result<Vec<(Pubkey, u64)>>;
}

/// 滑点模型接口Trait
pub trait SlippageModel: Send + Sync + Debug {
    /// 估算滑点（返回bps）
    fn estimate_slippage(&self, amount: u64, price: u64, liquidity: u64) -> u64;
}

/// 市场冲击模型接口Trait
pub trait MarketImpactModel: Send + Sync + Debug {
    /// 估算价格冲击（返回bps）
    fn estimate_impact(&self, amount: u64, liquidity: u64) -> u64;
} 