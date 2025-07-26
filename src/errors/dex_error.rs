//!
//! DEX错误类型模块
//!
//! 定义所有DEX/AMM相关的错误类型，包括交易、流动性管理、报价等操作的错误处理。

use anchor_lang::prelude::*;
use thiserror::Error;
use crate::errors::{ErrorConvertible, error_codes::DEX_ERROR_BASE};

/// DEX相关错误类型
#[derive(Debug, Error, Clone, AnchorSerialize, AnchorDeserialize)]
pub enum DexError {
    /// 无效的交易参数
    #[error("Invalid swap parameters: {message}")]
    InvalidSwapParams {
        /// 错误消息
        message: String,
    },
    
    /// 交易金额无效
    #[error("Invalid amount: {amount}")]
    InvalidAmount {
        /// 无效金额
        amount: u64,
    },
    
    /// 代币对无效
    #[error("Invalid token pair: {token_in} -> {token_out}")]
    InvalidTokenPair {
        /// 输入代币
        token_in: String,
        /// 输出代币
        token_out: String,
    },
    
    /// 滑点超出限制
    #[error("Slippage exceeded: {actual} > {max}")]
    SlippageExceeded {
        /// 实际滑点
        actual: u64,
        /// 最大允许滑点
        max: u64,
    },
    
    /// 流动性不足
    #[error("Insufficient liquidity: {required} > {available}")]
    InsufficientLiquidity {
        /// 所需流动性
        required: u64,
        /// 可用流动性
        available: u64,
    },
    
    /// 价格影响过大
    #[error("Price impact too high: {impact}%")]
    PriceImpactTooHigh {
        /// 价格影响百分比
        impact: f64,
    },
    
    /// 交易失败
    #[error("Swap failed: {reason}")]
    SwapFailed {
        /// 失败原因
        reason: String,
    },
    
    /// 报价失败
    #[error("Quote failed: {reason}")]
    QuoteFailed {
        /// 失败原因
        reason: String,
    },
    
    /// 流动性池不存在
    #[error("Pool not found: {pool_id}")]
    PoolNotFound {
        /// 池ID
        pool_id: String,
    },
    
    /// 流动性池已暂停
    #[error("Pool paused: {pool_id}")]
    PoolPaused {
        /// 池ID
        pool_id: String,
    },
    
    /// 不支持的资产类型
    #[error("Unsupported asset: {asset}")]
    UnsupportedAsset {
        /// 不支持的资产
        asset: String,
    },
    
    /// 不支持的DEX类型
    #[error("Unsupported DEX: {dex_name}")]
    UnsupportedDex {
        /// DEX名称
        dex_name: String,
    },
    
    /// 路由失败
    #[error("Routing failed: {reason}")]
    RoutingFailed {
        /// 失败原因
        reason: String,
    },
    
    /// 批量交易失败
    #[error("Batch swap failed: {failed_count}/{total_count}")]
    BatchSwapFailed {
        /// 失败数量
        failed_count: u32,
        /// 总数量
        total_count: u32,
    },
    
    /// 费用计算错误
    #[error("Fee calculation error: {reason}")]
    FeeCalculationError {
        /// 错误原因
        reason: String,
    },
    
    /// 价格计算错误
    #[error("Price calculation error: {reason}")]
    PriceCalculationError {
        /// 错误原因
        reason: String,
    },
    
    /// 超时错误
    #[error("Operation timeout: {operation}")]
    Timeout {
        /// 操作类型
        operation: String,
    },
    
    /// 网络错误
    #[error("Network error: {reason}")]
    NetworkError {
        /// 错误原因
        reason: String,
    },
    
    /// 配置错误
    #[error("Configuration error: {field}")]
    ConfigurationError {
        /// 配置字段
        field: String,
    },
    
    /// 权限错误
    #[error("Permission denied: {operation}")]
    PermissionDenied {
        /// 操作类型
        operation: String,
    },
    
    /// 状态错误
    #[error("Invalid state: {current} -> {expected}")]
    InvalidState {
        /// 当前状态
        current: String,
        /// 期望状态
        expected: String,
    },
    
    /// 余额不足
    #[error("Insufficient balance: {required} > {available}")]
    InsufficientBalance {
        /// 所需余额
        required: u64,
        /// 可用余额
        available: u64,
    },
    
    /// 未知错误
    #[error("Unknown DEX error: {message}")]
    Unknown {
        /// 错误消息
        message: String,
    },
}

