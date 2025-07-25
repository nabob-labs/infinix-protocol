//!
//! DEX Adapter Registry Module
//!
//! 本模块实现 DEX 适配器注册表，支持适配器的注册、注销、查找、批量管理等功能，确保适配器生命周期合规、可追溯、可维护。

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// DEX 适配器 trait，所有适配器需实现。
pub trait DexAdapterTrait: Send + Sync {
    /// 获取适配器名称。
    fn name(&self) -> &str;
    /// 获取适配器版本。
    fn version(&self) -> &str;
    /// 检查适配器是否可用。
    fn is_available(&self) -> bool;
}

/// 适配器注册表结构体。
pub struct AdapterRegistry {
    adapters: Mutex<HashMap<String, Arc<dyn DexAdapterTrait>>>, // 线程安全适配器映射表
}

impl AdapterRegistry {
    /// 创建新的适配器注册表。
    pub fn new() -> Self {
        Self {
            adapters: Mutex::new(HashMap::new()),
        }
    }
    /// 注册适配器。
    pub fn register(&self, adapter: Arc<dyn DexAdapterTrait>) {
        let mut map = self.adapters.lock().unwrap();
        map.insert(adapter.name().to_string(), adapter);
    }
    /// 注销适配器。
    pub fn unregister(&self, name: &str) {
        let mut map = self.adapters.lock().unwrap();
        map.remove(name);
    }
    /// 查找适配器。
    pub fn find(&self, name: &str) -> Option<Arc<dyn DexAdapterTrait>> {
        let map = self.adapters.lock().unwrap();
        map.get(name).cloned()
    }
    /// 获取所有适配器名称。
    pub fn list_names(&self) -> Vec<String> {
        let map = self.adapters.lock().unwrap();
        map.keys().cloned().collect()
    }
    /// 获取所有可用适配器。
    pub fn available_adapters(&self) -> Vec<Arc<dyn DexAdapterTrait>> {
        let map = self.adapters.lock().unwrap();
        map.values().filter(|a| a.is_available()).cloned().collect()
    }
    /// 清空所有适配器。
    pub fn clear(&self) {
        let mut map = self.adapters.lock().unwrap();
        map.clear();
    }
} 