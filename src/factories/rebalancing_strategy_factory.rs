//!
//! rebalancing_strategy_factory.rs - 再平衡策略工厂管理器
//!
//! 本文件实现再平衡策略工厂管理器及相关方法，严格遵循Rust、Solana、Anchor、SOLID最佳实践，
//! 并逐行专业注释，便于审计、维护、扩展。

use anchor_lang::prelude::*;
use crate::core::*;
use crate::error::StrategyError;
use crate::state::*;
use crate::strategies::*;
use crate::utils::price::{RebalanceAction, TokenWeight};
use crate::utils::{MathOps, PriceUtils, ValidationUtils};

/// 再平衡策略工厂管理器
/// - 负责再平衡策略的初始化、创建、执行等
pub struct RebalancingStrategyFactoryManager;

impl RebalancingStrategyFactoryManager {
    /// 初始化再平衡策略工厂
    pub fn initialize(
        factory: &mut RebalancingStrategyFactory,
        authority: Pubkey,
        factory_id: u64,
        bump: u8,
    ) -> StrategyResult<()> {
        factory.initialize(authority, factory_id, authority, bump)?;
        msg!(
            "Rebalancing strategy factory initialized: ID={}, Authority={}",
            factory_id,
            authority
        );
        Ok(())
    }
    /// 创建再平衡策略
    pub fn create_strategy(
        factory: &mut RebalancingStrategyFactory,
        strategy: &mut RebalancingStrategy,
        authority: Pubkey,
        weight_strategy: Pubkey,
        strategy_type: RebalancingStrategyType,
        parameters: Vec<u8>,
        rebalancing_threshold: u64,
        min_rebalance_interval: u64,
        max_slippage: u64,
        bump: u8,
    ) -> StrategyResult<()> {
        crate::factories::factory_utils::FactoryUtils::validate_factory_can_create(factory)?;
        ValidationUtils::validate_parameters_size(
            &parameters,
            RebalancingStrategy::MAX_PARAMETERS_SIZE,
        )?;
        ValidationUtils::validate_pubkey(&weight_strategy, "weight_strategy")?;
        if rebalancing_threshold == 0 || rebalancing_threshold > MAX_REBALANCE_THRESHOLD_BPS {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        ValidationUtils::validate_time_interval(
            min_rebalance_interval,
            MIN_REBALANCE_INTERVAL,
            MAX_REBALANCE_INTERVAL,
        )?;
        ValidationUtils::validate_slippage(max_slippage)?;
        strategy.initialize(
            authority,
            factory.base.authority,
            weight_strategy,
            strategy_type.clone(),
            parameters,
            rebalancing_threshold,
            min_rebalance_interval,
            max_slippage,
            bump,
        )?;
        let strategy_id = factory.create_strategy_id();
        factory.execution_stats.total_executions += 1;
        msg!(
            "Rebalancing strategy created: ID={}, Type={:?}, Threshold={}bp",
            strategy_id,
            strategy_type,
            rebalancing_threshold
        );
        Ok(())
    }
    /// 执行再平衡
    pub fn execute_rebalancing(
        rebalancing_strategy: &mut RebalancingStrategy,
        weight_strategy: &WeightStrategy,
        tokens: &[TokenWeight],
        total_portfolio_value: u64,
    ) -> StrategyResult<Vec<RebalanceAction>> {
        if !rebalancing_strategy.can_rebalance()? {
            return Err(StrategyError::RebalancingThresholdNotMet.into());
        }
        let current_weights: Vec<u64> = tokens.iter().map(|t| t.current_weight).collect();
        let target_weights: Vec<u64> = tokens.iter().map(|t| t.target_weight).collect();
        if !rebalancing_strategy.needs_rebalancing(&current_weights, &target_weights) {
            return Err(StrategyError::RebalancingThresholdNotMet.into());
        }
        let actions = Self::calculate_rebalancing_actions(
            rebalancing_strategy,
            tokens,
            total_portfolio_value,
        )?;
        rebalancing_strategy.update_rebalancing()?;
        Ok(actions)
    }
    /// 计算再平衡动作
    fn calculate_rebalancing_actions(
        strategy: &RebalancingStrategy,
        tokens: &[TokenWeight],
        total_portfolio_value: u64,
    ) -> StrategyResult<Vec<RebalanceAction>> {
        let mut actions = Vec::new();
        for token in tokens {
            let current_value = MathOps::mul(token.balance, token.price)?;
            let current_weight_actual = if total_portfolio_value > 0 {
                MathOps::div(
                    MathOps::mul(current_value, BASIS_POINTS_MAX)?,
                    total_portfolio_value,
                )?
            } else {
                0
            };
            let weight_diff = if current_weight_actual > token.target_weight {
                current_weight_actual - token.target_weight
            } else {
                token.target_weight - current_weight_actual
            };
            if weight_diff >= strategy.rebalancing_threshold {
                let target_value = MathOps::div(
                    MathOps::mul(total_portfolio_value, token.target_weight)?,
                    BASIS_POINTS_MAX,
                )?;
                let value_diff = if current_value > target_value {
                    current_value - target_value
                } else {
                    target_value - current_value
                };
                let action_type = if current_value > target_value { 1 } else { 0 };
                let amount = MathOps::div(value_diff, token.price)?;
                let price_impact = PriceUtils::calculate_price_impact(
                    amount,
                    token.balance * 10,
                    token.price,
                )?;
                if price_impact <= strategy.max_slippage {
                    actions.push(RebalanceAction {
                        token_mint: token.mint,
                        action_type,
                        amount,
                        price_impact,
                    });
                }
            }
        }
        Ok(actions)
    }
} 