impl DexError {
    /// 创建新的DEX错误
    pub fn new(error_type: DexErrorType, message: &str) -> Self {
        match error_type {
            DexErrorType::InvalidSwapParams => DexError::InvalidSwapParams {
                message: message.to_string(),
            },
            DexErrorType::InvalidAmount => DexError::InvalidAmount {
                amount: 0, // 需要从message中解析
            },
            DexErrorType::InvalidTokenPair => DexError::InvalidTokenPair {
                token_in: "".to_string(),
                token_out: "".to_string(),
            },
            DexErrorType::SlippageExceeded => DexError::SlippageExceeded {
                actual: 0,
                max: 0,
            },
            DexErrorType::InsufficientLiquidity => DexError::InsufficientLiquidity {
                required: 0,
                available: 0,
            },
            DexErrorType::PriceImpactTooHigh => DexError::PriceImpactTooHigh {
                impact: 0.0,
            },
            DexErrorType::SwapFailed => DexError::SwapFailed {
                reason: message.to_string(),
            },
            DexErrorType::QuoteFailed => DexError::QuoteFailed {
                reason: message.to_string(),
            },
            DexErrorType::PoolNotFound => DexError::PoolNotFound {
                pool_id: message.to_string(),
            },
            DexErrorType::PoolPaused => DexError::PoolPaused {
                pool_id: message.to_string(),
            },
            DexErrorType::UnsupportedAsset => DexError::UnsupportedAsset {
                asset: message.to_string(),
            },
            DexErrorType::UnsupportedDex => DexError::UnsupportedDex {
                dex_name: message.to_string(),
            },
            DexErrorType::RoutingFailed => DexError::RoutingFailed {
                reason: message.to_string(),
            },
            DexErrorType::BatchSwapFailed => DexError::BatchSwapFailed {
                failed_count: 0,
                total_count: 0,
            },
            DexErrorType::FeeCalculationError => DexError::FeeCalculationError {
                reason: message.to_string(),
            },
            DexErrorType::PriceCalculationError => DexError::PriceCalculationError {
                reason: message.to_string(),
            },
            DexErrorType::Timeout => DexError::Timeout {
                operation: message.to_string(),
            },
            DexErrorType::NetworkError => DexError::NetworkError {
                reason: message.to_string(),
            },
            DexErrorType::ConfigurationError => DexError::ConfigurationError {
                field: message.to_string(),
            },
            DexErrorType::PermissionDenied => DexError::PermissionDenied {
                operation: message.to_string(),
            },
            DexErrorType::InvalidState => DexError::InvalidState {
                current: "".to_string(),
                expected: "".to_string(),
            },
            DexErrorType::InsufficientBalance => DexError::InsufficientBalance {
                required: 0,
                available: 0,
            },
            DexErrorType::Unknown => DexError::Unknown {
                message: message.to_string(),
            },
        }
    }
    
    /// 创建带错误码的DEX错误
    pub fn new_with_code(error_type: DexErrorType, message: &str, code: u32) -> Self {
        let mut error = Self::new(error_type, message);
        // 这里可以设置自定义错误码
        error
    }
    
    /// 检查是否为可恢复错误
    pub fn is_recoverable(&self) -> bool {
        match self {
            DexError::InvalidSwapParams { .. } => true,
            DexError::InvalidAmount { .. } => true,
            DexError::InvalidTokenPair { .. } => true,
            DexError::SlippageExceeded { .. } => true,
            DexError::InsufficientLiquidity { .. } => false,
            DexError::PriceImpactTooHigh { .. } => true,
            DexError::SwapFailed { .. } => false,
            DexError::QuoteFailed { .. } => true,
            DexError::PoolNotFound { .. } => false,
            DexError::PoolPaused { .. } => true,
            DexError::UnsupportedAsset { .. } => false,
            DexError::UnsupportedDex { .. } => false,
            DexError::RoutingFailed { .. } => true,
            DexError::BatchSwapFailed { .. } => true,
            DexError::FeeCalculationError { .. } => true,
            DexError::PriceCalculationError { .. } => true,
            DexError::Timeout { .. } => true,
            DexError::NetworkError { .. } => true,
            DexError::ConfigurationError { .. } => false,
            DexError::PermissionDenied { .. } => false,
            DexError::InvalidState { .. } => false,
            DexError::InsufficientBalance { .. } => false,
            DexError::Unknown { .. } => false,
        }
    }
    
    /// 获取错误严重程度
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            DexError::InvalidSwapParams { .. } => ErrorSeverity::Warning,
            DexError::InvalidAmount { .. } => ErrorSeverity::Warning,
            DexError::InvalidTokenPair { .. } => ErrorSeverity::Warning,
            DexError::SlippageExceeded { .. } => ErrorSeverity::Warning,
            DexError::InsufficientLiquidity { .. } => ErrorSeverity::Error,
            DexError::PriceImpactTooHigh { .. } => ErrorSeverity::Warning,
            DexError::SwapFailed { .. } => ErrorSeverity::Error,
            DexError::QuoteFailed { .. } => ErrorSeverity::Warning,
            DexError::PoolNotFound { .. } => ErrorSeverity::Error,
            DexError::PoolPaused { .. } => ErrorSeverity::Warning,
            DexError::UnsupportedAsset { .. } => ErrorSeverity::Error,
            DexError::UnsupportedDex { .. } => ErrorSeverity::Error,
            DexError::RoutingFailed { .. } => ErrorSeverity::Warning,
            DexError::BatchSwapFailed { .. } => ErrorSeverity::Error,
            DexError::FeeCalculationError { .. } => ErrorSeverity::Warning,
            DexError::PriceCalculationError { .. } => ErrorSeverity::Warning,
            DexError::Timeout { .. } => ErrorSeverity::Warning,
            DexError::NetworkError { .. } => ErrorSeverity::Warning,
            DexError::ConfigurationError { .. } => ErrorSeverity::Error,
            DexError::PermissionDenied { .. } => ErrorSeverity::Error,
            DexError::InvalidState { .. } => ErrorSeverity::Error,
            DexError::InsufficientBalance { .. } => ErrorSeverity::Error,
            DexError::Unknown { .. } => ErrorSeverity::Error,
        }
    }
}

