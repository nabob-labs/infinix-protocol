/*!
 * Caching Utilities Module
 *
 * Efficient caching mechanisms for frequently accessed data.
 */

use crate::core::*;
use crate::core::constants::{DEFAULT_CACHE_TTL, MAX_CACHE_ENTRY_SIZE};
use anchor_lang::prelude::*;
use std::collections::HashMap;

/// Simple cache implementation with TTL support (not thread-safe, single-threaded use only)
pub struct SimpleCache<K, V>
where
    K: std::hash::Hash + Eq + Clone,
    V: Clone,
{
    entries: HashMap<K, CacheEntry<V>>,
    ttl_seconds: i64,
    max_size: usize,
    hits: u64,
    misses: u64,
}

/// Cache entry with timestamp
#[derive(Debug, Clone)]
struct CacheEntry<V> {
    value: V,
    timestamp: i64,
    access_count: u64,
}

impl<K, V> SimpleCache<K, V>
where
    K: std::hash::Hash + Eq + Clone,
    V: Clone,
{
    /// Create new cache with TTL and size limits
    pub fn new(ttl_seconds: i64, max_size: usize) -> Self {
        Self {
            entries: HashMap::new(),
            ttl_seconds,
            max_size,
            hits: 0,
            misses: 0,
        }
    }

    /// Get value from cache
    pub fn get(&mut self, key: &K) -> Option<V> {
        if let Some(entry) = self.entries.get_mut(key) {
            let current_time = Clock::get().ok()?.unix_timestamp;

            // Check if entry is still valid
            if current_time - entry.timestamp < self.ttl_seconds {
                entry.access_count += 1;
                self.hits += 1;
                return Some(entry.value.clone());
            } else {
                // Remove expired entry
                self.entries.remove(key);
            }
        }

        self.misses += 1;
        None
    }

    /// Set value in cache
    pub fn set(&mut self, key: K, value: V) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;

        // Check if we need to evict entries
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

    /// Remove expired entries
    pub fn cleanup_expired(&mut self) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;

        self.entries
            .retain(|_, entry| current_time - entry.timestamp < self.ttl_seconds);

        Ok(())
    }

    /// Evict least recently used entry
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

    /// Get cache statistics
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

    /// Clear all entries
    pub fn clear(&mut self) {
        self.entries.clear();
        self.hits = 0;
        self.misses = 0;
    }

    /// Check if cache contains key
    pub fn contains_key(&self, key: &K) -> bool {
        if let Some(entry) = self.entries.get(key) {
            if let Ok(clock) = Clock::get() {
                clock.unix_timestamp - entry.timestamp < self.ttl_seconds
            } else {
                false // Consider expired if we can't get current time
            }
        } else {
            false
        }
    }

    /// Get cache size
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub hit_rate: u64, // In basis points
    pub entry_count: usize,
    pub max_size: usize,
}

/// Price cache specifically for price feeds
pub type PriceCache = SimpleCache<Pubkey, PriceFeed>;

/// Weight cache for calculated weights
pub type WeightCache = SimpleCache<u64, Vec<u64>>;

/// Strategy result cache
pub type StrategyCache = SimpleCache<String, Vec<u8>>;

/// Cache manager for coordinating multiple caches
pub struct CacheManager {
    price_cache: PriceCache,
    weight_cache: WeightCache,
    strategy_cache: StrategyCache,
}

impl CacheManager {
    /// Create new cache manager
    pub fn new() -> Self {
        Self {
            price_cache: PriceCache::new(DEFAULT_CACHE_TTL, MAX_CACHE_ENTRY_SIZE),
            weight_cache: WeightCache::new(DEFAULT_CACHE_TTL, 500),
            strategy_cache: StrategyCache::new(DEFAULT_CACHE_TTL, 200),
        }
    }

    /// Get price from cache
    pub fn get_price(&mut self, mint: &Pubkey) -> Option<PriceFeed> {
        self.price_cache.get(mint)
    }

    /// Set price in cache
    pub fn set_price(&mut self, mint: Pubkey, price_feed: PriceFeed) -> Result<()> {
        self.price_cache.set(mint, price_feed)
    }

    /// Get weights from cache
    pub fn get_weights(&mut self, strategy_id: u64) -> Option<Vec<u64>> {
        self.weight_cache.get(&strategy_id)
    }

    /// Set weights in cache
    pub fn set_weights(&mut self, strategy_id: u64, weights: Vec<u64>) -> Result<()> {
        self.weight_cache.set(strategy_id, weights)
    }

    /// Cleanup all expired entries
    pub fn cleanup_all(&mut self) -> Result<()> {
        self.price_cache.cleanup_expired()?;
        self.weight_cache.cleanup_expired()?;
        self.strategy_cache.cleanup_expired()?;
        Ok(())
    }

    /// Get combined cache statistics
    pub fn get_combined_stats(&self) -> CombinedCacheStats {
        CombinedCacheStats {
            price_stats: self.price_cache.get_stats(),
            weight_stats: self.weight_cache.get_stats(),
            strategy_stats: self.strategy_cache.get_stats(),
        }
    }

    /// Clear all caches
    pub fn clear_all(&mut self) {
        self.price_cache.clear();
        self.weight_cache.clear();
        self.strategy_cache.clear();
    }
}

/// Combined cache statistics
#[derive(Debug, Clone)]
pub struct CombinedCacheStats {
    pub price_stats: CacheStats,
    pub weight_stats: CacheStats,
    pub strategy_stats: CacheStats,
}

impl Default for CacheManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache warming utilities
pub struct CacheWarmer;

impl CacheWarmer {
    /// Warm price cache with common tokens
    pub fn warm_price_cache(cache: &mut PriceCache, common_mints: &[Pubkey]) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;

        for mint in common_mints {
            // Create mock price feed for warming
            let price_feed = PriceFeed {
                mint: *mint,
                price: PRICE_PRECISION, // $1.00 default
                confidence: 1000,
                last_updated: current_time,
                is_valid: true,
                source: PriceFeedSource::Custom("cache_warmer".to_string()),
            };

            cache.set(*mint, price_feed)?;
        }

        Ok(())
    }

    /// Warm weight cache with common strategies
    pub fn warm_weight_cache(
        cache: &mut WeightCache,
        strategy_configs: &[(u64, usize)],
    ) -> Result<()> {
        for (strategy_id, token_count) in strategy_configs {
            // Create equal weight distribution
            let equal_weight = BASIS_POINTS_MAX / *token_count as u64;
            let weights = vec![equal_weight; *token_count];

            cache.set(*strategy_id, weights)?;
        }

        Ok(())
    }
}
