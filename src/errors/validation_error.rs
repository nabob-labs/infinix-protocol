//!
//! 验证错误类型模块
//!
//! 定义所有数据验证相关的错误类型，包括输入验证、业务规则验证、格式验证等操作的错误处理。

use anchor_lang::prelude::*;
use thiserror::Error;
use crate::errors::{ErrorConvertible, error_codes::VALIDATION_ERROR_BASE};

/// 验证相关错误类型
#[derive(Debug, Error, Clone, AnchorSerialize, AnchorDeserialize)]
pub enum ValidationError {
    /// 输入参数无效
    #[error("Invalid input parameter: {parameter} - {reason}")]
    InvalidInputParameter {
        /// 参数名称
        parameter: String,
        /// 无效原因
        reason: String,
    },
    
    /// 数值范围无效
    #[error("Value out of range: {value} not in [{min}, {max}]")]
    ValueOutOfRange {
        /// 实际值
        value: f64,
        /// 最小值
        min: f64,
        /// 最大值
        max: f64,
    },
    
    /// 字符串长度无效
    #[error("String length invalid: {length} not in [{min}, {max}]")]
    StringLengthInvalid {
        /// 实际长度
        length: usize,
        /// 最小长度
        min: usize,
        /// 最大长度
        max: usize,
    },
    
    /// 格式无效
    #[error("Invalid format: {field} - {reason}")]
    InvalidFormat {
        /// 字段名称
        field: String,
        /// 无效原因
        reason: String,
    },
    
    /// 必填字段缺失
    #[error("Required field missing: {field}")]
    RequiredFieldMissing {
        /// 缺失字段
        field: String,
    },
    
    /// 字段类型不匹配
    #[error("Field type mismatch: {field} expected {expected}, got {actual}")]
    FieldTypeMismatch {
        /// 字段名称
        field: String,
        /// 期望类型
        expected: String,
        /// 实际类型
        actual: String,
    },
    
    /// 业务规则违反
    #[error("Business rule violation: {rule} - {reason}")]
    BusinessRuleViolation {
        /// 违反的规则
        rule: String,
        /// 违反原因
        reason: String,
    },
    
    /// 数据一致性错误
    #[error("Data consistency error: {reason}")]
    DataConsistencyError {
        /// 错误原因
        reason: String,
    },
    
    /// 权限验证失败
    #[error("Permission validation failed: {operation} - {reason}")]
    PermissionValidationFailed {
        /// 操作名称
        operation: String,
        /// 失败原因
        reason: String,
    },
    
    /// 状态验证失败
    #[error("State validation failed: {current_state} - {expected_state}")]
    StateValidationFailed {
        /// 当前状态
        current_state: String,
        /// 期望状态
        expected_state: String,
    },
    
    /// 时间验证失败
    #[error("Time validation failed: {reason}")]
    TimeValidationFailed {
        /// 失败原因
        reason: String,
    },
    
    /// 地址验证失败
    #[error("Address validation failed: {address} - {reason}")]
    AddressValidationFailed {
        /// 无效地址
        address: String,
        /// 失败原因
        reason: String,
    },
    
    /// 签名验证失败
    #[error("Signature validation failed: {reason}")]
    SignatureValidationFailed {
        /// 失败原因
        reason: String,
    },
    
    /// 哈希验证失败
    #[error("Hash validation failed: {expected} != {actual}")]
    HashValidationFailed {
        /// 期望哈希
        expected: String,
        /// 实际哈希
        actual: String,
    },
    
    /// 数组长度无效
    #[error("Array length invalid: {length} not in [{min}, {max}]")]
    ArrayLengthInvalid {
        /// 实际长度
        length: usize,
        /// 最小长度
        min: usize,
        /// 最大长度
        max: usize,
    },
    
    /// 枚举值无效
    #[error("Invalid enum value: {value} not in {valid_values:?}")]
    InvalidEnumValue {
        /// 无效值
        value: String,
        /// 有效值列表
        valid_values: Vec<String>,
    },
    
    /// 正则表达式匹配失败
    #[error("Regex pattern mismatch: {value} does not match {pattern}")]
    RegexPatternMismatch {
        /// 不匹配的值
        value: String,
        /// 正则表达式模式
        pattern: String,
    },
    
    /// 自定义验证失败
    #[error("Custom validation failed: {validator} - {reason}")]
    CustomValidationFailed {
        /// 验证器名称
        validator: String,
        /// 失败原因
        reason: String,
    },
}

