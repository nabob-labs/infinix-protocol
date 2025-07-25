/// IndexToken 相关错误类型模块
pub mod index_token_error;
/// Basket 相关错误类型模块
pub mod basket_error;
/// Asset 相关错误类型模块
pub mod asset_error;

/// 主错误枚举，统一所有策略相关错误类型
/// - 用于 Anchor 指令返回链上通用错误，便于前端/客户端统一识别与处理
/// - 每个错误项均有详细注释，说明触发场景与设计意图
#[error_code]
pub enum ProgramError {
    /// 策略已暂停
    /// 触发场景：尝试对已暂停的策略执行操作
    #[msg("Strategy paused")] StrategyPaused,
    /// 溢出错误
    /// 触发场景：数值计算超出类型边界
    #[msg("Overflow")] Overflow,
    /// 供应不足
    /// 触发场景：操作所需供应量大于当前可用供应
    #[msg("Insufficient supply")] InsufficientSupply,
    /// 代币数量无效
    /// 触发场景：输入或计算得到的代币数量不符合业务规则
    #[msg("Invalid token count")] InvalidTokenCount,
    /// 权重和无效
    /// 触发场景：资产权重和不等于 100% 或不满足业务规则
    #[msg("Invalid weight sum")] InvalidWeightSum,
    /// 算法未找到
    /// 触发场景：指定算法名称或类型在注册表中不存在
    #[msg("Algorithm not found")] AlgorithmNotFound,
    /// 算法类型不匹配
    /// 触发场景：算法类型与预期不符
    #[msg("Algorithm type mismatch")] AlgorithmTypeMismatch,
    /// 流动性聚合失败
    /// 触发场景：DEX/AMM 流动性聚合过程中发生异常
    #[msg("Liquidity aggregation failed")] LiquidityError,
    /// 未知错误
    /// 触发场景：未被明确捕获的异常或内部错误
    #[msg("Unknown error")] UnknownError,
}

/// IndexTokenError 到 ProgramError 的转换实现
/// - 设计意图：便于统一错误处理，将子模块错误归一为主错误类型
impl From<crate::errors::index_token_error::IndexTokenError> for ProgramError {
    fn from(_e: crate::errors::index_token_error::IndexTokenError) -> Self {
        ProgramError::UnknownError
    }
}
/// BasketError 到 ProgramError 的转换实现
/// - 设计意图：便于统一错误处理，将子模块错误归一为主错误类型
impl From<crate::errors::basket_error::BasketError> for ProgramError {
    fn from(_e: crate::errors::basket_error::BasketError) -> Self {
        ProgramError::UnknownError
    }
}
/// AssetError 到 ProgramError 的转换实现
/// - 设计意图：便于统一错误处理，将子模块错误归一为主错误类型
impl From<crate::errors::asset_error::AssetError> for ProgramError {
    fn from(_e: crate::errors::asset_error::AssetError) -> Self {
        ProgramError::UnknownError
    }
} 