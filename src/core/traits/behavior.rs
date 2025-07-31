//!
//! behavior.rs - 通用行为Trait定义
//!
//! 本文件定义了系统通用行为Trait，严格遵循Rust、Anchor、SOLID最佳实践，
//! 并逐行专业注释，便于审计、维护、扩展。

use anchor_lang::prelude::*;
// use crate::errors::strategy_error::StrategyError; // 暂时注释掉
use std::cmp::Ordering;
use std::collections::HashMap;

/// 可验证对象 trait
/// - 要求实现 validate 方法，统一参数合法性校验
pub trait Validatable {
    /// 校验对象参数合法性，返回 Result
    fn validate(&self) -> anchor_lang::Result<()>;
}

/// 可初始化对象 trait
/// - 要求实现 initialize 方法，支持参数化初始化
pub trait Initializable {
    /// 使用初始化参数初始化对象
    fn initialize(&mut self, params: crate::core::traits::types::InitializationParams) -> anchor_lang::Result<()>;
}

/// 可执行对象 trait
/// - 要求实现 execute/can_execute 方法，支持输入输出类型泛型
pub trait Executable {
    type Input;
    type Output;
    /// 执行操作，返回结果
    fn execute(&mut self, input: Self::Input) -> anchor_lang::Result<Self::Output>;
    /// 判断是否允许执行
    fn can_execute(&self) -> bool;
}

/// 可优化对象 trait
/// - 要求实现 optimize/optimization_score 方法
pub trait Optimizable {
    /// 执行优化操作
    fn optimize(&mut self) -> anchor_lang::Result<()>;
    /// 获取优化得分（0-10000）
    fn optimization_score(&self) -> u32;
}

/// 可缓存对象 trait
/// - 要求实现 cache_key/is_cache_valid 方法
pub trait Cacheable {
    type Key;
    /// 获取缓存 key
    fn cache_key(&self) -> Self::Key;
    /// 校验缓存是否有效
    fn is_cache_valid(&self, timestamp: i64) -> bool;
}

/// 可监控对象 trait
/// - 要求实现 collect_metrics 方法
pub trait Monitorable {
    /// 收集当前性能指标
    fn collect_metrics(&self) -> crate::core::traits::types::Metrics;
}

/// 可加锁对象 trait
/// - 支持锁定/解锁/锁拥有者查询
pub trait Lockable {
    /// 是否已锁定
    fn is_locked(&self) -> bool;
    /// 锁定对象
    fn lock(&mut self, authority: &Pubkey) -> anchor_lang::Result<()>;
    /// 解锁对象
    fn unlock(&mut self, authority: &Pubkey) -> anchor_lang::Result<()>;
    /// 查询锁拥有者
    fn lock_owner(&self) -> Option<Pubkey>;
}

/// 限流对象 trait
/// - 支持速率限制配置与检查
pub trait RateLimited {
    /// 检查速率限制
    fn check_rate_limit(&mut self) -> anchor_lang::Result<()>;
    /// 获取限流配置
    fn rate_limit_config(&self) -> crate::core::traits::types::RateLimitConfig;
    /// 更新限流配置
    fn update_rate_limit(&mut self, config: crate::core::traits::types::RateLimitConfig, authority: &Pubkey) -> anchor_lang::Result<()>;
}

/// 支持熔断 trait
/// - 支持熔断器配置与检查
pub trait CircuitBreaker {
    /// 检查熔断器状态
    fn check_circuit_breaker(&self) -> anchor_lang::Result<()>;
    /// 获取熔断器配置
    fn circuit_breaker_config(&self) -> crate::core::traits::types::CircuitBreakerConfig;
    /// 更新熔断器配置
    fn update_circuit_breaker(&mut self, config: crate::core::traits::types::CircuitBreakerConfig, authority: &Pubkey) -> anchor_lang::Result<()>;
}

/// 事件发布 trait
pub trait EventEmitter {
    type Event;
    /// 发布事件
    fn emit_event(&self, event: Self::Event) -> anchor_lang::Result<()>;
}

/// 通知处理 trait
pub trait NotificationHandler {
    type Notification;
    /// 处理通知
    fn handle_notification(&mut self, notification: Self::Notification) -> anchor_lang::Result<()>;
    /// 判断是否能处理该类型通知
    fn can_handle(&self, notification: &Self::Notification) -> bool;
}

/// 版本管理 trait
pub trait Versioned {
    /// 获取当前版本号
    fn version(&self) -> u32;
}

/// 可暂停 trait
pub trait Pausable {
    /// 检查是否暂停
    fn is_paused(&self) -> bool;
    /// 暂停
    fn pause(&mut self) -> anchor_lang::Result<()>;
    /// 恢复
    fn resume(&mut self) -> anchor_lang::Result<()>;
    /// 取消暂停
    fn unpause(&mut self) -> anchor_lang::Result<()>;
}

/// 可激活 trait
pub trait Activatable {
    /// 检查是否激活
    fn is_active(&self) -> bool;
    /// 激活
    fn activate(&mut self) -> anchor_lang::Result<()>;
    /// 失活
    fn deactivate(&mut self) -> anchor_lang::Result<()>;
}

/// 授权 trait
pub trait Authorizable {
    /// 获取当前权限
    fn authority(&self) -> Pubkey;
    /// 检查是否授权
    fn is_authorized(&self, authority: &Pubkey) -> bool {
        self.authority() == *authority
    }
    /// 转移权限
    fn transfer_authority(&mut self, new_authority: Pubkey) -> anchor_lang::Result<()>;
}

/// 配置 trait
pub trait Configurable<T> {
    /// 获取配置
    fn config(&self) -> &T;
    /// 更新配置
    fn update_config(&mut self, config: T) -> anchor_lang::Result<()>;
    /// 校验配置
    fn validate_config(config: &T) -> anchor_lang::Result<()>;
}

