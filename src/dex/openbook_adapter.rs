//!
//! OpenBook DEX Adapter Bridge Module
//!
//! 本模块为 OpenBook DEX 提供 Anchor 兼容的桥接适配器，实现统一接口、自动注册、CPI集成（预留），确保可插拔、合规、可维护。

use anchor_lang::prelude::*; // Anchor 预导入，包含 Result、Context 等
use crate::core::adapter::AdapterTrait;
use crate::dex::adapter::{DexAdapter, DexSwapResult};
use crate::core::types::{TradeParams, BatchTradeParams, DexParams};
// use crate::core::adapter: // 暂时注释掉:AdapterTrait; // 适配器元信息 trait，统一接口
// 移除未找到的ctor属性
// use ctor::ctor; // ctor 宏用于自动注册

/// OpenBook DEX 适配器结构体。
/// 用于对接 Solana 链上的 OpenBook DEX，实现统一的 DEX 适配接口。
pub struct OpenBookAdapter;

/// 实现 AdapterTrait，提供适配器元信息。
impl AdapterTrait for OpenBookAdapter {
    fn name(&self) -> &str { "openbook_adapter" }
    fn version(&self) -> &str { "1.0.0" }
    fn is_available(&self) -> bool { true }
    fn initialize(&mut self) -> anchor_lang::Result<()> { Ok(()) }
    fn cleanup(&mut self) -> anchor_lang::Result<()> { Ok(()) }
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
    fn swap(&self, params: &crate::core::types::TradeParams) -> anchor_lang::Result<DexSwapResult> {
        // TODO: 实现实际的 swap 逻辑
        Ok(DexSwapResult {
            executed_amount: params.amount_in,
            avg_price: 1_000_000,
            fee: 1000,
            dex_name: "openbook".to_string(),
        })
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