//! Asset Errors
//! 资产相关错误类型定义，所有资产操作相关错误均在此枚举中统一管理。

use anchor_lang::prelude::*; // Anchor 预导入，包含 #[error_code]、Result、msg! 等

#[error_code]
/// 资产相关错误枚举，统一所有资产操作相关错误类型
/// - 用于 Anchor 指令返回链上错误，便于前端/客户端识别与处理
/// - 每个错误项均有详细注释，说明触发场景与设计意图
pub enum AssetError {
    /// 资产成分无效
    /// 触发场景：资产组合不满足业务规则或校验失败
    #[msg("Invalid asset composition")] InvalidAssets,
    /// 资产余额不足
    /// 触发场景：操作所需资产数量大于当前余额
    #[msg("Insufficient value")] InsufficientValue,
    /// 未授权操作
    /// 触发场景：非权限账户尝试执行受限操作
    #[msg("Unauthorized")] Unauthorized,
    /// 操作不允许
    /// 触发场景：当前状态下禁止该操作，如冻结、暂停等
    #[msg("Operation not allowed")] NotAllowed,
    /// 未知错误
    /// 触发场景：未被明确捕获的异常或内部错误
    #[msg("Unknown error")] Unknown,
    /// 资产操作未授权
    /// 触发场景：对资产执行特定操作（如转移、销毁）时权限不足
    #[msg("Unauthorized operation for asset.")]
    UnauthorizedOperation,
    /// swap 操作失败
    /// 触发场景：资产兑换（swap）过程中发生异常
    #[msg("Swap operation failed.")]
    SwapFailed,
    /// 合并操作失败
    /// 触发场景：资产合并过程中发生异常
    #[msg("Combine operation failed.")]
    CombineFailed,
    /// 拆分操作失败
    /// 触发场景：资产拆分过程中发生异常
    #[msg("Split operation failed.")]
    SplitFailed,
    /// 买入操作失败
    /// 触发场景：资产买入过程中发生异常
    #[msg("Buy operation failed.")]
    BuyFailed,
    /// 卖出操作失败
    /// 触发场景：资产卖出过程中发生异常
    #[msg("Sell operation failed.")]
    SellFailed,
    /// 授权操作失败
    /// 触发场景：资产授权变更过程中发生异常
    #[msg("Authorization failed.")]
    AuthorizationFailed,
    /// 冻结操作失败
    /// 触发场景：资产冻结过程中发生异常
    #[msg("Freeze operation failed.")]
    FreezeFailed,
    /// 解冻操作失败
    /// 触发场景：资产解冻过程中发生异常
    #[msg("Unfreeze operation failed.")]
    UnfreezeFailed,
    /// 批量转账操作失败
    /// 触发场景：资产批量转移过程中发生异常
    #[msg("Batch transfer operation failed.")]
    BatchTransferFailed,
    /// 批量交换操作失败
    /// 触发场景：资产批量兑换过程中发生异常
    #[msg("Batch swap operation failed.")]
    BatchSwapFailed,
    /// 策略交易操作失败
    /// 触发场景：资产通过策略执行交易时发生异常
    #[msg("Strategy trade operation failed.")]
    StrategyTradeFailed,
} 