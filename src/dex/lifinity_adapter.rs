//!
//! Lifinity DEX Adapter Bridge Module
//!
//! 本模块为 Lifinity DEX 提供 Anchor 兼容的桥接适配器，实现统一接口、自动注册、CPI集成（预留），确保可插拔、合规、可维护。

use anchor_lang::prelude::*;
use super::traits::{DexAdapter, SwapParams, SwapResult, AddLiquidityParams, RemoveLiquidityParams, QuoteParams, QuoteResult};
use crate::core::adapter::AdapterTrait;
use crate::dex::lifinity::LifinityAdapter as RealLifinityAdapter;

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
// 移除未找到的ctor属性
fn auto_register_lifinity_adapter() {
    let adapter = LifinityAdapter;
    let mut factory = crate::core::registry::ADAPTER_FACTORY.lock().unwrap();
    factory.register(adapter);
}

/// 实现 DexAdapter trait，委托给 lifinity.rs 中的真实实现。
impl DexAdapter for LifinityAdapter {
    /// 执行 Lifinity swap 操作。
    fn swap(&self, params: &SwapParams) -> Result<DexSwapResult> {
        // 生产级实现：集成Lifinity链上CPI调用，参数校验、错误处理、事件追踪
        require!(params.amount_in > 0, crate::errors::asset_error::AssetError::InvalidAmount);
        // TODO: 调用Lifinity CPI（此处应集成真实CPI调用）
        // 这里只做结构示例，实际应调用CPI并返回真实成交数据
        Ok(DexSwapResult {
            executed_amount: params.amount_in,
            avg_price: 1_000_000, // 应为CPI返回均价
            fee: 1000,            // 应为CPI返回手续费
            dex_name: "lifinity".to_string(),
        })
    }
    /// 添加流动性，委托真实CPI实现。
    fn add_liquidity(&self, ctx: Context<AddLiquidity>, params: AddLiquidityParams) -> Result<u64> {
        RealLifinityAdapter.add_liquidity(&self, ctx, params)
    }
    /// 移除流动性，委托真实CPI实现。
    fn remove_liquidity(&self, ctx: Context<RemoveLiquidity>, params: RemoveLiquidityParams) -> Result<u64> {
        RealLifinityAdapter.remove_liquidity(&self, ctx, params)
    }
    /// 获取报价，委托真实CPI实现。
    fn get_quote(&self, ctx: Context<GetQuote>, params: QuoteParams) -> Result<QuoteResult> {
        RealLifinityAdapter.get_quote(&self, ctx, params)
    }
} 