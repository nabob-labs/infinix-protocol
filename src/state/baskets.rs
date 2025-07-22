/*!
 * Basket State Structures
 *
 * State definitions for basket management and trading.
 */

use crate::core::*;
use crate::error::StrategyError;
use crate::state::common::*;
use crate::version::{ProgramVersion, Versioned};
use anchor_lang::prelude::*;
// Removed conflicting borsh import

/// Basket manager account
#[account]
#[derive(InitSpace)]
pub struct BasketManager {
    /// Base account fields
    pub base: BaseAccount,

    /// Number of baskets created
    pub basket_count: u64,

    /// Total value locked across all baskets
    pub total_value_locked: u64,

    /// Fee collector for basket operations
    pub fee_collector: Pubkey,

    /// Default fee in basis points
    pub default_fee_bps: u16,

    /// Execution statistics
    pub execution_stats: ExecutionStats,
}

impl BasketManager {
    pub const INIT_SPACE: usize = 8 + // discriminator
        std::mem::size_of::<BaseAccount>() +
        8 + // basket_count
        8 + // total_value_locked
        32 + // fee_collector
        2 + // default_fee_bps
        std::mem::size_of::<ExecutionStats>();

    /// Initialize the basket manager
    pub fn initialize(&mut self, authority: Pubkey, fee_collector: Pubkey, bump: u8) -> Result<()> {
        self.base = BaseAccount::new(authority, bump)?;
        self.basket_count = 0;
        self.total_value_locked = 0;
        self.fee_collector = fee_collector;
        self.default_fee_bps = DEFAULT_FEE_BPS;
        self.execution_stats = ExecutionStats::default();

        Ok(())
    }

    /// Create a new basket ID
    pub fn create_basket_id(&mut self) -> u64 {
        let id = self.basket_count;
        self.basket_count += 1;
        id
    }

    /// Update total value locked
    pub fn update_tvl(&mut self, amount: u64, is_addition: bool) -> Result<()> {
        if is_addition {
            self.total_value_locked = self.total_value_locked.saturating_add(amount);
        } else {
            self.total_value_locked = self.total_value_locked.saturating_sub(amount);
        }
        self.base.touch()?;
        Ok(())
    }
}

