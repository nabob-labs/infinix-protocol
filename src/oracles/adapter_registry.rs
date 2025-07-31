//!
//! 预言机适配器注册表模块
//! - 提供Oracle适配器的动态注册、元数据管理、批量操作、热插拔、事件通知等功能
//! - 设计意图：极致可插拔、动态扩展、链上可观测性、线程安全、Anchor集成友好

use anchor_lang::prelude::*; // Anchor预导入，包含Result、Clock、emit!等
use std::collections::HashMap; // HashMap用于名称到实例/元数据映射
use std::sync::{Arc, RwLock}; // Arc/RwLock保证线程安全
use crate::oracles::traits::OracleAdapter; // OracleAdapter trait
use once_cell::sync::Lazy; // Lazy单例
use log::info; // 日志输出

/// 适配器状态枚举
/// - 表示Oracle适配器的当前运行状态，用于动态管理和监控。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdapterStatus {
    /// 适配器处于活跃状态，可正常处理请求
    Active,   // 活跃
    /// 适配器处于非活跃状态，暂不可用
    Inactive, // 不活跃
    /// 适配器发生错误，附带错误原因
    Error(String), // 错误及原因
}

/// 适配器元数据结构体
/// - 用于描述Oracle适配器的基本信息、支持资产、状态等元数据，便于注册表统一管理
#[derive(Debug, Clone)]
pub struct AdapterMeta {
    /// 适配器名称（唯一标识）
    pub name: String,
    /// 适配器版本号
    pub version: String,
    /// 支持的资产列表（如SOL、USDC等）
    pub supported_assets: Vec<String>,
    /// 当前状态（活跃/不活跃/错误）
    pub status: AdapterStatus,
    /// 最后一次状态更新时间戳（Unix时间戳，秒）
    pub last_updated: i64,
}

/// 预言机适配器注册表
/// - 支持动态注册、注销、元数据管理、批量操作、热插拔、事件通知
/// - 线程安全，适用于多线程环境
pub struct OracleAdapterRegistry {
    /// 名称到适配器实例的映射（线程安全）
    adapters: RwLock<HashMap<String, Arc<dyn OracleAdapter + Send + Sync>>>,
    /// 名称到适配器元数据的映射（线程安全）
    metadata: RwLock<HashMap<String, AdapterMeta>>,
}

