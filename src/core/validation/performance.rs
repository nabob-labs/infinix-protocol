//!
//! performance.rs - 性能校验器实现
//!
//! 本文件实现PerformanceValidator结构体及其所有性能校验方法，严格遵循Rust、Anchor、SOLID最佳实践，
//! 并逐行专业注释，便于审计、维护、扩展。

use anchor_lang::prelude::*;
use crate::error::StrategyError;

/// 性能校验器结构体
pub struct PerformanceValidator;

impl PerformanceValidator {
    /// 校验计算预算是否充足
    pub fn validate_compute_budget(required_units: u32, available_units: u32) -> Result<()> {
        if required_units > available_units {
            return Err(StrategyError::InsufficientComputeBudget.into());
        }
        Ok(())
    }

    /// 校验内存使用是否超限
    pub fn validate_memory_usage(used_memory: usize, max_memory: usize) -> Result<()> {
        if used_memory > max_memory {
            return Err(StrategyError::MemoryLimitExceeded.into());
        }
        Ok(())
    }

    /// 校验缓存命中率
    pub fn validate_cache_performance(hit_rate_bps: u32, min_hit_rate_bps: u32) -> Result<()> {
        if hit_rate_bps < min_hit_rate_bps {
            return Err(StrategyError::CachePerformanceLow.into());
        }
        Ok(())
    }

    /// 校验执行时间
    pub fn validate_execution_time(execution_time_ms: u64, max_time_ms: u64) -> Result<()> {
        if execution_time_ms > max_time_ms {
            return Err(StrategyError::ExecutionTimeout.into());
        }
        Ok(())
    }
} 