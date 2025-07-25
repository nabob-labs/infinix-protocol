//! Index Token Errors
//! 指数代币相关错误类型定义，所有指数代币操作相关错误均在此枚举中统一管理。

use anchor_lang::prelude::*; // Anchor 预导入，包含 #[error_code]、Result、msg! 等

#[error_code]
/// 指数代币相关错误枚举，统一所有指数代币操作相关错误类型
/// - 用于 Anchor 指令返回链上错误，便于前端/客户端识别与处理
/// - 每个错误项均有详细注释，说明触发场景与设计意图
pub enum IndexTokenError {
    /// 资产成分无效
    /// 触发场景：指数代币资产组合不满足业务规则或校验失败
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
    /// 指数代币操作未授权
    /// 触发场景：对指数代币执行特定操作（如转移、销毁）时权限不足
    #[msg("Unauthorized operation for index token.")]
    UnauthorizedOperation,
    /// swap 操作失败
    /// 触发场景：指数代币兑换（swap）过程中发生异常
    #[msg("Swap operation failed.")]
    SwapFailed,
    /// 合并操作失败
    /// 触发场景：指数代币合并过程中发生异常
    #[msg("Combine operation failed.")]
    CombineFailed,
    /// 拆分操作失败
    /// 触发场景：指数代币拆分过程中发生异常
    #[msg("Split operation failed.")]
    SplitFailed,
    /// 买入操作失败
    /// 触发场景：指数代币买入过程中发生异常
    #[msg("Buy operation failed.")]
    BuyFailed,
    /// 卖出操作失败
    /// 触发场景：指数代币卖出过程中发生异常
    #[msg("Sell operation failed.")]
    SellFailed,
    /// 授权操作失败
    /// 触发场景：指数代币授权变更过程中发生异常
    #[msg("Authorization failed.")]
    AuthorizationFailed,
    /// 冻结操作失败
    /// 触发场景：指数代币冻结过程中发生异常
    #[msg("Freeze operation failed.")]
    FreezeFailed,
    /// 解冻操作失败
    /// 触发场景：指数代币解冻过程中发生异常
    #[msg("Unfreeze operation failed.")]
    UnfreezeFailed,
    /// 批量转账操作失败
    /// 触发场景：指数代币批量转移过程中发生异常
    #[msg("Batch transfer operation failed.")]
    BatchTransferFailed,
    /// 批量申购操作失败
    /// 触发场景：指数代币批量申购过程中发生异常
    #[msg("Batch subscribe operation failed.")]
    BatchSubscribeFailed,
    /// 批量赎回操作失败
    /// 触发场景：指数代币批量赎回过程中发生异常
    #[msg("Batch redeem operation failed.")]
    BatchRedeemFailed,
    /// 策略申购操作失败
    /// 触发场景：指数代币通过策略执行申购时发生异常
    #[msg("Strategy subscribe operation failed.")]
    StrategySubscribeFailed,
    /// 策略赎回操作失败
    /// 触发场景：指数代币通过策略执行赎回时发生异常
    #[msg("Strategy redeem operation failed.")]
    StrategyRedeemFailed,
    /// 批量合并操作失败
    /// 触发场景：多个指数代币合并过程中发生异常
    #[msg("Batch combine operation failed.")]
    BatchCombineFailed,
    /// 批量拆分操作失败
    /// 触发场景：指数代币批量拆分过程中发生异常
    #[msg("Batch split operation failed.")]
    BatchSplitFailed,
} 