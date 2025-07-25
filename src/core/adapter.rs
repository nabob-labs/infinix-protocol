//!
//! Adapter Core Module
//!
//! 本模块定义适配器基础 trait、结构体与通用逻辑，支持 DEX、Oracle、策略等多种适配器的标准化、可插拔实现，确保系统扩展性、合规性与安全性。

// 引入 Anchor 依赖与核心类型。
use anchor_lang::prelude::*;

/// 适配器 trait，定义所有适配器的通用接口。
pub trait AdapterTrait {
    /// 获取适配器名称。
    fn name(&self) -> &str;
    /// 获取适配器版本。
    fn version(&self) -> &str;
    /// 检查适配器是否可用。
    fn is_available(&self) -> bool;
    /// 执行适配器初始化逻辑。
    fn initialize(&mut self) -> Result<()>;
    /// 执行适配器清理逻辑。
    fn cleanup(&mut self) -> Result<()>;
}

/// 适配器元信息结构体。
#[derive(Debug, Clone)]
pub struct AdapterMeta {
    pub name: String,      // 适配器名称
    pub version: String,   // 适配器版本
    pub is_active: bool,   // 是否激活
}

impl AdapterMeta {
    /// 创建新的适配器元信息。
    pub fn new(name: &str, version: &str) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
            is_active: true,
        }
    }
    /// 激活适配器。
    pub fn activate(&mut self) {
        self.is_active = true;
    }
    /// 停用适配器。
    pub fn deactivate(&mut self) {
        self.is_active = false;
    }
}

/// 适配器注册表结构体。
pub struct AdapterRegistry {
    pub adapters: Vec<AdapterMeta>, // 已注册适配器元信息列表
}

impl AdapterRegistry {
    /// 创建新的适配器注册表。
    pub fn new() -> Self {
        Self {
            adapters: Vec::new(),
        }
    }
    /// 注册适配器。
    pub fn register(&mut self, meta: AdapterMeta) {
        self.adapters.push(meta);
    }
    /// 注销适配器。
    pub fn unregister(&mut self, name: &str) {
        self.adapters.retain(|m| m.name != name);
    }
    /// 查找适配器。
    pub fn find(&self, name: &str) -> Option<&AdapterMeta> {
        self.adapters.iter().find(|m| m.name == name)
    }
    /// 检查适配器是否已注册。
    pub fn is_registered(&self, name: &str) -> bool {
        self.adapters.iter().any(|m| m.name == name)
    }
    /// 获取所有激活适配器。
    pub fn active_adapters(&self) -> Vec<&AdapterMeta> {
        self.adapters.iter().filter(|m| m.is_active).collect()
    }
} 