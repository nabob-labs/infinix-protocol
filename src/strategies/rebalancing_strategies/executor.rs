//!
//! executor.rs - 再平衡策略执行器实现
//!
//! 本文件实现RebalancingStrategyExecutor及其所有再平衡算法与辅助函数，严格遵循Rust、Anchor、SOLID最佳实践，
//! 并逐行专业注释，便于审计、维护、扩展。

use crate::core::*;
use crate::error::StrategyError;
use crate::strategies::*;
use anchor_lang::prelude::*;

/// 再平衡策略执行器结构体，支持多种再平衡算法。
pub struct RebalancingStrategyExecutor;

impl RebalancingStrategyExecutor {
    /// 执行阈值再平衡。
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
        for (i, (&current, &target)) in current_weights.iter().zip(target_weights.iter()).enumerate() {
            let deviation = if current > target { current - target } else { target - current };
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
        actions.sort_by(|a, b| b.priority.cmp(&a.priority));
        Ok(actions)
    }
    /// 执行定时再平衡。
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
            return Ok(Vec::new());
        }
        Self::execute_full_rebalancing(current_weights, target_weights, portfolio_value)
    }
    /// 执行波动率触发再平衡。
    pub fn execute_volatility_triggered_rebalancing(
        current_weights: &[u64],
        target_weights: &[u64],
        volatility_data: &[u64],
        volatility_threshold: u64,
        portfolio_value: u64,
    ) -> StrategyResult<Vec<RebalancingAction>> {
        let avg_volatility = if volatility_data.is_empty() {
            0
        } else {
            volatility_data.iter().sum::<u64>() / volatility_data.len() as u64
        };
        if avg_volatility < volatility_threshold {
            return Ok(Vec::new());
        }
        Self::execute_full_rebalancing(current_weights, target_weights, portfolio_value)
    }
    /// 执行漂移触发再平衡。
    pub fn execute_drift_based_rebalancing(
        current_weights: &[u64],
        target_weights: &[u64],
        drift_history: &[WeightDrift],
        drift_threshold: u64,
        portfolio_value: u64,
    ) -> StrategyResult<Vec<RebalancingAction>> {
        let cumulative_drift = Self::calculate_cumulative_drift(drift_history);
        if cumulative_drift < drift_threshold {
            return Ok(Vec::new());
        }
        Self::execute_full_rebalancing(current_weights, target_weights, portfolio_value)
    }
    /// 执行混合再平衡。
    pub fn execute_hybrid_rebalancing(
        current_weights: &[u64],
        target_weights: &[u64],
        hybrid_params: &HybridRebalancingParams,
        market_context: &MarketContext,
        portfolio_value: u64,
    ) -> StrategyResult<Vec<RebalancingAction>> {
        let mut actions = Vec::new();
        if hybrid_params.enable_threshold {
            let threshold_actions = Self::execute_threshold_rebalancing(
                current_weights,
                target_weights,
                hybrid_params.threshold_bps,
                portfolio_value,
            )?;
            actions.extend(threshold_actions);
        }
        if hybrid_params.enable_time {
            let time_actions = Self::execute_time_based_rebalancing(
                current_weights,
                target_weights,
                market_context.last_rebalance,
                hybrid_params.time_interval,
                portfolio_value,
            )?;
            actions.extend(time_actions);
        }
        if hybrid_params.enable_volatility {
            let vol_actions = Self::execute_volatility_triggered_rebalancing(
                current_weights,
                target_weights,
                &market_context.volatility_data,
                hybrid_params.volatility_threshold,
                portfolio_value,
            )?;
            actions.extend(vol_actions);
        }
        Self::combine_actions(actions, BASIS_POINTS_MAX)
    }
    /// 执行全量再平衡。
    fn execute_full_rebalancing(
        current_weights: &[u64],
        target_weights: &[u64],
        portfolio_value: u64,
    ) -> StrategyResult<Vec<RebalancingAction>> {
        if current_weights.len() != target_weights.len() {
            return Err(StrategyError::InvalidTokenCount.into());
        }
        let mut actions = Vec::new();
        for (i, (&current, &target)) in current_weights.iter().zip(target_weights.iter()).enumerate() {
            if current != target {
                let action_type = if current > target {
                    RebalancingActionType::Sell
                } else {
                    RebalancingActionType::Buy
                };
                let deviation = if current > target { current - target } else { target - current };
                let amount = (portfolio_value * deviation) / BASIS_POINTS_MAX;
                actions.push(RebalancingAction {
                    token_index: i,
                    action_type,
                    amount,
                    priority: Self::calculate_priority(deviation, 1),
                });
            }
        }
        Ok(actions)
    }
    /// 计算优先级。
    fn calculate_priority(deviation: u64, threshold: u64) -> u64 {
        deviation / threshold
    }
    /// 计算累计漂移。
    fn calculate_cumulative_drift(drift_history: &[WeightDrift]) -> u64 {
        drift_history.iter().map(|d| d.magnitude).sum()
    }
    /// 计算单资产漂移因子。
    fn calculate_drift_factor(token_index: usize, drift_history: &[WeightDrift]) -> u64 {
        drift_history
            .iter()
            .filter(|d| d.token_index == token_index)
            .map(|d| d.magnitude)
            .sum()
    }
    /// 合并所有操作。
    fn combine_actions(
        mut all_actions: Vec<RebalancingAction>,
        total_weight: u64,
    ) -> StrategyResult<Vec<RebalancingAction>> {
        if all_actions.is_empty() {
            return Ok(Vec::new());
        }
        all_actions.sort_by(|a, b| b.priority.cmp(&a.priority));
        Ok(all_actions)
    }
    /// 合并单资产操作。
    fn combine_token_actions(
        actions: Vec<RebalancingAction>,
        total_weight: u64,
    ) -> StrategyResult<RebalancingAction> {
        let mut total_amount = 0u64;
        let mut priority = 0u64;
        let mut action_type = RebalancingActionType::Buy;
        let mut token_index = 0usize;
        for action in actions {
            total_amount += action.amount;
            priority += action.priority;
            action_type = action.action_type;
            token_index = action.token_index;
        }
        Ok(RebalancingAction {
            token_index,
            action_type,
            amount: total_amount,
            priority,
        })
    }
} 