/// 生命周期 trait
pub trait Lifecycle {
    /// 初始化
    fn init(&mut self) -> anchor_lang::Result<()>;
    /// 启动
    fn start(&mut self) -> anchor_lang::Result<()>;
    /// 停止
    fn stop(&mut self) -> anchor_lang::Result<()>;
    /// 清理
    fn cleanup(&mut self) -> anchor_lang::Result<()>;
}

/// 风险感知 trait
pub trait RiskAware {
    /// 获取风险等级（0-10000）
    fn risk_level(&self) -> u32;
    /// 检查操作是否在风险限制内
    fn within_risk_limits(&self, operation_risk: u32) -> bool;
    /// 获取风险指标
    fn risk_metrics(&self) -> crate::core::types::RiskMetrics;
}

/// 费用感知 trait
pub trait FeeAware {
    /// 计算费用
    fn calculate_fees(&self, amount: u64) -> anchor_lang::Result<u64>;
    /// 获取费用配置
    fn fee_config(&self) -> crate::core::traits::types::FeeConfig;
    /// 更新费用配置
    fn update_fee_config(&mut self, config: crate::core::traits::types::FeeConfig) -> anchor_lang::Result<()>;
}

/// 时间感知 trait
pub trait TimeAware {
    /// 获取创建时间戳
    fn created_at(&self) -> i64;
    /// 获取更新时间戳
    fn updated_at(&self) -> i64;
    /// 更新时间戳
    fn touch(&mut self) -> anchor_lang::Result<()>;
    /// 检查是否过期
    fn is_stale(&self, max_age_seconds: i64) -> bool {
        let current_time = Clock::get().unwrap().unix_timestamp;
        current_time - self.updated_at() > max_age_seconds
    }
}

/// 可序列化 trait
pub trait Serializable {
    /// 序列化为字节
    fn serialize(&self) -> anchor_lang::Result<Vec<u8>>;
    /// 从字节反序列化
    fn deserialize(data: &[u8]) -> anchor_lang::Result<Self>
    where
        Self: Sized;
}

/// 可重置 trait
pub trait Resettable {
    /// 重置为默认状态
    fn reset(&mut self) -> anchor_lang::Result<()>;
    /// 重置为指定状态
    fn reset_to(&mut self, state: &Self) -> anchor_lang::Result<()>
    where
        Self: Clone,
    {
        *self = state.clone();
        Ok(())
    }
}

/// 可比较 trait
pub trait Comparable<T> {
    /// 比较
    fn compare(&self, other: &T) -> Ordering;
    /// 是否相等
    fn equals(&self, other: &T) -> bool {
        self.compare(other) == Ordering::Equal
    }
}

/// 可合并 trait
pub trait Mergeable<T> {
    /// 合并
    fn merge(&mut self, other: &T) -> anchor_lang::Result<()>;
    /// 是否可合并
    fn can_merge(&self, other: &T) -> bool;
}

/// 可拆分 trait
pub trait Splittable {
    /// 拆分
    fn split(&self, parts: usize) -> anchor_lang::Result<Vec<Self>>
    where
        Self: Sized;
    /// 是否可拆分
    fn can_split(&self, parts: usize) -> bool;
}

/// 备份 trait
pub trait Backup {
    type BackupData;
    /// 创建备份
    fn create_backup(&self) -> anchor_lang::Result<Self::BackupData>;
    /// 恢复备份
    fn restore_from_backup(&mut self, backup: Self::BackupData) -> anchor_lang::Result<()>;
    /// 校验备份完整性
    fn validate_backup(&self, backup: &Self::BackupData) -> anchor_lang::Result<()>;
}

/// 审计 trait
pub trait Auditable {
    type AuditEntry;
    /// 记录审计
    fn record_audit(&mut self, entry: Self::AuditEntry) -> anchor_lang::Result<()>;
    /// 获取审计轨迹
    fn audit_trail(&self) -> Vec<Self::AuditEntry>;
    /// 清空审计轨迹
    fn clear_audit_trail(&mut self, authority: &Pubkey) -> anchor_lang::Result<()>;
}

/// 配额管理 trait
pub trait QuotaManaged {
    /// 当前用量
    fn current_usage(&self) -> u64;
    /// 配额上限
    fn quota_limit(&self) -> u64;
    /// 检查是否在配额内
    fn within_quota(&self, additional_usage: u64) -> bool {
        self.current_usage() + additional_usage <= self.quota_limit()
    }
    /// 更新配额
    fn update_quota(&mut self, new_limit: u64, authority: &Pubkey) -> anchor_lang::Result<()>;
}

/// 健康检查 trait
pub trait HealthCheck {
    /// 执行健康检查
    fn health_check(&self) -> crate::core::traits::types::HealthCheckResult;
    /// 获取详细健康信息
    fn detailed_health(&self) -> HashMap<String, crate::core::traits::types::HealthMetric>;
    /// 注册健康检查回调
    fn register_health_callback(&mut self, callback: Box<dyn Fn(&crate::core::traits::types::HealthCheckResult)>) -> anchor_lang::Result<u64>;
}

/// 依赖管理 trait
pub trait DependencyManaged {
    type Dependency;
    /// 添加依赖
    fn add_dependency(&mut self, dependency: Self::Dependency) -> anchor_lang::Result<()>;
    /// 移除依赖
    fn remove_dependency(&mut self, dependency_id: &str) -> anchor_lang::Result<()>;
    /// 检查所有依赖是否健康
    fn dependencies_healthy(&self) -> bool;
    /// 获取依赖状态
    fn dependency_status(&self) -> HashMap<String, crate::core::traits::types::HealthStatus>;
} 