//!
//! 算法错误类型模块
//!
//! 定义所有交易算法相关的错误类型，包括TWAP、VWAP、智能路由、风险管理等算法的错误处理。

use anchor_lang::prelude::*;
use thiserror::Error;
use crate::errors::{ErrorConvertible, error_codes::ALGORITHM_ERROR_BASE};

/// 算法相关错误类型
#[derive(Debug, Error, Clone, AnchorSerialize, AnchorDeserialize)]
pub enum AlgorithmError {
    /// 算法参数无效
    #[error("Invalid algorithm parameters: {reason}")]
    InvalidParameters {
        /// 无效原因
        reason: String,
    },
    
    /// 算法执行超时
    #[error("Algorithm execution timeout: {algorithm_name}")]
    ExecutionTimeout {
        /// 算法名称
        algorithm_name: String,
    },
    
    /// 算法资源不足
    #[error("Insufficient resources for algorithm: {algorithm_name}")]
    InsufficientResources {
        /// 算法名称
        algorithm_name: String,
    },
    
    /// 算法状态无效
    #[error("Invalid algorithm state: {current_state}")]
    InvalidState {
        /// 当前状态
        current_state: String,
    },
    
    /// 算法配置错误
    #[error("Algorithm configuration error: {reason}")]
    ConfigurationError {
        /// 错误原因
        reason: String,
    },
    
    /// 算法依赖缺失
    #[error("Missing algorithm dependency: {dependency}")]
    MissingDependency {
        /// 缺失的依赖
        dependency: String,
    },
    
    /// 算法版本不兼容
    #[error("Incompatible algorithm version: {current} vs {required}")]
    VersionIncompatible {
        /// 当前版本
        current: String,
        /// 要求版本
        required: String,
    },
    
    /// 算法执行失败
    #[error("Algorithm execution failed: {reason}")]
    ExecutionFailed {
        /// 失败原因
        reason: String,
    },
    
    /// 算法结果无效
    #[error("Invalid algorithm result: {reason}")]
    InvalidResult {
        /// 无效原因
        reason: String,
    },
    
    /// 算法优化失败
    #[error("Algorithm optimization failed: {reason}")]
    OptimizationFailed {
        /// 失败原因
        reason: String,
    },
    
    /// 算法收敛失败
    #[error("Algorithm convergence failed: {iterations} iterations")]
    ConvergenceFailed {
        /// 迭代次数
        iterations: u32,
    },
    
    /// 算法内存不足
    #[error("Algorithm out of memory: {required} > {available}")]
    OutOfMemory {
        /// 需要内存
        required: u64,
        /// 可用内存
        available: u64,
    },
    
    /// 算法计算错误
    #[error("Algorithm computation error: {reason}")]
    ComputationError {
        /// 错误原因
        reason: String,
    },
    
    /// 算法输入数据无效
    #[error("Invalid algorithm input data: {reason}")]
    InvalidInputData {
        /// 无效原因
        reason: String,
    },
    
    /// 算法输出数据无效
    #[error("Invalid algorithm output data: {reason}")]
    InvalidOutputData {
        /// 无效原因
        reason: String,
    },
    
    /// 算法权限错误
    #[error("Algorithm permission error: {reason}")]
    PermissionError {
        /// 错误原因
        reason: String,
    },
    
    /// 算法安全错误
    #[error("Algorithm security error: {reason}")]
    SecurityError {
        /// 错误原因
        reason: String,
    },
}

