//!
//! 策略错误类型模块
//!
//! 定义所有交易策略相关的错误类型，包括权重策略、再平衡策略、风险管理策略等操作的错误处理。

use anchor_lang::prelude::*;
use thiserror::Error;
use crate::errors::{ErrorConvertible, error_codes::STRATEGY_ERROR_BASE};

/// 策略相关错误类型
#[derive(Debug, Error, Clone, AnchorSerialize, AnchorDeserialize)]
pub enum StrategyError {
    /// 策略参数无效
    #[error("Invalid strategy parameters: {reason}")]
    InvalidParameters {
        /// 无效原因
        reason: String,
    },
    
    /// 策略配置错误
    #[error("Strategy configuration error: {reason}")]
    ConfigurationError {
        /// 错误原因
        reason: String,
    },
    
    /// 策略状态无效
    #[error("Invalid strategy state: {current_state}")]
    InvalidState {
        /// 当前状态
        current_state: String,
    },
    
    /// 策略执行失败
    #[error("Strategy execution failed: {reason}")]
    ExecutionFailed {
        /// 失败原因
        reason: String,
    },
    
    /// 策略验证失败
    #[error("Strategy validation failed: {reason}")]
    ValidationFailed {
        /// 失败原因
        reason: String,
    },
    
    /// 策略依赖缺失
    #[error("Missing strategy dependency: {dependency}")]
    MissingDependency {
        /// 缺失的依赖
        dependency: String,
    },
    
    /// 策略版本不兼容
    #[error("Incompatible strategy version: {current} vs {required}")]
    VersionIncompatible {
        /// 当前版本
        current: String,
        /// 要求版本
        required: String,
    },
    
    /// 策略资源不足
    #[error("Insufficient resources for strategy: {strategy_name}")]
    InsufficientResources {
        /// 策略名称
        strategy_name: String,
    },
    
    /// 策略执行超时
    #[error("Strategy execution timeout: {strategy_name}")]
    ExecutionTimeout {
        /// 策略名称
        strategy_name: String,
    },
    
    /// 策略结果无效
    #[error("Invalid strategy result: {reason}")]
    InvalidResult {
        /// 无效原因
        reason: String,
    },
    
    /// 策略优化失败
    #[error("Strategy optimization failed: {reason}")]
    OptimizationFailed {
        /// 失败原因
        reason: String,
    },
    
    /// 策略风险超限
    #[error("Strategy risk limit exceeded: {risk_type} = {value} > {limit}")]
    RiskLimitExceeded {
        /// 风险类型
        risk_type: String,
        /// 实际值
        value: f64,
        /// 限制值
        limit: f64,
    },
    
    /// 策略权重无效
    #[error("Invalid strategy weights: {reason}")]
    InvalidWeights {
        /// 无效原因
        reason: String,
    },
    
    /// 策略再平衡失败
    #[error("Strategy rebalancing failed: {reason}")]
    RebalancingFailed {
        /// 失败原因
        reason: String,
    },
    
    /// 策略对冲失败
    #[error("Strategy hedging failed: {reason}")]
    HedgingFailed {
        /// 失败原因
        reason: String,
    },
    
    /// 策略套利失败
    #[error("Strategy arbitrage failed: {reason}")]
    ArbitrageFailed {
        /// 失败原因
        reason: String,
    },
    
    /// 策略流动性不足
    #[error("Insufficient liquidity for strategy: {strategy_name}")]
    InsufficientLiquidity {
        /// 策略名称
        strategy_name: String,
    },
    
    /// 策略市场条件不满足
    #[error("Market conditions not met for strategy: {strategy_name}")]
    MarketConditionsNotMet {
        /// 策略名称
        strategy_name: String,
    },
    
    /// 策略权限错误
    #[error("Strategy permission error: {reason}")]
    PermissionError {
        /// 错误原因
        reason: String,
    },
    
    /// 策略安全错误
    #[error("Strategy security error: {reason}")]
    SecurityError {
        /// 错误原因
        reason: String,
    },
    
    /// 策略性能错误
    #[error("Strategy performance error: {reason}")]
    PerformanceError {
        /// 错误原因
        reason: String,
    },
}

