//!
//! Strategy Registry Implementation
//!
//! 本模块实现策略注册表，用于管理和追踪策略实例，支持批量注册、状态管理、性能统计等功能，确保策略生命周期合规、可追溯、可维护。

// 引入核心模块、错误类型、策略模块和 Anchor 依赖。
use crate::core::*;
use crate::error::StrategyError;
use crate::strategies::*;
use anchor_lang::prelude::*;
use log::info;

/// 策略状态枚举，标识策略生命周期各阶段。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StrategyStatus {
    Active,    // 激活
    Paused,    // 暂停
    Archived,  // 归档
    Error,     // 错误
}

/// 策略元信息结构体，记录策略标识、名称、版本、类型、状态和更新时间。
#[derive(Debug, Clone)]
pub struct StrategyMeta {
    pub id: u64,              // 策略唯一标识
    pub name: String,         // 策略名称
    pub version: String,      // 策略版本
    pub strategy_type: String,// 策略类型
    pub status: StrategyStatus,// 当前状态
    pub last_updated: i64,    // 最后更新时间戳
}

/// 策略注册表结构体，管理所有策略实例。
pub struct StrategyRegistry {
    strategies: Vec<RegistryEntry>, // 策略实例列表
    next_id: u64,                  // 下一个可用策略 ID
    metadata: std::collections::HashMap<u64, StrategyMeta>, // 策略元信息表
}

