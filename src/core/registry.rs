//!
//! 核心注册表与工厂模块
//! 提供算法、DEX、预言机等全局注册表与统一 Adapter 工厂，支持动态注册、查询、批量管理。

use once_cell::sync::Lazy;
use std::sync::Arc;
use crate::algorithms::algorithm_registry::AlgorithmRegistry;
use crate::dex::adapter::DexAdapterRegistry;
use crate::oracles::adapter_registry::OracleAdapterRegistry;
use crate::core::adapter::AdapterTrait;
use std::collections::HashMap;

/// 全局算法注册表单例（线程安全，模块加载时自动初始化）
/// - 用于统一管理所有算法适配器
/// - 使用 once_cell::sync::Lazy + Arc 保证线程安全和惰性初始化
pub static ALGORITHM_REGISTRY: Lazy<Arc<AlgorithmRegistry>> = Lazy::new(|| {
    // 可在此注册默认算法
    Arc::new(AlgorithmRegistry::new())
});

/// 全局 DEX 适配器注册表单例
/// - 用于统一管理所有 DEX 适配器
/// - 使用 once_cell::sync::Lazy + Arc 保证线程安全和惰性初始化
pub static DEX_ADAPTER_REGISTRY: Lazy<Arc<DexAdapterRegistry>> = Lazy::new(|| {
    // 可在此注册默认DEX适配器
    Arc::new(DexAdapterRegistry::new())
});

/// 全局 Oracle 适配器注册表单例
/// - 用于统一管理所有 Oracle 适配器
/// - 使用 once_cell::sync::Lazy + Arc 保证线程安全和惰性初始化
pub static ORACLE_ADAPTER_REGISTRY: Lazy<Arc<OracleAdapterRegistry>> = Lazy::new(|| {
    // 可在此注册默认Oracle适配器
    Arc::new(OracleAdapterRegistry::new())
});

/// 统一 Adapter 工厂，支持通过名称/类型/元数据动态创建和注册 adapter/algorithm/strategy
/// - 内部维护 HashMap，键为 adapter 名称，值为 Arc 包裹的 trait 对象
pub struct AdapterFactory {
    /// 注册表，存储所有已注册的 adapter
    registry: HashMap<String, Arc<dyn AdapterTrait + Send + Sync>>,
}

impl AdapterFactory {
    /// 创建新工厂实例
    pub fn new() -> Self {
        Self { registry: HashMap::new() }
    }
    /// 注册 adapter
    /// - 参数 adapter: 实现 AdapterTrait 的实例
    /// - trait 对象需 Send + Sync + 'static 以便多线程和 trait object 安全
    pub fn register<T: AdapterTrait + Send + Sync + 'static>(&mut self, adapter: T) {
        self.registry.insert(adapter.name().to_string(), Arc::new(adapter));
    }
    /// 通过名称获取 adapter
    /// - 返回 Arc 包裹的 trait 对象，便于多线程共享
    pub fn get(&self, name: &str) -> Option<Arc<dyn AdapterTrait + Send + Sync>> {
        self.registry.get(name).cloned()
    }
    /// 按版本筛选 adapter
    /// - 便于多版本管理和升级
    pub fn list_by_version(&self, version: &str) -> Vec<Arc<dyn AdapterTrait + Send + Sync>> {
        self.registry.values().filter(|a| a.version() == version).cloned().collect()
    }
    /// 按资产筛选 adapter
    /// - 便于资产类型动态适配
    pub fn list_by_asset(&self, asset: &str) -> Vec<Arc<dyn AdapterTrait + Send + Sync>> {
        self.registry.values().filter(|a| a.supported_assets().contains(&asset.to_string())).cloned().collect()
    }
    /// 列出所有已注册 adapter
    pub fn list_all(&self) -> Vec<Arc<dyn AdapterTrait + Send + Sync>> {
        self.registry.values().cloned().collect()
    }
    /// 自动注册（可用于模块加载时批量注册）
    /// - 参数 adapters: 实现 AdapterTrait 的实例列表
    pub fn auto_register<T: AdapterTrait + Send + Sync + 'static>(&mut self, adapters: Vec<T>) {
        for adapter in adapters {
            self.register(adapter);
        }
    }
}

/// 全局 Adapter 工厂单例（线程安全，支持动态注册与查询）
/// - 推荐通过该工厂统一管理所有 adapter/algorithm/strategy 实例
/// - 使用 std::sync::Mutex 包裹，保证多线程安全
pub static ADAPTER_FACTORY: Lazy<std::sync::Mutex<AdapterFactory>> = Lazy::new(|| {
    let mut factory = AdapterFactory::new();
    // 可在此自动注册所有adapter/algorithm/strategy实现
    std::sync::Mutex::new(factory)
}); 