impl OracleAdapterRegistry {
    /// 创建新的Oracle适配器注册表实例
    /// 返回：OracleAdapterRegistry对象
    pub fn new() -> Self {
        Self {
            adapters: RwLock::new(HashMap::new()),
            metadata: RwLock::new(HashMap::new()),
        }
    }
    /// 注册适配器及其元数据
    /// 参数：
    ///   - name: 适配器名称
    ///   - adapter: 适配器实例（需实现OracleAdapter trait）
    ///   - meta: 适配器元数据
    /// - 设计意图：支持运行时热插拔、链上可观测性、事件通知
    pub fn register(&self, name: &str, adapter: Arc<dyn OracleAdapter + Send + Sync>, meta: AdapterMeta) {
        self.adapters.write().unwrap().insert(name.to_string(), adapter);
        self.metadata.write().unwrap().insert(name.to_string(), meta.clone());
        info!("Registered Oracle adapter: {} v{}", name, meta.version);
        emit!(OracleAdapterRegistered { name: name.to_string(), version: meta.version });
    }
    /// 查询适配器实例
    /// 参数：name 适配器名称
    /// 返回：Option<Arc<dyn OracleAdapter + Send + Sync>>
    pub fn get(&self, name: &str) -> Option<Arc<dyn OracleAdapter + Send + Sync>> {
        self.adapters.read().unwrap().get(name).cloned()
    }
    /// 查询适配器元数据
    /// 参数：name 适配器名称
    /// 返回：Option<AdapterMeta>
    pub fn get_meta(&self, name: &str) -> Option<AdapterMeta> {
        self.metadata.read().unwrap().get(name).cloned()
    }
    /// 注销适配器及其元数据
    /// 参数：name 适配器名称
    /// - 设计意图：支持运行时卸载、链上事件通知
    pub fn remove(&self, name: &str) {
        self.adapters.write().unwrap().remove(name);
        self.metadata.write().unwrap().remove(name);
        info!("Removed Oracle adapter: {}", name);
        emit!(OracleAdapterRemoved { name: name.to_string() });
    }
    /// 设置适配器状态
    /// 参数：
    ///   - name: 适配器名称
    ///   - status: 新状态
    /// - 设计意图：链上可观测性、状态变更事件
    pub fn set_status(&self, name: &str, status: AdapterStatus) {
        if let Some(meta) = self.metadata.write().unwrap().get_mut(name) {
            meta.status = status.clone();
            meta.last_updated = Clock::get().map(|c| c.unix_timestamp).unwrap_or(0);
            info!("Set status for Oracle adapter {}: {:?}", name, status);
            emit!(OracleAdapterStatusChanged { name: name.to_string(), status });
        }
    }
    /// 按状态筛选适配器元数据
    /// 参数：status 目标状态
    /// 返回：所有处于该状态的适配器元数据列表
    pub fn list_by_status(&self, status: AdapterStatus) -> Vec<AdapterMeta> {
        self.metadata.read().unwrap().values().filter(|m| m.status == status).cloned().collect()
    }
    /// 按资产筛选适配器元数据
    /// 参数：asset 资产名称
    /// 返回：支持该资产的适配器元数据列表
    pub fn list_by_asset(&self, asset: &str) -> Vec<AdapterMeta> {
        self.metadata.read().unwrap().values().filter(|m| m.supported_assets.contains(&asset.to_string())).cloned().collect()
    }
    /// 批量注册适配器
    /// 参数：adapters (名称, 实例, 元数据)元组列表
    pub fn batch_register(&self, adapters: Vec<(String, Arc<dyn OracleAdapter + Send + Sync>, AdapterMeta)>) {
        for (name, adapter, meta) in adapters {
            self.register(&name, adapter, meta);
        }
    }
    /// 批量注销适配器
    /// 参数：names 适配器名称列表
    pub fn batch_remove(&self, names: Vec<String>) {
        for name in names {
            self.remove(&name);
        }
    }
    /// 热插拔替换适配器及元数据
    /// 参数：
    ///   - name: 适配器名称
    ///   - new_adapter: 新适配器实例
    ///   - new_meta: 新适配器元数据
    /// - 设计意图：支持不停机升级、链上事件通知
    pub fn hot_swap(&self, name: &str, new_adapter: Arc<dyn OracleAdapter + Send + Sync>, new_meta: AdapterMeta) {
        self.adapters.write().unwrap().insert(name.to_string(), new_adapter);
        self.metadata.write().unwrap().insert(name.to_string(), new_meta.clone());
        info!("Hot-swapped Oracle adapter: {} v{}", name, new_meta.version);
        emit!(OracleAdapterHotSwapped { name: name.to_string(), version: new_meta.version });
    }
}

/// 预言机适配器注册相关事件（Anchor事件）
/// - 用于链上追踪适配器注册、注销、状态变更、热插拔等操作，便于前端/监控系统监听

#[event]
pub struct OracleAdapterRegistered {
    pub name: String,
    pub version: String,
}

#[event]
pub struct OracleAdapterRemoved {
    pub name: String,
}

#[event]
pub struct OracleAdapterStatusChanged {
    pub name: String,
    pub status: AdapterStatus,
}

#[event]
pub struct OracleAdapterHotSwapped {
    pub name: String,
    pub version: String,
}

/// 全局预言机适配器注册表（单例，带示例注册）
/// - 使用once_cell::sync::Lazy实现全局唯一实例
/// - 启动时自动注册主流Oracle适配器，便于统一管理和动态扩展
pub static ORACLE_ADAPTER_REGISTRY: Lazy<OracleAdapterRegistry> = Lazy::new(|| {
    let registry = OracleAdapterRegistry::new();
    // Pyth适配器注册，含元数据
    registry.register(
        "pyth",
        Arc::new(crate::oracles::pyth_adapter::PythAdapter),
        AdapterMeta {
            name: "pyth".to_string(),
            version: "1.0.0".to_string(),
            supported_assets: vec!["SOL".to_string(), "USDC".to_string()],
            status: AdapterStatus::Active,
            last_updated: Clock::get().map(|c| c.unix_timestamp).unwrap_or(0),
        },
    );
    // ... 其他Oracle adapter注册 ...
    registry
}); 