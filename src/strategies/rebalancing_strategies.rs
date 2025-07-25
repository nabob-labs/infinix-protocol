//!
//! Rebalancing Strategy Implementations
//!
//! 本模块实现多种再平衡算法与执行策略，支持阈值、定时、波动率、漂移、混合等多种再平衡逻辑，确保资产配置动态合规、可追溯、可维护。

// 引入核心模块、错误类型、策略模块和 Anchor 依赖。
use crate::core::*;
use crate::error::StrategyError;
use crate::strategies::*;
use anchor_lang::prelude::*;

/// 再平衡策略执行器结构体，支持多种再平衡算法。
pub struct RebalancingStrategyExecutor;

impl RebalancingStrategyExecutor {
    /// 执行阈值再平衡。
    ///
    /// # 参数
    /// * `current_weights` - 当前权重数组。
    /// * `target_weights` - 目标权重数组。
    /// * `threshold_bps` - 触发阈值（基点）。
    /// * `portfolio_value` - 组合总价值。
    /// # 返回
    /// * 再平衡操作列表或错误。
    pub fn execute_threshold_rebalancing(
        current_weights: &[u64],
        target_weights: &[u64],
        threshold_bps: u64,
        portfolio_value: u64,
    ) -> StrategyResult<Vec<RebalancingAction>> {
        // 校验权重数组长度一致。
        if current_weights.len() != target_weights.len() {
            return Err(StrategyError::InvalidTokenCount.into());
        }
        let mut actions = Vec::new();
        // 遍历每个资产，计算偏离度。
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
            // 偏离超过阈值则生成操作。
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
        // 按优先级降序排序。
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
        // 未到再平衡时间则不操作。
        if time_since_last < rebalance_interval {
            return Ok(Vec::new());
        }
        // 到期则全量再平衡。
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
        // 计算平均波动率。
        let avg_volatility = if volatility_data.is_empty() {
            0
        } else {
            volatility_data.iter().sum::<u64>() / volatility_data.len() as u64
        };
        // 未超阈值不触发。
        if avg_volatility < volatility_threshold {
            return Ok(Vec::new());
        }
        // 执行再平衡并按波动率调整操作规模。
        let mut actions =
            Self::execute_full_rebalancing(current_weights, target_weights, portfolio_value)?;
        let volatility_factor = (avg_volatility * 10000) / volatility_threshold;
        for action in &mut actions {
            action.amount = (action.amount * volatility_factor) / 10000;
        }
        Ok(actions)
    }
    /// 执行漂移触发再平衡。
    pub fn execute_drift_based_rebalancing(
        current_weights: &[u64],
        target_weights: &[u64],
        drift_history: &[WeightDrift],
        drift_threshold: u64,
        portfolio_value: u64,
    ) -> StrategyResult<Vec<RebalancingAction>> {
        // 计算累计漂移。
        let cumulative_drift = Self::calculate_cumulative_drift(drift_history);
        if cumulative_drift < drift_threshold {
            return Ok(Vec::new());
        }
        // 执行再平衡并按漂移调整优先级。
        let mut actions =
            Self::execute_full_rebalancing(current_weights, target_weights, portfolio_value)?;
        for action in &mut actions {
            let drift_factor = Self::calculate_drift_factor(action.token_index, drift_history);
            action.priority = (action.priority * drift_factor) / 10000;
        }
        actions.sort_by(|a, b| b.priority.cmp(&a.priority));
        Ok(actions)
    }
    /// 执行混合再平衡策略。
    pub fn execute_hybrid_rebalancing(
        current_weights: &[u64],
        target_weights: &[u64],
        hybrid_params: &HybridRebalancingParams,
        market_context: &MarketContext,
        portfolio_value: u64,
    ) -> StrategyResult<Vec<RebalancingAction>> {
        let mut all_actions = Vec::new();
        let mut total_weight = 0u64;
        // 阈值分量。
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
        // 定时分量。
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
        // 波动率分量。
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
        // 合并去重操作。
        let combined_actions = Self::combine_actions(all_actions, total_weight)?;
        Ok(combined_actions)
    }
    /// 执行全量再平衡，将所有权重调整至目标。
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
                    priority: deviation, // 以偏离度为优先级
                });
            }
        }
        Ok(actions)
    }
    /// 计算优先级，基于偏离度和阈值。
    fn calculate_priority(deviation: u64, threshold: u64) -> u64 {
        if threshold == 0 {
            return deviation;
        }
        // 偏离度越大优先级越高。
        (deviation * 10000) / threshold
    }
    /// 计算累计漂移。
    fn calculate_cumulative_drift(drift_history: &[WeightDrift]) -> u64 {
        if drift_history.is_empty() {
            return 0;
        }
        drift_history.iter().map(|drift| drift.magnitude).sum()
    }
    /// 计算指定资产的漂移因子。
    fn calculate_drift_factor(token_index: usize, drift_history: &[WeightDrift]) -> u64 {
        let token_drifts: Vec<&WeightDrift> = drift_history
            .iter()
            .filter(|drift| drift.token_index == token_index)
            .collect();
        if token_drifts.is_empty() {
            return 10000; // 中性因子
        }
        let avg_drift = token_drifts
            .iter()
            .map(|drift| drift.magnitude)
            .sum::<u64>()
            / token_drifts.len() as u64;
        // 漂移越大因子越高。
        10000 + (avg_drift / 10).min(5000)
    }
    /// 合并并去重多组操作。
    fn combine_actions(
        mut all_actions: Vec<RebalancingAction>,
        total_weight: u64,
    ) -> StrategyResult<Vec<RebalancingAction>> {
        if total_weight == 0 {
            return Ok(Vec::new());
        }
        // 按资产分组。
        let mut token_actions: std::collections::HashMap<usize, Vec<RebalancingAction>> =
            std::collections::HashMap::new();
        for action in all_actions {
            token_actions
                .entry(action.token_index)
                .or_insert_with(Vec::new)
                .push(action);
        }
        // 合并每个资产的操作。
        let mut combined_actions = Vec::new();
        for (token_index, actions) in token_actions {
            let combined_action = Self::combine_token_actions(actions, total_weight)?;
            if combined_action.amount > 0 {
                combined_actions.push(combined_action);
            }
        }
        // 按优先级排序。
        combined_actions.sort_by(|a, b| b.priority.cmp(&a.priority));
        Ok(combined_actions)
    }
    /// 合并同一资产的多组操作。
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

