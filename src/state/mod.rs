// ========================= State 管理模块主入口 =========================
// 本模块为链上所有状态结构体、校验工具、事件等提供统一入口，
// 每个 struct、trait、方法、参数、用途、边界、Anchor 相关点、事件、错误、测试等均有详细注释。
// - 设计意图：极致可插拔、最小功能单元、统一接口、Anchor集成友好、可观测性、可维护性、可审计性
/*!
 * State Management Module for Solana AMM Index Token Strategies
 *
 * 本模块将所有链上状态结构体按逻辑子模块组织，便于维护和扩展。
 * 结构层次清晰，基础类型在底层，专用类型在子模块。
 */

use crate::core::*;         // 引入核心trait、类型、错误等
use crate::version::*;      // 版本管理相关
use anchor_lang::prelude::*; // Anchor预导入，包含Result、Context、Pubkey等

/// StateValidator 提供状态迁移和初始化的校验工具
/// - 统一所有链上状态结构体的初始化、迁移、权限、批量校验、健康检查等
/// - 设计意图：极致可插拔、统一校验、便于测试、审计、扩展
pub struct StateValidator;

impl StateValidator {
    /// 校验初始化参数（简化版）
    /// - params: 需实现 Validatable trait 的参数对象
    /// - 返回: 校验通过则 Ok，否则返回错误
    pub fn validate_init_params<T: crate::core::traits::Validatable>(
        params: &T,
    ) -> StrategyResult<()> {
        params.validate()
    }
    /// 校验状态迁移是否合法
    /// - old_state: 旧状态
    /// - new_state: 新状态
    /// - allowed_transitions: 允许的新状态列表
    /// - 返回: 合法则 Ok，否则返回错误
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
    /// 校验权限（简化版）
    /// - expected: 期望的 pubkey
    /// - actual: 实际 pubkey
    /// - 返回: 权限一致则 Ok，否则返回错误
    pub fn validate_authority(expected: &Pubkey, actual: &Pubkey) -> StrategyResult<()> {
        require!(
            *expected == *actual,
            crate::error::StrategyError::Unauthorized
        );
        Ok(())
    }
    /// 校验版本兼容性（简化版）
    /// - account: 需实现 Versioned trait 的账户对象
    /// - 返回: 版本兼容则 Ok，否则返回错误
    pub fn validate_version_compatibility<T: crate::core::traits::Versioned>(
        account: &T,
    ) -> StrategyResult<()> {
        let version = account.version();
        // 生产环境应实现更严格的版本校验
        if version.major < 1 {
            return Err(crate::error::StrategyError::IncompatibleVersion.into());
        }
        Ok(())
    }
    /// 自动迁移（简化版，带日志）
    /// - account: 需实现 Versioned trait 的账户对象
    /// - 返回: 迁移成功则 Ok，否则返回错误
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
    /// 批量账户校验（简化版）
    /// - accounts: 账户对象数组
    /// - 返回: 校验全部通过则 Ok，否则返回错误
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
    /// 账户健康检查（简化版）
    /// - account: 需实现 Versioned、Authorizable、Pausable trait 的账户对象
    /// - 返回: 健康则 Ok，否则返回错误
    pub fn health_check<
        T: crate::core::traits::Versioned
            + crate::core::traits::Authorizable
            + crate::core::traits::Pausable,
    >(
        account: &T,
    ) -> StrategyResult<()> {
        // 校验版本兼容性
        Self::validate_version_compatibility(account)?;
        // 校验账户未暂停
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

/// 账户迁移事件
/// - 记录链上账户自动迁移的详细信息，便于审计和可观测性
#[event]
pub struct AccountMigrated {
    /// 账户类型
    pub account_type: String,
    /// 迁移前版本
    pub from_version: Version,
    /// 迁移后版本
    pub to_version: Version,
    /// 迁移时间戳
    pub timestamp: i64,
}
