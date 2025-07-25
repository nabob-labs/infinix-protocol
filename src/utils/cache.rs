//!
//! Caching Utilities Module
//!
//! 本模块实现高效缓存机制，支持 TTL、LRU、统计、批量管理等，适用于频繁访问的数据缓存，提升性能与可用性。

// 引入核心模块、常量、Anchor 依赖和标准库。
use crate::core::*;
use crate::core::constants::{DEFAULT_CACHE_TTL, MAX_CACHE_ENTRY_SIZE};
use anchor_lang::prelude::*;
use std::collections::HashMap;

/// 简单缓存实现，支持 TTL（非线程安全，仅限单线程场景）。
pub struct SimpleCache<K, V>
where
    K: std::hash::Hash + Eq + Clone,
    V: Clone,
{
    entries: HashMap<K, CacheEntry<V>>, // 缓存条目表
    ttl_seconds: i64,                   // 每条目有效期（秒）
    max_size: usize,                    // 最大缓存容量
    hits: u64,                          // 命中次数
    misses: u64,                        // 未命中次数
}

/// 缓存条目结构体，包含值、时间戳、访问计数。
#[derive(Debug, Clone)]
struct CacheEntry<V> {
    value: V,           // 缓存值
    timestamp: i64,     // 写入时间戳
    access_count: u64,  // 访问次数
}

impl<K, V> SimpleCache<K, V>
where
    K: std::hash::Hash + Eq + Clone,
    V: Clone,
{
    /// 创建新缓存，指定 TTL 和最大容量。
    pub fn new(ttl_seconds: i64, max_size: usize) -> Self {
        Self {
            entries: HashMap::new(),
            ttl_seconds,
            max_size,
            hits: 0,
            misses: 0,
        }
    }
    /// 从缓存获取值。
    pub fn get(&mut self, key: &K) -> Option<V> {
        if let Some(entry) = self.entries.get_mut(key) {
            let current_time = Clock::get().ok()?.unix_timestamp;
            // 检查条目是否过期。
            if current_time - entry.timestamp < self.ttl_seconds {
                entry.access_count += 1;
                self.hits += 1;
                return Some(entry.value.clone());
            } else {
                // 移除过期条目。
                self.entries.remove(key);
            }
        }
        self.misses += 1;
        None
    }
    /// 设置缓存值。
    pub fn set(&mut self, key: K, value: V) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;
        // 超容量时淘汰最少使用条目。
        if self.entries.len() >= self.max_size && !self.entries.contains_key(&key) {
            self.evict_lru();
        }
        let entry = CacheEntry {
            value,
            timestamp: current_time,
            access_count: 1,
        };
        self.entries.insert(key, entry);
        Ok(())
    }
    /// 清理所有过期条目。
    pub fn cleanup_expired(&mut self) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;
        self.entries
            .retain(|_, entry| current_time - entry.timestamp < self.ttl_seconds);
        Ok(())
    }
    /// 淘汰最少使用（LRU）条目。
    fn evict_lru(&mut self) {
        if let Some((key_to_remove, _)) = self
            .entries
            .iter()
            .min_by_key(|(_, entry)| entry.access_count)
            .map(|(k, v)| (k.clone(), v.clone()))
        {
            self.entries.remove(&key_to_remove);
        }
    }
    /// 获取缓存统计信息。
    pub fn get_stats(&self) -> CacheStats {
        let total_requests = self.hits + self.misses;
        let hit_rate = if total_requests > 0 {
            (self.hits * BASIS_POINTS_MAX) / total_requests
        } else {
            0
        };
        CacheStats {
            hits: self.hits,
            misses: self.misses,
            hit_rate,
            entry_count: self.entries.len(),
            max_size: self.max_size,
        }
    }
    /// 清空所有缓存条目。
    pub fn clear(&mut self) {
        self.entries.clear();
        self.hits = 0;
        self.misses = 0;
    }
    /// 检查缓存是否包含指定 key，且未过期。
    pub fn contains_key(&self, key: &K) -> bool {
        if let Some(entry) = self.entries.get(key) {
            if let Ok(clock) = Clock::get() {
                clock.unix_timestamp - entry.timestamp < self.ttl_seconds
            } else {
                false // 无法获取时间时视为过期
            }
        } else {
            false
        }
    }
    /// 获取缓存当前条目数。
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    /// 检查缓存是否为空。
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// 缓存统计信息结构体。
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub hits: u64,           // 命中次数
    pub misses: u64,         // 未命中次数
    pub hit_rate: u64,       // 命中率（基点）
    pub entry_count: usize,  // 当前条目数
    pub max_size: usize,     // 最大容量
}