impl ErrorConvertible for StrategyError {
    fn error_code(&self) -> u32 {
        match self {
            StrategyError::InvalidParameters { .. } => STRATEGY_ERROR_BASE + 1,
            StrategyError::ConfigurationError { .. } => STRATEGY_ERROR_BASE + 2,
            StrategyError::InvalidState { .. } => STRATEGY_ERROR_BASE + 3,
            StrategyError::ExecutionFailed { .. } => STRATEGY_ERROR_BASE + 4,
            StrategyError::ValidationFailed { .. } => STRATEGY_ERROR_BASE + 5,
            StrategyError::MissingDependency { .. } => STRATEGY_ERROR_BASE + 6,
            StrategyError::VersionIncompatible { .. } => STRATEGY_ERROR_BASE + 7,
            StrategyError::InsufficientResources { .. } => STRATEGY_ERROR_BASE + 8,
            StrategyError::ExecutionTimeout { .. } => STRATEGY_ERROR_BASE + 9,
            StrategyError::InvalidResult { .. } => STRATEGY_ERROR_BASE + 10,
            StrategyError::OptimizationFailed { .. } => STRATEGY_ERROR_BASE + 11,
            StrategyError::RiskLimitExceeded { .. } => STRATEGY_ERROR_BASE + 12,
            StrategyError::InvalidWeights { .. } => STRATEGY_ERROR_BASE + 13,
            StrategyError::RebalancingFailed { .. } => STRATEGY_ERROR_BASE + 14,
            StrategyError::HedgingFailed { .. } => STRATEGY_ERROR_BASE + 15,
            StrategyError::ArbitrageFailed { .. } => STRATEGY_ERROR_BASE + 16,
            StrategyError::InsufficientLiquidity { .. } => STRATEGY_ERROR_BASE + 17,
            StrategyError::MarketConditionsNotMet { .. } => STRATEGY_ERROR_BASE + 18,
            StrategyError::PermissionError { .. } => STRATEGY_ERROR_BASE + 19,
            StrategyError::SecurityError { .. } => STRATEGY_ERROR_BASE + 20,
            StrategyError::PerformanceError { .. } => STRATEGY_ERROR_BASE + 21,
        }
    }
    
    fn error_message(&self) -> String {
        self.to_string()
    }
    
    fn is_recoverable(&self) -> bool {
        matches!(
            self,
            StrategyError::ExecutionTimeout { .. } |
            StrategyError::InsufficientResources { .. } |
            StrategyError::InsufficientLiquidity { .. } |
            StrategyError::MarketConditionsNotMet { .. }
        )
    }
    
    fn retry_after(&self) -> Option<u64> {
        match self {
            StrategyError::ExecutionTimeout { .. } => Some(15), // 15秒后重试
            StrategyError::InsufficientResources { .. } => Some(60), // 60秒后重试
            StrategyError::InsufficientLiquidity { .. } => Some(30), // 30秒后重试
            StrategyError::MarketConditionsNotMet { .. } => Some(300), // 5分钟后重试
            _ => None,
        }
    }
}

impl From<StrategyError> for ProgramError {
    fn from(err: StrategyError) -> Self {
        ProgramError::Strategy(err)
    }
}

impl From<StrategyError> for Error {
    fn from(err: StrategyError) -> Self {
        Error::from(err)
    }
}

/// 策略错误扩展方法
impl StrategyError {
    /// 检查是否为可恢复错误
    pub fn can_retry(&self) -> bool {
        self.is_recoverable()
    }
    
    /// 获取错误严重程度
    pub fn severity(&self) -> StrategyErrorSeverity {
        match self {
            StrategyError::ExecutionTimeout { .. } => StrategyErrorSeverity::Warning,
            StrategyError::InsufficientResources { .. } => StrategyErrorSeverity::Warning,
            StrategyError::InsufficientLiquidity { .. } => StrategyErrorSeverity::Warning,
            StrategyError::MarketConditionsNotMet { .. } => StrategyErrorSeverity::Warning,
            StrategyError::InvalidParameters { .. } => StrategyErrorSeverity::Error,
            StrategyError::ConfigurationError { .. } => StrategyErrorSeverity::Error,
            StrategyError::InvalidState { .. } => StrategyErrorSeverity::Error,
            StrategyError::ValidationFailed { .. } => StrategyErrorSeverity::Error,
            StrategyError::MissingDependency { .. } => StrategyErrorSeverity::Error,
            StrategyError::VersionIncompatible { .. } => StrategyErrorSeverity::Error,
            StrategyError::ExecutionFailed { .. } => StrategyErrorSeverity::Error,
            StrategyError::InvalidResult { .. } => StrategyErrorSeverity::Error,
            StrategyError::OptimizationFailed { .. } => StrategyErrorSeverity::Error,
            StrategyError::RiskLimitExceeded { .. } => StrategyErrorSeverity::Error,
            StrategyError::InvalidWeights { .. } => StrategyErrorSeverity::Error,
            StrategyError::RebalancingFailed { .. } => StrategyErrorSeverity::Error,
            StrategyError::HedgingFailed { .. } => StrategyErrorSeverity::Error,
            StrategyError::ArbitrageFailed { .. } => StrategyErrorSeverity::Error,
            StrategyError::PerformanceError { .. } => StrategyErrorSeverity::Error,
            StrategyError::PermissionError { .. } => StrategyErrorSeverity::Critical,
            StrategyError::SecurityError { .. } => StrategyErrorSeverity::Critical,
        }
    }
    
