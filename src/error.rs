//!
//! Global Error Definitions
//!
//! 本模块定义全局错误类型、错误码、错误映射与 trait 实现，确保错误处理合规、可追溯、可维护。

// 引入 Anchor 依赖。
use anchor_lang::prelude::*;

/// 全局错误类型枚举。
#[error_code]
pub enum ErrorCode {
    /// 未知错误。
    UnknownError,
    /// 权限不足。
    Unauthorized,
    /// 输入参数无效。
    InvalidInput,
    /// 账户不存在。
    AccountNotFound,
    /// 账户余额不足。
    InsufficientBalance,
    /// 操作超时。
    OperationTimeout,
    /// 状态无效。
    InvalidState,
    /// 资产无效。
    InvalidAsset,
    /// 权重无效。
    InvalidWeight,
    /// 策略参数无效。
    InvalidStrategyParameter,
    /// 策略执行失败。
    StrategyExecutionFailed,
    /// 预言机数据无效。
    InvalidOracleData,
    /// 价格无效。
    InvalidPrice,
    /// 价格过期。
    StalePrice,
    /// 再平衡阈值无效。
    InvalidRebalanceThreshold,
    /// 超出最大资产数量。
    ExceedMaxTokens,
    /// 超出最大批量处理大小。
    ExceedMaxBatchSize,
    /// 缓存未命中。
    CacheMiss,
    /// 缓存已过期。
    CacheExpired,
    /// 计算溢出。
    MathOverflow,
    /// 除零错误。
    DivisionByZero,
    /// 策略未注册。
    StrategyNotRegistered,
    /// 策略已存在。
    StrategyAlreadyExists,
    /// 策略不可用。
    StrategyUnavailable,
    /// 策略校验失败。
    StrategyValidationFailed,
    /// 策略参数超出最大长度。
    ExceedMaxStrategyParametersSize,
    /// 策略参数过小。
    StrategyParameterTooSmall,
    /// 策略参数过大。
    StrategyParameterTooLarge,
    /// 策略执行超时。
    StrategyExecutionTimeout,
    /// 策略执行被中断。
    StrategyExecutionInterrupted,
    /// 策略执行被拒绝。
    StrategyExecutionRejected,
    /// 策略执行未完成。
    StrategyExecutionIncomplete,
    /// 策略执行结果无效。
    InvalidStrategyExecutionResult,
    /// 策略执行结果过期。
    StaleStrategyExecutionResult,
    /// 策略执行结果冲突。
    ConflictingStrategyExecutionResult,
    /// 策略执行结果不一致。
    InconsistentStrategyExecutionResult,
    /// 策略执行结果缺失。
    MissingStrategyExecutionResult,
    /// 策略执行结果重复。
    DuplicateStrategyExecutionResult,
    /// 策略执行结果不支持。
    UnsupportedStrategyExecutionResult,
    /// 策略执行结果未实现。
    UnimplementedStrategyExecutionResult,
    /// 策略执行结果未定义。
    UndefinedStrategyExecutionResult,
    /// 策略执行结果未初始化。
    UninitializedStrategyExecutionResult,
    /// 策略执行结果未激活。
    UnactivatedStrategyExecutionResult,
    /// 策略执行结果未授权。
    UnauthorizedStrategyExecutionResult,
    /// 策略执行结果未验证。
    UnverifiedStrategyExecutionResult,
    /// 策略执行结果未签名。
    UnsingedStrategyExecutionResult,
    /// 策略执行结果未提交。
    UncommittedStrategyExecutionResult,
    /// 策略执行结果未确认。
    UnconfirmedStrategyExecutionResult,
    /// 策略执行结果未同步。
    UnsyncedStrategyExecutionResult,
    /// 策略执行结果未持久化。
    UnpersistedStrategyExecutionResult,
    /// 策略执行结果未归档。
    UnarchivedStrategyExecutionResult,
    /// 策略执行结果未清理。
    UncleanedStrategyExecutionResult,
    /// 策略执行结果未回滚。
    UnrolledBackStrategyExecutionResult,
    /// 策略执行结果未恢复。
    UnrecoveredStrategyExecutionResult,
    /// 策略执行结果未重试。
    UnretriedStrategyExecutionResult,
    /// 策略执行结果未通知。
    UnnotifiedStrategyExecutionResult,
    /// 策略执行结果未广播。
    UnbroadcastedStrategyExecutionResult,
    /// 策略执行结果未上链。
    UnchainedStrategyExecutionResult,
    /// 策略执行结果未下链。
    UnoffchainedStrategyExecutionResult,
    /// 策略执行结果未同步到链上。
    UnchainSyncedStrategyExecutionResult,
    /// 策略执行结果未同步到链下。
    UnoffchainSyncedStrategyExecutionResult,
    /// 策略执行结果未同步到缓存。
    UncacheSyncedStrategyExecutionResult,
    /// 策略执行结果未同步到数据库。
    UndbSyncedStrategyExecutionResult,
    /// 策略执行结果未同步到日志。
    UnlogSyncedStrategyExecutionResult,
    /// 策略执行结果未同步到监控。
    UnmonitorSyncedStrategyExecutionResult,
    /// 策略执行结果未同步到告警。
    UnalertSyncedStrategyExecutionResult,
    /// 策略执行结果未同步到通知。
    UnnotifySyncedStrategyExecutionResult,
    /// 策略执行结果未同步到备份。
    UnbackupSyncedStrategyExecutionResult,
    /// 策略执行结果未同步到归档。
    UnarchiveSyncedStrategyExecutionResult,
    /// 策略执行结果未同步到恢复。
    UnrecoverSyncedStrategyExecutionResult,
    /// 策略执行结果未同步到重试。
    UnretrySyncedStrategyExecutionResult,
    /// 策略执行结果未同步到回滚。
    UnrollbackSyncedStrategyExecutionResult,
    /// 策略执行结果未同步到清理。
    UncleanSyncedStrategyExecutionResult,
    /// 策略执行结果未同步到持久化。
    UnpersistSyncedStrategyExecutionResult,
    /// 策略执行结果未同步到归档。
    UnarchiveSyncedStrategyExecutionResult,
    /// 策略执行结果未同步到未定义。
    UnundefinedSyncedStrategyExecutionResult,
}
