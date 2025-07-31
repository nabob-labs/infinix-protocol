//!
//! rebalance_utils.rs - 再平衡工具集
//!
//! 本文件实现再平衡工具集及相关方法，严格遵循Rust、Solana、Anchor、SOLID最佳实践，
//! 并逐行专业注释，便于审计、维护、扩展。

use crate::core::*;
// use crate::errors::strategy_error::StrategyError; // 暂时注释掉
use crate::utils::price::{RebalanceAction, TokenWeight};
use crate::utils::{MathOps, PriceUtils};

/// 再平衡工具集
/// - 提供再平衡动作校验、成本计算、执行顺序优化等方法
pub struct RebalanceUtils;

impl RebalanceUtils {
    /// 校验再平衡动作
    pub fn validate_actions(
        actions: &[RebalanceAction],
        tokens: &[TokenWeight],
        max_slippage: u64,
    ) -> StrategyResult<()> {
        for action in actions {
            if !tokens.iter().any(|t| t.mint == action.token_mint) {
                return Err(StrategyError::InvalidStrategyParameters.into());
            }
            if action.price_impact > max_slippage {
                return Err(StrategyError::SlippageExceeded.into());
            }
            if action.amount == 0 {
                return Err(StrategyError::InvalidStrategyParameters.into());
            }
        }
        Ok(())
    }
    /// 计算再平衡总成本
    pub fn calculate_total_cost(actions: &[RebalanceAction]) -> StrategyResult<u64> {
        let mut total_cost = 0u64;
        for action in actions {
            let action_cost = MathOps::mul(action.amount, action.price_impact)?;
            total_cost = MathOps::add(total_cost, action_cost)?;
        }
        Ok(total_cost)
    }
    /// 优化再平衡执行顺序
    pub fn optimize_execution_order(actions: &mut [RebalanceAction]) {
        actions.sort_by(|a, b| a.price_impact.cmp(&b.price_impact));
    }
    /// 计算再平衡效率得分
    pub fn calculate_efficiency_score(
        actions: &[RebalanceAction],
        total_cost: u64,
        portfolio_value: u64,
    ) -> u32 {
        if portfolio_value == 0 {
            return 0;
        }
        let cost_ratio = (total_cost * BASIS_POINTS_MAX) / portfolio_value;
        let action_efficiency = if actions.is_empty() {
            0
        } else {
            let avg_impact = actions.iter().map(|a| a.price_impact).sum::<u64>() / actions.len() as u64;
            BASIS_POINTS_MAX - std::cmp::min(avg_impact, BASIS_POINTS_MAX)
        };
        let cost_efficiency = BASIS_POINTS_MAX - std::cmp::min(cost_ratio, BASIS_POINTS_MAX);
        ((cost_efficiency + action_efficiency) / 2) as u32
    }
} 