/// 价格缓存类型，key 为资产 mint。
pub type PriceCache = SimpleCache<Pubkey, PriceFeed>;
/// 权重缓存类型，key 为策略 ID。
pub type WeightCache = SimpleCache<u64, Vec<u64>>;
/// 策略结果缓存类型，key 为策略名。
pub type StrategyCache = SimpleCache<String, Vec<u8>>;

/// 缓存管理器，协调多种缓存。
pub struct CacheManager {
    price_cache: PriceCache,         // 价格缓存
    weight_cache: WeightCache,       // 权重缓存
    strategy_cache: StrategyCache,   // 策略缓存
}

impl CacheManager {
    /// 创建新缓存管理器。
    pub fn new() -> Self {
        Self {
            price_cache: PriceCache::new(DEFAULT_CACHE_TTL, MAX_CACHE_ENTRY_SIZE),
            weight_cache: WeightCache::new(DEFAULT_CACHE_TTL, 500),
            strategy_cache: StrategyCache::new(DEFAULT_CACHE_TTL, 200),
        }
    }
    /// 获取价格缓存。
    pub fn get_price(&mut self, mint: &Pubkey) -> Option<PriceFeed> {
        self.price_cache.get(mint)
    }
    /// 设置价格缓存。
    pub fn set_price(&mut self, mint: Pubkey, price_feed: PriceFeed) -> Result<()> {
        self.price_cache.set(mint, price_feed)
    }
    /// 获取权重缓存。
    pub fn get_weights(&mut self, strategy_id: u64) -> Option<Vec<u64>> {
        self.weight_cache.get(&strategy_id)
    }
    /// 设置权重缓存。
    pub fn set_weights(&mut self, strategy_id: u64, weights: Vec<u64>) -> Result<()> {
        self.weight_cache.set(strategy_id, weights)
    }
    /// 清理所有过期条目。
    pub fn cleanup_all(&mut self) -> Result<()> {
        self.price_cache.cleanup_expired()?;
        self.weight_cache.cleanup_expired()?;
        self.strategy_cache.cleanup_expired()?;
        Ok(())
    }
    /// 获取所有缓存的统计信息。
    pub fn get_combined_stats(&self) -> CombinedCacheStats {
        CombinedCacheStats {
            price_stats: self.price_cache.get_stats(),
            weight_stats: self.weight_cache.get_stats(),
            strategy_stats: self.strategy_cache.get_stats(),
        }
    }
    /// 清空所有缓存。
    pub fn clear_all(&mut self) {
        self.price_cache.clear();
        self.weight_cache.clear();
        self.strategy_cache.clear();
    }
}

/// 组合缓存统计信息结构体。
#[derive(Debug, Clone)]
pub struct CombinedCacheStats {
    pub price_stats: CacheStats,     // 价格缓存统计
    pub weight_stats: CacheStats,    // 权重缓存统计
    pub strategy_stats: CacheStats,  // 策略缓存统计
}

impl Default for CacheManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 缓存预热工具。
pub struct CacheWarmer;

impl CacheWarmer {
    /// 预热价格缓存。
    pub fn warm_price_cache(cache: &mut PriceCache, common_mints: &[Pubkey]) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;
        for mint in common_mints {
            // 构造 mock 价格数据。
            let price_feed = PriceFeed {
                mint: *mint,
                price: PRICE_PRECISION, // 默认 $1.00
                confidence: 1000,
                last_updated: current_time,
                is_valid: true,
                source: PriceFeedSource::Custom("cache_warmer".to_string()),
            };
            cache.set(*mint, price_feed)?;
        }
        Ok(())
    }
    /// 预热权重缓存。
    pub fn warm_weight_cache(
        cache: &mut WeightCache,
        strategy_configs: &[(u64, usize)],
    ) -> Result<()> {
        for (strategy_id, token_count) in strategy_configs {
            // 构造等权重分布。
            let equal_weight = BASIS_POINTS_MAX / *token_count as u64;
            let weights = vec![equal_weight; *token_count];
            cache.set(*strategy_id, weights)?;
        }
        Ok(())
    }
}