    /// 获取错误分类
    pub fn category(&self) -> &'static str {
        match self {
            StrategyError::InvalidParameters { .. } => "Validation",
            StrategyError::ConfigurationError { .. } => "Configuration",
            StrategyError::InvalidState { .. } => "State",
            StrategyError::ExecutionFailed { .. } => "Execution",
            StrategyError::ValidationFailed { .. } => "Validation",
            StrategyError::MissingDependency { .. } => "Dependency",
            StrategyError::VersionIncompatible { .. } => "Compatibility",
            StrategyError::InsufficientResources { .. } => "Resource",
            StrategyError::ExecutionTimeout { .. } => "Performance",
            StrategyError::InvalidResult { .. } => "Validation",
            StrategyError::OptimizationFailed { .. } => "Optimization",
            StrategyError::RiskLimitExceeded { .. } => "Risk",
            StrategyError::InvalidWeights { .. } => "Validation",
            StrategyError::RebalancingFailed { .. } => "Rebalancing",
            StrategyError::HedgingFailed { .. } => "Hedging",
            StrategyError::ArbitrageFailed { .. } => "Arbitrage",
            StrategyError::InsufficientLiquidity { .. } => "Liquidity",
            StrategyError::MarketConditionsNotMet { .. } => "Market",
            StrategyError::PermissionError { .. } => "Security",
            StrategyError::SecurityError { .. } => "Security",
            StrategyError::PerformanceError { .. } => "Performance",
        }
    }
    
    /// 获取策略名称
    pub fn strategy_name(&self) -> Option<&str> {
        match self {
            StrategyError::InsufficientResources { strategy_name } => Some(strategy_name),
            StrategyError::ExecutionTimeout { strategy_name } => Some(strategy_name),
            StrategyError::InsufficientLiquidity { strategy_name } => Some(strategy_name),
            StrategyError::MarketConditionsNotMet { strategy_name } => Some(strategy_name),
            _ => None,
        }
    }
    
    /// 检查是否为风险相关错误
    pub fn is_risk_related(&self) -> bool {
        matches!(
            self,
            StrategyError::RiskLimitExceeded { .. } |
            StrategyError::InsufficientLiquidity { .. }
        )
    }
    
    /// 检查是否为执行相关错误
    pub fn is_execution_related(&self) -> bool {
        matches!(
            self,
            StrategyError::ExecutionFailed { .. } |
            StrategyError::ExecutionTimeout { .. } |
            StrategyError::RebalancingFailed { .. } |
            StrategyError::HedgingFailed { .. } |
            StrategyError::ArbitrageFailed { .. }
        )
    }
}

/// 策略错误严重程度
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrategyErrorSeverity {
    /// 警告级别
    Warning,
    /// 错误级别
    Error,
    /// 严重级别
    Critical,
}

/// 策略错误统计
#[derive(Debug, Default)]
pub struct StrategyErrorStats {
    /// 总错误数
    pub total_errors: u64,
    /// 可恢复错误数
    pub recoverable_errors: u64,
    /// 严重错误数
    pub critical_errors: u64,
    /// 按策略分组的错误数
    pub errors_by_strategy: std::collections::HashMap<String, u64>,
    /// 按类型分组的错误数
    pub errors_by_type: std::collections::HashMap<String, u64>,
    /// 按严重程度分组的错误数
    pub errors_by_severity: std::collections::HashMap<StrategyErrorSeverity, u64>,
    /// 风险相关错误数
    pub risk_related_errors: u64,
    /// 执行相关错误数
    pub execution_related_errors: u64,
}

impl StrategyErrorStats {
    /// 记录错误
    pub fn record_error(&mut self, error: &StrategyError) {
        self.total_errors += 1;
        
        if error.is_recoverable() {
            self.recoverable_errors += 1;
        }
        
        if error.severity() == StrategyErrorSeverity::Critical {
            self.critical_errors += 1;
        }
        
        if error.is_risk_related() {
            self.risk_related_errors += 1;
        }
        
        if error.is_execution_related() {
            self.execution_related_errors += 1;
        }
        
        // 按策略分组
        if let Some(strategy_name) = error.strategy_name() {
            *self.errors_by_strategy.entry(strategy_name.to_string()).or_insert(0) += 1;
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
    
    /// 获取风险错误率
    pub fn risk_error_rate(&self) -> f64 {
        if self.total_errors == 0 {
            return 0.0;
        }
        self.risk_related_errors as f64 / self.total_errors as f64
    }
    
    /// 获取执行错误率
    pub fn execution_error_rate(&self) -> f64 {
        if self.total_errors == 0 {
            return 0.0;
        }
        self.execution_related_errors as f64 / self.total_errors as f64
    }
    
    /// 获取最常出错的策略
    pub fn most_error_prone_strategy(&self) -> Option<(&String, &u64)> {
        self.errors_by_strategy.iter().max_by_key(|(_, &count)| count)
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
        self.errors_by_strategy.clear();
        self.errors_by_type.clear();
        self.errors_by_severity.clear();
        self.risk_related_errors = 0;
        self.execution_related_errors = 0;
    }
} 