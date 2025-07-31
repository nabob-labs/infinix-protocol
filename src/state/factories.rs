// ========================= 策略工厂状态管理 =========================
// 本模块管理链上策略工厂状态，包括权重策略工厂、再平衡策略工厂
// 每个 struct、trait、方法、参数、用途、边界、Anchor 相关点均有详细注释

use anchor_lang::prelude::*;
use crate::state::common::*;
use crate::core::traits::*;
use anchor_lang::prelude::ProgramError;
use crate::version::{ProgramVersion, Versioned};

/// 权重策略工厂账户
#[account]
#[derive(Debug, InitSpace, PartialEq, Eq)]
pub struct WeightStrategyFactory {
    /// 通用账户基础信息
    pub base: BaseAccount,
    /// 工厂唯一ID
    pub factory_id: u64,
    /// 已创建策略数量
    pub strategy_count: u64,
    /// 最大可创建策略数量
    pub max_strategies: u64,
    /// 工厂操作费用收集账户
    pub fee_collector: Pubkey,
    /// 工厂费用（bps）
    pub factory_fee_bps: u16,
    /// 执行统计
    pub execution_stats: ExecutionStats,
}

impl WeightStrategyFactory {
    /// 初始化工厂
    pub fn initialize(&mut self, authority: Pubkey, factory_id: u64, fee_collector: Pubkey, max_strategies: u64, factory_fee_bps: u16, bump: u8) -> anchor_lang::Result<()> {
        self.base = BaseAccount::new(authority, bump)?;
        self.factory_id = factory_id;
        self.strategy_count = 0;
        self.max_strategies = max_strategies;
        self.fee_collector = fee_collector;
        self.factory_fee_bps = factory_fee_bps;
        self.execution_stats = ExecutionStats::default();
        Ok(())
    }
    /// 创建新策略ID
    pub fn create_strategy_id(&mut self) -> u64 {
        let id = self.strategy_count;
        self.strategy_count += 1;
        id
    }
    /// 是否可创建新策略
    pub fn can_create_strategy(&self) -> bool {
        self.strategy_count < self.max_strategies && self.base.is_active && !self.base.is_paused
    }
}

/// 工厂参数校验
impl Validatable for WeightStrategyFactory {
    fn validate(&self) -> anchor_lang::Result<()> {
        self.base.validate()?;
        if self.factory_id == 0 {
            return Err(ProgramError::InvalidStrategyParameters.into());
        }
        if self.fee_collector == Pubkey::default() {
            return Err(ProgramError::InvalidStrategyParameters.into());
        }
        if self.factory_fee_bps > 10_000 {
            return Err(ProgramError::InvalidStrategyParameters.into());
        }
        Ok(())
    }
}

/// 权限 trait 实现
impl Authorizable for WeightStrategyFactory {
    fn authority(&self) -> Pubkey { self.base.authority }
    fn transfer_authority(&mut self, new_authority: Pubkey) -> anchor_lang::Result<()> {
        self.base.authority = new_authority;
        self.base.touch()?;
        Ok(())
    }
}

/// 暂停 trait 实现
impl Pausable for WeightStrategyFactory {
    fn is_paused(&self) -> bool { self.base.is_paused }
    fn pause(&mut self) -> anchor_lang::Result<()> { self.base.pause() }
    fn unpause(&mut self) -> anchor_lang::Result<()> { self.base.unpause() }
    fn resume(&mut self) -> anchor_lang::Result<()> { self.unpause() }
}

/// 激活 trait 实现
impl Activatable for WeightStrategyFactory {
    fn is_active(&self) -> bool { self.base.is_active }
    fn activate(&mut self) -> anchor_lang::Result<()> { self.base.activate() }
    fn deactivate(&mut self) -> anchor_lang::Result<()> { self.base.deactivate() }
}

/// 版本 trait 实现
impl Versioned for WeightStrategyFactory {
    fn version(&self) -> ProgramVersion { self.base.version }
    fn set_version(&mut self, version: ProgramVersion) { self.base.set_version(version); }
}

// ========================= 再平衡策略工厂 =========================

/// 再平衡策略工厂账户
#[account]
#[derive(Debug, InitSpace, PartialEq, Eq)]
pub struct RebalancingStrategyFactory {
    /// 通用账户基础信息
    pub base: BaseAccount,
    /// 工厂唯一ID
    pub factory_id: u64,
    /// 已创建策略数量
    pub strategy_count: u64,
    /// 最大可创建策略数量
    pub max_strategies: u64,
    /// 工厂操作费用收集账户
    pub fee_collector: Pubkey,
    /// 工厂费用（bps）
    pub factory_fee_bps: u16,
    /// 执行统计
    pub execution_stats: ExecutionStats,
}

