/*!
 * Rebalancing Strategy Implementations
 *
 * Contains various rebalancing algorithms and execution strategies.
 */

use crate::core::*;
use crate::error::StrategyError;
use crate::strategies::*;
use anchor_lang::prelude::*;

/// Rebalancing strategy executor
pub struct RebalancingStrategyExecutor;

impl RebalancingStrategyExecutor {
    /// Execute threshold-based rebalancing
    pub fn execute_threshold_rebalancing(
        current_weights: &[u64],
        target_weights: &[u64],
        threshold_bps: u64,
        portfolio_value: u64,
    ) -> StrategyResult<Vec<RebalancingAction>> {
        if current_weights.len() != target_weights.len() {
            return Err(StrategyError::InvalidTokenCount.into());
        }

        let mut actions = Vec::new();

        for (i, (&current, &target)) in current_weights
            .iter()
            .zip(target_weights.iter())
            .enumerate()
        {
            let deviation = if current > target {
                current - target
            } else {
                target - current
            };

            if deviation >= threshold_bps {
                let action_type = if current > target {
                    RebalancingActionType::Sell
                } else {
                    RebalancingActionType::Buy
                };

                let amount = (portfolio_value * deviation) / BASIS_POINTS_MAX;

                actions.push(RebalancingAction {
                    token_index: i,
                    action_type,
                    amount,
                    priority: Self::calculate_priority(deviation, threshold_bps),
                });
            }
        }

        // Sort by priority (highest first)
        actions.sort_by(|a, b| b.priority.cmp(&a.priority));

        Ok(actions)
    }

    /// Execute time-based rebalancing
    pub fn execute_time_based_rebalancing(
        current_weights: &[u64],
        target_weights: &[u64],
        last_rebalance: i64,
        rebalance_interval: u64,
        portfolio_value: u64,
    ) -> StrategyResult<Vec<RebalancingAction>> {
        let current_time = Clock::get()?.unix_timestamp;
        let time_since_last = (current_time - last_rebalance) as u64;

        if time_since_last < rebalance_interval {
            return Ok(Vec::new()); // Not time to rebalance yet
        }

        // Perform full rebalancing regardless of deviation
        Self::execute_full_rebalancing(current_weights, target_weights, portfolio_value)
    }

    /// Execute volatility-triggered rebalancing
    pub fn execute_volatility_triggered_rebalancing(
        current_weights: &[u64],
        target_weights: &[u64],
        volatility_data: &[u64],
        volatility_threshold: u64,
        portfolio_value: u64,
    ) -> StrategyResult<Vec<RebalancingAction>> {
        // Check if market volatility exceeds threshold
        let avg_volatility = if volatility_data.is_empty() {
            0
        } else {
            volatility_data.iter().sum::<u64>() / volatility_data.len() as u64
        };

        if avg_volatility < volatility_threshold {
            return Ok(Vec::new()); // Volatility too low to trigger rebalancing
        }

        // Execute rebalancing with volatility-adjusted parameters
        let mut actions =
            Self::execute_full_rebalancing(current_weights, target_weights, portfolio_value)?;

        // Adjust action sizes based on volatility
        let volatility_factor = (avg_volatility * 10000) / volatility_threshold;
        for action in &mut actions {
            action.amount = (action.amount * volatility_factor) / 10000;
        }

        Ok(actions)
    }

    /// Execute drift-based rebalancing
    pub fn execute_drift_based_rebalancing(
        current_weights: &[u64],
        target_weights: &[u64],
        drift_history: &[WeightDrift],
        drift_threshold: u64,
        portfolio_value: u64,
    ) -> StrategyResult<Vec<RebalancingAction>> {
        // Calculate cumulative drift
        let cumulative_drift = Self::calculate_cumulative_drift(drift_history);

        if cumulative_drift < drift_threshold {
            return Ok(Vec::new()); // Drift not significant enough
        }

        // Execute rebalancing with drift-adjusted priorities
        let mut actions =
            Self::execute_full_rebalancing(current_weights, target_weights, portfolio_value)?;

        // Adjust priorities based on drift patterns
        for action in &mut actions {
            let drift_factor = Self::calculate_drift_factor(action.token_index, drift_history);
            action.priority = (action.priority * drift_factor) / 10000;
        }

        // Re-sort by adjusted priority
        actions.sort_by(|a, b| b.priority.cmp(&a.priority));

        Ok(actions)
    }

