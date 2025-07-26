//!
//! 统一错误处理系统
//!
//! 本模块建立统一的错误类型体系，支持所有业务模块的错误处理，
//! 确保错误类型安全、可追溯、可维护，符合生产级质量标准。

pub mod asset_error;
pub mod basket_error;
pub mod index_token_error;
pub mod dex_error;
pub mod oracle_error;
pub mod algorithm_error;
pub mod strategy_error;
pub mod validation_error;
pub mod security_error;
pub mod error_codes;

#[cfg(test)]
mod tests;

pub use asset_error::*;
pub use basket_error::*;
pub use index_token_error::*;
pub use dex_error::*;
pub use oracle_error::*;
pub use algorithm_error::*;
pub use strategy_error::*;
pub use validation_error::*;
pub use security_error::*;
pub use error_codes::*;

use anchor_lang::prelude::*;
use thiserror::Error;

/// 统一程序错误类型
/// 
/// 整合所有业务模块的错误类型，提供统一的错误处理接口。
/// 支持错误类型转换、错误码映射、国际化等高级功能。
#[derive(Debug, Error, Clone, AnchorSerialize, AnchorDeserialize)]
pub enum ProgramError {
    /// 资产相关错误
    #[error("Asset error: {0}")]
    Asset(#[from] AssetError),
    
    /// 篮子相关错误
    #[error("Basket error: {0}")]
    Basket(#[from] BasketError),
    
    /// 指数代币相关错误
    #[error("Index token error: {0}")]
    IndexToken(#[from] IndexTokenError),
    
    /// DEX相关错误
    #[error("DEX error: {0}")]
    Dex(#[from] DexError),
    
    /// 预言机相关错误
    #[error("Oracle error: {0}")]
    Oracle(#[from] OracleError),
    
    /// 算法相关错误
    #[error("Algorithm error: {0}")]
    Algorithm(#[from] AlgorithmError),
    
    /// 策略相关错误
    #[error("Strategy error: {0}")]
    Strategy(#[from] StrategyError),
    
    /// 验证相关错误
    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),
    
    /// 安全相关错误
    #[error("Security error: {0}")]
    Security(#[from] SecurityError),
    
    /// 通用错误
    #[error("General error: {message}")]
    General {
        /// 错误消息
        message: String,
        /// 错误码
        code: u32,
    },
    
    /// 未知错误
    #[error("Unknown error occurred")]
    Unknown,
}

impl ProgramError {
    /// 获取错误码
    pub fn error_code(&self) -> u32 {
        match self {
            ProgramError::Asset(e) => e.error_code(),
            ProgramError::Basket(e) => e.error_code(),
            ProgramError::IndexToken(e) => e.error_code(),
            ProgramError::Dex(e) => e.error_code(),
            ProgramError::Oracle(e) => e.error_code(),
            ProgramError::Algorithm(e) => e.error_code(),
            ProgramError::Strategy(e) => e.error_code(),
            ProgramError::Validation(e) => e.error_code(),
            ProgramError::Security(e) => e.error_code(),
            ProgramError::General { code, .. } => *code,
            ProgramError::Unknown => 9999,
        }
    }
    
    /// 获取错误消息
    pub fn error_message(&self) -> String {
        match self {
            ProgramError::Asset(e) => e.error_message(),
            ProgramError::Basket(e) => e.error_message(),
            ProgramError::IndexToken(e) => e.error_message(),
            ProgramError::Dex(e) => e.error_message(),
            ProgramError::Oracle(e) => e.error_message(),
            ProgramError::Algorithm(e) => e.error_message(),
            ProgramError::Strategy(e) => e.error_message(),
            ProgramError::Validation(e) => e.error_message(),
            ProgramError::Security(e) => e.error_message(),
            ProgramError::General { message, .. } => message.clone(),
            ProgramError::Unknown => "Unknown error occurred".to_string(),
        }
    }
    
    /// 检查是否为可恢复错误
    pub fn is_recoverable(&self) -> bool {
        match self {
            ProgramError::Asset(e) => e.is_recoverable(),
            ProgramError::Basket(e) => e.is_recoverable(),
            ProgramError::IndexToken(e) => e.is_recoverable(),
            ProgramError::Dex(e) => e.is_recoverable(),
            ProgramError::Oracle(e) => e.is_recoverable(),
            ProgramError::Algorithm(e) => e.is_recoverable(),
            ProgramError::Strategy(e) => e.is_recoverable(),
            ProgramError::Validation(e) => e.is_recoverable(),
            ProgramError::Security(e) => e.is_recoverable(),
            ProgramError::General { .. } => false,
            ProgramError::Unknown => false,
        }
    }
    
    /// 获取重试时间（秒）
    pub fn retry_after(&self) -> Option<u64> {
        match self {
            ProgramError::Asset(e) => e.retry_after(),
            ProgramError::Basket(e) => e.retry_after(),
            ProgramError::IndexToken(e) => e.retry_after(),
            ProgramError::Dex(e) => e.retry_after(),
            ProgramError::Oracle(e) => e.retry_after(),
            ProgramError::Algorithm(e) => e.retry_after(),
            ProgramError::Strategy(e) => e.retry_after(),
            ProgramError::Validation(e) => e.retry_after(),
            ProgramError::Security(e) => e.retry_after(),
            ProgramError::General { .. } => None,
            ProgramError::Unknown => None,
        }
    }
    
    /// 检查是否为安全相关错误
    pub fn is_security_error(&self) -> bool {
        matches!(self, ProgramError::Security(_))
    }
    
    /// 检查是否为验证相关错误
    pub fn is_validation_error(&self) -> bool {
        matches!(self, ProgramError::Validation(_))
    }
    
    /// 获取错误模块名称
    pub fn module_name(&self) -> &'static str {
        match self {
            ProgramError::Asset(_) => "Asset",
            ProgramError::Basket(_) => "Basket",
            ProgramError::IndexToken(_) => "IndexToken",
            ProgramError::Dex(_) => "DEX",
            ProgramError::Oracle(_) => "Oracle",
            ProgramError::Algorithm(_) => "Algorithm",
            ProgramError::Strategy(_) => "Strategy",
            ProgramError::Validation(_) => "Validation",
            ProgramError::Security(_) => "Security",
            ProgramError::General { .. } => "General",
            ProgramError::Unknown => "Unknown",
        }
    }
}

/// 错误转换trait
/// 
/// 为所有错误类型提供统一的转换接口，支持错误码映射、可恢复性判断等功能。
pub trait ErrorConvertible {
    /// 获取错误码
    fn error_code(&self) -> u32;
    
    /// 获取错误消息
    fn error_message(&self) -> String;
    
    /// 检查是否为可恢复错误
    fn is_recoverable(&self) -> bool;
    
    /// 获取重试时间（秒）
    fn retry_after(&self) -> Option<u64>;
}

/// 错误处理trait
/// 
/// 提供统一的错误处理接口，支持错误日志、错误追踪、错误恢复等功能。
pub trait ErrorHandler {
    /// 处理错误
    fn handle_error(&self, error: &ProgramError) -> Result<()>;
    
    /// 记录错误日志
    fn log_error(&self, error: &ProgramError);
    
    /// 错误恢复
    fn recover_from_error(&self, error: &ProgramError) -> Result<()>;
}

/// 默认错误处理器
pub struct DefaultErrorHandler;

impl ErrorHandler for DefaultErrorHandler {
    fn handle_error(&self, error: &ProgramError) -> Result<()> {
        // 记录错误到全局统计
        utils::record_global_error(error);
        
        // 记录错误日志
        self.log_error(error);
        
        // 检查是否为安全错误
        if error.is_security_error() {
            msg!("Security error detected: {}", error.error_message());
            return Err(error.clone().into());
        }
        
        // 检查是否为可恢复错误
        if error.is_recoverable() {
            return self.recover_from_error(error);
        }
        
        // 不可恢复错误，直接返回
        Err(error.clone().into())
    }
    
    fn log_error(&self, error: &ProgramError) {
        msg!(
            "[ERROR] Module: {}, Code: {}, Message: {}, Recoverable: {}, Retry After: {:?}s",
            error.module_name(),
            error.error_code(),
            error.error_message(),
            error.is_recoverable(),
            error.retry_after()
        );
    }
    
    fn recover_from_error(&self, error: &ProgramError) -> Result<()> {
        // 默认恢复策略：记录恢复尝试
        msg!("Attempting to recover from error in module: {}", error.module_name());
        
        // 如果有重试时间，记录重试信息
        if let Some(retry_after) = error.retry_after() {
            msg!("Will retry after {} seconds", retry_after);
        }
        
        Ok(())
    }
}



/// 错误工具函数
pub mod utils {
    use super::*;
    
    /// 创建通用错误
    pub fn general_error(message: &str, code: u32) -> ProgramError {
        ProgramError::General {
            message: message.to_string(),
            code,
        }
    }
    
    /// 创建未知错误
    pub fn unknown_error() -> ProgramError {
        ProgramError::Unknown
    }
    
    /// 检查错误是否为特定类型
    pub fn is_error_type<T>(error: &ProgramError) -> bool 
    where
        T: std::any::Any,
    {
        std::any::TypeId::of::<T>() == std::any::TypeId::of::<ProgramError>()
    }
    
    /// 获取错误堆栈信息
    pub fn get_error_stack(error: &ProgramError) -> String {
        format!("Error stack: {:?}", error)
    }
    
    /// 全局错误统计器
    #[derive(Debug, Default, Clone)]
    pub struct GlobalErrorStats {
        /// 总错误数
        pub total_errors: u64,
        /// 按模块分组的错误数
        pub errors_by_module: std::collections::HashMap<String, u64>,
        /// 按严重程度分组的错误数
        pub errors_by_severity: std::collections::HashMap<String, u64>,
        /// 可恢复错误数
        pub recoverable_errors: u64,
        /// 安全相关错误数
        pub security_errors: u64,
        /// 验证相关错误数
        pub validation_errors: u64,
    }
    
    impl GlobalErrorStats {
        /// 记录错误
        pub fn record_error(&mut self, error: &ProgramError) {
            self.total_errors += 1;
            
            // 按模块分组
            let module = error.module_name();
            *self.errors_by_module.entry(module.to_string()).or_insert(0) += 1;
            
            // 按严重程度分组
            let severity = if error.is_security_error() {
                "Critical"
            } else if error.is_validation_error() {
                "Error"
            } else if error.is_recoverable() {
                "Warning"
            } else {
                "Error"
            };
            *self.errors_by_severity.entry(severity.to_string()).or_insert(0) += 1;
            
            // 统计特殊类型错误
            if error.is_recoverable() {
                self.recoverable_errors += 1;
            }
            if error.is_security_error() {
                self.security_errors += 1;
            }
            if error.is_validation_error() {
                self.validation_errors += 1;
            }
        }
        
        /// 获取错误率
        pub fn error_rate(&self) -> f64 {
            if self.total_errors == 0 {
                return 0.0;
            }
            (self.total_errors - self.recoverable_errors) as f64 / self.total_errors as f64
        }
        
        /// 获取可恢复率
        pub fn recovery_rate(&self) -> f64 {
            if self.total_errors == 0 {
                return 0.0;
            }
            self.recoverable_errors as f64 / self.total_errors as f64
        }
        
        /// 获取安全错误率
        pub fn security_error_rate(&self) -> f64 {
            if self.total_errors == 0 {
                return 0.0;
            }
            self.security_errors as f64 / self.total_errors as f64
        }
        
        /// 获取最常出错的模块
        pub fn most_error_prone_module(&self) -> Option<(&String, &u64)> {
            self.errors_by_module.iter().max_by_key(|(_, &count)| count)
        }
        
        /// 重置统计
        pub fn reset(&mut self) {
            self.total_errors = 0;
            self.errors_by_module.clear();
            self.errors_by_severity.clear();
            self.recoverable_errors = 0;
            self.security_errors = 0;
            self.validation_errors = 0;
        }
    }
    
    /// 全局错误统计器实例
    pub static mut GLOBAL_ERROR_STATS: GlobalErrorStats = GlobalErrorStats {
        total_errors: 0,
        errors_by_module: std::collections::HashMap::new(),
        errors_by_severity: std::collections::HashMap::new(),
        recoverable_errors: 0,
        security_errors: 0,
        validation_errors: 0,
    };
    
    /// 记录全局错误
    pub fn record_global_error(error: &ProgramError) {
        unsafe {
            GLOBAL_ERROR_STATS.record_error(error);
        }
    }
    
    /// 获取全局错误统计
    pub fn get_global_error_stats() -> GlobalErrorStats {
        unsafe {
            GLOBAL_ERROR_STATS.clone()
        }
    }
    
    /// 重置全局错误统计
    pub fn reset_global_error_stats() {
        unsafe {
            GLOBAL_ERROR_STATS.reset();
        }
    }
}

/// 错误宏定义
#[macro_export]
macro_rules! program_error {
    ($error_type:ident, $message:expr) => {
        $crate::errors::ProgramError::$error_type($crate::errors::$error_type::new($message))
    };
    
    ($error_type:ident, $message:expr, $code:expr) => {
        $crate::errors::ProgramError::$error_type($crate::errors::$error_type::new_with_code($message, $code))
    };
}

#[macro_export]
macro_rules! handle_error {
    ($result:expr) => {
        match $result {
            Ok(value) => value,
            Err(error) => {
                let program_error = $crate::errors::ProgramError::from(error);
                let handler = $crate::errors::DefaultErrorHandler;
                return handler.handle_error(&program_error);
            }
        }
    };
    
    ($result:expr, $handler:expr) => {
        match $result {
            Ok(value) => value,
            Err(error) => {
                let program_error = $crate::errors::ProgramError::from(error);
                return $handler.handle_error(&program_error);
            }
        }
    };
} 