impl StrategyRegistry {
    /// 创建新的策略注册表。
    pub fn new() -> Self {
        Self {
            strategies: Vec::new(),
            next_id: 1,
            metadata: std::collections::HashMap::new(),
        }
    }
    /// 注册新策略，返回分配的策略 ID。
    pub fn register_strategy(
        &mut self,
        config: StrategyConfig,
        creator: Pubkey,
    ) -> StrategyResult<u64> {
        let strategy_id = self.next_id;
        self.next_id += 1;
        let entry = RegistryEntry {
            id: strategy_id,
            config,
            creator,
            created_at: Clock::get()?.unix_timestamp,
            last_accessed: Clock::get()?.unix_timestamp,
            access_count: 0,
            status: StrategyStatus::Active,
            performance_metrics: None,
        };
        self.strategies.push(entry);
        Ok(strategy_id)
    }
    /// 通过 ID 获取策略的可变引用，并更新访问统计。
    pub fn get_strategy(&mut self, strategy_id: u64) -> Option<&mut RegistryEntry> {
        if let Some(entry) = self.strategies.iter_mut().find(|e| e.id == strategy_id) {
            entry.last_accessed = Clock::get().unwrap().unix_timestamp;
            entry.access_count += 1;
            Some(entry)
        } else {
            None
        }
    }
    /// 获取指定创建者的所有策略。
    pub fn get_strategies_by_creator(&self, creator: &Pubkey) -> Vec<&RegistryEntry> {
        self.strategies
            .iter()
            .filter(|entry| entry.creator == *creator)
            .collect()
    }
    /// 获取指定权重类型的所有策略。
    pub fn get_strategies_by_weight_type(
        &self,
        strategy_type: &WeightStrategyType,
    ) -> Vec<&RegistryEntry> {
        self.strategies
            .iter()
            .filter(|entry| entry.config.weight_config.strategy_type == *strategy_type)
            .collect()
    }
    /// 获取所有激活状态的策略。
    pub fn get_active_strategies(&self) -> Vec<&RegistryEntry> {
        self.strategies
            .iter()
            .filter(|entry| entry.status == StrategyStatus::Active)
            .collect()
    }
    /// 更新策略状态。
    pub fn update_strategy_status(
        &mut self,
        strategy_id: u64,
        status: StrategyStatus,
    ) -> StrategyResult<()> {
        if let Some(entry) = self.strategies.iter_mut().find(|e| e.id == strategy_id) {
            entry.status = status;
            entry.last_accessed = Clock::get()?.unix_timestamp;
            Ok(())
        } else {
            Err(StrategyError::InvalidStrategyParameters.into())
        }
    }
    /// 从注册表移除指定策略。
    pub fn remove_strategy(&mut self, strategy_id: u64) {
        self.strategies.retain(|e| e.id != strategy_id);
    }
    /// 列出所有策略 ID。
    pub fn list_strategies(&self) -> Vec<u64> {
        self.strategies.iter().map(|e| e.id).collect()
    }
    /// 获取注册表统计信息。
    pub fn get_statistics(&self) -> RegistryStatistics {
        let total_strategies = self.strategies.len();
        let active_strategies = self
            .strategies
            .iter()
            .filter(|e| e.status == StrategyStatus::Active)
            .count();
        let paused_strategies = self
            .strategies
            .iter()
            .filter(|e| e.status == StrategyStatus::Paused)
            .count();
        let archived_strategies = self
            .strategies
            .iter()
            .filter(|e| e.status == StrategyStatus::Archived)
            .count();
        let total_access_count = self.strategies.iter().map(|e| e.access_count).sum();
        let avg_access_count = if total_strategies > 0 {
            total_access_count / total_strategies as u64
        } else {
            0
        };
        // 统计各类型策略数量。
        let mut weight_type_counts = std::collections::HashMap::new();
        let mut rebalancing_type_counts = std::collections::HashMap::new();
        for entry in &self.strategies {
            let weight_type = format!("{:?}", entry.config.weight_config.strategy_type);
            *weight_type_counts.entry(weight_type).or_insert(0) += 1;
            let rebalancing_type = format!("{:?}", entry.config.rebalancing_config.strategy_type);
            *rebalancing_type_counts.entry(rebalancing_type).or_insert(0) += 1;
        }
        RegistryStatistics {
            total_strategies,
            active_strategies,
            paused_strategies,
            archived_strategies,
            total_access_count,
            avg_access_count,
            weight_type_counts,
            rebalancing_type_counts,
        }
    }
    /// 清理超龄或归档策略。
    pub fn cleanup_strategies(&mut self, max_age_seconds: i64) -> StrategyResult<usize> {
        let current_time = Clock::get()?.unix_timestamp;
        let initial_count = self.strategies.len();
        self.strategies.retain(|entry| {
            // 激活策略始终保留。
            if entry.status == StrategyStatus::Active {
                return true;
            }
            // 归档策略超龄则移除。
            if entry.status == StrategyStatus::Archived {
                let age = current_time - entry.last_accessed;
                return age < max_age_seconds;
            }
            // 暂停策略保留。
            true
        });
        let removed_count = initial_count - self.strategies.len();
        Ok(removed_count)
    }
    /// 查找需要再平衡的策略。
    pub fn find_strategies_needing_rebalancing(&self) -> Vec<&RegistryEntry> {
        let current_time = Clock::get().unwrap().unix_timestamp;
        self.strategies
            .iter()
            .filter(|entry| {
                entry.status == StrategyStatus::Active
                    && entry.config.rebalancing_config.is_active
                    && current_time >= entry.config.rebalancing_config.next_rebalance
            })
            .collect()
    }
    /// 更新策略性能指标。
    pub fn update_performance_metrics(
        &mut self,
        strategy_id: u64,
        metrics: crate::core::traits::StrategyPerformanceMetrics,
    ) -> StrategyResult<()> {
        if let Some(entry) = self.strategies.iter_mut().find(|e| e.id == strategy_id) {
            entry.performance_metrics = Some(metrics);
            entry.last_accessed = Clock::get()?.unix_timestamp;
            Ok(())
        } else {
            Err(StrategyError::InvalidStrategyParameters.into())
        }
    }
    /// 批量注册策略。
    pub fn batch_register(&mut self, configs: Vec<(StrategyConfig, Pubkey, StrategyMeta)>) -> Vec<u64> {
        let mut ids = Vec::new();
        for (config, creator, meta) in configs {
            let id = self.register_strategy(config, creator).unwrap();
            self.metadata.insert(id, meta);
            info!("Batch registered strategy: {} v{}", id, meta.version);
            emit!(StrategyEvent::Registered { id, version: meta.version.clone() });
            ids.push(id);
        }
        ids
    }
    /// 批量移除策略。
    pub fn batch_remove(&mut self, ids: Vec<u64>) {
        for id in ids {
            self.remove_strategy(id);
            self.metadata.remove(&id);
            info!("Batch removed strategy: {}", id);
            emit!(StrategyEvent::Removed { id });
        }
    }
    /// 设置策略状态。
    pub fn set_status(&mut self, id: u64, status: StrategyStatus) {
        if let Some(meta) = self.metadata.get_mut(&id) {
            meta.status = status.clone();
            meta.last_updated = Clock::get().map(|c| c.unix_timestamp).unwrap_or(0);
            info!("Set status for strategy {}: {:?}", id, status);
            emit!(StrategyEvent::StatusChanged { id, status });
        }
    }
    /// 按状态列出策略元信息。
    pub fn list_by_status(&self, status: StrategyStatus) -> Vec<StrategyMeta> {
        self.metadata.values().filter(|m| m.status == status).cloned().collect()
    }
    /// 按类型列出策略元信息。
    pub fn list_by_type(&self, strategy_type: &str) -> Vec<StrategyMeta> {
        self.metadata.values().filter(|m| m.strategy_type == strategy_type).cloned().collect()
    }
    /// 热插拔替换策略配置和元信息。
    pub fn hot_swap(&mut self, id: u64, new_config: StrategyConfig, new_meta: StrategyMeta) {
        if let Some(entry) = self.strategies.iter_mut().find(|e| e.id == id) {
            entry.config = new_config;
            self.metadata.insert(id, new_meta.clone());
            info!("Hot-swapped strategy: {} v{}", id, new_meta.version);
            emit!(StrategyEvent::HotSwapped { id, version: new_meta.version });
        }
    }
}

