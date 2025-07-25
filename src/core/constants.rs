//!
//! Core Constants Module
//!
//! 本模块定义全局常量、边界值、精度、阈值等，确保系统参数合规、安全、可维护。

// Anchor 依赖。
use anchor_lang::prelude::*;

/// 最大基点数（10000 = 100%）。
pub const BASIS_POINTS_MAX: u64 = 10_000;
/// 价格精度（1e8，适用于主流预言机）。
pub const PRICE_PRECISION: u64 = 100_000_000;
/// 最大支持资产数量。
pub const MAX_TOKENS: usize = 16;
/// 策略参数最大字节数。
pub const MAX_STRATEGY_PARAMETERS_SIZE: usize = 256;
/// 最小再平衡间隔（秒）。
pub const MIN_REBALANCE_INTERVAL: u64 = 60;
/// 最大再平衡阈值（基点）。
pub const MAX_REBALANCE_THRESHOLD_BPS: u64 = 2_000;
/// 默认批量处理大小。
pub const DEFAULT_BATCH_SIZE: usize = 8;
/// 默认集中度限制（基点）。
pub const DEFAULT_CONCENTRATION_LIMIT_BPS: u64 = 3_000;
/// 电路断路器阈值（基点）。
pub const CIRCUIT_BREAKER_THRESHOLD_BPS: u64 = 5_000;
/// 单资产最大权重（基点）。
pub const MAX_TOKEN_WEIGHT_BPS: u64 = 10_000;
/// 最大批量处理大小。
pub const MAX_BATCH_SIZE: usize = 32;
/// 价格喂价过期阈值（秒）。
pub const PRICE_FEED_STALENESS_THRESHOLD: i64 = 60;
/// 最大滑点（基点）。
pub const MAX_SLIPPAGE_BPS: u64 = 500;
/// 最大费用（基点）。
pub const MAX_FEE_BPS: u64 = 200;
/// 缓存命中率阈值（基点）。
pub const CACHE_HIT_RATE_THRESHOLD: u32 = 9_000;

// 其他核心常量可按需扩展。
