/*!
 * Index Token State Structures
 * 
 * State definitions for index token management and operations.
 */

use crate::core::*;
use crate::state::common::*;
use crate::state::common::Validatable;
use crate::basket::BasketComposition;
use anchor_lang::prelude::*;
use borsh::{BorshDeserialize, BorshSerialize};

/// Index token manager state
/// Updated for Anchor 0.32 with InitSpace trait implementation
#[account]
#[derive(Debug, InitSpace)]
pub struct IndexTokenManager {
    /// Manager authority
    pub authority: Pubkey,
    /// Number of index tokens created
    pub token_count: u64,
    /// Whether manager is active
    pub is_active: bool,
    /// Fee collector address
    pub fee_collector: Pubkey,
    /// Creation fee in basis points
    pub creation_fee_bps: u16,
    /// Redemption fee in basis points
    pub redemption_fee_bps: u16,
    /// Total value locked across all tokens
    pub total_value_locked: u64,
    /// Creation timestamp
    pub created_at: i64,
    /// Last update timestamp
    pub updated_at: i64,
    /// Execution statistics
    pub execution_stats: ExecutionStats,
    /// PDA bump
    pub bump: u8,
}

impl IndexTokenManager {
    // Legacy LEN for backward compatibility
    pub const LEN: usize = 8 + Self::INIT_SPACE;
    
    pub fn initialize(
        &mut self,
        authority: Pubkey,
        fee_collector: Pubkey,
        creation_fee_bps: u16,
        redemption_fee_bps: u16,
        bump: u8,
    ) -> StrategyResult<()> {
        let now = Clock::get()?.unix_timestamp;
        
        self.authority = authority;
        self.token_count = 0;
        self.is_active = true;
        self.fee_collector = fee_collector;
        self.creation_fee_bps = creation_fee_bps;
        self.redemption_fee_bps = redemption_fee_bps;
        self.total_value_locked = 0;
        self.created_at = now;
        self.updated_at = now;
        self.execution_stats = ExecutionStats::default();
        self.bump = bump;
        
        Ok(())
    }
    
    pub fn create_token_id(&mut self) -> u64 {
        let id = self.token_count;
        self.token_count += 1;
        self.updated_at = Clock::get().map(|c| c.unix_timestamp).unwrap_or(0);
        id
    }
    
    pub fn update_tvl(&mut self, amount: u64, is_increase: bool) -> StrategyResult<()> {
        if is_increase {
            self.total_value_locked = safe_math!(self.total_value_locked + amount)?;
        } else {
            self.total_value_locked = safe_math!(self.total_value_locked - amount)?;
        }
        self.updated_at = Clock::get().map(|c| c.unix_timestamp).unwrap_or(0);
        Ok(())
    }
}

impl Validatable for IndexTokenManager {
    fn validate(&self) -> StrategyResult<()> {
        if self.authority == Pubkey::default() {
            return Err(crate::error::StrategyError::InvalidStrategyParameters.into());
        }
        
        if self.fee_collector == Pubkey::default() {
            return Err(crate::error::StrategyError::InvalidStrategyParameters.into());
        }
        
        if self.creation_fee_bps > MAX_FEE_BPS {
            return Err(crate::error::StrategyError::InvalidStrategyParameters.into());
        }
        
        if self.redemption_fee_bps > MAX_FEE_BPS {
            return Err(crate::error::StrategyError::InvalidStrategyParameters.into());
        }
        
        Ok(())
    }
}

/// Individual index token state
/// Updated for Anchor 0.32 with InitSpace trait implementation
#[account]
#[derive(Debug, InitSpace)]
pub struct IndexToken {
    /// Token authority
    pub authority: Pubkey,
    /// Parent manager
    pub manager: Pubkey,
    /// Unique token identifier
    pub token_id: u64,
    /// Token mint address
    pub token_mint: Pubkey,
    /// Basket composition
    pub composition: BasketComposition,
    /// Associated weight strategy (optional)
    pub weight_strategy: Option<Pubkey>,
    /// Associated rebalancing strategy (optional)
    pub rebalancing_strategy: Option<Pubkey>,
    /// Total supply of index tokens
    pub total_supply: u64,
    /// Net Asset Value per token
    pub nav_per_token: u64,
    /// Total fees collected
    pub fees_collected: u64,
    /// Number of operations performed
    pub operation_count: u64,
    /// Whether token is active
    pub is_active: bool,
    /// Whether token is paused
    pub is_paused: bool,
    /// Last rebalancing timestamp
    pub last_rebalanced: i64,
    /// Creation timestamp
    pub created_at: i64,
    /// Last update timestamp
    pub updated_at: i64,
    /// Execution statistics
    pub execution_stats: ExecutionStats,
    /// Risk metrics
    pub risk_metrics: RiskMetrics,
    /// PDA bump
    pub bump: u8,
}

