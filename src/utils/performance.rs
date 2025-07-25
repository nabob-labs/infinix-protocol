//!
//! Performance Monitoring and Optimization Utilities
//!
//! 本模块实现程序性能监控与优化工具，支持执行时间、Gas、命中率、批量优化、执行策略等度量与分析，提升系统可观测性与运行效率。

// 引入核心模块和 Anchor 依赖。
use crate::core::*;
use anchor_lang::prelude::*;

/// 性能监控工具结构体。
pub struct PerformanceMonitor;

impl PerformanceMonitor {
    /// 启动性能测量，记录起始时间与 slot。
    pub fn start_measurement() -> PerformanceMeasurement {
        let clock = Clock::get().unwrap_or_default();
        PerformanceMeasurement {
            start_time: clock.unix_timestamp, // 起始时间戳
            start_slot: clock.slot,           // 起始 slot
            compute_units_start: 0,           // 生产环境可记录实际消耗
        }
    }
    /// 计算执行度量指标。
    pub fn calculate_metrics(
        measurement: &PerformanceMeasurement,
        gas_used: u64,
        success: bool,
    ) -> PerformanceMetrics {
        let current_time = Clock::get().unwrap().unix_timestamp;
        // 计算执行时间（毫秒）。
        let execution_time = (current_time - measurement.start_time).max(0) as u64 * 1000;
        PerformanceMetrics {
            gas_used,
            execution_time_ms: execution_time,
            slippage_bps: 0, // 实际应由成交数据计算
            success_rate_bps: if success { BASIS_POINTS_MAX as u16 } else { 0 },
            mev_protection_score: 8000, // 默认良好
        }
    }
    /// 日志输出性能指标。
    pub fn log_metrics(metrics: &PerformanceMetrics, operation: &str) {
        msg!(
            "Performance - {}: Gas={}, Time={}ms, Success={}%",
            operation,
            metrics.gas_used,
            metrics.execution_time_ms,
            metrics.success_rate_bps / 100
        );
    }
    /// 检查性能是否达标。
    pub fn meets_performance_thresholds(
        metrics: &PerformanceMetrics,
        thresholds: &PerformanceThresholds,
    ) -> bool {
        metrics.gas_used <= thresholds.max_gas
            && metrics.execution_time_ms <= thresholds.max_execution_time_ms
            && u64::from(metrics.success_rate_bps) >= thresholds.min_success_rate_bps
    }
}

/// 性能测量上下文结构体。
#[derive(Debug, Clone)]
pub struct PerformanceMeasurement {
    pub start_time: i64,         // 起始时间戳
    pub start_slot: u64,         // 起始 slot
    pub compute_units_start: u32,// 起始计算单元
}

/// 性能阈值结构体。
#[derive(Debug, Clone)]
pub struct PerformanceThresholds {
    pub max_gas: u64,                // 最大 Gas
    pub max_execution_time_ms: u64,  // 最大执行时间（毫秒）
    pub min_success_rate_bps: u64,   // 最小成功率（基点）
    pub max_slippage_bps: u64,       // 最大滑点（基点）
}

impl Default for PerformanceThresholds {
    fn default() -> Self {
        Self {
            max_gas: 1_000_000,
            max_execution_time_ms: 30_000, // 30 秒
            min_success_rate_bps: 9000,    // 90%
            max_slippage_bps: 500,         // 5%
        }
    }
}

/// 性能指标结构体。
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub gas_used: u64,                // Gas 消耗
    pub execution_time_ms: u64,       // 执行时间（毫秒）
    pub slippage_bps: u16,            // 滑点（基点）
    pub success_rate_bps: u16,        // 成功率（基点）
    pub mev_protection_score: u16,    // MEV 防护分数
}

/// 性能优化工具结构体。
pub struct PerformanceOptimizer;

impl PerformanceOptimizer {
    /// 根据计算资源优化批量大小。
    pub fn optimize_batch_size(
        item_count: usize,
        compute_per_item: u32,
        available_compute: u32,
    ) -> usize {
        if compute_per_item == 0 {
            return item_count;
        }
        let max_items = (available_compute / compute_per_item) as usize;
        max_items.min(item_count).max(1)
    }
    /// 计算最优执行策略。
    pub fn calculate_execution_strategy(
        total_amount: u64,
        market_liquidity: u64,
        time_constraint: u64,
    ) -> ExecutionStrategy {
        let urgency = if time_constraint < 300 {
            // 小于 5 分钟
            ExecutionUrgency::High
        } else if time_constraint < 3600 {
            // 小于 1 小时
            ExecutionUrgency::Medium
        } else {
            ExecutionUrgency::Low
        };
        let batch_count = if market_liquidity > total_amount * 10 {
            1 // 高流动性单批次
        } else if market_liquidity > total_amount * 3 {
            3 // 中等流动性多批次
        } else {
            5 // 低流动性多小批次
        };
        let max_slippage_per_batch = match urgency {
            ExecutionUrgency::Low => 100,    // 1%
            ExecutionUrgency::Medium => 200, // 2%
            ExecutionUrgency::High => 500,   // 5%
        };
        ExecutionStrategy {
            urgency,
            batch_count,
            time_per_batch: time_constraint / batch_count,
            max_slippage_per_batch,
        }
    }
}

/// 执行策略结构体。
#[derive(Debug, Clone)]
pub struct ExecutionStrategy {
    pub urgency: ExecutionUrgency,        // 紧急程度
    pub batch_count: u64,                 // 批次数
    pub time_per_batch: u64,              // 每批次时间
    pub max_slippage_per_batch: u64,      // 每批次最大滑点
}

/// 执行紧急程度枚举。
#[derive(Debug, Clone)]
pub enum ExecutionUrgency {
    Low,    // 低
    Medium, // 中
    High,   // 高
}

/// 缓存性能监控工具结构体。
pub struct CacheMonitor;

impl CacheMonitor {
    /// 计算缓存命中率。
    pub fn calculate_hit_rate(hits: u64, total_requests: u64) -> u32 {
        if total_requests == 0 {
            return 0;
        }
        ((hits * BASIS_POINTS_MAX) / total_requests) as u32
    }
    /// 检查缓存性能是否达标。
    pub fn is_performance_acceptable(hit_rate: u32) -> bool {
        hit_rate >= CACHE_HIT_RATE_THRESHOLD
    }
    /// 日志输出缓存性能。
    pub fn log_cache_performance(hits: u64, misses: u64, evictions: u64) {
        let total = hits + misses;
        let hit_rate = if total > 0 { (hits * 100) / total } else { 0 };
        msg!(
            "Cache Performance - Hit Rate: {}%, Evictions: {}",
            hit_rate,
            evictions
        );
    }
}
