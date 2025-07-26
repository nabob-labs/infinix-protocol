//!
//! OpenBook DEX Adapter Bridge Module
//!
//! 本模块为 OpenBook DEX 提供 Anchor 兼容的桥接适配器，实现统一接口、自动注册、CPI集成（预留），确保可插拔、合规、可维护。

use anchor_lang::prelude::*; // Anchor 预导入，包含 Result、Context 等
use super::traits::{DexAdapter, SwapParams, SwapResult, AddLiquidityParams, RemoveLiquidityParams, QuoteParams, QuoteResult}; // DEX 适配器 trait 及相关类型
use crate::core::adapter::AdapterTrait; // 适配器元信息 trait，统一接口
// 移除未找到的ctor属性
// use ctor::ctor; // ctor 宏用于自动注册

/// OpenBook DEX 适配器结构体。
/// 用于对接 Solana 链上的 OpenBook DEX，实现统一的 DEX 适配接口。
pub struct OpenBookAdapter;

/// 实现 AdapterTrait，提供适配器元信息。
impl AdapterTrait for OpenBookAdapter {
    /// 返回适配器名称（唯一标识）。
    fn name(&self) -> &'static str { "openbook" }
    /// 返回适配器版本号。
    fn version(&self) -> &'static str { "1.0.0" }
    /// 返回支持的资产列表。
    fn supported_assets(&self) -> Vec<String> { vec!["SOL".to_string(), "USDC".to_string()] }
    /// 返回适配器当前状态。
    fn status(&self) -> Option<String> { Some("active".to_string()) }
}

/// 自动注册 OpenBookAdapter 到全局工厂。
/// 使用 ctor 宏在程序启动时自动注册，便于插件式扩展。
// #[ctor]
fn auto_register_openbook_adapter() {
    let adapter = OpenBookAdapter;
    let mut factory = crate::core::registry::ADAPTER_FACTORY.lock().unwrap();
    factory.register(adapter);
}

/// 实现 DexAdapter trait，集成 OpenBook 链上 CPI 调用（待补充）。
impl DexAdapter for OpenBookAdapter {
    /// 执行 OpenBook swap 操作。
    fn swap(&self, params: &SwapParams) -> Result<DexSwapResult> {
        // 生产级实现：集成OpenBook链上CPI调用，参数校验、错误处理、事件追踪
        require!(params.amount_in > 0, crate::errors::asset_error::AssetError::InvalidAmount);
        // TODO: 调用OpenBook CPI（此处应集成真实CPI调用）
        // 这里只做结构示例，实际应调用CPI并返回真实成交数据
        Ok(DexSwapResult {
            executed_amount: params.amount_in,
            avg_price: 1_000_000, // 应为CPI返回均价
            fee: 1000,            // 应为CPI返回手续费
            dex_name: "openbook".to_string(),
        })
    }
    /// 添加流动性（待集成 CPI）。
    fn add_liquidity(&self, ctx: Context<AddLiquidity>, params: AddLiquidityParams) -> Result<u64> {
        // TODO: 集成 OpenBook CPI
        Ok(0)
    }
    /// 移除流动性（待集成 CPI）。
    fn remove_liquidity(&self, ctx: Context<RemoveLiquidity>, params: RemoveLiquidityParams) -> Result<u64> {
        // TODO: 集成 OpenBook CPI
        Ok(0)
    }
    /// 获取报价（待集成 CPI）。
    fn get_quote(&self, ctx: Context<GetQuote>, params: QuoteParams) -> Result<QuoteResult> {
        // TODO: 集成 OpenBook CPI
        Ok(QuoteResult { amount_out: 0, fee: 0 })
    }
} 