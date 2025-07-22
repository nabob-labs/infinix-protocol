/*!
 * Performance Monitoring and Optimization Utilities
 *
 * Tools for measuring and optimizing program performance.
 */

use crate::core::*;
use anchor_lang::prelude::*;

/// Performance monitoring utilities
pub struct PerformanceMonitor;

impl PerformanceMonitor {
    /// Start performance measurement
    pub fn start_measurement() -> PerformanceMeasurement {
        let clock = Clock::get().unwrap_or_default();
        PerformanceMeasurement {
            start_time: clock.unix_timestamp,
            start_slot: clock.slot,
            compute_units_start: 0, // Would be actual compute units in production
        }
    }

    /// Calculate execution metrics
    pub fn calculate_metrics(
        measurement: &PerformanceMeasurement,
        gas_used: u64,
        success: bool,
    ) -> PerformanceMetrics {
        let current_time = Clock::get().unwrap().unix_timestamp;
        let execution_time = (current_time - measurement.start_time).max(0) as u64 * 1000; // Convert to ms

        PerformanceMetrics {
            gas_used,
            execution_time_ms: execution_time,
            slippage_bps: 0, // Would be calculated from actual trades
            success_rate_bps: if success { BASIS_POINTS_MAX as u16 } else { 0 },
            mev_protection_score: 8000, // Default good score
        }
    }

    /// Log performance metrics
    pub fn log_metrics(metrics: &PerformanceMetrics, operation: &str) {
        msg!(
            "Performance - {}: Gas={}, Time={}ms, Success={}%",
            operation,
            metrics.gas_used,
            metrics.execution_time_ms,
            metrics.success_rate_bps / 100
        );
    }

    /// Check if performance meets thresholds
    pub fn meets_performance_thresholds(
        metrics: &PerformanceMetrics,
        thresholds: &PerformanceThresholds,
    ) -> bool {
        metrics.gas_used <= thresholds.max_gas
            && metrics.execution_time_ms <= thresholds.max_execution_time_ms
            && u64::from(metrics.success_rate_bps) >= thresholds.min_success_rate_bps
    }
}

/// Performance measurement context
#[derive(Debug, Clone)]
pub struct PerformanceMeasurement {
    pub start_time: i64,
    pub start_slot: u64,
    pub compute_units_start: u32,
}

/// Performance thresholds for validation
#[derive(Debug, Clone)]
pub struct PerformanceThresholds {
    pub max_gas: u64,
    pub max_execution_time_ms: u64,
    pub min_success_rate_bps: u64,
    pub max_slippage_bps: u64,
}

impl Default for PerformanceThresholds {
    fn default() -> Self {
        Self {
            max_gas: 1_000_000,
            max_execution_time_ms: 30_000, // 30 seconds
            min_success_rate_bps: 9000,    // 90%
            max_slippage_bps: 500,         // 5%
        }
    }
}

/// Performance optimization utilities
pub struct PerformanceOptimizer;

impl PerformanceOptimizer {
    /// Optimize batch size based on compute constraints
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

    /// Calculate optimal execution strategy
    pub fn calculate_execution_strategy(
        total_amount: u64,
        market_liquidity: u64,
        time_constraint: u64,
    ) -> ExecutionStrategy {
        let urgency = if time_constraint < 300 {
            // Less than 5 minutes
            ExecutionUrgency::High
        } else if time_constraint < 3600 {
            // Less than 1 hour
            ExecutionUrgency::Medium
        } else {
            ExecutionUrgency::Low
        };

        let batch_count = if market_liquidity > total_amount * 10 {
            1 // Single batch if high liquidity
        } else if market_liquidity > total_amount * 3 {
            3 // Multiple batches for medium liquidity
        } else {
            5 // Many small batches for low liquidity
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

/// Execution strategy for performance optimization
#[derive(Debug, Clone)]
pub struct ExecutionStrategy {
    pub urgency: ExecutionUrgency,
    pub batch_count: u64,
    pub time_per_batch: u64,
    pub max_slippage_per_batch: u64,
}

/// Execution urgency levels
#[derive(Debug, Clone)]
pub enum ExecutionUrgency {
    Low,
    Medium,
    High,
}

/// Cache performance monitoring
pub struct CacheMonitor;

impl CacheMonitor {
    /// Calculate cache hit rate
    pub fn calculate_hit_rate(hits: u64, total_requests: u64) -> u32 {
        if total_requests == 0 {
            return 0;
        }

        ((hits * BASIS_POINTS_MAX) / total_requests) as u32
    }

    /// Check if cache performance is acceptable
    pub fn is_performance_acceptable(hit_rate: u32) -> bool {
        hit_rate >= CACHE_HIT_RATE_THRESHOLD
    }

    /// Log cache performance
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
