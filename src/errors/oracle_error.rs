//!
//! Oracle错误类型模块
//!
//! 定义所有价格预言机相关的错误类型，包括数据获取、验证、聚合等操作的错误处理。

use anchor_lang::prelude::*;
use thiserror::Error;
use crate::errors::{ErrorConvertible, error_codes::ORACLE_ERROR_BASE};

/// Oracle相关错误类型
#[derive(Debug, Error, Clone, AnchorSerialize, AnchorDeserialize)]
pub enum OracleError {
    /// 价格数据过期
    #[error("Price data stale: {age_seconds}s > {max_age_seconds}s")]
    PriceStale {
        /// 数据年龄（秒）
        age_seconds: u64,
        /// 最大允许年龄（秒）
        max_age_seconds: u64,
    },
    
    /// 价格数据无效
    #[error("Invalid price data: {reason}")]
    InvalidPriceData {
        /// 无效原因
        reason: String,
    },
    
    /// 价格偏差过大
    #[error("Price deviation too large: {deviation}% > {max_deviation}%")]
    PriceDeviationTooLarge {
        /// 实际偏差百分比
        deviation: f64,
        /// 最大允许偏差百分比
        max_deviation: f64,
    },
    
    /// 预言机不可用
    #[error("Oracle unavailable: {oracle_name}")]
    OracleUnavailable {
        /// 预言机名称
        oracle_name: String,
    },
    
    /// 聚合失败
    #[error("Price aggregation failed: {reason}")]
    AggregationFailed {
        /// 失败原因
        reason: String,
    },
    
    /// 权重配置无效
    #[error("Invalid oracle weights: {reason}")]
    InvalidWeights {
        /// 无效原因
        reason: String,
    },
    
    /// 预言机数量不足
    #[error("Insufficient oracles: {count} < {min_required}")]
    InsufficientOracles {
        /// 当前数量
        count: u8,
        /// 最小要求数量
        min_required: u8,
    },
    
    /// 预言机响应超时
    #[error("Oracle timeout: {oracle_name}")]
    OracleTimeout {
        /// 预言机名称
        oracle_name: String,
    },
    
    /// 预言机数据格式错误
    #[error("Oracle data format error: {oracle_name} - {reason}")]
    DataFormatError {
        /// 预言机名称
        oracle_name: String,
        /// 错误原因
        reason: String,
    },
    
    /// 预言机权限错误
    #[error("Oracle permission error: {oracle_name}")]
    PermissionError {
        /// 预言机名称
        oracle_name: String,
    },
    
    /// 预言机配置错误
    #[error("Oracle configuration error: {reason}")]
    ConfigurationError {
        /// 错误原因
        reason: String,
    },
}

impl ErrorConvertible for OracleError {
    fn error_code(&self) -> u32 {
        match self {
            OracleError::PriceStale { .. } => ORACLE_ERROR_BASE + 1,
            OracleError::InvalidPriceData { .. } => ORACLE_ERROR_BASE + 2,
            OracleError::PriceDeviationTooLarge { .. } => ORACLE_ERROR_BASE + 3,
            OracleError::OracleUnavailable { .. } => ORACLE_ERROR_BASE + 4,
            OracleError::AggregationFailed { .. } => ORACLE_ERROR_BASE + 5,
            OracleError::InvalidWeights { .. } => ORACLE_ERROR_BASE + 6,
            OracleError::InsufficientOracles { .. } => ORACLE_ERROR_BASE + 7,
            OracleError::OracleTimeout { .. } => ORACLE_ERROR_BASE + 8,
            OracleError::DataFormatError { .. } => ORACLE_ERROR_BASE + 9,
            OracleError::PermissionError { .. } => ORACLE_ERROR_BASE + 10,
            OracleError::ConfigurationError { .. } => ORACLE_ERROR_BASE + 11,
        }
    }
    
    fn error_message(&self) -> String {
        self.to_string()
    }
    
    fn is_recoverable(&self) -> bool {
        matches!(
            self,
            OracleError::OracleTimeout { .. } | 
            OracleError::OracleUnavailable { .. }
        )
    }
    
    fn retry_after(&self) -> Option<u64> {
        match self {
            OracleError::OracleTimeout { .. } => Some(5), // 5秒后重试
            OracleError::OracleUnavailable { .. } => Some(30), // 30秒后重试
            _ => None,
        }
    }
}

impl From<OracleError> for ProgramError {
    fn from(err: OracleError) -> Self {
        ProgramError::Oracle(err)
    }
}

