/*!
 * Common State Types and Utilities
 *
 * Shared state structures and traits used across the application.
 */

use crate::core::*;
use crate::error::StrategyError;
use crate::version::{ProgramVersion, Versioned, CURRENT_VERSION};
use anchor_lang::prelude::*;
// Removed conflicting borsh import

/// Base account structure with common fields
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, InitSpace)]
pub struct BaseAccount {
    /// Account version for migration support
    pub version: ProgramVersion,
    /// Account authority
    pub authority: Pubkey,
    /// Whether the account is active
    pub is_active: bool,
    /// Whether the account is paused
    pub is_paused: bool,
    /// Creation timestamp
    pub created_at: i64,
    /// Last update timestamp
    pub updated_at: i64,
    /// PDA bump seed
    pub bump: u8,
}

impl BaseAccount {
    /// Initialize base account
    pub fn new(authority: Pubkey, bump: u8) -> Result<Self> {
        let current_time = Clock::get()?.unix_timestamp;

        Ok(Self {
            version: CURRENT_VERSION,
            authority,
            is_active: true,
            is_paused: false,
            created_at: current_time,
            updated_at: current_time,
            bump,
        })
    }

    /// Update timestamp
    pub fn touch(&mut self) -> Result<()> {
        self.updated_at = Clock::get()?.unix_timestamp;
        Ok(())
    }
}

impl crate::core::traits::Validatable for BaseAccount {
    fn validate(&self) -> Result<()> {
        if self.authority == Pubkey::default() {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(())
    }
}

impl crate::core::traits::Pausable for BaseAccount {
    fn is_paused(&self) -> bool {
        self.is_paused
    }

    fn pause(&mut self) -> Result<()> {
        self.is_paused = true;
        self.touch()
    }

    fn unpause(&mut self) -> Result<()> {
        self.is_paused = false;
        self.touch()
    }

    fn resume(&mut self) -> StrategyResult<()> {
        self.unpause()
    }
}

impl crate::core::traits::Activatable for BaseAccount {
    fn is_active(&self) -> bool {
        self.is_active
    }

    fn activate(&mut self) -> Result<()> {
        self.is_active = true;
        self.touch()
    }

    fn deactivate(&mut self) -> Result<()> {
        self.is_active = false;
        self.touch()
    }
}

impl crate::core::traits::Authorizable for BaseAccount {
    fn authority(&self) -> Pubkey {
        self.authority
    }

    fn transfer_authority(&mut self, new_authority: Pubkey) -> StrategyResult<()> {
        self.authority = new_authority;
        self.touch()?;
        Ok(())
    }
}

impl crate::core::traits::Versioned for BaseAccount {
    fn version(&self) -> u32 {
        self.version.as_u32()
    }

    fn set_version(&mut self, version: ProgramVersion) {
        self.version = version;
        if let Ok(clock) = Clock::get() {
            self.updated_at = clock.unix_timestamp;
        }
    }
}

/// Execution statistics for tracking performance
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, Default, InitSpace)]
pub struct ExecutionStats {
    /// Total number of executions
    pub total_executions: u64,
    /// Total successful executions
    pub successful_executions: u64,
    /// Total failed executions
    pub failed_executions: u64,
    /// Total gas used
    pub total_gas_used: u64,
    /// Average execution time in milliseconds
    pub avg_execution_time_ms: u64,
    /// Last execution timestamp
    pub last_execution: i64,
}

impl ExecutionStats {
    /// Record successful execution
    pub fn record_success(&mut self, gas_used: u64, execution_time_ms: u64) -> Result<()> {
        self.total_executions += 1;
        self.successful_executions += 1;
        self.total_gas_used += gas_used;

        // Update average execution time
        if self.total_executions > 0 {
            self.avg_execution_time_ms =
                ((self.avg_execution_time_ms * (self.total_executions - 1)) + execution_time_ms)
                    / self.total_executions;
        }

        self.last_execution = Clock::get()?.unix_timestamp;
        Ok(())
    }

    /// Record failed execution
    pub fn record_failure(&mut self) -> Result<()> {
        self.total_executions += 1;
        self.failed_executions += 1;
        self.last_execution = Clock::get()?.unix_timestamp;
        Ok(())
    }

    /// Get success rate in basis points
    pub fn success_rate_bps(&self) -> u64 {
        if self.total_executions == 0 {
            return 0;
        }
        (self.successful_executions * BASIS_POINTS_MAX) / self.total_executions
    }

    /// Get average gas per execution
    pub fn avg_gas_per_execution(&self) -> u64 {
        if self.total_executions == 0 {
            return 0;
        }
        self.total_gas_used / self.total_executions
    }
}

/// Account initialization helper
pub struct AccountInitializer;

impl AccountInitializer {
    /// Initialize account with base fields
    pub fn init_base_account(authority: Pubkey, bump: u8) -> Result<BaseAccount> {
        BaseAccount::new(authority, bump)
    }

    /// Validate account initialization parameters
    pub fn validate_init_params(authority: &Pubkey, bump: u8) -> Result<()> {
        if *authority == Pubkey::default() {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        if bump == 0 {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(())
    }
}

/// Common account space calculations
pub struct SpaceCalculator;

impl SpaceCalculator {
    /// Calculate space for vector of pubkeys
    pub fn pubkey_vec_space(max_items: usize) -> usize {
        4 + (32 * max_items) // 4 bytes for length + 32 bytes per pubkey
    }

    /// Calculate space for vector of u64
    pub fn u64_vec_space(max_items: usize) -> usize {
        4 + (8 * max_items) // 4 bytes for length + 8 bytes per u64
    }

    /// Calculate space for vector of bytes
    pub fn bytes_vec_space(max_bytes: usize) -> usize {
        4 + max_bytes // 4 bytes for length + max bytes
    }

    /// Calculate space for string
    pub fn string_space(max_length: usize) -> usize {
        4 + max_length // 4 bytes for length + max string length
    }

    /// Calculate space for option type
    pub fn option_space<T>(inner_space: usize) -> usize {
        1 + inner_space // 1 byte for Some/None + inner type space
    }
}
