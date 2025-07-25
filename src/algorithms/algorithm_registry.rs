//! 算法注册表模块
//! 支持 ExecutionStrategy、RiskManagement 等多 trait 算法的注册和查找。
//! 线程安全，支持动态注册、热插拔、批量操作、状态管理等。

use anchor_lang::prelude::*; // Anchor 预导入，包含事件、时间戳等
use std::collections::HashMap; // HashMap 用于算法名称到实例/元数据的映射
use std::sync::{Arc, RwLock}; // Arc+RwLock 实现线程安全的全局注册表
use once_cell::sync::Lazy; // Lazy 用于全局单例懒加载
use crate::algorithms::traits::{ExecutionStrategy, RiskManagement}; // 引入算法 trait，便于类型安全注册
use log::info; // 日志输出

/// 算法状态枚举
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AlgorithmStatus {
    Active,                 // 激活
    Inactive,               // 未激活
    Error(String),          // 错误（附带原因）
}

/// 算法元数据结构体
#[derive(Debug, Clone)]
pub struct AlgorithmMeta {
    pub name: String,           // 算法名称
    pub version: String,        // 算法版本
    pub algo_type: String,      // 算法类型
    pub status: AlgorithmStatus,// 算法状态
    pub last_updated: i64,      // 最后更新时间戳
}

/// 算法注册表主结构体
/// - 支持多 trait 算法注册与查找
/// - 内部采用 RwLock+Arc，线程安全，支持并发注册/查询
pub struct AlgorithmRegistry {
    /// 执行类算法注册表（名称->算法实例）
    execution_algorithms: RwLock<HashMap<String, Arc<dyn ExecutionStrategy + Send + Sync>>>,
    /// 路由类算法注册表
    routing_algorithms: RwLock<HashMap<String, Arc<dyn ExecutionStrategy + Send + Sync>>>,
    /// 风控类算法注册表
    risk_algorithms: RwLock<HashMap<String, Arc<dyn RiskManagement + Send + Sync>>>,
    /// 优化器类算法注册表
    optimizer_algorithms: RwLock<HashMap<String, Arc<dyn ExecutionStrategy + Send + Sync>>>,
    /// 算法元数据表
    metadata: RwLock<HashMap<String, AlgorithmMeta>>,
}

