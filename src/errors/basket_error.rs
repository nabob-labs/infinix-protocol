//! Basket Errors
//! 篮子相关错误类型定义，所有篮子操作相关错误均在此枚举中统一管理。

use anchor_lang::prelude::*; // Anchor 预导入，包含 #[error_code]、Result、msg! 等

#[error_code]
/// 篮子相关错误枚举，统一所有篮子操作相关错误类型
/// - 用于 Anchor 指令返回链上错误，便于前端/客户端识别与处理
/// - 每个错误项均有详细注释，说明触发场景与设计意图
pub enum BasketError {
    /// 资产成分无效
    /// 触发场景：篮子资产组合不满足业务规则或校验失败
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
    /// 权重和无效
    /// 触发场景：篮子资产权重和不等于 100% 或不满足业务规则
    #[msg("Invalid weight sum")] InvalidWeightSum,
    /// 已处于暂停状态
    /// 触发场景：重复暂停已暂停的篮子
    #[msg("Already paused")] AlreadyPaused,
    /// 未处于暂停状态
    /// 触发场景：恢复未暂停的篮子
    #[msg("Not paused")] NotPaused,
    /// 未知错误
    /// 触发场景：未被明确捕获的异常或内部错误
    #[msg("Unknown error")] Unknown,
    /// 篮子操作未授权
    /// 触发场景：对篮子执行特定操作（如转移、销毁）时权限不足
    #[msg("Unauthorized operation for basket.")]
    UnauthorizedOperation,
    /// swap 操作失败
    /// 触发场景：篮子兑换（swap）过程中发生异常
    #[msg("Swap operation failed.")]
    SwapFailed,
    /// 合并操作失败
    /// 触发场景：篮子合并过程中发生异常
    #[msg("Combine operation failed.")]
    CombineFailed,
    /// 拆分操作失败
    /// 触发场景：篮子拆分过程中发生异常
    #[msg("Split operation failed.")]
    SplitFailed,
    /// 买入操作失败
    /// 触发场景：篮子买入过程中发生异常
    #[msg("Buy operation failed.")]
    BuyFailed,
    /// 卖出操作失败
    /// 触发场景：篮子卖出过程中发生异常
    #[msg("Sell operation failed.")]
    SellFailed,
    /// 授权操作失败
    /// 触发场景：篮子授权变更过程中发生异常
    #[msg("Authorization failed.")]
    AuthorizationFailed,
    /// 冻结操作失败
    /// 触发场景：篮子冻结过程中发生异常
    #[msg("Freeze operation failed.")]
    FreezeFailed,
    /// 解冻操作失败
    /// 触发场景：篮子解冻过程中发生异常
    #[msg("Unfreeze operation failed.")]
    UnfreezeFailed,
    /// 批量转账操作失败
    /// 触发场景：篮子批量转移过程中发生异常
    #[msg("Batch transfer operation failed.")]
    BatchTransferFailed,
    /// 批量再平衡操作失败
    /// 触发场景：篮子批量再平衡过程中发生异常
    #[msg("Batch rebalance operation failed.")]
    BatchRebalanceFailed,
    /// 策略再平衡操作失败
    /// 触发场景：篮子通过策略执行再平衡时发生异常
    #[msg("Strategy rebalance operation failed.")]
    StrategyRebalanceFailed,
    /// 批量申购操作失败
    /// 触发场景：篮子批量申购过程中发生异常
    #[msg("Batch subscribe operation failed.")]
    BatchSubscribeFailed,
    /// 批量赎回操作失败
    /// 触发场景：篮子批量赎回过程中发生异常
    #[msg("Batch redeem operation failed.")]
    BatchRedeemFailed,
    /// 批量合并操作失败
    /// 触发场景：多个篮子合并过程中发生异常
    #[msg("Batch combine operation failed.")]
    BatchCombineFailed,
    /// 批量拆分操作失败
    /// 触发场景：篮子批量拆分过程中发生异常
    #[msg("Batch split operation failed.")]
    BatchSplitFailed,
} 