impl ErrorConvertible for AlgorithmError {
    fn error_code(&self) -> u32 {
        match self {
            AlgorithmError::InvalidParameters { .. } => ALGORITHM_ERROR_BASE + 1,
            AlgorithmError::ExecutionTimeout { .. } => ALGORITHM_ERROR_BASE + 2,
            AlgorithmError::InsufficientResources { .. } => ALGORITHM_ERROR_BASE + 3,
            AlgorithmError::InvalidState { .. } => ALGORITHM_ERROR_BASE + 4,
            AlgorithmError::ConfigurationError { .. } => ALGORITHM_ERROR_BASE + 5,
            AlgorithmError::MissingDependency { .. } => ALGORITHM_ERROR_BASE + 6,
            AlgorithmError::VersionIncompatible { .. } => ALGORITHM_ERROR_BASE + 7,
            AlgorithmError::ExecutionFailed { .. } => ALGORITHM_ERROR_BASE + 8,
            AlgorithmError::InvalidResult { .. } => ALGORITHM_ERROR_BASE + 9,
            AlgorithmError::OptimizationFailed { .. } => ALGORITHM_ERROR_BASE + 10,
            AlgorithmError::ConvergenceFailed { .. } => ALGORITHM_ERROR_BASE + 11,
            AlgorithmError::OutOfMemory { .. } => ALGORITHM_ERROR_BASE + 12,
            AlgorithmError::ComputationError { .. } => ALGORITHM_ERROR_BASE + 13,
            AlgorithmError::InvalidInputData { .. } => ALGORITHM_ERROR_BASE + 14,
            AlgorithmError::InvalidOutputData { .. } => ALGORITHM_ERROR_BASE + 15,
            AlgorithmError::PermissionError { .. } => ALGORITHM_ERROR_BASE + 16,
            AlgorithmError::SecurityError { .. } => ALGORITHM_ERROR_BASE + 17,
        }
    }
    
    fn error_message(&self) -> String {
        self.to_string()
    }
    
    fn is_recoverable(&self) -> bool {
        matches!(
            self,
            AlgorithmError::ExecutionTimeout { .. } |
            AlgorithmError::InsufficientResources { .. } |
            AlgorithmError::ConvergenceFailed { .. } |
            AlgorithmError::OutOfMemory { .. }
        )
    }
    
    fn retry_after(&self) -> Option<u64> {
        match self {
            AlgorithmError::ExecutionTimeout { .. } => Some(10), // 10秒后重试
            AlgorithmError::InsufficientResources { .. } => Some(60), // 60秒后重试
            AlgorithmError::ConvergenceFailed { .. } => Some(30), // 30秒后重试
            AlgorithmError::OutOfMemory { .. } => Some(120), // 120秒后重试
            _ => None,
        }
    }
}

impl From<AlgorithmError> for ProgramError {
    fn from(err: AlgorithmError) -> Self {
        ProgramError::Algorithm(err)
    }
}

impl From<AlgorithmError> for Error {
    fn from(err: AlgorithmError) -> Self {
        Error::from(err)
    }
}

/// 算法错误扩展方法
impl AlgorithmError {
    /// 检查是否为可恢复错误
    pub fn can_retry(&self) -> bool {
        self.is_recoverable()
    }
    
    /// 获取错误严重程度
    pub fn severity(&self) -> AlgorithmErrorSeverity {
        match self {
            AlgorithmError::ExecutionTimeout { .. } => AlgorithmErrorSeverity::Warning,
            AlgorithmError::InsufficientResources { .. } => AlgorithmErrorSeverity::Warning,
            AlgorithmError::ConvergenceFailed { .. } => AlgorithmErrorSeverity::Warning,
            AlgorithmError::OutOfMemory { .. } => AlgorithmErrorSeverity::Warning,
            AlgorithmError::InvalidParameters { .. } => AlgorithmErrorSeverity::Error,
            AlgorithmError::InvalidState { .. } => AlgorithmErrorSeverity::Error,
            AlgorithmError::ConfigurationError { .. } => AlgorithmErrorSeverity::Error,
            AlgorithmError::MissingDependency { .. } => AlgorithmErrorSeverity::Error,
            AlgorithmError::VersionIncompatible { .. } => AlgorithmErrorSeverity::Error,
            AlgorithmError::ExecutionFailed { .. } => AlgorithmErrorSeverity::Error,
            AlgorithmError::InvalidResult { .. } => AlgorithmErrorSeverity::Error,
            AlgorithmError::OptimizationFailed { .. } => AlgorithmErrorSeverity::Error,
            AlgorithmError::ComputationError { .. } => AlgorithmErrorSeverity::Error,
            AlgorithmError::InvalidInputData { .. } => AlgorithmErrorSeverity::Error,
            AlgorithmError::InvalidOutputData { .. } => AlgorithmErrorSeverity::Error,
            AlgorithmError::PermissionError { .. } => AlgorithmErrorSeverity::Critical,
            AlgorithmError::SecurityError { .. } => AlgorithmErrorSeverity::Critical,
        }
    }
    