impl ErrorConvertible for ValidationError {
    fn error_code(&self) -> u32 {
        match self {
            ValidationError::InvalidInputParameter { .. } => VALIDATION_ERROR_BASE + 1,
            ValidationError::ValueOutOfRange { .. } => VALIDATION_ERROR_BASE + 2,
            ValidationError::StringLengthInvalid { .. } => VALIDATION_ERROR_BASE + 3,
            ValidationError::InvalidFormat { .. } => VALIDATION_ERROR_BASE + 4,
            ValidationError::RequiredFieldMissing { .. } => VALIDATION_ERROR_BASE + 5,
            ValidationError::FieldTypeMismatch { .. } => VALIDATION_ERROR_BASE + 6,
            ValidationError::BusinessRuleViolation { .. } => VALIDATION_ERROR_BASE + 7,
            ValidationError::DataConsistencyError { .. } => VALIDATION_ERROR_BASE + 8,
            ValidationError::PermissionValidationFailed { .. } => VALIDATION_ERROR_BASE + 9,
            ValidationError::StateValidationFailed { .. } => VALIDATION_ERROR_BASE + 10,
            ValidationError::TimeValidationFailed { .. } => VALIDATION_ERROR_BASE + 11,
            ValidationError::AddressValidationFailed { .. } => VALIDATION_ERROR_BASE + 12,
            ValidationError::SignatureValidationFailed { .. } => VALIDATION_ERROR_BASE + 13,
            ValidationError::HashValidationFailed { .. } => VALIDATION_ERROR_BASE + 14,
            ValidationError::ArrayLengthInvalid { .. } => VALIDATION_ERROR_BASE + 15,
            ValidationError::InvalidEnumValue { .. } => VALIDATION_ERROR_BASE + 16,
            ValidationError::RegexPatternMismatch { .. } => VALIDATION_ERROR_BASE + 17,
            ValidationError::CustomValidationFailed { .. } => VALIDATION_ERROR_BASE + 18,
        }
    }
    
    fn error_message(&self) -> String {
        self.to_string()
    }
    
    fn is_recoverable(&self) -> bool {
        matches!(
            self,
            ValidationError::StringLengthInvalid { .. } |
            ValidationError::ArrayLengthInvalid { .. } |
            ValidationError::TimeValidationFailed { .. }
        )
    }
    
    fn retry_after(&self) -> Option<u64> {
        match self {
            ValidationError::TimeValidationFailed { .. } => Some(1), // 1秒后重试
            _ => None,
        }
    }
}

impl From<ValidationError> for ProgramError {
    fn from(err: ValidationError) -> Self {
        ProgramError::Validation(err)
    }
}

impl From<ValidationError> for Error {
    fn from(err: ValidationError) -> Self {
        Error::from(err)
    }
}

/// 验证错误扩展方法
impl ValidationError {
    /// 检查是否为可恢复错误
    pub fn can_retry(&self) -> bool {
        self.is_recoverable()
    }
    
    /// 获取错误严重程度
    pub fn severity(&self) -> ValidationErrorSeverity {
        match self {
            ValidationError::StringLengthInvalid { .. } => ValidationErrorSeverity::Warning,
            ValidationError::ArrayLengthInvalid { .. } => ValidationErrorSeverity::Warning,
            ValidationError::TimeValidationFailed { .. } => ValidationErrorSeverity::Warning,
            ValidationError::InvalidInputParameter { .. } => ValidationErrorSeverity::Error,
            ValidationError::ValueOutOfRange { .. } => ValidationErrorSeverity::Error,
            ValidationError::InvalidFormat { .. } => ValidationErrorSeverity::Error,
            ValidationError::RequiredFieldMissing { .. } => ValidationErrorSeverity::Error,
            ValidationError::FieldTypeMismatch { .. } => ValidationErrorSeverity::Error,
            ValidationError::BusinessRuleViolation { .. } => ValidationErrorSeverity::Error,
            ValidationError::DataConsistencyError { .. } => ValidationErrorSeverity::Error,
            ValidationError::StateValidationFailed { .. } => ValidationErrorSeverity::Error,
            ValidationError::AddressValidationFailed { .. } => ValidationErrorSeverity::Error,
            ValidationError::InvalidEnumValue { .. } => ValidationErrorSeverity::Error,
            ValidationError::RegexPatternMismatch { .. } => ValidationErrorSeverity::Error,
            ValidationError::CustomValidationFailed { .. } => ValidationErrorSeverity::Error,
            ValidationError::PermissionValidationFailed { .. } => ValidationErrorSeverity::Critical,
            ValidationError::SignatureValidationFailed { .. } => ValidationErrorSeverity::Critical,
            ValidationError::HashValidationFailed { .. } => ValidationErrorSeverity::Critical,
        }
    }
    