impl From<OracleError> for Error {
    fn from(err: OracleError) -> Self {
        Error::from(err)
    }
}

/// Oracle错误扩展方法
impl OracleError {
    /// 检查是否为可恢复错误
    pub fn can_retry(&self) -> bool {
        self.is_recoverable()
    }
    
    /// 获取错误严重程度
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            OracleError::PriceStale { .. } => ErrorSeverity::Warning,
            OracleError::OracleTimeout { .. } => ErrorSeverity::Warning,
            OracleError::OracleUnavailable { .. } => ErrorSeverity::Warning,
            OracleError::InvalidPriceData { .. } => ErrorSeverity::Critical,
            OracleError::PriceDeviationTooLarge { .. } => ErrorSeverity::Critical,
            OracleError::AggregationFailed { .. } => ErrorSeverity::Critical,
            OracleError::InvalidWeights { .. } => ErrorSeverity::Critical,
            OracleError::InsufficientOracles { .. } => ErrorSeverity::Critical,
            OracleError::DataFormatError { .. } => ErrorSeverity::Critical,
            OracleError::PermissionError { .. } => ErrorSeverity::Critical,
            OracleError::ConfigurationError { .. } => ErrorSeverity::Critical,
        }
    }
    
    /// 获取错误分类
    pub fn category(&self) -> &'static str {
        match self {
            OracleError::PriceStale { .. } => "DataQuality",
            OracleError::InvalidPriceData { .. } => "DataQuality",
            OracleError::PriceDeviationTooLarge { .. } => "DataQuality",
            OracleError::OracleUnavailable { .. } => "Availability",
            OracleError::AggregationFailed { .. } => "Processing",
            OracleError::InvalidWeights { .. } => "Configuration",
            OracleError::InsufficientOracles { .. } => "Configuration",
            OracleError::OracleTimeout { .. } => "Availability",
            OracleError::DataFormatError { .. } => "DataQuality",
            OracleError::PermissionError { .. } => "Security",
            OracleError::ConfigurationError { .. } => "Configuration",
        }
    }
}

/// 错误严重程度
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    /// 警告级别
    Warning,
    /// 错误级别
    Error,
    /// 严重级别
    Critical,
}

/// Oracle错误统计
#[derive(Debug, Default)]
pub struct OracleErrorStats {
    /// 总错误数
    pub total_errors: u64,
    /// 可恢复错误数
    pub recoverable_errors: u64,
    /// 严重错误数
    pub critical_errors: u64,
    /// 按预言机分组的错误数
    pub errors_by_oracle: std::collections::HashMap<String, u64>,
    /// 按类型分组的错误数
    pub errors_by_type: std::collections::HashMap<String, u64>,
}

impl OracleErrorStats {
    /// 记录错误
    pub fn record_error(&mut self, error: &OracleError) {
        self.total_errors += 1;
        
        if error.is_recoverable() {
            self.recoverable_errors += 1;
        }
        
        if error.severity() == ErrorSeverity::Critical {
            self.critical_errors += 1;
        }
        
        // 按预言机分组
        if let Some(oracle_name) = self.extract_oracle_name(error) {
            *self.errors_by_oracle.entry(oracle_name).or_insert(0) += 1;
        }
        
        // 按类型分组
        let error_type = error.category();
        *self.errors_by_type.entry(error_type.to_string()).or_insert(0) += 1;
    }
    
    /// 提取预言机名称
    fn extract_oracle_name(&self, error: &OracleError) -> Option<String> {
        match error {
            OracleError::OracleUnavailable { oracle_name } => Some(oracle_name.clone()),
            OracleError::OracleTimeout { oracle_name } => Some(oracle_name.clone()),
            OracleError::DataFormatError { oracle_name, .. } => Some(oracle_name.clone()),
            OracleError::PermissionError { oracle_name } => Some(oracle_name.clone()),
            _ => None,
        }
    }
    
    /// 获取错误率
    pub fn error_rate(&self) -> f64 {
        if self.total_errors == 0 {
            return 0.0;
        }
        self.critical_errors as f64 / self.total_errors as f64
    }
    
    /// 获取可恢复率
    pub fn recovery_rate(&self) -> f64 {
        if self.total_errors == 0 {
            return 0.0;
        }
        self.recoverable_errors as f64 / self.total_errors as f64
    }
    
    /// 重置统计
    pub fn reset(&mut self) {
        self.total_errors = 0;
        self.recoverable_errors = 0;
        self.critical_errors = 0;
        self.errors_by_oracle.clear();
        self.errors_by_type.clear();
    }
} 