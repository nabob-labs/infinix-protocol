//!
//! Offchain Integration Error Definitions
//!
//! 本模块定义离线集成服务的错误类型、错误码与 trait 实现，确保错误处理合规、可追溯、可维护。

/// 离线集成错误类型枚举。
#[derive(Debug)]
pub enum OffchainIntegrationError {
    /// 未知错误。
    Unknown,
    /// 网络请求失败。
    NetworkError,
    /// 数据解析失败。
    ParseError,
    /// 权限不足。
    Unauthorized,
    /// 输入参数无效。
    InvalidInput,
    /// 预言机数据无效。
    InvalidOracleData,
    /// DEX 操作失败。
    DexOperationFailed,
}