impl IndexToken {
    // Legacy LEN for backward compatibility
    pub const LEN: usize = 8 + Self::INIT_SPACE;
    
    pub fn initialize(
        &mut self,
        authority: Pubkey,
        manager: Pubkey,
        token_id: u64,
        token_mint: Pubkey,
        composition: BasketComposition,
        weight_strategy: Option<Pubkey>,
        rebalancing_strategy: Option<Pubkey>,
        bump: u8,
    ) -> StrategyResult<()> {
        let now = Clock::get()?.unix_timestamp;
        
        self.authority = authority;
        self.manager = manager;
        self.token_id = token_id;
        self.token_mint = token_mint;
        self.composition = composition;
        self.weight_strategy = weight_strategy;
        self.rebalancing_strategy = rebalancing_strategy;
        self.total_supply = 0;
        self.nav_per_token = PRICE_PRECISION; // Start at 1.0
        self.fees_collected = 0;
        self.operation_count = 0;
        self.is_active = true;
        self.is_paused = false;
        self.last_rebalanced = 0;
        self.created_at = now;
        self.updated_at = now;
        self.execution_stats = ExecutionStats::default();
        self.risk_metrics = RiskMetrics::default();
        self.bump = bump;
        
        Ok(())
    }
    
    pub fn calculate_nav(&self, price_feeds: &[PriceFeed]) -> StrategyResult<u64> {
        let mut total_value = 0u64;
        
        for constituent in &self.composition.constituents {
            if let Some(price_feed) = price_feeds.iter().find(|pf| pf.mint == constituent.mint) {
                price_feed.validate()?;
                let constituent_value = safe_math!(constituent.balance * price_feed.price / PRICE_PRECISION)?;
                total_value = safe_math!(total_value + constituent_value)?;
            } else {
                return Err(crate::error::StrategyError::PriceFeedUnavailable.into());
            }
        }
        
        if self.total_supply == 0 {
            Ok(PRICE_PRECISION)
        } else {
            safe_math!(total_value * PRICE_PRECISION / self.total_supply)
        }
    }
    
    pub fn update_nav(&mut self, price_feeds: &[PriceFeed]) -> StrategyResult<()> {
        self.nav_per_token = self.calculate_nav(price_feeds)?;
        self.updated_at = Clock::get().map(|c| c.unix_timestamp).unwrap_or(0);
        Ok(())
    }
    
    pub fn mint_tokens(&mut self, amount: u64) -> StrategyResult<()> {
        self.total_supply = safe_math!(self.total_supply + amount)?;
        self.operation_count += 1;
        self.updated_at = Clock::get().map(|c| c.unix_timestamp).unwrap_or(0);
        Ok(())
    }
    
    pub fn burn_tokens(&mut self, amount: u64) -> StrategyResult<()> {
        if amount > self.total_supply {
            return Err(crate::error::StrategyError::BasketRedemptionExceedsSupply.into());
        }
        
        self.total_supply = safe_math!(self.total_supply - amount)?;
        self.operation_count += 1;
        self.updated_at = Clock::get().map(|c| c.unix_timestamp).unwrap_or(0);
        Ok(())
    }
    
    pub fn collect_fees(&mut self, amount: u64) -> StrategyResult<()> {
        self.fees_collected = safe_math!(self.fees_collected + amount)?;
        self.updated_at = Clock::get().map(|c| c.unix_timestamp).unwrap_or(0);
        Ok(())
    }
    
    pub fn validate_can_operate(&self) -> StrategyResult<()> {
        if !self.is_active {
            return Err(crate::error::StrategyError::StrategyPaused.into());
        }
        
        if self.is_paused {
            return Err(crate::error::StrategyError::StrategyPaused.into());
        }
        
        Ok(())
    }
}

impl Validatable for IndexToken {
    fn validate(&self) -> StrategyResult<()> {
        if self.authority == Pubkey::default() {
            return Err(crate::error::StrategyError::InvalidStrategyParameters.into());
        }
        
        if self.manager == Pubkey::default() {
            return Err(crate::error::StrategyError::InvalidStrategyParameters.into());
        }
        
        if self.token_mint == Pubkey::default() {
            return Err(crate::error::StrategyError::InvalidStrategyParameters.into());
        }
        
        if self.composition.constituents.is_empty() {
            return Err(crate::error::StrategyError::InvalidTokenCount.into());
        }
        
        // Validate composition weights sum to 100%
        let total_weight: u64 = self.composition.constituents.iter().map(|c| c.weight).sum();
        if total_weight != BASIS_POINTS_MAX {
            return Err(crate::error::StrategyError::InvalidWeightSum.into());
        }
        
        Ok(())
    }
}