/*!
 * Factory State Management Module
 *
 * This module manages the on-chain state for strategy factories,
 * including weight strategy factories and rebalancing strategy factories.
 */

use crate::core::traits::{Activatable, Pausable, Validatable};
use crate::core::types::{FeeConfig, RiskLimits};
use crate::error::StrategyError;
use crate::version::{ProgramVersion, Versioned};
use anchor_lang::prelude::*;
// Removed conflicting borsh import

/// Weight strategy factory account
#[account]
pub struct WeightStrategyFactory {
    /// Base account fields
    pub base: BaseAccount,

    /// Factory identifier
    pub factory_id: u64,

    /// Number of strategies created
    pub strategy_count: u64,

    /// Maximum strategies allowed
    pub max_strategies: u64,

    /// Fee collector for factory operations
    pub fee_collector: Pubkey,

    /// Factory fee in basis points
    pub factory_fee_bps: u16,

    /// Execution statistics
    pub execution_stats: ExecutionStats,
}

impl WeightStrategyFactory {
    pub const INIT_SPACE: usize = 8 + // discriminator
        std::mem::size_of::<BaseAccount>() +
        8 + // factory_id
        8 + // strategy_count
        8 + // max_strategies
        32 + // fee_collector
        2 + // factory_fee_bps
        std::mem::size_of::<ExecutionStats>();

    /// Initialize a new weight strategy factory
    pub fn initialize(&mut self, factory_id: u64, authority: Pubkey) -> Result<()> {
        self.factory_id = factory_id;
        self.authority = authority;
        self.base.initialize()?;
        self.factory_fee_bps = DEFAULT_FEE_BPS as u16;
        self.base.validate()?;
        Ok(())
    }

    /// Validate factory configuration
    pub fn validate_config(&self) -> Result<()> {
        self.base.validate()?;
        if self.factory_fee_bps > MAX_FEE_BPS as u16 {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(())
    }

    /// Pause the factory
    pub fn pause_factory(&mut self) -> Result<()> {
        self.base.pause()
    }

    /// Unpause the factory
    pub fn unpause_factory(&mut self) -> Result<()> {
        self.base.unpause()
    }

    /// Activate the factory
    pub fn activate_factory(&mut self) -> Result<()> {
        self.base.activate()
    }

    /// Deactivate the factory
    pub fn deactivate_factory(&mut self) -> Result<()> {
        self.base.deactivate()
    }

    /// Create a new strategy ID
    pub fn create_strategy_id(&mut self) -> u64 {
        let id = self.strategy_count;
        self.strategy_count += 1;
        id
    }

    /// Check if factory can create more strategies
    pub fn can_create_strategy(&self) -> bool {
        self.strategy_count < self.max_strategies && self.base.is_active && !self.base.is_paused
    }
}

impl Validatable for WeightStrategyFactory {
    fn validate(&self) -> Result<()> {
        self.base.validate()?;

        if self.factory_id == 0 {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        if self.fee_collector == Pubkey::default() {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        if self.factory_fee_bps > MAX_FEE_BPS {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        Ok(())
    }
}

impl Authorizable for WeightStrategyFactory {
    fn authority(&self) -> Pubkey {
        self.base.authority
    }

    fn transfer_authority(&mut self, new_authority: Pubkey) -> StrategyResult<()> {
        self.base.authority = new_authority;
        self.base.touch()?;
        Ok(())
    }
}

impl Pausable for WeightStrategyFactory {
    fn is_paused(&self) -> bool {
        self.base.is_paused
    }

    fn pause(&mut self) -> Result<()> {
        self.base.pause()
    }

    fn unpause(&mut self) -> Result<()> {
        self.base.unpause()
    }

    fn resume(&mut self) -> StrategyResult<()> {
        self.unpause()
    }
}

impl Activatable for WeightStrategyFactory {
    fn is_active(&self) -> bool {
        self.base.is_active
    }

    fn activate(&mut self) -> Result<()> {
        self.base.activate()
    }

    fn deactivate(&mut self) -> Result<()> {
        self.base.deactivate()
    }
}

impl Versioned for WeightStrategyFactory {
    fn version(&self) -> ProgramVersion {
        self.base.version
    }

    fn set_version(&mut self, version: ProgramVersion) {
        self.base.set_version(version);
    }
}

/// Rebalancing strategy factory account
#[account]
pub struct RebalancingStrategyFactory {
    /// Base account fields
    pub base: BaseAccount,

    /// Factory identifier
    pub factory_id: u64,

    /// Number of strategies created
    pub strategy_count: u64,

    /// Maximum strategies allowed
    pub max_strategies: u64,

    /// Fee collector for factory operations
    pub fee_collector: Pubkey,

    /// Factory fee in basis points
    pub factory_fee_bps: u16,

    /// Execution statistics
    pub execution_stats: ExecutionStats,
}

impl RebalancingStrategyFactory {
    pub const INIT_SPACE: usize = 8 + // discriminator
        std::mem::size_of::<BaseAccount>() +
        8 + // factory_id
        8 + // strategy_count
        8 + // max_strategies
        32 + // fee_collector
        2 + // factory_fee_bps
        std::mem::size_of::<ExecutionStats>();

    /// Initialize the factory
    pub fn initialize(
        &mut self,
        authority: Pubkey,
        factory_id: u64,
        fee_collector: Pubkey,
        bump: u8,
    ) -> Result<()> {
        self.base = BaseAccount::new(authority, bump)?;
        self.factory_id = factory_id;
        self.strategy_count = 0;
        self.max_strategies = MAX_STRATEGIES_PER_FACTORY;
        self.fee_collector = fee_collector;
        self.factory_fee_bps = DEFAULT_FEE_BPS;
        self.execution_stats = ExecutionStats::default();

        Ok(())
    }

    /// Create a new strategy ID
    pub fn create_strategy_id(&mut self) -> u64 {
        let id = self.strategy_count;
        self.strategy_count += 1;
        id
    }

    /// Check if factory can create more strategies
    pub fn can_create_strategy(&self) -> bool {
        self.strategy_count < self.max_strategies && self.base.is_active && !self.base.is_paused
    }
}

impl Validatable for RebalancingStrategyFactory {
    fn validate(&self) -> Result<()> {
        self.base.validate()?;

        if self.factory_id == 0 {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        if self.fee_collector == Pubkey::default() {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        if self.factory_fee_bps > MAX_FEE_BPS {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        Ok(())
    }
}

impl Authorizable for RebalancingStrategyFactory {
    fn authority(&self) -> Pubkey {
        self.base.authority
    }

    fn transfer_authority(&mut self, new_authority: Pubkey) -> StrategyResult<()> {
        self.base.authority = new_authority;
        self.base.touch()?;
        Ok(())
    }
}

impl Pausable for RebalancingStrategyFactory {
    fn is_paused(&self) -> bool {
        self.base.is_paused
    }

    fn pause(&mut self) -> Result<()> {
        self.base.pause()
    }

    fn unpause(&mut self) -> Result<()> {
        self.base.unpause()
    }

    fn resume(&mut self) -> StrategyResult<()> {
        self.unpause()
    }
}

impl Activatable for RebalancingStrategyFactory {
    fn is_active(&self) -> bool {
        self.base.is_active
    }

    fn activate(&mut self) -> Result<()> {
        self.base.activate()
    }

    fn deactivate(&mut self) -> Result<()> {
        self.base.deactivate()
    }
}

impl Versioned for RebalancingStrategyFactory {
    fn version(&self) -> ProgramVersion {
        self.base.version
    }

    fn set_version(&mut self, version: ProgramVersion) {
        self.base.set_version(version);
    }
}