/// 再平衡操作类型枚举。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RebalancingActionType {
    Buy,  // 买入
    Sell, // 卖出
}

/// 再平衡操作结构体。
#[derive(Debug, Clone)]
pub struct RebalancingAction {
    pub token_index: usize,           // 资产索引
    pub action_type: RebalancingActionType, // 操作类型
    pub amount: u64,                  // 操作金额
    pub priority: u64,                // 优先级
}

/// 权重漂移跟踪结构体。
#[derive(Debug, Clone)]
pub struct WeightDrift {
    pub token_index: usize,           // 资产索引
    pub magnitude: u64,               // 漂移幅度
    pub direction: DriftDirection,    // 漂移方向
    pub timestamp: i64,               // 时间戳
}

/// 漂移方向枚举。
#[derive(Debug, Clone, Copy)]
pub enum DriftDirection {
    Positive, // 权重增加
    Negative, // 权重减少
}

/// 混合再平衡参数结构体。
#[derive(Debug, Clone)]
pub struct HybridRebalancingParams {
    pub enable_threshold: bool,       // 启用阈值分量
    pub enable_time: bool,            // 启用定时分量
    pub enable_volatility: bool,      // 启用波动率分量
    pub threshold_bps: u64,           // 阈值（基点）
    pub time_interval: u64,           // 定时间隔（秒）
    pub volatility_threshold: u64,    // 波动率阈值（基点）
    pub threshold_weight: u64,        // 阈值分量权重
    pub time_weight: u64,             // 定时分量权重
    pub volatility_weight: u64,       // 波动率分量权重
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

/// 再平衡决策市场上下文结构体。
#[derive(Debug, Clone)]
pub struct MarketContext {
    pub last_rebalance: i64,              // 上次再平衡时间
    pub volatility_data: Vec<u64>,        // 波动率数据
    pub market_trend: MarketTrend,        // 市场趋势
    pub liquidity_conditions: LiquidityCondition, // 流动性状况
}

/// 市场趋势枚举。
#[derive(Debug, Clone, Copy)]
pub enum MarketTrend {
    Bullish,   // 多头
    Bearish,   // 空头
    Sideways,  // 震荡
}

/// 流动性状况枚举。
#[derive(Debug, Clone, Copy)]
pub enum LiquidityCondition {
    High,   // 高流动性
    Medium, // 中等流动性
    Low,    // 低流动性
}
