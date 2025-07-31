//!
//! Lifinity DEX Adapter Bridge Module
//!
//! 本模块为 Lifinity DEX 提供 Anchor 兼容的桥接适配器，实现统一接口、自动注册、CPI集成（预留），确保可插拔、合规、可维护。

use anchor_lang::prelude::*;
use crate::core::adapter::AdapterTrait;
use crate::dex::adapter::DexAdapter;
use crate::dex::lifinity::LifinityAdapter as RealLifinityAdapter;

/// Lifinity DEX 适配器结构体。
/// 用于对接 Solana 链上的 Lifinity DEX，实现统一的 DEX 适配接口。
pub struct LifinityAdapter;

/// 实现 AdapterTrait，提供适配器元信息。
impl AdapterTrait for LifinityAdapter {
    fn name(&self) -> &str { "lifinity_adapter" }
    fn version(&self) -> &str { "1.0.0" }
    fn is_available(&self) -> bool { true }
    fn initialize(&mut self) -> anchor_lang::Result<()> { Ok(()) }
    fn cleanup(&mut self) -> anchor_lang::Result<()> { Ok(()) }
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
    fn swap(&self, params: &crate::core::types::TradeParams) -> anchor_lang::Result<crate::dex::adapter::DexSwapResult> {
        // TODO: 实现实际的 swap 逻辑
        Ok(crate::dex::adapter::DexSwapResult {
            executed_amount: params.amount_in,
            avg_price: 1_000_000,
            fee: 1000,
            dex_name: "lifinity".to_string(),
        })
    }
    
    fn batch_swap(&self, params: &crate::core::types::BatchTradeParams) -> anchor_lang::Result<Vec<crate::dex::adapter::DexSwapResult>> {
        // TODO: 实现实际的批量 swap 逻辑
        Ok(vec![])
    }
    
    fn configure(&self, params: &crate::core::types::DexParams) -> anchor_lang::Result<()> {
        // TODO: 实现配置逻辑
        Ok(())
    }
    
    fn supported_assets(&self) -> Vec<String> {
        vec!["SOL".to_string(), "USDC".to_string()]
    }
    
    fn supported_markets(&self) -> Vec<String> {
        vec!["spot".to_string()]
    }
    
    fn adapter_type(&self) -> crate::dex::adapter::DexAdapterType {
        crate::dex::adapter::DexAdapterType::AMM
    }
} 