impl Default for StrategyRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 策略注册表条目结构体，记录策略实例及其状态。
#[derive(Debug, Clone)]
pub struct RegistryEntry {
    pub id: u64,           // 策略唯一标识
    pub config: StrategyConfig, // 策略配置
    pub creator: Pubkey,   // 创建者
    pub created_at: i64,   // 创建时间
    pub last_accessed: i64,// 最后访问时间
    pub access_count: u64, // 访问次数
    pub status: StrategyStatus, // 当前状态
    pub performance_metrics: Option<crate::core::traits::StrategyPerformanceMetrics>, // 性能指标
}

/// 策略状态枚举（副本，兼容事件）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StrategyStatus {
    Active,
    Paused,
    Archived,
    Error,
}

/// 注册表统计信息结构体。
#[derive(Debug, Clone)]
pub struct RegistryStatistics {
    pub total_strategies: usize, // 策略总数
    pub active_strategies: usize, // 激活策略数
    pub paused_strategies: usize, // 暂停策略数
    pub archived_strategies: usize, // 归档策略数
    pub total_access_count: u64, // 总访问次数
    pub avg_access_count: u64,   // 平均访问次数
    pub weight_type_counts: std::collections::HashMap<String, usize>, // 权重类型分布
    pub rebalancing_type_counts: std::collections::HashMap<String, usize>, // 再平衡类型分布
}

/// 策略搜索条件结构体。
#[derive(Debug, Clone)]
pub struct SearchCriteria {
    pub creator: Option<Pubkey>,
    pub weight_strategy_type: Option<WeightStrategyType>,
    pub rebalancing_strategy_type: Option<RebalancingStrategyType>,
    pub status: Option<StrategyStatus>,
    pub min_access_count: Option<u64>,
    pub created_after: Option<i64>,
    pub created_before: Option<i64>,
}

impl StrategyRegistry {
    /// 按条件搜索策略。
    pub fn search_strategies(&self, criteria: &SearchCriteria) -> Vec<&RegistryEntry> {
        self.strategies
            .iter()
            .filter(|entry| {
                // 创建者过滤。
                if let Some(creator) = criteria.creator {
                    if entry.creator != creator {
                        return false;
                    }
                }
                // 权重类型过滤。
                if let Some(weight_type) = &criteria.weight_strategy_type {
                    if entry.config.weight_config.strategy_type != *weight_type {
                        return false;
                    }
                }
                // 再平衡类型过滤。
                if let Some(rebalancing_type) = &criteria.rebalancing_strategy_type {
                    if entry.config.rebalancing_config.strategy_type != *rebalancing_type {
                        return false;
                    }
                }
                // 状态过滤。
                if let Some(status) = criteria.status {
                    if entry.status != status {
                        return false;
                    }
                }
                // 最小访问次数过滤。
                if let Some(min_access) = criteria.min_access_count {
                    if entry.access_count < min_access {
                        return false;
                    }
                }
                // 创建时间范围过滤。
                if let Some(created_after) = criteria.created_after {
                    if entry.created_at < created_after {
                        return false;
                    }
                }
                if let Some(created_before) = criteria.created_before {
                    if entry.created_at > created_before {
                        return false;
                    }
                }
                true
            })
            .collect()
    }
}

/// 策略相关事件枚举。
#[event]
pub enum StrategyEvent {
    Registered { id: u64, version: String },
    Removed { id: u64 },
    StatusChanged { id: u64, status: StrategyStatus },
    HotSwapped { id: u64, version: String },
}
