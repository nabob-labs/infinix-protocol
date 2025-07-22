/*!
 * Strategy State Structures
 *
 * State definitions for weight and rebalancing strategies.
 */

use crate::core::traits::{Activatable, Pausable, Validatable};
use crate::core::*;
use crate::error::StrategyError;
use crate::strategies::{RebalancingStrategyType, WeightStrategyType};
use crate::version::{ProgramVersion, Versioned};
use anchor_lang::prelude::*;
// Removed conflicting borsh import

/// Weight strategy account
#[account]
#[derive(InitSpace)]
pub struct WeightStrategy {
    /// Base account fields
    pub base: BaseAccount,

    /// Factory that created this strategy
    pub factory: Pubkey,

    /// Strategy type
    pub strategy_type: WeightStrategyType,

    /// Strategy parameters (serialized)
    #[max_len(MAX_STRATEGY_PARAMETERS_SIZE)]
    pub parameters: Vec<u8>,

    /// Token mints included in this strategy
    #[max_len(MAX_TOKENS)]
    pub token_mints: Vec<Pubkey>,

    /// Current calculated weights
    #[max_len(MAX_TOKENS)]
    pub current_weights: Vec<u64>,

    /// Last calculation timestamp
    pub last_calculated: i64,

    /// Execution statistics
    pub execution_stats: ExecutionStats,

    /// AI/ML权重建议
    pub ai_weights: Option<Vec<u64>>,

    /// 外部信号
    pub external_signals: Option<Vec<u64>>,
}

impl WeightStrategy {
    pub const MAX_PARAMETERS_SIZE: usize = MAX_STRATEGY_PARAMETERS_SIZE;
    pub const MAX_TOKENS: usize = crate::core::MAX_TOKENS;

    pub const INIT_SPACE: usize = 8 + // discriminator
        std::mem::size_of::<BaseAccount>() +
        32 + // factory
        1 + // strategy_type (enum)
        4 + MAX_STRATEGY_PARAMETERS_SIZE + // parameters vec
        4 + (32 * MAX_TOKENS) + // token_mints vec
        4 + (8 * MAX_TOKENS) + // current_weights vec
        8 + // last_calculated
        std::mem::size_of::<ExecutionStats>();

    /// Initialize the strategy
    pub fn initialize(
        &mut self,
        authority: Pubkey,
        factory: Pubkey,
        strategy_type: WeightStrategyType,
        parameters: Vec<u8>,
        token_mints: Vec<Pubkey>,
        bump: u8,
    ) -> Result<()> {
        self.base = BaseAccount::new(authority, bump)?;
        self.factory = factory;
        self.strategy_type = strategy_type;
        self.parameters = parameters;
        self.token_mints = token_mints.clone();
        self.current_weights = vec![0; token_mints.len()];
        self.last_calculated = 0;
        self.execution_stats = ExecutionStats::default();
        self.ai_weights = None;
        self.external_signals = None;

        Ok(())
    }

    /// Check if strategy can execute
    pub fn validate_can_execute(&self) -> Result<()> {
        if !self.base.is_active {
            return Err(StrategyError::StrategyPaused.into());
        }
        if self.base.is_paused {
            return Err(StrategyError::StrategyPaused.into());
        }
        if self.token_mints.is_empty() {
            return Err(StrategyError::InvalidTokenCount.into());
        }
        Ok(())
    }

    /// Update current weights (多因子聚合)
    pub fn update_weights(
        &mut self,
        new_weights: Vec<u64>,
        ai_weights: Option<Vec<u64>>,
        external_signals: Option<Vec<u64>>,
    ) -> Result<()> {
        if new_weights.len() != self.token_mints.len() {
            return Err(StrategyError::InvalidTokenCount.into());
        }

        let total: u64 = new_weights.iter().sum();
        if total != BASIS_POINTS_MAX {
            return Err(StrategyError::InvalidWeightSum.into());
        }

        // 多因子聚合
        let final_weights = if let Some(ai) = &ai_weights {
            ai.iter()
                .zip(new_weights.iter())
                .map(|(a, n)| ((*a + *n) / 2))
                .collect()
        } else {
            new_weights.clone()
        };
        self.current_weights = final_weights;
        self.last_calculated = Clock::get()?.unix_timestamp;
        self.base.touch()?;
        self.ai_weights = ai_weights;
        self.external_signals = external_signals;

        Ok(())
    }
}