impl crate::core::traits::Validatable for BasketManager {
    fn validate(&self) -> Result<()> {
        self.base.validate()?;

        if self.fee_collector == Pubkey::default() {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        if self.default_fee_bps > MAX_FEE_BPS {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        Ok(())
    }
}

impl crate::core::traits::Authorizable for BasketManager {
    fn authority(&self) -> Pubkey {
        self.base.authority
    }

    fn transfer_authority(&mut self, new_authority: Pubkey) -> StrategyResult<()> {
        self.base.authority = new_authority;
        self.base.touch()?;
        Ok(())
    }
}

impl crate::core::traits::Pausable for BasketManager {
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

impl crate::core::traits::Activatable for BasketManager {
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

impl crate::core::traits::Versioned for BasketManager {
    fn version(&self) -> u32 {
        self.base.version.as_u32()
    }

    fn set_version(&mut self, version: ProgramVersion) {
        self.base.set_version(version);
    }
}

/// Individual basket instance
#[account]
#[derive(InitSpace)]
pub struct BasketInstance {
    /// Base account fields
    pub base: BaseAccount,

    /// Manager that created this basket
    pub manager: Pubkey,

    /// Basket identifier
    pub basket_id: u64,

    /// Basket token mint
    pub basket_mint: Pubkey,

    /// Basket composition
    #[max_len(MAX_TOKENS)]
    pub composition: Vec<BasketConstituent>,

    /// Total supply of basket tokens
    pub total_supply: u64,

    /// Net Asset Value per token
    pub nav_per_token: u64,

    /// Total fees collected
    pub fees_collected: u64,

    /// Number of operations performed
    pub operation_count: u64,

    /// Last rebalance timestamp
    pub last_rebalanced: i64,

    /// Execution statistics
    pub execution_stats: ExecutionStats,
}

impl BasketInstance {
    pub const INIT_SPACE: usize = 8 + // discriminator
        std::mem::size_of::<BaseAccount>() +
        32 + // manager
        8 + // basket_id
        32 + // basket_mint
        4 + (std::mem::size_of::<BasketConstituent>() * MAX_TOKENS) + // composition vec
        8 + // total_supply
        8 + // nav_per_token
        8 + // fees_collected
        8 + // operation_count
        8 + // last_rebalanced
        std::mem::size_of::<ExecutionStats>();

    /// Initialize the basket
    pub fn initialize(
        &mut self,
        authority: Pubkey,
        manager: Pubkey,
        basket_id: u64,
        basket_mint: Pubkey,
        composition: Vec<BasketConstituent>,
        bump: u8,
    ) -> Result<()> {
        self.base = BaseAccount::new(authority, bump)?;
        self.manager = manager;
        self.basket_id = basket_id;
        self.basket_mint = basket_mint;
        self.composition = composition;
        self.total_supply = 0;
        self.nav_per_token = PRICE_PRECISION; // Start at $1.00
        self.fees_collected = 0;
        self.operation_count = 0;
        self.last_rebalanced = 0;
        self.execution_stats = ExecutionStats::default();

        Ok(())
    }

    /// Update basket supply
    pub fn update_supply(&mut self, amount: u64, is_creation: bool) -> Result<()> {
        if is_creation {
            self.total_supply = self.total_supply.saturating_add(amount);
        } else {
            if amount > self.total_supply {
                return Err(StrategyError::BasketRedemptionExceedsSupply.into());
            }
            self.total_supply = self.total_supply.saturating_sub(amount);
        }

        self.operation_count += 1;
        self.base.touch()?;
        Ok(())
    }

    /// Update NAV per token
    pub fn update_nav(&mut self, new_nav: u64) -> Result<()> {
        self.nav_per_token = new_nav;
        self.base.touch()?;
        Ok(())
    }

    /// Add fees collected
    pub fn add_fees(&mut self, fee_amount: u64) -> Result<()> {
        self.fees_collected = self.fees_collected.saturating_add(fee_amount);
        self.base.touch()?;
        Ok(())
    }

    /// Update rebalance timestamp
    pub fn update_rebalance(&mut self) -> Result<()> {
        self.last_rebalanced = Clock::get()?.unix_timestamp;
        self.base.touch()?;
        Ok(())
    }

    /// Check if basket can be operated on
    pub fn validate_can_operate(&self) -> Result<()> {
        if !self.base.is_active {
            return Err(StrategyError::StrategyPaused.into());
        }
        if self.base.is_paused {
            return Err(StrategyError::StrategyPaused.into());
        }
        Ok(())
    }
}

impl crate::core::traits::Validatable for BasketInstance {
    fn validate(&self) -> Result<()> {
        self.base.validate()?;

        if self.manager == Pubkey::default() {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        if self.basket_mint == Pubkey::default() {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        if self.composition.is_empty() || self.composition.len() > MAX_TOKENS {
            return Err(StrategyError::InvalidTokenCount.into());
        }

        // Validate composition weights sum to 100%
        let total_weight: u64 = self.composition.iter().map(|c| c.weight).sum();
        if total_weight != BASIS_POINTS_MAX {
            return Err(StrategyError::InvalidWeightSum.into());
        }

        Ok(())
    }
}

impl crate::core::traits::Authorizable for BasketInstance {
    fn authority(&self) -> Pubkey {
        self.base.authority
    }

    fn transfer_authority(&mut self, new_authority: Pubkey) -> StrategyResult<()> {
        self.base.authority = new_authority;
        self.base.touch()?;
        Ok(())
    }
}

impl crate::core::traits::Pausable for BasketInstance {
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

impl crate::core::traits::Activatable for BasketInstance {
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

impl crate::core::traits::Versioned for BasketInstance {
    fn version(&self) -> u32 {
        self.base.version.as_u32()
    }

    fn set_version(&mut self, version: ProgramVersion) {
        self.base.set_version(version);
    }
}

/// Basket constituent with enhanced fields
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, InitSpace)]
pub struct BasketConstituent {
    /// Token mint address
    pub token_mint: Pubkey,

    /// Weight in basis points
    pub weight: u64,

    /// Current balance
    pub balance: u64,

    /// Target allocation
    pub target_allocation: u64,

    /// Last update timestamp
    pub last_updated: i64,
}

impl BasketConstituent {
    /// Create new constituent
    pub fn new(
        token_mint: Pubkey,
        weight: u64,
        balance: u64,
        target_allocation: u64,
    ) -> Result<Self> {
        if weight > MAX_TOKEN_WEIGHT_BPS {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        Ok(Self {
            token_mint,
            weight,
            balance,
            target_allocation,
            last_updated: Clock::get()?.unix_timestamp,
        })
    }

    /// Update balance
    pub fn update_balance(&mut self, new_balance: u64) -> Result<()> {
        self.balance = new_balance;
        self.last_updated = Clock::get()?.unix_timestamp;
        Ok(())
    }

    /// Update target allocation
    pub fn update_target(&mut self, new_target: u64) -> Result<()> {
        self.target_allocation = new_target;
        self.last_updated = Clock::get()?.unix_timestamp;
        Ok(())
    }

    /// Calculate deviation from target
    pub fn calculate_deviation(&self) -> u64 {
        if self.target_allocation > self.balance {
            self.target_allocation - self.balance
        } else {
            self.balance - self.target_allocation
        }
    }

    /// Check if rebalancing is needed
    pub fn needs_rebalancing(&self, threshold_bps: u64) -> bool {
        if self.target_allocation == 0 {
            return false;
        }

        let deviation_bps =
            (self.calculate_deviation() * BASIS_POINTS_MAX) / self.target_allocation;
        deviation_bps >= threshold_bps
    }
}

impl crate::core::traits::Validatable for BasketConstituent {
    fn validate(&self) -> Result<()> {
        if self.token_mint == Pubkey::default() {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        if self.weight > MAX_TOKEN_WEIGHT_BPS {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        Ok(())
    }
}