    /// Execute hybrid rebalancing strategy
    pub fn execute_hybrid_rebalancing(
        current_weights: &[u64],
        target_weights: &[u64],
        hybrid_params: &HybridRebalancingParams,
        market_context: &MarketContext,
        portfolio_value: u64,
    ) -> StrategyResult<Vec<RebalancingAction>> {
        let mut all_actions = Vec::new();
        let mut total_weight = 0u64;

        // Threshold-based component
        if hybrid_params.enable_threshold {
            let threshold_actions = Self::execute_threshold_rebalancing(
                current_weights,
                target_weights,
                hybrid_params.threshold_bps,
                portfolio_value,
            )?;

            for mut action in threshold_actions {
                action.priority = (action.priority * hybrid_params.threshold_weight) / 10000;
                all_actions.push(action);
            }
            total_weight += hybrid_params.threshold_weight;
        }

        // Time-based component
        if hybrid_params.enable_time {
            let time_actions = Self::execute_time_based_rebalancing(
                current_weights,
                target_weights,
                market_context.last_rebalance,
                hybrid_params.time_interval,
                portfolio_value,
            )?;

            for mut action in time_actions {
                action.priority = (action.priority * hybrid_params.time_weight) / 10000;
                all_actions.push(action);
            }
            total_weight += hybrid_params.time_weight;
        }

        // Volatility-based component
        if hybrid_params.enable_volatility {
            let vol_actions = Self::execute_volatility_triggered_rebalancing(
                current_weights,
                target_weights,
                &market_context.volatility_data,
                hybrid_params.volatility_threshold,
                portfolio_value,
            )?;

            for mut action in vol_actions {
                action.priority = (action.priority * hybrid_params.volatility_weight) / 10000;
                all_actions.push(action);
            }
            total_weight += hybrid_params.volatility_weight;
        }

        // Combine and deduplicate actions
        let combined_actions = Self::combine_actions(all_actions, total_weight)?;

        Ok(combined_actions)
    }

    /// Execute full rebalancing (bring all weights to target)
    fn execute_full_rebalancing(
        current_weights: &[u64],
        target_weights: &[u64],
        portfolio_value: u64,
    ) -> StrategyResult<Vec<RebalancingAction>> {
        if current_weights.len() != target_weights.len() {
            return Err(StrategyError::InvalidTokenCount.into());
        }

        let mut actions = Vec::new();

        for (i, (&current, &target)) in current_weights
            .iter()
            .zip(target_weights.iter())
            .enumerate()
        {
            if current != target {
                let action_type = if current > target {
                    RebalancingActionType::Sell
                } else {
                    RebalancingActionType::Buy
                };

                let deviation = if current > target {
                    current - target
                } else {
                    target - current
                };

                let amount = (portfolio_value * deviation) / BASIS_POINTS_MAX;

                actions.push(RebalancingAction {
                    token_index: i,
                    action_type,
                    amount,
                    priority: deviation, // Use deviation as priority
                });
            }
        }

        Ok(actions)
    }

    /// Calculate priority based on deviation and threshold
    fn calculate_priority(deviation: u64, threshold: u64) -> u64 {
        if threshold == 0 {
            return deviation;
        }

        // Higher priority for larger deviations relative to threshold
        (deviation * 10000) / threshold
    }

    /// Calculate cumulative drift from history
    fn calculate_cumulative_drift(drift_history: &[WeightDrift]) -> u64 {
        if drift_history.is_empty() {
            return 0;
        }

        drift_history.iter().map(|drift| drift.magnitude).sum()
    }

    /// Calculate drift factor for a specific token
    fn calculate_drift_factor(token_index: usize, drift_history: &[WeightDrift]) -> u64 {
        let token_drifts: Vec<&WeightDrift> = drift_history
            .iter()
            .filter(|drift| drift.token_index == token_index)
            .collect();

        if token_drifts.is_empty() {
            return 10000; // Neutral factor
        }

        let avg_drift = token_drifts
            .iter()
            .map(|drift| drift.magnitude)
            .sum::<u64>()
            / token_drifts.len() as u64;

        // Higher factor for tokens with more drift
        10000 + (avg_drift / 10).min(5000)
    }