impl Validatable for WeightStrategy {
    fn validate(&self) -> Result<()> {
        self.base.validate()?;

        if self.factory == Pubkey::default() {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        if self.token_mints.is_empty() || self.token_mints.len() > MAX_TOKENS {
            return Err(StrategyError::InvalidTokenCount.into());
        }

        if self.parameters.len() > MAX_STRATEGY_PARAMETERS_SIZE {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        // Validate no duplicate token mints
        let mut seen = std::collections::HashSet::new();
        for mint in &self.token_mints {
            if !seen.insert(*mint) {
                return Err(StrategyError::InvalidStrategyParameters.into());
            }
        }

        Ok(())
    }
}

impl Authorizable for WeightStrategy {
    fn authority(&self) -> Pubkey {
        self.base.authority
    }

    fn transfer_authority(&mut self, new_authority: Pubkey) -> StrategyResult<()> {
        self.base.authority = new_authority;
        self.base.touch()?;
        Ok(())
    }
}

impl Pausable for WeightStrategy {
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

impl Activatable for WeightStrategy {
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

impl Versioned for WeightStrategy {
    fn version(&self) -> ProgramVersion {
        self.base.version
    }

    fn set_version(&mut self, version: ProgramVersion) {
        self.base.set_version(version);
    }
}

/// Rebalancing strategy account
#[account]
#[derive(InitSpace)]
pub struct RebalancingStrategy {
    /// Base account fields
    pub base: BaseAccount,

    /// Factory that created this strategy
    pub factory: Pubkey,

    /// Associated weight strategy
    pub weight_strategy: Pubkey,

    /// Strategy type
    pub strategy_type: RebalancingStrategyType,

    /// Strategy parameters (serialized)
    #[max_len(MAX_STRATEGY_PARAMETERS_SIZE)]
    pub parameters: Vec<u8>,

    /// Rebalancing threshold in basis points
    pub rebalancing_threshold: u64,

    /// Minimum interval between rebalances (seconds)
    pub min_rebalance_interval: u64,

    /// Maximum slippage tolerance
    pub max_slippage: u64,

    /// Last rebalance timestamp
    pub last_rebalanced: i64,

    /// Execution statistics
    pub execution_stats: ExecutionStats,

    /// AI/ML信号
    pub ai_signals: Option<Vec<u64>>,

    /// 外部信号
    pub external_signals: Option<Vec<u64>>,
}

impl RebalancingStrategy {
    pub const MAX_PARAMETERS_SIZE: usize = MAX_STRATEGY_PARAMETERS_SIZE;

    pub const INIT_SPACE: usize = 8 + // discriminator
        std::mem::size_of::<BaseAccount>() +
        32 + // factory
        32 + // weight_strategy
        1 + // strategy_type (enum)
        4 + MAX_STRATEGY_PARAMETERS_SIZE + // parameters vec
        8 + // rebalancing_threshold
        8 + // min_rebalance_interval
        8 + // max_slippage
        8 + // last_rebalanced
        std::mem::size_of::<ExecutionStats>();

    /// Initialize the strategy
    pub fn initialize(
        &mut self,
        authority: Pubkey,
        factory: Pubkey,
        weight_strategy: Pubkey,
        strategy_type: RebalancingStrategyType,
        parameters: Vec<u8>,
        rebalancing_threshold: u64,
        min_rebalance_interval: u64,
        max_slippage: u64,
        bump: u8,
    ) -> Result<()> {
        self.base = BaseAccount::new(authority, bump)?;
        self.factory = factory;
        self.weight_strategy = weight_strategy;
        self.strategy_type = strategy_type;
        self.parameters = parameters;
        self.rebalancing_threshold = rebalancing_threshold;
        self.min_rebalance_interval = min_rebalance_interval;
        self.max_slippage = max_slippage;
        self.last_rebalanced = 0;
        self.execution_stats = ExecutionStats::default();
        self.ai_signals = None;
        self.external_signals = None;

        Ok(())
    }

    /// Check if rebalancing is allowed (多因子聚合)
    pub fn can_rebalance(
        &self,
        ai_signals: Option<Vec<u64>>,
        external_signals: Option<Vec<u64>>,
    ) -> Result<bool> {
        if !self.base.is_active || self.base.is_paused {
            return Ok(false);
        }

        let current_time = Clock::get()?.unix_timestamp;
        let time_since_last = current_time - self.last_rebalanced;

        // 多因子聚合
        let ai_factor = ai_signals
            .as_ref()
            .and_then(|v| v.first().cloned())
            .unwrap_or(1);
        let ext_factor = external_signals
            .as_ref()
            .and_then(|v| v.first().cloned())
            .unwrap_or(1);
        let allow = time_since_last >= self.min_rebalance_interval as i64
            && ai_factor > 0
            && ext_factor > 0;

        Ok(allow)
    }

    /// Check if rebalancing is needed based on weight deviation
    pub fn needs_rebalancing(&self, current_weights: &[u64], target_weights: &[u64]) -> bool {
        if current_weights.len() != target_weights.len() {
            return false;
        }

        for (current, target) in current_weights.iter().zip(target_weights.iter()) {
            let deviation = if current > target {
                current - target
            } else {
                target - current
            };

            if deviation >= self.rebalancing_threshold {
                return true;
            }
        }

        false
    }

    /// Update rebalancing timestamp
    pub fn update_rebalancing(&mut self) -> Result<()> {
        self.last_rebalanced = Clock::get()?.unix_timestamp;
        self.base.touch()?;
        Ok(())
    }
}

impl Validatable for RebalancingStrategy {
    fn validate(&self) -> Result<()> {
        self.base.validate()?;

        if self.factory == Pubkey::default() {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        if self.weight_strategy == Pubkey::default() {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        if self.rebalancing_threshold == 0
            || self.rebalancing_threshold > MAX_REBALANCE_THRESHOLD_BPS
        {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        if self.min_rebalance_interval < MIN_REBALANCE_INTERVAL {
            return Err(StrategyError::InvalidTimeWindow.into());
        }

        if self.max_slippage > MAX_SLIPPAGE_BPS {
            return Err(StrategyError::SlippageExceeded.into());
        }

        if self.parameters.len() > MAX_STRATEGY_PARAMETERS_SIZE {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        Ok(())
    }
}

impl Authorizable for RebalancingStrategy {
    fn authority(&self) -> Pubkey {
        self.base.authority
    }

    fn transfer_authority(&mut self, new_authority: Pubkey) -> StrategyResult<()> {
        self.base.authority = new_authority;
        self.base.touch()?;
        Ok(())
    }
}

impl Pausable for RebalancingStrategy {
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

impl Activatable for RebalancingStrategy {
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

impl Versioned for RebalancingStrategy {
    fn version(&self) -> ProgramVersion {
        self.base.version
    }

    fn set_version(&mut self, version: ProgramVersion) {
        self.base.set_version(version);
    }
}
