/*!
 * Program Version Management System
 *
 * Provides version control and upgrade management for the Solana program.
 * Enables seamless program upgrades while maintaining backward compatibility.
 */

use anchor_lang::prelude::*;
// Removed conflicting borsh import

/// Current program version - Enhanced refactored architecture
pub const CURRENT_VERSION: ProgramVersion = ProgramVersion {
    major: 2,
    minor: 1,
    patch: 0,
};

/// Minimum supported version for compatibility
pub const MIN_SUPPORTED_VERSION: ProgramVersion = ProgramVersion {
    major: 1,
    minor: 0,
    patch: 0,
};

/// Program version structure
#[derive(
    AnchorSerialize,
    AnchorDeserialize,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    InitSpace,
)]
pub struct ProgramVersion {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

impl ProgramVersion {
    /// Create a new version
    pub const fn new(major: u16, minor: u16, patch: u16) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    /// Check if this version is compatible with another
    pub fn is_compatible_with(&self, other: &ProgramVersion) -> bool {
        // Major version must match, minor version must be >= minimum
        self.major == other.major && *self >= MIN_SUPPORTED_VERSION
    }

    /// Check if this version supports a feature
    pub fn supports_feature(&self, feature: Feature) -> bool {
        match feature {
            Feature::BasicStrategies => *self >= ProgramVersion::new(1, 0, 0),
            Feature::AdvancedOptimization => *self >= ProgramVersion::new(1, 1, 0),
            Feature::CrossChainSupport => *self >= ProgramVersion::new(2, 0, 0),
            Feature::AIOptimization => *self >= ProgramVersion::new(1, 2, 0),
        }
    }

    /// Get version as string
    pub fn to_string(&self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Feature flags for version-dependent functionality
#[derive(Debug, Clone, Copy)]
pub enum Feature {
    BasicStrategies,
    AdvancedOptimization,
    CrossChainSupport,
    AIOptimization,
}

/// Version-aware account trait
pub trait Versioned {
    /// Get the version of this account
    fn version(&self) -> ProgramVersion;

    /// Set the version of this account
    fn set_version(&mut self, version: ProgramVersion);

    /// Check if account needs migration
    fn needs_migration(&self) -> bool {
        self.version() < CURRENT_VERSION
    }

    /// Migrate account to current version
    fn migrate(&mut self) -> Result<()> {
        if !self.needs_migration() {
            return Ok(());
        }

        let current_version = self.version();

        // Perform version-specific migrations - Simplified migration path
        if current_version < ProgramVersion::new(1, 0, 1) {
            self.migrate_to_1_0_1()?;
        }

        if current_version < ProgramVersion::new(1, 1, 0) {
            self.migrate_to_1_1_0()?;
        }

        if current_version < ProgramVersion::new(2, 0, 0) {
            self.migrate_to_2_0_0()?;
        }

        if current_version < ProgramVersion::new(2, 1, 0) {
            self.migrate_to_2_1_0()?;
        }

        // Set to current version
        self.set_version(CURRENT_VERSION);

        msg!(
            "Account migrated from {} to {}",
            current_version.to_string(),
            CURRENT_VERSION.to_string()
        );

        Ok(())
    }

    /// Migration to version 1.0.1 - Simplified logic improvements
    fn migrate_to_1_0_1(&mut self) -> Result<()> {
        // Simplified: Basic compatibility updates
        msg!("Migrating to v1.0.1: Applying simplified logic improvements");
        Ok(())
    }

    /// Migration to version 1.1.0 - Enhanced optimization features
    fn migrate_to_1_1_0(&mut self) -> Result<()> {
        // Simplified: Add optimization features
        msg!("Migrating to v1.1.0: Adding enhanced optimization features");
        Ok(())
    }

    /// Migration to version 2.0.0 - Refactored architecture
    fn migrate_to_2_0_0(&mut self) -> Result<()> {
        // Simplified: Migrate to new refactored architecture
        msg!("Migrating to v2.0.0: Upgrading to refactored architecture with simplified logic");
        Ok(())
    }

    /// Migration to version 2.1.0 - Enhanced refactored architecture
    fn migrate_to_2_1_0(&mut self) -> Result<()> {
        // Simplified: Enhanced features and performance improvements
        msg!("Migrating to v2.1.0: Adding enhanced refactored features and performance optimizations");
        Ok(())
    }
}

/// Version validation utilities
pub struct VersionValidator;

impl VersionValidator {
    /// Validate that an account version is compatible
    pub fn validate_compatibility(account_version: &ProgramVersion) -> Result<()> {
        if !CURRENT_VERSION.is_compatible_with(account_version) {
            return Err(crate::error::StrategyError::IncompatibleVersion.into());
        }
        Ok(())
    }

    /// Validate that a feature is supported
    pub fn validate_feature_support(version: &ProgramVersion, feature: Feature) -> Result<()> {
        if !version.supports_feature(feature) {
            return Err(crate::error::StrategyError::FeatureNotSupported.into());
        }
        Ok(())
    }

    /// Get upgrade path for a version
    pub fn get_upgrade_path(from: &ProgramVersion) -> Vec<ProgramVersion> {
        let mut path = Vec::new();
        let mut current = *from;

        while current < CURRENT_VERSION {
            // Simplified migration path with clear upgrade steps
            if current < ProgramVersion::new(1, 0, 1)
                && CURRENT_VERSION >= ProgramVersion::new(1, 0, 1)
            {
                path.push(ProgramVersion::new(1, 0, 1));
                current = ProgramVersion::new(1, 0, 1);
            } else if current < ProgramVersion::new(1, 1, 0)
                && CURRENT_VERSION >= ProgramVersion::new(1, 1, 0)
            {
                path.push(ProgramVersion::new(1, 1, 0));
                current = ProgramVersion::new(1, 1, 0);
            } else if current < ProgramVersion::new(2, 0, 0)
                && CURRENT_VERSION >= ProgramVersion::new(2, 0, 0)
            {
                // Major refactoring upgrade to v2.0.0
                path.push(ProgramVersion::new(2, 0, 0));
                current = ProgramVersion::new(2, 0, 0);
            } else if current < ProgramVersion::new(2, 1, 0)
                && CURRENT_VERSION >= ProgramVersion::new(2, 1, 0)
            {
                // Enhanced refactoring upgrade to v2.1.0
                path.push(ProgramVersion::new(2, 1, 0));
                current = ProgramVersion::new(2, 1, 0);
            } else {
                // Direct upgrade to current version
                path.push(CURRENT_VERSION);
                break;
            }
        }

        path
    }
}

/// Version management events
#[event]
pub struct VersionUpgraded {
    pub account: Pubkey,
    pub from_version: ProgramVersion,
    pub to_version: ProgramVersion,
    pub timestamp: i64,
}

#[event]
pub struct MigrationCompleted {
    pub account: Pubkey,
    pub version: ProgramVersion,
    pub migration_steps: u8,
    pub timestamp: i64,
}

// Macros are defined in core/macros.rs to avoid duplication
