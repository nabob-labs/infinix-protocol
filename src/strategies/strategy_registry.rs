/*!
 * Strategy Registry Implementation
 *
 * Registry for managing and tracking strategy instances.
 */

use crate::core::*;
use crate::error::StrategyError;
use crate::strategies::*;
use anchor_lang::prelude::*;

/// Strategy registry for tracking and managing strategies
pub struct StrategyRegistry {
    strategies: Vec<RegistryEntry>,
    next_id: u64,
}

impl StrategyRegistry {
    /// Create a new strategy registry
    pub fn new() -> Self {
        Self {
            strategies: Vec::new(),
            next_id: 1,
        }
    }

    /// Register a new strategy
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

    /// Get strategy by ID
    pub fn get_strategy(&mut self, strategy_id: u64) -> Option<&mut RegistryEntry> {
        if let Some(entry) = self.strategies.iter_mut().find(|e| e.id == strategy_id) {
            entry.last_accessed = Clock::get().unwrap().unix_timestamp;
            entry.access_count += 1;
            Some(entry)
        } else {
            None
        }
    }

    /// Get strategies by creator
    pub fn get_strategies_by_creator(&self, creator: &Pubkey) -> Vec<&RegistryEntry> {
        self.strategies
            .iter()
            .filter(|entry| entry.creator == *creator)
            .collect()
    }

    /// Get strategies by type
    pub fn get_strategies_by_weight_type(
        &self,
        strategy_type: &WeightStrategyType,
    ) -> Vec<&RegistryEntry> {
        self.strategies
            .iter()
            .filter(|entry| entry.config.weight_config.strategy_type == *strategy_type)
            .collect()
    }

    /// Get active strategies
    pub fn get_active_strategies(&self) -> Vec<&RegistryEntry> {
        self.strategies
            .iter()
            .filter(|entry| entry.status == StrategyStatus::Active)
            .collect()
    }

    /// Update strategy status
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

    /// Remove strategy from registry
    pub fn remove_strategy(&mut self, strategy_id: u64) -> StrategyResult<()> {
        let initial_len = self.strategies.len();
        self.strategies.retain(|entry| entry.id != strategy_id);

        if self.strategies.len() < initial_len {
            Ok(())
        } else {
            Err(StrategyError::InvalidStrategyParameters.into())
        }
    }

    /// Get registry statistics
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

        // Count strategies by type
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

    /// Cleanup old or unused strategies
    pub fn cleanup_strategies(&mut self, max_age_seconds: i64) -> StrategyResult<usize> {
        let current_time = Clock::get()?.unix_timestamp;
        let initial_count = self.strategies.len();

        self.strategies.retain(|entry| {
            // Keep active strategies regardless of age
            if entry.status == StrategyStatus::Active {
                return true;
            }

            // Remove old archived strategies
            if entry.status == StrategyStatus::Archived {
                let age = current_time - entry.last_accessed;
                return age < max_age_seconds;
            }

            // Keep paused strategies (they might be reactivated)
            true
        });

        let removed_count = initial_count - self.strategies.len();
        Ok(removed_count)
    }

    /// Find strategies that need rebalancing
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

    /// Update strategy performance metrics
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
}

impl Default for StrategyRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Registry entry for a strategy
#[derive(Debug, Clone)]
pub struct RegistryEntry {
    pub id: u64,
    pub config: StrategyConfig,
    pub creator: Pubkey,
    pub created_at: i64,
    pub last_accessed: i64,
    pub access_count: u64,
    pub status: StrategyStatus,
    pub performance_metrics: Option<crate::core::traits::StrategyPerformanceMetrics>,
}

/// Strategy status enumeration
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StrategyStatus {
    Active,
    Paused,
    Archived,
    Error,
}

/// Registry statistics
#[derive(Debug, Clone)]
pub struct RegistryStatistics {
    pub total_strategies: usize,
    pub active_strategies: usize,
    pub paused_strategies: usize,
    pub archived_strategies: usize,
    pub total_access_count: u64,
    pub avg_access_count: u64,
    pub weight_type_counts: std::collections::HashMap<String, usize>,
    pub rebalancing_type_counts: std::collections::HashMap<String, usize>,
}

/// Strategy search criteria
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
    /// Search strategies based on criteria
    pub fn search_strategies(&self, criteria: &SearchCriteria) -> Vec<&RegistryEntry> {
        self.strategies
            .iter()
            .filter(|entry| {
                // Filter by creator
                if let Some(creator) = criteria.creator {
                    if entry.creator != creator {
                        return false;
                    }
                }

                // Filter by weight strategy type
                if let Some(weight_type) = &criteria.weight_strategy_type {
                    if entry.config.weight_config.strategy_type != *weight_type {
                        return false;
                    }
                }

                // Filter by rebalancing strategy type
                if let Some(rebalancing_type) = &criteria.rebalancing_strategy_type {
                    if entry.config.rebalancing_config.strategy_type != *rebalancing_type {
                        return false;
                    }
                }

                // Filter by status
                if let Some(status) = criteria.status {
                    if entry.status != status {
                        return false;
                    }
                }

                // Filter by minimum access count
                if let Some(min_access) = criteria.min_access_count {
                    if entry.access_count < min_access {
                        return false;
                    }
                }

                // Filter by creation date range
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
