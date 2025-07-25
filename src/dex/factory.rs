//!
//! DEX Factory Module
//!
//! 本模块实现 DEX 工厂模式，支持多种 DEX 适配器的创建、注册与管理，确保可扩展性、合规性与可维护性。

use crate::dex::adapter_registry::{AdapterRegistry, DexAdapterTrait};
use std::sync::Arc;

/// DEX 工厂结构体。
pub struct DexFactory {
    pub registry: Arc<AdapterRegistry>, // 适配器注册表
}

impl DexFactory {
    /// 创建新的 DEX 工厂。
    pub fn new(registry: Arc<AdapterRegistry>) -> Self {
        Self { registry }
    }
    /// 注册适配器。
    pub fn register_adapter(&self, adapter: Arc<dyn DexAdapterTrait>) {
        self.registry.register(adapter);
    }
    /// 注销适配器。
    pub fn unregister_adapter(&self, name: &str) {
        self.registry.unregister(name);
    }
    /// 获取适配器。
    pub fn get_adapter(&self, name: &str) -> Option<Arc<dyn DexAdapterTrait>> {
        self.registry.find(name)
    }
    /// 获取所有适配器名称。
    pub fn list_adapter_names(&self) -> Vec<String> {
        self.registry.list_names()
    }
    /// 获取所有可用适配器。
    pub fn available_adapters(&self) -> Vec<Arc<dyn DexAdapterTrait>> {
        self.registry.available_adapters()
    }
    /// 清空所有适配器。
    pub fn clear_adapters(&self) {
        self.registry.clear();
    }
} 