impl AlgorithmRegistry {
    /// 仅供全局单例初始化使用，推荐通过 core/registry.rs 的 ALGORITHM_REGISTRY 获取实例。
    pub fn new() -> Self {
        Self {
            execution_algorithms: RwLock::new(HashMap::new()), // 初始化执行类算法表
            routing_algorithms: RwLock::new(HashMap::new()),   // 初始化路由类算法表
            risk_algorithms: RwLock::new(HashMap::new()),      // 初始化风控类算法表
            optimizer_algorithms: RwLock::new(HashMap::new()), // 初始化优化器类算法表
            metadata: RwLock::new(HashMap::new()),             // 初始化元数据表
        }
    }
    /// 注册执行类算法
    /// - 参数 name: 算法名称
    /// - 参数 algo: 算法实例（需实现 ExecutionStrategy + Send + Sync）
    /// - 参数 meta: 算法元数据
    pub fn register_execution(&self, name: &str, algo: Arc<dyn ExecutionStrategy + Send + Sync>, meta: AlgorithmMeta) {
        self.execution_algorithms
            .write()
            .unwrap()
            .insert(name.to_string(), algo); // 注册算法实例
        self.metadata.write().unwrap().insert(name.to_string(), meta.clone()); // 注册元数据
        info!("Registered execution algorithm: {} v{}", name, meta.version); // 日志输出
        emit!(AlgorithmEvent::Registered { name: name.to_string(), version: meta.version }); // 触发注册事件
    }
    /// 获取执行类算法实例
    pub fn get_execution(&self, name: &str) -> Option<Arc<dyn ExecutionStrategy + Send + Sync>> {
        self.execution_algorithms.read().unwrap().get(name).cloned() // 线程安全读取
    }
    /// 获取算法元数据
    pub fn get_meta(&self, name: &str) -> Option<AlgorithmMeta> {
        self.metadata.read().unwrap().get(name).cloned() // 线程安全读取
    }
    /// 注册风控类算法
    pub fn register_risk(&self, name: &str, algo: Arc<dyn RiskManagement + Send + Sync>) {
        self.risk_algorithms
            .write()
            .unwrap()
            .insert(name.to_string(), algo); // 注册风控算法
    }
    /// 获取风控类算法实例
    pub fn get_risk(&self, name: &str) -> Option<Arc<dyn RiskManagement + Send + Sync>> {
        self.risk_algorithms.read().unwrap().get(name).cloned() // 线程安全读取
    }
    /// 移除执行类算法
    pub fn remove_execution(&self, name: &str) {
        self.execution_algorithms.write().unwrap().remove(name); // 移除算法实例
        self.metadata.write().unwrap().remove(name); // 移除元数据
        info!("Removed execution algorithm: {}", name); // 日志输出
        emit!(AlgorithmEvent::Removed { name: name.to_string() }); // 触发移除事件
    }
    /// 移除路由类算法
    pub fn remove_routing(&self, name: &str) {
        self.routing_algorithms.write().unwrap().remove(name); // 移除路由算法
    }
    /// 移除风控类算法
    pub fn remove_risk(&self, name: &str) {
        self.risk_algorithms.write().unwrap().remove(name); // 移除风控算法
    }
    /// 移除优化器类算法
    pub fn remove_optimizer(&self, name: &str) {
        self.optimizer_algorithms.write().unwrap().remove(name); // 移除优化器算法
    }
    /// 列出所有已注册执行类算法名称
    pub fn list_executions(&self) -> Vec<String> {
        self.execution_algorithms.read().unwrap().keys().cloned().collect() // 返回所有执行类算法名称
    }
    /// 列出所有已注册路由类算法名称
    pub fn list_routings(&self) -> Vec<String> {
        self.routing_algorithms.read().unwrap().keys().cloned().collect() // 返回所有路由类算法名称
    }
    /// 列出所有已注册风控类算法名称
    pub fn list_risks(&self) -> Vec<String> {
        self.risk_algorithms.read().unwrap().keys().cloned().collect() // 返回所有风控类算法名称
    }
    /// 列出所有已注册优化器类算法名称
    pub fn list_optimizers(&self) -> Vec<String> {
        self.optimizer_algorithms.read().unwrap().keys().cloned().collect() // 返回所有优化器类算法名称
    }
    /// 设置算法状态
    /// - 参数 name: 算法名称
    /// - 参数 status: 新状态
    pub fn set_status(&self, name: &str, status: AlgorithmStatus) {
        if let Some(meta) = self.metadata.write().unwrap().get_mut(name) {
            meta.status = status.clone(); // 更新状态
            meta.last_updated = Clock::get().map(|c| c.unix_timestamp).unwrap_or(0); // 更新时间戳
            info!("Set status for algorithm {}: {:?}", name, status); // 日志输出
            emit!(AlgorithmEvent::StatusChanged { name: name.to_string(), status }); // 触发状态变更事件
        }
    }
    /// 按状态筛选算法元数据
    pub fn list_by_status(&self, status: AlgorithmStatus) -> Vec<AlgorithmMeta> {
        self.metadata.read().unwrap().values().filter(|m| m.status == status).cloned().collect() // 返回所有指定状态的算法元数据
    }
    /// 按类型筛选算法元数据
    pub fn list_by_type(&self, algo_type: &str) -> Vec<AlgorithmMeta> {
        self.metadata.read().unwrap().values().filter(|m| m.algo_type == algo_type).cloned().collect() // 返回所有指定类型的算法元数据
    }
    /// 批量注册执行类算法
    pub fn batch_register(&self, algos: Vec<(String, Arc<dyn ExecutionStrategy + Send + Sync>, AlgorithmMeta)>) {
        for (name, algo, meta) in algos {
            self.register_execution(&name, algo, meta); // 依次注册
        }
    }
    /// 批量移除执行类算法
    pub fn batch_remove(&self, names: Vec<String>) {
        for name in names {
            self.remove_execution(&name); // 依次移除
        }
    }
    /// 热插拔替换执行类算法
    /// - 参数 name: 算法名称
    /// - 参数 new_algo: 新算法实例
    /// - 参数 new_meta: 新算法元数据
    pub fn hot_swap(&self, name: &str, new_algo: Arc<dyn ExecutionStrategy + Send + Sync>, new_meta: AlgorithmMeta) {
        self.execution_algorithms.write().unwrap().insert(name.to_string(), new_algo); // 替换算法实例
        self.metadata.write().unwrap().insert(name.to_string(), new_meta.clone()); // 替换元数据
        info!("Hot-swapped execution algorithm: {} v{}", name, new_meta.version); // 日志输出
        emit!(AlgorithmEvent::HotSwapped { name: name.to_string(), version: new_meta.version }); // 触发热插拔事件
    }
}

/// 算法注册相关事件（Anchor 事件）
#[event]
pub enum AlgorithmEvent {
    Registered { name: String, version: String },      // 注册事件
    Removed { name: String },                          // 移除事件
    StatusChanged { name: String, status: AlgorithmStatus }, // 状态变更事件
    HotSwapped { name: String, version: String },      // 热插拔事件
}

/// 全局算法注册表单例（线程安全，模块加载时自动初始化）
pub static ALGORITHM_REGISTRY: Lazy<AlgorithmRegistry> = Lazy::new(|| {
    let registry = AlgorithmRegistry::new(); // 创建注册表实例
    // 示例注册，实际应补充元数据
    registry.register_execution(
        "twap",
        Arc::new(crate::algorithms::twap::TwapAlgorithm::default()),
        AlgorithmMeta {
            name: "twap".to_string(),
            version: "1.0.0".to_string(),
            algo_type: "execution".to_string(),
            status: AlgorithmStatus::Active,
            last_updated: Clock::get().map(|c| c.unix_timestamp).unwrap_or(0),
        },
    );
    registry.register_execution(
        "vwap",
        Arc::new(crate::algorithms::vwap::VwapAlgorithm::default()),
        AlgorithmMeta {
            name: "vwap".to_string(),
            version: "1.0.0".to_string(),
            algo_type: "execution".to_string(),
            status: AlgorithmStatus::Active,
            last_updated: Clock::get().map(|c| c.unix_timestamp).unwrap_or(0),
        },
    );
    // 可继续注册其他算法adapter
    registry
});
