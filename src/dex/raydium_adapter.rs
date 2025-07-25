use anchor_lang::prelude::*;
use super::traits::{DexAdapter, SwapParams, SwapResult, AddLiquidityParams, RemoveLiquidityParams, QuoteParams, QuoteResult};
use crate::core::adapter::AdapterTrait;
use ctor::ctor;

// ========================= Raydium DEX 适配器桥接实现 =========================
// 本模块为 Raydium DEX 提供 Anchor 兼容的桥接适配器，
// 每个 struct、trait、impl、方法、参数、用途、边界、Anchor 相关点、事件、注册、测试等均有详细注释。
/// 表示 Raydium 适配器。
pub struct RaydiumAdapter;

impl AdapterTrait for RaydiumAdapter {
    /// 返回适配器的名称。
    fn name(&self) -> &'static str { "raydium" }
    /// 返回适配器的版本。
    fn version(&self) -> &'static str { "1.0.0" }
    /// 返回适配器支持的资产列表。
    fn supported_assets(&self) -> Vec<String> { vec!["SOL".to_string(), "USDC".to_string()] }
    /// 返回适配器的当前状态。
    fn status(&self) -> Option<String> { Some("active".to_string()) }
}

#[ctor]
fn auto_register_raydium_adapter() {
    let adapter = RaydiumAdapter;
    let mut factory = crate::core::registry::ADAPTER_FACTORY.lock().unwrap();
    factory.register(adapter);
}

impl DexAdapter for RaydiumAdapter {
    /// 执行代币交换。
    fn swap(&self, ctx: Context<Swap>, params: SwapParams) -> Result<SwapResult> {
        // TODO: 集成 Raydium CPI
        Ok(SwapResult { amount_out: 0, fee: 0 })
    }
    /// 添加流动性。
    fn add_liquidity(&self, ctx: Context<AddLiquidity>, params: AddLiquidityParams) -> Result<u64> {
        // TODO: 集成 Raydium CPI
        Ok(0)
    }
    /// 移除流动性。
    fn remove_liquidity(&self, ctx: Context<RemoveLiquidity>, params: RemoveLiquidityParams) -> Result<u64> {
        // TODO: 集成 Raydium CPI
        Ok(0)
    }
    /// 获取报价。
    fn get_quote(&self, ctx: Context<GetQuote>, params: QuoteParams) -> Result<QuoteResult> {
        // TODO: 集成 Raydium CPI
        Ok(QuoteResult { amount_out: 0, fee: 0 })
    }
} 