impl RebalancingStrategyFactory {
    /// 初始化工厂
    pub fn initialize(&mut self, authority: Pubkey, factory_id: u64, fee_collector: Pubkey, max_strategies: u64, factory_fee_bps: u16, bump: u8) -> anchor_lang::Result<()> {
        self.base = BaseAccount::new(authority, bump)?;
        self.factory_id = factory_id;
        self.strategy_count = 0;
        self.max_strategies = max_strategies;
        self.fee_collector = fee_collector;
        self.factory_fee_bps = factory_fee_bps;
        self.execution_stats = ExecutionStats::default();
        Ok(())
    }
    /// 创建新策略ID
    pub fn create_strategy_id(&mut self) -> u64 {
        let id = self.strategy_count;
        self.strategy_count += 1;
        id
    }
    /// 是否可创建新策略
    pub fn can_create_strategy(&self) -> bool {
        self.strategy_count < self.max_strategies && self.base.is_active && !self.base.is_paused
    }
}

/// 工厂参数校验
impl Validatable for RebalancingStrategyFactory {
    fn validate(&self) -> anchor_lang::Result<()> {
        self.base.validate()?;
        if self.factory_id == 0 {
            return Err(ProgramError::InvalidStrategyParameters.into());
        }
        if self.fee_collector == Pubkey::default() {
            return Err(ProgramError::InvalidStrategyParameters.into());
        }
        if self.factory_fee_bps > 10_000 {
            return Err(ProgramError::InvalidStrategyParameters.into());
        }
        Ok(())
    }
}

/// 权限 trait 实现
impl Authorizable for RebalancingStrategyFactory {
    fn authority(&self) -> Pubkey { self.base.authority }
    fn transfer_authority(&mut self, new_authority: Pubkey) -> anchor_lang::Result<()> {
        self.base.authority = new_authority;
        self.base.touch()?;
        Ok(())
    }
}

/// 暂停 trait 实现
impl Pausable for RebalancingStrategyFactory {
    fn is_paused(&self) -> bool { self.base.is_paused }
    fn pause(&mut self) -> anchor_lang::Result<()> { self.base.pause() }
    fn unpause(&mut self) -> anchor_lang::Result<()> { self.base.unpause() }
    fn resume(&mut self) -> anchor_lang::Result<()> { self.unpause() }
}

/// 激活 trait 实现
impl Activatable for RebalancingStrategyFactory {
    fn is_active(&self) -> bool { self.base.is_active }
    fn activate(&mut self) -> anchor_lang::Result<()> { self.base.activate() }
    fn deactivate(&mut self) -> anchor_lang::Result<()> { self.base.deactivate() }
}

/// 版本 trait 实现
impl Versioned for RebalancingStrategyFactory {
    fn version(&self) -> ProgramVersion { self.base.version }
    fn set_version(&mut self, version: ProgramVersion) { self.base.set_version(version); }
}

// ========================= 单元测试 =========================
#[cfg(test)]
mod tests {
    use super::*;
    use anchor_lang::prelude::Pubkey;

    #[test]
    fn test_weight_factory_validate() {
        let mut factory = WeightStrategyFactory {
            base: BaseAccount::new(Pubkey::new_unique(), 1).unwrap(),
            factory_id: 1,
            strategy_count: 0,
            max_strategies: 10,
            fee_collector: Pubkey::new_unique(),
            factory_fee_bps: 100,
            execution_stats: ExecutionStats::default(),
        };
        assert!(factory.validate().is_ok());
        factory.factory_fee_bps = 20_000;
        assert!(factory.validate().is_err());
    }

    #[test]
    fn test_rebalancing_factory_validate() {
        let mut factory = RebalancingStrategyFactory {
            base: BaseAccount::new(Pubkey::new_unique(), 1).unwrap(),
            factory_id: 1,
            strategy_count: 0,
            max_strategies: 10,
            fee_collector: Pubkey::new_unique(),
            factory_fee_bps: 100,
            execution_stats: ExecutionStats::default(),
        };
        assert!(factory.validate().is_ok());
        factory.factory_id = 0;
        assert!(factory.validate().is_err());
    }
}