    /// Combine multiple action lists and deduplicate
    fn combine_actions(
        mut all_actions: Vec<RebalancingAction>,
        total_weight: u64,
    ) -> StrategyResult<Vec<RebalancingAction>> {
        if total_weight == 0 {
            return Ok(Vec::new());
        }

        // Group actions by token index
        let mut token_actions: std::collections::HashMap<usize, Vec<RebalancingAction>> =
            std::collections::HashMap::new();

        for action in all_actions {
            token_actions
                .entry(action.token_index)
                .or_insert_with(Vec::new)
                .push(action);
        }

        // Combine actions for each token
        let mut combined_actions = Vec::new();
        for (token_index, actions) in token_actions {
            let combined_action = Self::combine_token_actions(actions, total_weight)?;
            if combined_action.amount > 0 {
                combined_actions.push(combined_action);
            }
        }

        // Sort by priority
        combined_actions.sort_by(|a, b| b.priority.cmp(&a.priority));

        Ok(combined_actions)
    }

    /// Combine multiple actions for the same token
    fn combine_token_actions(
        actions: Vec<RebalancingAction>,
        total_weight: u64,
    ) -> StrategyResult<RebalancingAction> {
        if actions.is_empty() {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        let token_index = actions[0].token_index;
        let mut net_buy_amount = 0i64;
        let mut weighted_priority = 0u64;

        for action in &actions {
            let signed_amount = match action.action_type {
                RebalancingActionType::Buy => action.amount as i64,
                RebalancingActionType::Sell => -(action.amount as i64),
            };

            net_buy_amount += signed_amount;
            weighted_priority += action.priority;
        }

        let (action_type, amount) = if net_buy_amount >= 0 {
            (RebalancingActionType::Buy, net_buy_amount as u64)
        } else {
            (RebalancingActionType::Sell, (-net_buy_amount) as u64)
        };

        let avg_priority = if total_weight > 0 {
            weighted_priority / total_weight
        } else {
            weighted_priority / actions.len() as u64
        };

        Ok(RebalancingAction {
            token_index,
            action_type,
            amount,
            priority: avg_priority,
        })
    }
}

/// Rebalancing action types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RebalancingActionType {
    Buy,
    Sell,
}

/// Rebalancing action structure
#[derive(Debug, Clone)]
pub struct RebalancingAction {
    pub token_index: usize,
    pub action_type: RebalancingActionType,
    pub amount: u64,
    pub priority: u64,
}

/// Weight drift tracking structure
#[derive(Debug, Clone)]
pub struct WeightDrift {
    pub token_index: usize,
    pub magnitude: u64,
    pub direction: DriftDirection,
    pub timestamp: i64,
}

/// Drift direction enumeration
#[derive(Debug, Clone, Copy)]
pub enum DriftDirection {
    Positive, // Weight increased
    Negative, // Weight decreased
}

/// Hybrid rebalancing parameters
#[derive(Debug, Clone)]
pub struct HybridRebalancingParams {
    pub enable_threshold: bool,
    pub enable_time: bool,
    pub enable_volatility: bool,
    pub threshold_bps: u64,
    pub time_interval: u64,
    pub volatility_threshold: u64,
    pub threshold_weight: u64,
    pub time_weight: u64,
    pub volatility_weight: u64,
}

impl Default for HybridRebalancingParams {
    fn default() -> Self {
        Self {
            enable_threshold: true,
            enable_time: true,
            enable_volatility: false,
            threshold_bps: 500,         // 5%
            time_interval: 86400,       // 24 hours
            volatility_threshold: 2000, // 20%
            threshold_weight: 5000,     // 50%
            time_weight: 3000,          // 30%
            volatility_weight: 2000,    // 20%
        }
    }
}

/// Market context for rebalancing decisions
#[derive(Debug, Clone)]
pub struct MarketContext {
    pub last_rebalance: i64,
    pub volatility_data: Vec<u64>,
    pub market_trend: MarketTrend,
    pub liquidity_conditions: LiquidityCondition,
}

/// Market trend enumeration
#[derive(Debug, Clone, Copy)]
pub enum MarketTrend {
    Bullish,
    Bearish,
    Sideways,
}

/// Liquidity condition enumeration
#[derive(Debug, Clone, Copy)]
pub enum LiquidityCondition {
    High,
    Medium,
    Low,
}
