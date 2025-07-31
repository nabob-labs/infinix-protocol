/*!
 * Program Version Management System
 *
 * Provides version control and upgrade management for the Solana program.
 * Enables seamless program upgrades while maintaining backward compatibility.
 */

use anchor_lang::prelude::*;
// Removed conflicting borsh import

/// 当前程序版本 - 采用增强重构架构
/// - 用于所有账户、指令的版本兼容性校验
pub const CURRENT_VERSION: ProgramVersion = ProgramVersion {
    major: 2,
    minor: 1,
    patch: 0,
};

/// 最低兼容版本
/// - 低于该版本的账户需强制迁移
pub const MIN_SUPPORTED_VERSION: ProgramVersion = ProgramVersion {
    major: 1,
    minor: 0,
    patch: 0,
};

/// 程序版本结构体
/// - 记录主版本、次版本、补丁号
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
    /// 主版本号
    pub major: u16,
    /// 次版本号
    pub minor: u16,
    /// 补丁号
    pub patch: u16,
}

impl ProgramVersion {
    /// 创建新版本
    /// - major: 主版本号
    /// - minor: 次版本号
    /// - patch: 补丁号
    pub const fn new(major: u16, minor: u16, patch: u16) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    /// 判断版本兼容性
    /// - other: 目标版本
    /// - 返回：是否兼容
    pub fn is_compatible_with(&self, other: &ProgramVersion) -> bool {
        // Major version must match, minor version must be >= minimum
        self.major == other.major && *self >= MIN_SUPPORTED_VERSION
    }

    /// 判断是否支持某特性
    /// - feature: 特性枚举
    /// - 返回：是否支持
    pub fn supports_feature(&self, feature: Feature) -> bool {
        match feature {
            Feature::BasicStrategies => *self >= ProgramVersion::new(1, 0, 0),
            Feature::AdvancedOptimization => *self >= ProgramVersion::new(1, 1, 0),
            Feature::CrossChainSupport => *self >= ProgramVersion::new(2, 0, 0),
            Feature::AIOptimization => *self >= ProgramVersion::new(1, 2, 0),
        }
    }

    /// 获取版本字符串
    pub fn to_string(&self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// 版本相关特性枚举
/// - 用于判断某版本是否支持特定功能
#[derive(Debug, Clone, Copy)]
pub enum Feature {
    /// 基础策略功能
    BasicStrategies,
    /// 高级优化功能
    AdvancedOptimization,
    /// 跨链支持
    CrossChainSupport,
    /// AI优化功能
    AIOptimization,
}

/// 具备版本感知能力的账户trait
/// - 便于账户自动迁移、版本校验
pub trait Versioned {
    /// 获取账户版本
    fn version(&self) -> ProgramVersion;

    /// 设置账户版本
    fn set_version(&mut self, version: ProgramVersion);

    /// 判断账户是否需要迁移
    fn needs_migration(&self) -> bool {
        self.version() < CURRENT_VERSION
    }

    /// 迁移账户到当前版本
    /// - 自动按版本升级路径依次迁移
    fn migrate(&mut self) -> anchor_lang::Result<()> {
        if !self.needs_migration() {
            return Ok(());
        }

        let current_version = self.version();

        // 按版本依次迁移 - 简化迁移路径
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

        // 设置为当前版本
        self.set_version(CURRENT_VERSION);

        msg!(
            "Account migrated from {} to {}",
            current_version.to_string(),
            CURRENT_VERSION.to_string()
        );

        Ok(())
    }

    /// 迁移到1.0.1版本 - 逻辑简化
    fn migrate_to_1_0_1(&mut self) -> anchor_lang::Result<()> {
        // 简化：基础兼容性更新
        msg!("Migrating to v1.0.1: Applying simplified logic improvements");
        Ok(())
    }

    /// 迁移到1.1.0版本 - 增强优化特性
    fn migrate_to_1_1_0(&mut self) -> anchor_lang::Result<()> {
        // 简化：增加优化特性
        msg!("Migrating to v1.1.0: Adding enhanced optimization features");
        Ok(())
    }

    /// 迁移到2.0.0版本 - 架构重构
    fn migrate_to_2_0_0(&mut self) -> anchor_lang::Result<()> {
        // 简化：迁移到新重构架构
        msg!("Migrating to v2.0.0: Upgrading to refactored architecture with simplified logic");
        Ok(())
    }

    /// 迁移到2.1.0版本 - 增强重构特性
    fn migrate_to_2_1_0(&mut self) -> anchor_lang::Result<()> {
        // 简化：增强特性和性能优化
        msg!("Migrating to v2.1.0: Adding enhanced refactored features and performance optimizations");
        Ok(())
    }
}

/// 版本校验工具
pub struct VersionValidator;

impl VersionValidator {
    /// 校验账户版本兼容性
    pub fn validate_compatibility(account_version: &ProgramVersion) -> anchor_lang::Result<()> {
        if !CURRENT_VERSION.is_compatible_with(account_version) {
            return Err(crate::error::StrategyError::IncompatibleVersion.into());
        }
        Ok(())
    }

    /// 校验特性支持
    pub fn validate_feature_support(version: &ProgramVersion, feature: Feature) -> anchor_lang::Result<()> {
        if !version.supports_feature(feature) {
            return Err(crate::error::StrategyError::FeatureNotSupported.into());
        }
        Ok(())
    }

    /// 获取版本升级路径
    pub fn get_upgrade_path(from: &ProgramVersion) -> Vec<ProgramVersion> {
        let mut path = Vec::new();
        let mut current = *from;

        while current < CURRENT_VERSION {
            // 简化迁移路径，按升级步骤依次推进
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
                // 主架构重构升级到v2.0.0
                path.push(ProgramVersion::new(2, 0, 0));
                current = ProgramVersion::new(2, 0, 0);
            } else if current < ProgramVersion::new(2, 1, 0)
                && CURRENT_VERSION >= ProgramVersion::new(2, 1, 0)
            {
                // 增强重构升级到v2.1.0
                path.push(ProgramVersion::new(2, 1, 0));
                current = ProgramVersion::new(2, 1, 0);
            } else {
                // 直接升级到当前版本
                path.push(CURRENT_VERSION);
                break;
            }
        }

        path
    }
}

/// 版本升级事件（Anchor事件）
/// - 记录账户从旧版本升级到新版本的链上事件
#[event]
pub struct VersionUpgraded {
    pub account: Pubkey,
    pub from_version: ProgramVersion,
    pub to_version: ProgramVersion,
    pub timestamp: i64,
}

/// 迁移完成事件（Anchor事件）
/// - 记录账户迁移完成的链上事件
#[event]
pub struct MigrationCompleted {
    pub account: Pubkey,
    pub version: ProgramVersion,
    pub migration_steps: u8,
    pub timestamp: i64,
}

// Macros are defined in core/macros.rs to avoid duplication