    /// 获取错误分类
    pub fn category(&self) -> &'static str {
        match self {
            AlgorithmError::InvalidParameters { .. } => "Validation",
            AlgorithmError::ExecutionTimeout { .. } => "Performance",
            AlgorithmError::InsufficientResources { .. } => "Resource",
            AlgorithmError::InvalidState { .. } => "State",
            AlgorithmError::ConfigurationError { .. } => "Configuration",
            AlgorithmError::MissingDependency { .. } => "Dependency",
            AlgorithmError::VersionIncompatible { .. } => "Compatibility",
            AlgorithmError::ExecutionFailed { .. } => "Execution",
            AlgorithmError::InvalidResult { .. } => "Validation",
            AlgorithmError::OptimizationFailed { .. } => "Optimization",
            AlgorithmError::ConvergenceFailed { .. } => "Optimization",
            AlgorithmError::OutOfMemory { .. } => "Resource",
            AlgorithmError::ComputationError { .. } => "Computation",
            AlgorithmError::InvalidInputData { .. } => "Validation",
            AlgorithmError::InvalidOutputData { .. } => "Validation",
            AlgorithmError::PermissionError { .. } => "Security",
            AlgorithmError::SecurityError { .. } => "Security",
        }
    }
    
    /// 获取算法名称
    pub fn algorithm_name(&self) -> Option<&str> {
        match self {
            AlgorithmError::ExecutionTimeout { algorithm_name } => Some(algorithm_name),
            AlgorithmError::InsufficientResources { algorithm_name } => Some(algorithm_name),
            _ => None,
        }
    }
}

/// 算法错误严重程度
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlgorithmErrorSeverity {
    /// 警告级别
    Warning,
    /// 错误级别
    Error,
    /// 严重级别
    Critical,
}

/// 算法错误统计
#[derive(Debug, Default)]
pub struct AlgorithmErrorStats {
    /// 总错误数
    pub total_errors: u64,
    /// 可恢复错误数
    pub recoverable_errors: u64,
    /// 严重错误数
    pub critical_errors: u64,
    /// 按算法分组的错误数
    pub errors_by_algorithm: std::collections::HashMap<String, u64>,
    /// 按类型分组的错误数
    pub errors_by_type: std::collections::HashMap<String, u64>,
    /// 按严重程度分组的错误数
    pub errors_by_severity: std::collections::HashMap<AlgorithmErrorSeverity, u64>,
}

impl AlgorithmErrorStats {
    /// 记录错误
    pub fn record_error(&mut self, error: &AlgorithmError) {
        self.total_errors += 1;
        
        if error.is_recoverable() {
            self.recoverable_errors += 1;
        }
        
        if error.severity() == AlgorithmErrorSeverity::Critical {
            self.critical_errors += 1;
        }
        
        // 按算法分组
        if let Some(algorithm_name) = error.algorithm_name() {
            *self.errors_by_algorithm.entry(algorithm_name.to_string()).or_insert(0) += 1;
        }
        
        // 按类型分组
        let error_type = error.category();
        *self.errors_by_type.entry(error_type.to_string()).or_insert(0) += 1;
        
        // 按严重程度分组
        let severity = error.severity();
        *self.errors_by_severity.entry(severity).or_insert(0) += 1;
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
    
    /// 获取最常出错的算法
    pub fn most_error_prone_algorithm(&self) -> Option<(&String, &u64)> {
        self.errors_by_algorithm.iter().max_by_key(|(_, &count)| count)
    }
    
    /// 获取最常出错的错误类型
    pub fn most_common_error_type(&self) -> Option<(&String, &u64)> {
        self.errors_by_type.iter().max_by_key(|(_, &count)| count)
    }
    
    /// 重置统计
    pub fn reset(&mut self) {
        self.total_errors = 0;
        self.recoverable_errors = 0;
        self.critical_errors = 0;
        self.errors_by_algorithm.clear();
        self.errors_by_type.clear();
        self.errors_by_severity.clear();
    }
} 