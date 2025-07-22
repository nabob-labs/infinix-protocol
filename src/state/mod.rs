/*!
 * State Management Module for Solana AMM Index Token Strategies
 *
 * This module organizes all on-chain state structures into logical sub-modules
 * for better maintainability and organization. The structure follows a clear
 * hierarchy with common types at the base and specialized types in sub-modules.
 */

pub mod baskets;
pub mod common;
pub mod factories;
pub mod optimizers;
pub mod strategies;

// Re-export common types first
pub use common::*;

// Re-export specialized types
pub use baskets::*;
pub use factories::*;
pub use optimizers::*;
pub use strategies::*;

use crate::core::*;
use crate::version::*;
use anchor_lang::prelude::*;

/// StateValidator provides utilities for validating state transitions and initialization.
///
/// - `validate_init_params`: Validates initialization parameters using the Validatable trait.
/// - `validate_state_transition`: Checks if a state transition is allowed based on a whitelist.
/// - `validate_authority`: Validates authority matches expected pubkey.
pub struct StateValidator;

impl StateValidator {
    /// Simplified validation of initialization parameters
    pub fn validate_init_params<T: crate::core::traits::Validatable>(
        params: &T,
    ) -> StrategyResult<()> {
        params.validate()
    }

    /// Validate state transition between two states.
    ///
    /// # Arguments
    /// - `old_state`: The current state before transition
    /// - `new_state`: The desired state after transition
    /// - `allowed_transitions`: List of allowed new states
    ///
    /// # Returns
    /// - `Ok(())` if the transition is allowed
    /// - `Err(StrategyError::InvalidStrategyParameters)` if not allowed
    ///
    /// # Example
    /// ```
    /// let allowed = vec![State::Active, State::Paused];
    /// StateValidator::validate_state_transition(&State::Init, &State::Active, &allowed)?;
    /// ```
    pub fn validate_state_transition<T: std::fmt::Debug + PartialEq>(
        old_state: &T,
        new_state: &T,
        allowed_transitions: &[T],
    ) -> StrategyResult<()> {
        if !allowed_transitions.contains(new_state) {
            msg!(
                "Invalid state transition: {:?} -> {:?}",
                old_state,
                new_state
            );
            return Err(crate::error::StrategyError::InvalidStrategyParameters.into());
        }
        Ok(())
    }

    /// Simplified authority validation
    pub fn validate_authority(expected: &Pubkey, actual: &Pubkey) -> StrategyResult<()> {
        require!(
            *expected == *actual,
            crate::error::StrategyError::Unauthorized
        );
        Ok(())
    }

    /// Simplified version compatibility check
    pub fn validate_version_compatibility<T: crate::core::traits::Versioned>(
        account: &T,
    ) -> StrategyResult<()> {
        let version = account.version();
        // Simplified version check - in production, implement proper version validation
        if version.major < 1 {
            return Err(crate::error::StrategyError::IncompatibleVersion.into());
        }
        Ok(())
    }

    /// Simplified automatic migration with enhanced logging
    pub fn auto_migrate<T: crate::core::traits::Versioned + crate::core::traits::Versioned>(
        account: &mut T,
    ) -> StrategyResult<()> {
        if account.needs_migration() {
            let old_version = account.version();
            let target_version = crate::version::CURRENT_VERSION;
            account.migrate(target_version)?;

            emit!(AccountMigrated {
                account_type: std::any::type_name::<T>().to_string(),
                from_version: old_version,
                to_version: target_version,
                timestamp: Clock::get()?.unix_timestamp,
            });

            msg!(
                "Account migrated (simplified): {:?} -> {:?}",
                old_version,
                target_version
            );
        }
        Ok(())
    }

    /// Simplified batch validation for multiple accounts
    pub fn validate_accounts_batch<
        T: crate::core::traits::Versioned + crate::core::traits::Versioned,
    >(
        accounts: &mut [&mut T],
    ) -> StrategyResult<()> {
        let count = accounts.len();
        for account in accounts {
            Self::auto_migrate(*account)?;
            Self::validate_version_compatibility(*account)?;
        }
        msg!(
            "Batch account validation completed (simplified): {} accounts",
            count
        );
        Ok(())
    }

    /// Simplified account health check
    pub fn health_check<
        T: crate::core::traits::Versioned
            + crate::core::traits::Authorizable
            + crate::core::traits::Pausable,
    >(
        account: &T,
    ) -> StrategyResult<()> {
        // Check version compatibility
        Self::validate_version_compatibility(account)?;

        // Check if account is not paused
        require!(
            !account.is_paused(),
            crate::error::StrategyError::StrategyPaused
        );

        msg!("Account health check passed (simplified)");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, Clone)]
    enum DummyState {
        Init,
        Active,
        Paused,
        Closed,
    }

    #[test]
    fn test_validate_state_transition_valid() {
        let old_state = DummyState::Init;
        let new_state = DummyState::Active;
        let allowed = vec![DummyState::Active, DummyState::Paused];
        assert!(
            StateValidator::validate_state_transition(&old_state, &new_state, &allowed).is_ok()
        );
    }

    #[test]
    fn test_validate_state_transition_invalid() {
        let old_state = DummyState::Active;
        let new_state = DummyState::Closed;
        let allowed = vec![DummyState::Active, DummyState::Paused];
        assert!(
            StateValidator::validate_state_transition(&old_state, &new_state, &allowed).is_err()
        );
    }
}

/// Account migration event
#[event]
pub struct AccountMigrated {
    pub account_type: String,
    pub from_version: Version,
    pub to_version: Version,
    pub timestamp: i64,
}