    /// 获取错误分类
    pub fn category(&self) -> &'static str {
        match self {
            ValidationError::InvalidInputParameter { .. } => "Input",
            ValidationError::ValueOutOfRange { .. } => "Range",
            ValidationError::StringLengthInvalid { .. } => "Length",
            ValidationError::InvalidFormat { .. } => "Format",
            ValidationError::RequiredFieldMissing { .. } => "Required",
            ValidationError::FieldTypeMismatch { .. } => "Type",
            ValidationError::BusinessRuleViolation { .. } => "Business",
            ValidationError::DataConsistencyError { .. } => "Consistency",
            ValidationError::PermissionValidationFailed { .. } => "Permission",
            ValidationError::StateValidationFailed { .. } => "State",
            ValidationError::TimeValidationFailed { .. } => "Time",
            ValidationError::AddressValidationFailed { .. } => "Address",
            ValidationError::SignatureValidationFailed { .. } => "Signature",
            ValidationError::HashValidationFailed { .. } => "Hash",
            ValidationError::ArrayLengthInvalid { .. } => "Length",
            ValidationError::InvalidEnumValue { .. } => "Enum",
            ValidationError::RegexPatternMismatch { .. } => "Pattern",
            ValidationError::CustomValidationFailed { .. } => "Custom",
        }
    }
    
    /// 获取字段名称
    pub fn field_name(&self) -> Option<&str> {
        match self {
            ValidationError::InvalidInputParameter { parameter, .. } => Some(parameter),
            ValidationError::InvalidFormat { field, .. } => Some(field),
            ValidationError::RequiredFieldMissing { field } => Some(field),
            ValidationError::FieldTypeMismatch { field, .. } => Some(field),
            ValidationError::AddressValidationFailed { address, .. } => Some(address),
            _ => None,
        }
    }
    
    /// 检查是否为安全相关错误
    pub fn is_security_related(&self) -> bool {
        matches!(
            self,
            ValidationError::PermissionValidationFailed { .. } |
            ValidationError::SignatureValidationFailed { .. } |
            ValidationError::HashValidationFailed { .. }
        )
    }
    
    /// 检查是否为数据完整性错误
    pub fn is_data_integrity_related(&self) -> bool {
        matches!(
            self,
            ValidationError::DataConsistencyError { .. } |
            ValidationError::HashValidationFailed { .. } |
            ValidationError::SignatureValidationFailed { .. }
        )
    }
}

/// 验证错误严重程度
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationErrorSeverity {
    /// 警告级别
    Warning,
    /// 错误级别
    Error,
    /// 严重级别
    Critical,
}

/// 验证错误统计
#[derive(Debug, Default)]
pub struct ValidationErrorStats {
    /// 总错误数
    pub total_errors: u64,
    /// 可恢复错误数
    pub recoverable_errors: u64,
    /// 严重错误数
    pub critical_errors: u64,
    /// 按字段分组的错误数
    pub errors_by_field: std::collections::HashMap<String, u64>,
    /// 按类型分组的错误数
    pub errors_by_type: std::collections::HashMap<String, u64>,
    /// 按严重程度分组的错误数
    pub errors_by_severity: std::collections::HashMap<ValidationErrorSeverity, u64>,
    /// 安全相关错误数
    pub security_related_errors: u64,
    /// 数据完整性错误数
    pub data_integrity_errors: u64,
}

impl ValidationErrorStats {
    /// 记录错误
    pub fn record_error(&mut self, error: &ValidationError) {
        self.total_errors += 1;
        
        if error.is_recoverable() {
            self.recoverable_errors += 1;
        }
        
        if error.severity() == ValidationErrorSeverity::Critical {
            self.critical_errors += 1;
        }
        
        if error.is_security_related() {
            self.security_related_errors += 1;
        }
        
        if error.is_data_integrity_related() {
            self.data_integrity_errors += 1;
        }
        
        // 按字段分组
        if let Some(field_name) = error.field_name() {
            *self.errors_by_field.entry(field_name.to_string()).or_insert(0) += 1;
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
    
    /// 获取安全错误率
    pub fn security_error_rate(&self) -> f64 {
        if self.total_errors == 0 {
            return 0.0;
        }
        self.security_related_errors as f64 / self.total_errors as f64
    }
    
    /// 获取数据完整性错误率
    pub fn data_integrity_error_rate(&self) -> f64 {
        if self.total_errors == 0 {
            return 0.0;
        }
        self.data_integrity_errors as f64 / self.total_errors as f64
    }
    
    /// 获取最常出错的字段
    pub fn most_error_prone_field(&self) -> Option<(&String, &u64)> {
        self.errors_by_field.iter().max_by_key(|(_, &count)| count)
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
        self.errors_by_field.clear();
        self.errors_by_type.clear();
        self.errors_by_severity.clear();
        self.security_related_errors = 0;
        self.data_integrity_errors = 0;
    }
} 