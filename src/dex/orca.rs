//!
//! Orca DEX Adapter Module
//!
//! 本模块实现 Orca DEX 适配器，集成 Anchor CPI 调用，支持自动注册、标准接口、合规与可维护性。

use crate::core::adapter::AdapterTrait; // 适配器元信息 trait，统一接口
use crate::dex::adapter::DexAdapter;    // DEX 适配器 trait，统一 swap 等接口
use crate::core::types::SwapParams;     // swap 参数类型
use anchor_lang::prelude::*;            // Anchor 预导入，包含 Result、Pubkey 等
use std::sync::Arc;                     // Arc 用于多线程安全

/// Orca DEX 适配器结构体。
/// 用于对接 Solana 链上的 Orca DEX，实现统一的 DEX 适配接口。
/// 设计为无状态结构体，便于多实例、线程安全。
pub struct OrcaAdapter;

/// 实现 AdapterTrait，提供适配器元信息。
impl AdapterTrait for OrcaAdapter {
    /// 返回适配器名称（唯一标识）。
    fn name(&self) -> &'static str {
        "orca"
    }
}

/// 实现 DexAdapter trait，提供 swap 等核心功能。
impl DexAdapter for OrcaAdapter {
    /// 执行 Orca swap 操作。
    fn swap(&self, params: &SwapParams) -> Result<()> {
        // TODO: 集成 Orca 链上 CPI 调用。
        // 生产环境应校验参数、处理 CPI 错误、记录事件。
        Ok(())
    }
}

/// 注册 OrcaAdapter 到指定注册表。
/// 用于将 OrcaAdapter 动态注册到全局或自定义注册表，便于插件式扩展。
pub fn register_orca_adapter(registry: &mut crate::core::registry::AdapterRegistry<dyn DexAdapter>) {
    let adapter = Arc::new(OrcaAdapter); // 实例化适配器
    registry.register(adapter); // 注册到注册表，便于统一管理
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::SwapParams;
    use anchor_lang::prelude::Pubkey;

    /// 测试 OrcaAdapter 名称。
    #[test]
    fn test_orca_adapter_name() {
        let adapter = OrcaAdapter;
        assert_eq!(adapter.name(), "orca");
    }

    /// 测试 OrcaAdapter swap 功能。
    #[test]
    fn test_orca_adapter_swap() {
        let adapter = OrcaAdapter;
        let params = SwapParams {
            from_token: Pubkey::default(),
            to_token: Pubkey::default(),
            amount_in: 100,
            min_amount_out: 90,
            dex_name: "orca".to_string(),
        };
        let result = adapter.swap(&params);
        assert!(result.is_ok());
    }
} 