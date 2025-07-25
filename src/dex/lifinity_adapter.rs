//!
//! Lifinity DEX Adapter Bridge Module
//!
//! 本模块为 Lifinity DEX 提供 Anchor 兼容的桥接适配器，实现统一接口、自动注册、CPI集成（预留），确保可插拔、合规、可维护。

use anchor_lang::prelude::*;
use super::traits::{DexAdapter, SwapParams, SwapResult, AddLiquidityParams, RemoveLiquidityParams, QuoteParams, QuoteResult};
use crate::core::adapter::AdapterTrait;
use ctor::ctor;

/// Lifinity DEX 适配器结构体。
/// 用于对接 Solana 链上的 Lifinity DEX，实现统一的 DEX 适配接口。
pub struct LifinityAdapter;

/// 实现 AdapterTrait，提供适配器元信息。
impl AdapterTrait for LifinityAdapter {
    /// 返回适配器名称。
    fn name(&self) -> &'static str { "lifinity" }
    /// 返回适配器版本号。
    fn version(&self) -> &'static str { "1.0.0" }
    /// 返回支持的资产列表。
    fn supported_assets(&self) -> Vec<String> { vec!["SOL".to_string(), "USDC".to_string()] }
    /// 返回适配器当前状态。
    fn status(&self) -> Option<String> { Some("active".to_string()) }
}

/// 自动注册 LifinityAdapter 到全局工厂。
/// 使用 ctor 宏在程序启动时自动注册，便于插件式扩展。
#[ctor]
fn auto_register_lifinity_adapter() {
    let adapter = LifinityAdapter;
    let mut factory = crate::core::registry::ADAPTER_FACTORY.lock().unwrap();
    factory.register(adapter);
}

/// 实现 DexAdapter trait，集成 Lifinity 链上 CPI 调用（待补充）。
impl DexAdapter for LifinityAdapter {
    /// 执行 Lifinity swap 操作（待集成 CPI）。
    fn swap(&self, ctx: Context<Swap>, params: SwapParams) -> Result<SwapResult> {
        // TODO: 集成 Lifinity CPI
        Ok(SwapResult { amount_out: 0, fee: 0 })
    }
    /// 添加流动性（待集成 CPI）。
    fn add_liquidity(&self, ctx: Context<AddLiquidity>, params: AddLiquidityParams) -> Result<u64> {
        // TODO: 集成 Lifinity CPI
        Ok(0)
    }
    /// 移除流动性（待集成 CPI）。
    fn remove_liquidity(&self, ctx: Context<RemoveLiquidity>, params: RemoveLiquidityParams) -> Result<u64> {
        // TODO: 集成 Lifinity CPI
        Ok(0)
    }
    /// 获取报价（待集成 CPI）。
    fn get_quote(&self, ctx: Context<GetQuote>, params: QuoteParams) -> Result<QuoteResult> {
        // TODO: 集成 Lifinity CPI
        Ok(QuoteResult { amount_out: 0, fee: 0 })
    }
} 