impl ErrorConvertible for DexError {
    fn into_program_error(self) -> crate::errors::ProgramError {
        crate::errors::ProgramError::Dex(self)
    }
    
    fn error_code(&self) -> u32 {
        let base_code = match self {
            DexError::InvalidSwapParams { .. } => DEX_ERROR_BASE + 1,
            DexError::InvalidAmount { .. } => DEX_ERROR_BASE + 2,
            DexError::InvalidTokenPair { .. } => DEX_ERROR_BASE + 3,
            DexError::SlippageExceeded { .. } => DEX_ERROR_BASE + 4,
            DexError::InsufficientLiquidity { .. } => DEX_ERROR_BASE + 5,
            DexError::PriceImpactTooHigh { .. } => DEX_ERROR_BASE + 6,
            DexError::SwapFailed { .. } => DEX_ERROR_BASE + 7,
            DexError::QuoteFailed { .. } => DEX_ERROR_BASE + 8,
            DexError::PoolNotFound { .. } => DEX_ERROR_BASE + 9,
            DexError::PoolPaused { .. } => DEX_ERROR_BASE + 10,
            DexError::UnsupportedAsset { .. } => DEX_ERROR_BASE + 11,
            DexError::UnsupportedDex { .. } => DEX_ERROR_BASE + 12,
            DexError::RoutingFailed { .. } => DEX_ERROR_BASE + 13,
            DexError::BatchSwapFailed { .. } => DEX_ERROR_BASE + 14,
            DexError::FeeCalculationError { .. } => DEX_ERROR_BASE + 15,
            DexError::PriceCalculationError { .. } => DEX_ERROR_BASE + 16,
            DexError::Timeout { .. } => DEX_ERROR_BASE + 17,
            DexError::NetworkError { .. } => DEX_ERROR_BASE + 18,
            DexError::ConfigurationError { .. } => DEX_ERROR_BASE + 19,
            DexError::PermissionDenied { .. } => DEX_ERROR_BASE + 20,
            DexError::InvalidState { .. } => DEX_ERROR_BASE + 21,
            DexError::InsufficientBalance { .. } => DEX_ERROR_BASE + 22,
            DexError::Unknown { .. } => DEX_ERROR_BASE + 99,
        };
        base_code
    }
    
    fn is_recoverable(&self) -> bool {
        self.is_recoverable()
    }
}

/// DEX错误类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DexErrorType {
    InvalidSwapParams,
    InvalidAmount,
    InvalidTokenPair,
    SlippageExceeded,
    InsufficientLiquidity,
    PriceImpactTooHigh,
    SwapFailed,
    QuoteFailed,
    PoolNotFound,
    PoolPaused,
    UnsupportedAsset,
    UnsupportedDex,
    RoutingFailed,
    BatchSwapFailed,
    FeeCalculationError,
    PriceCalculationError,
    Timeout,
    NetworkError,
    ConfigurationError,
    PermissionDenied,
    InvalidState,
    InsufficientBalance,
    Unknown,
}

/// 错误严重程度
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    /// 警告级别
    Warning,
    /// 错误级别
    Error,
    /// 严重错误级别
    Critical,
}

/// DEX错误工具函数
pub mod utils {
    use super::*;
    
    /// 创建滑点错误
    pub fn slippage_error(actual: u64, max: u64) -> DexError {
        DexError::SlippageExceeded { actual, max }
    }
    
    /// 创建流动性不足错误
    pub fn liquidity_error(required: u64, available: u64) -> DexError {
        DexError::InsufficientLiquidity { required, available }
    }
    
    /// 创建余额不足错误
    pub fn balance_error(required: u64, available: u64) -> DexError {
        DexError::InsufficientBalance { required, available }
    }
    
    /// 检查是否为网络相关错误
    pub fn is_network_error(error: &DexError) -> bool {
        matches!(error, DexError::NetworkError { .. } | DexError::Timeout { .. })
    }
    
    /// 检查是否为配置相关错误
    pub fn is_configuration_error(error: &DexError) -> bool {
        matches!(error, DexError::ConfigurationError { .. } | DexError::UnsupportedDex { .. })
    }
    
    /// 检查是否为权限相关错误
    pub fn is_permission_error(error: &DexError) -> bool {
        matches!(error, DexError::PermissionDenied { .. })
    }
} 