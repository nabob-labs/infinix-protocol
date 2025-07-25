//!
//! Jupiter DEX Adapter Module
//!
//! 本模块实现 Jupiter DEX 适配器，提供与 Jupiter 聚合器的链上集成接口，确保交易路由与聚合合规、可维护。

use crate::core::adapter::AdapterTrait;
use crate::core::types::{TradeParams, BatchTradeParams, DexParams};
use crate::dex::adapter::{DexAdapter, DexSwapResult, DexAdapterType};
use anchor_lang::prelude::*;

/// Jupiter DEX 适配器结构体。
pub struct JupiterAdapter;

impl AdapterTrait for JupiterAdapter {
    /// 返回适配器名称。
    fn name(&self) -> &'static str { "jupiter" }
    /// 返回适配器版本。
    fn version(&self) -> &'static str { "1.0.0" }
    /// 返回支持的资产列表。
    fn supported_assets(&self) -> Vec<String> { vec!["SOL".to_string(), "USDC".to_string()] }
    /// 返回适配器状态。
    fn status(&self) -> Option<String> { Some("active".to_string()) }
}

impl DexAdapter for JupiterAdapter {
    /// 执行 swap 操作，集成 Jupiter 聚合器。
    fn swap(&self, params: &TradeParams) -> anchor_lang::Result<DexSwapResult> {
        // 实际应集成 Jupiter 聚合器链上路由逻辑，此处为示例
        Ok(DexSwapResult {
            executed_amount: params.amount_in, // 假设全部成交
            avg_price: 1_000_000, // 示例均价
            fee: 1000, // 示例手续费
            dex_name: "jupiter".to_string(),
        })
    }
    /// 批量 swap 操作。
    fn batch_swap(&self, params: &BatchTradeParams) -> anchor_lang::Result<Vec<DexSwapResult>> {
        Ok(params.swaps.iter().map(|p| DexSwapResult {
            executed_amount: p.amount_in,
            avg_price: 1_000_000,
            fee: 1000,
            dex_name: "jupiter".to_string(),
        }).collect())
    }
    /// 配置 Jupiter 适配器（无实际效果，示例）。
    fn configure(&self, _params: &DexParams) -> anchor_lang::Result<()> { Ok(()) }
    /// 返回支持的资产列表。
    fn supported_assets(&self) -> Vec<String> { vec!["SOL".to_string(), "USDC".to_string()] }
    /// 返回支持的市场类型。
    fn supported_markets(&self) -> Vec<String> { vec!["spot".to_string()] }
    /// 返回适配器类型。
    fn adapter_type(&self) -> DexAdapterType { DexAdapterType::AMM }
}

// 自动注册 JupiterAdapter。
use crate::auto_register_dex_adapter;
auto_register_dex_adapter!("jupiter", JupiterAdapter);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::SwapParams;
    use anchor_lang::prelude::Pubkey;

    /// 测试 JupiterAdapter 名称。
    /// - 设计意图：保证 name 方法返回唯一标识，便于注册表/工厂识别。
    #[test]
    fn test_jupiter_adapter_name() {
        let adapter = JupiterAdapter;
        assert_eq!(adapter.name(), "jupiter");
    }

    /// 测试 JupiterAdapter swap 功能。
    /// - 设计意图：保证 swap 方法可正常调用，便于持续集成。
    #[test]
    fn test_jupiter_adapter_swap() {
        let adapter = JupiterAdapter;
        let params = SwapParams {
            from_token: Pubkey::default(), // 测试用默认 token
            to_token: Pubkey::default(),
            amount_in: 100,
            min_amount_out: 90,
            dex_name: "jupiter".to_string(),
        };
        let result = adapter.swap(&params);
        assert!(result.is_ok());
    }
} 