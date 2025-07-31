//!
//! factory_utils.rs - 工厂工具集
//!
//! 本文件实现工厂工具集及相关方法，严格遵循Rust、Solana、Anchor、SOLID最佳实践，
//! 并逐行专业注释，便于审计、维护、扩展。

use crate::core::traits::*;
use crate::core::StrategyResult;
use crate::state::strategies::{WeightStrategy, RebalancingStrategy};
// use crate::errors::strategy_error::StrategyError; // 暂时注释掉

/// 工厂工具集
/// - 提供策略工厂通用校验、兼容性检查等方法
pub struct FactoryUtils;

impl FactoryUtils {
    /// 校验工厂是否可创建策略
    pub fn validate_factory_can_create<T: Validatable>(factory: &T) -> StrategyResult<()> {
        factory.validate()
    }
    /// 校验策略兼容性
    pub fn validate_strategy_compatibility(
        weight_strategy: &WeightStrategy,
        rebalancing_strategy: &RebalancingStrategy,
    ) -> StrategyResult<()> {
        if weight_strategy.base.authority != rebalancing_strategy.base.authority {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        if !weight_strategy.is_active() || !rebalancing_strategy.is_active() {
            return Err(StrategyError::StrategyPaused.into());
        }
        Ok(())
    }
} 