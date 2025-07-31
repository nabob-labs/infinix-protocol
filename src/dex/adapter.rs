//!
//! DEX Adapter Trait & Registry
//!
//! 本模块统一抽象所有 DEX/AMM 链上适配器接口，支持多资产、多市场、多功能类型参数化，
//! 并实现注册表、自动注册宏、Mock 适配器等，确保生产级集成、测试与合规。

use anchor_lang::prelude::*; // Anchor 预导入，包含 Result、msg!、账户声明等
use crate::core::adapter::AdapterTrait;
use crate::core::types::{TradeParams, BatchTradeParams, DexParams};
use std::collections::HashMap; // HashMap：适配器名称到实例的映射
use std::sync::{Arc, RwLock}; // Arc/RwLock：线程安全全局注册表

/// DEX/AMM 适配器 Trait（标准化、可扩展）
/// 统一所有 DEX/AMM 的链上适配接口，支持 swap、批量 swap、配置、资产/市场类型查询等。
pub trait DexAdapter: AdapterTrait {
    /// 执行 swap 的最小功能单元接口。
    fn swap(&self, params: &TradeParams) -> anchor_lang::Result<DexSwapResult>;
    /// 批量 swap 接口。
    fn batch_swap(&self, params: &BatchTradeParams) -> anchor_lang::Result<Vec<DexSwapResult>>;
    /// 配置 DEX adapter。
    fn configure(&self, params: &DexParams) -> anchor_lang::Result<()>;
    /// 支持的资产类型。
    fn supported_assets(&self) -> Vec<String> { vec![] }
    /// 支持的市场类型。
    fn supported_markets(&self) -> Vec<String> { vec![] }
    /// DEX adapter 类型。
    fn adapter_type(&self) -> DexAdapterType { DexAdapterType::Other }
}

/// DEX adapter 类型枚举。
#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize, PartialEq, Eq)]
pub enum DexAdapterType {
    Spot,   // 现货
    Perp,   // 永续合约
    AMM,    // 自动做市商
    CLOB,   // 集中订单簿
    Other,  // 其他
}

/// swap 结果结构体。
#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize)]
pub struct DexSwapResult {
    pub executed_amount: u64, // 实际成交数量
    pub avg_price: u64,       // 平均成交价格
    pub fee: u64,             // 手续费
    pub dex_name: String,     // DEX 名称
}

// === Anchor账户声明（可扩展） ===
#[derive(Accounts)]
pub struct Swap<'info> {
    // 相关账户声明（如用户、池、token账户等）
}

#[derive(Accounts)]
pub struct BatchSwap<'info> {
    // 相关账户声明
}

#[derive(Accounts)]
pub struct Configure<'info> {
    // 相关账户声明
}

/// DEX 适配器注册表，支持动态注册、注销、查询、事件日志，线程安全。
pub struct DexAdapterRegistry {
    adapters: RwLock<HashMap<String, Arc<dyn DexAdapter + Send + Sync>>>, // 适配器名称到实例的映射
}

impl DexAdapterRegistry {
    /// 创建新注册表。
    pub fn new() -> Self {
        Self { adapters: RwLock::new(HashMap::new()) }
    }
    /// 注册适配器。
    pub fn register(&self, name: &str, adapter: Arc<dyn DexAdapter + Send + Sync>) {
        self.adapters.write().unwrap().insert(name.to_string(), adapter);
        msg!("[DexAdapterRegistry] Registered adapter: {}", name);
    }
    /// 注销适配器。
    pub fn unregister(&self, name: &str) {
        self.adapters.write().unwrap().remove(name);
        msg!("[DexAdapterRegistry] Unregistered adapter: {}", name);
    }
    /// 查询适配器。
    pub fn get(&self, name: &str) -> Option<Arc<dyn DexAdapter + Send + Sync>> {
        self.adapters.read().unwrap().get(name).cloned()
    }
    /// 列出所有已注册适配器名称。
    pub fn list(&self) -> Vec<String> {
        self.adapters.read().unwrap().keys().cloned().collect()
    }
}

/// 全局 DEX 适配器注册表（单例）。
lazy_static::lazy_static! {
    pub static ref DEX_ADAPTER_REGISTRY: DexAdapterRegistry = DexAdapterRegistry::new();
}

/// 自动注册 DEX 适配器宏。
#[macro_export]
macro_rules! auto_register_dex_adapter {
    ($name:expr, $adapter:expr) => {
        // #[ctor::ctor]
        fn auto_register() {
            $crate::dex::adapter::DEX_ADAPTER_REGISTRY.register($name, std::sync::Arc::new($adapter));
        }
    };
}

/// Mock DEX 适配器示例，实现 AdapterTrait 与 DexAdapter。
pub struct MockDexAdapter;
impl AdapterTrait for MockDexAdapter {
    fn name(&self) -> &str { "mock_dex" }
    fn version(&self) -> &str { "1.0.0" }
    fn is_available(&self) -> bool { true }
    fn initialize(&mut self) -> anchor_lang::Result<()> { Ok(()) }
    fn cleanup(&mut self) -> anchor_lang::Result<()> { Ok(()) }
}
impl DexAdapter for MockDexAdapter {
    fn swap(&self, params: &TradeParams) -> anchor_lang::Result<DexSwapResult> {
        Ok(DexSwapResult {
            executed_amount: params.amount_in,
            avg_price: 1_000_000,
            fee: 1000,
            dex_name: "mock_dex".to_string(),
        })
    }
    fn batch_swap(&self, params: &BatchTradeParams) -> anchor_lang::Result<Vec<DexSwapResult>> {
        Ok(params.swaps.iter().map(|p| DexSwapResult {
            executed_amount: p.amount_in,
            avg_price: 1_000_000,
            fee: 1000,
            dex_name: "mock_dex".to_string(),
        }).collect())
    }
    fn configure(&self, _params: &DexParams) -> anchor_lang::Result<()> { Ok(()) }
    fn supported_assets(&self) -> Vec<String> { vec!["SOL".to_string(), "USDC".to_string()] }
    fn supported_markets(&self) -> Vec<String> { vec!["spot".to_string()] }
    fn adapter_type(&self) -> DexAdapterType { DexAdapterType::AMM }
}

// 自动注册 MockDexAdapter，便于开发环境自动集成，无需手动注册。
auto_register_dex_adapter!("mock_dex", MockDexAdapter);

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_mock_dex_adapter() {
        let adapter = MockDexAdapter;
        let params = TradeParams {
            from_token: Default::default(),
            to_token: Default::default(),
            amount_in: 100,
            min_amount_out: 90,
            dex_name: "mock_dex".to_string(),
        };
        let result = adapter.swap(&params).unwrap();
        assert_eq!(result.executed_amount, 100);
        assert_eq!(result.avg_price, 1_000_000);
        assert_eq!(result.fee, 1000);
        assert_eq!(result.dex_name, "mock_dex");
    }
} 