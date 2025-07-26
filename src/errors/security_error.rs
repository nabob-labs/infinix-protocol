//!
//! 安全错误类型模块
//!
//! 定义所有安全相关的错误类型，包括权限验证、访问控制、加密解密、签名验证等安全操作的错误处理。

use anchor_lang::prelude::*;
use thiserror::Error;
use crate::errors::{ErrorConvertible, error_codes::SECURITY_ERROR_BASE};

/// 安全相关错误类型
#[derive(Debug, Error, Clone, AnchorSerialize, AnchorDeserialize)]
pub enum SecurityError {
    /// 权限不足
    #[error("Insufficient permissions: {operation} requires {required_permission}")]
    InsufficientPermissions {
        /// 操作名称
        operation: String,
        /// 需要的权限
        required_permission: String,
    },
    
    /// 访问被拒绝
    #[error("Access denied: {resource} - {reason}")]
    AccessDenied {
        /// 被拒绝访问的资源
        resource: String,
        /// 拒绝原因
        reason: String,
    },
    
    /// 签名验证失败
    #[error("Signature verification failed: {reason}")]
    SignatureVerificationFailed {
        /// 失败原因
        reason: String,
    },
    
    /// 签名过期
    #[error("Signature expired: {expired_at} < {current_time}")]
    SignatureExpired {
        /// 过期时间
        expired_at: i64,
        /// 当前时间
        current_time: i64,
    },
    
    /// 签名格式无效
    #[error("Invalid signature format: {reason}")]
    InvalidSignatureFormat {
        /// 无效原因
        reason: String,
    },
    
    /// 公钥无效
    #[error("Invalid public key: {key} - {reason}")]
    InvalidPublicKey {
        /// 无效的公钥
        key: String,
        /// 无效原因
        reason: String,
    },
    
    /// 私钥无效
    #[error("Invalid private key: {reason}")]
    InvalidPrivateKey {
        /// 无效原因
        reason: String,
    },
    
    /// 加密失败
    #[error("Encryption failed: {reason}")]
    EncryptionFailed {
        /// 失败原因
        reason: String,
    },
    
    /// 解密失败
    #[error("Decryption failed: {reason}")]
    DecryptionFailed {
        /// 失败原因
        reason: String,
    },
    
    /// 密钥不匹配
    #[error("Key mismatch: expected {expected}, got {actual}")]
    KeyMismatch {
        /// 期望的密钥
        expected: String,
        /// 实际的密钥
        actual: String,
    },
    
    /// 哈希验证失败
    #[error("Hash verification failed: expected {expected}, got {actual}")]
    HashVerificationFailed {
        /// 期望的哈希
        expected: String,
        /// 实际的哈希
        actual: String,
    },
    
    /// 哈希算法不支持
    #[error("Unsupported hash algorithm: {algorithm}")]
    UnsupportedHashAlgorithm {
        /// 不支持的算法
        algorithm: String,
    },
    
    /// 加密算法不支持
    #[error("Unsupported encryption algorithm: {algorithm}")]
    UnsupportedEncryptionAlgorithm {
        /// 不支持的算法
        algorithm: String,
    },
    
    /// 随机数生成失败
    #[error("Random number generation failed: {reason}")]
    RandomNumberGenerationFailed {
        /// 失败原因
        reason: String,
    },
    
    /// 随机数重复
    #[error("Nonce reuse detected: {nonce}")]
    NonceReuse {
        /// 重复的随机数
        nonce: String,
    },
    
    /// 随机数过期
    #[error("Nonce expired: {nonce} at {expired_at}")]
    NonceExpired {
        /// 过期的随机数
        nonce: String,
        /// 过期时间
        expired_at: i64,
    },
    
    /// 时间戳验证失败
    #[error("Timestamp validation failed: {current} not in [{min}, {max}]")]
    TimestampValidationFailed {
        /// 当前时间戳
        current: i64,
        /// 最小允许时间戳
        min: i64,
        /// 最大允许时间戳
        max: i64,
    },
    
    /// 重放攻击检测
    #[error("Replay attack detected: {transaction_id}")]
    ReplayAttackDetected {
        /// 重放的交易ID
        transaction_id: String,
    },
    
    /// 暴力破解检测
    #[error("Brute force attack detected: {attempts} attempts in {time_window}s")]
    BruteForceAttackDetected {
        /// 尝试次数
        attempts: u32,
        /// 时间窗口（秒）
        time_window: u64,
    },
    
    /// 速率限制超限
    #[error("Rate limit exceeded: {operation} - {limit} per {time_window}s")]
    RateLimitExceeded {
        /// 操作名称
        operation: String,
        /// 限制次数
        limit: u32,
        /// 时间窗口（秒）
        time_window: u64,
    },
    
    /// 会话过期
    #[error("Session expired: {session_id} at {expired_at}")]
    SessionExpired {
        /// 会话ID
        session_id: String,
        /// 过期时间
        expired_at: i64,
    },
    
    /// 会话无效
    #[error("Invalid session: {session_id} - {reason}")]
    InvalidSession {
        /// 会话ID
        session_id: String,
        /// 无效原因
        reason: String,
    },
    
    /// 令牌无效
    #[error("Invalid token: {token} - {reason}")]
    InvalidToken {
        /// 令牌
        token: String,
        /// 无效原因
        reason: String,
    },
    
    /// 令牌过期
    #[error("Token expired: {token} at {expired_at}")]
    TokenExpired {
        /// 令牌
        token: String,
        /// 过期时间
        expired_at: i64,
    },
    
    /// 证书验证失败
    #[error("Certificate verification failed: {reason}")]
    CertificateVerificationFailed {
        /// 失败原因
        reason: String,
    },
    
    /// 证书过期
    #[error("Certificate expired: {certificate} at {expired_at}")]
    CertificateExpired {
        /// 证书
        certificate: String,
        /// 过期时间
        expired_at: i64,
    },
    
    /// 证书链验证失败
    #[error("Certificate chain verification failed: {reason}")]
    CertificateChainVerificationFailed {
        /// 失败原因
        reason: String,
    },
    
    /// 安全策略违反
    #[error("Security policy violation: {policy} - {reason}")]
    SecurityPolicyViolation {
        /// 违反的策略
        policy: String,
        /// 违反原因
        reason: String,
    },
    
    /// 审计日志失败
    #[error("Audit log failure: {reason}")]
    AuditLogFailure {
        /// 失败原因
        reason: String,
    },
    
    /// 安全配置错误
    #[error("Security configuration error: {reason}")]
    SecurityConfigurationError {
        /// 错误原因
        reason: String,
    },
}

impl ErrorConvertible for SecurityError {
    fn error_code(&self) -> u32 {
        match self {
            SecurityError::InsufficientPermissions { .. } => SECURITY_ERROR_BASE + 1,
            SecurityError::AccessDenied { .. } => SECURITY_ERROR_BASE + 2,
            SecurityError::SignatureVerificationFailed { .. } => SECURITY_ERROR_BASE + 3,
            SecurityError::SignatureExpired { .. } => SECURITY_ERROR_BASE + 4,
            SecurityError::InvalidSignatureFormat { .. } => SECURITY_ERROR_BASE + 5,
            SecurityError::InvalidPublicKey { .. } => SECURITY_ERROR_BASE + 6,
            SecurityError::InvalidPrivateKey { .. } => SECURITY_ERROR_BASE + 7,
            SecurityError::EncryptionFailed { .. } => SECURITY_ERROR_BASE + 8,
            SecurityError::DecryptionFailed { .. } => SECURITY_ERROR_BASE + 9,
            SecurityError::KeyMismatch { .. } => SECURITY_ERROR_BASE + 10,
            SecurityError::HashVerificationFailed { .. } => SECURITY_ERROR_BASE + 11,
            SecurityError::UnsupportedHashAlgorithm { .. } => SECURITY_ERROR_BASE + 12,
            SecurityError::UnsupportedEncryptionAlgorithm { .. } => SECURITY_ERROR_BASE + 13,
            SecurityError::RandomNumberGenerationFailed { .. } => SECURITY_ERROR_BASE + 14,
            SecurityError::NonceReuse { .. } => SECURITY_ERROR_BASE + 15,
            SecurityError::NonceExpired { .. } => SECURITY_ERROR_BASE + 16,
            SecurityError::TimestampValidationFailed { .. } => SECURITY_ERROR_BASE + 17,
            SecurityError::ReplayAttackDetected { .. } => SECURITY_ERROR_BASE + 18,
            SecurityError::BruteForceAttackDetected { .. } => SECURITY_ERROR_BASE + 19,
            SecurityError::RateLimitExceeded { .. } => SECURITY_ERROR_BASE + 20,
            SecurityError::SessionExpired { .. } => SECURITY_ERROR_BASE + 21,
            SecurityError::InvalidSession { .. } => SECURITY_ERROR_BASE + 22,
            SecurityError::InvalidToken { .. } => SECURITY_ERROR_BASE + 23,
            SecurityError::TokenExpired { .. } => SECURITY_ERROR_BASE + 24,
            SecurityError::CertificateVerificationFailed { .. } => SECURITY_ERROR_BASE + 25,
            SecurityError::CertificateExpired { .. } => SECURITY_ERROR_BASE + 26,
            SecurityError::CertificateChainVerificationFailed { .. } => SECURITY_ERROR_BASE + 27,
            SecurityError::SecurityPolicyViolation { .. } => SECURITY_ERROR_BASE + 28,
            SecurityError::AuditLogFailure { .. } => SECURITY_ERROR_BASE + 29,
            SecurityError::SecurityConfigurationError { .. } => SECURITY_ERROR_BASE + 30,
        }
    }
    
    fn error_message(&self) -> String {
        self.to_string()
    }
    
    fn is_recoverable(&self) -> bool {
        matches!(
            self,
            SecurityError::RateLimitExceeded { .. } |
            SecurityError::TimestampValidationFailed { .. } |
            SecurityError::SessionExpired { .. } |
            SecurityError::TokenExpired { .. }
        )
    }
    
    fn retry_after(&self) -> Option<u64> {
        match self {
            SecurityError::RateLimitExceeded { time_window, .. } => Some(*time_window),
            SecurityError::TimestampValidationFailed { .. } => Some(1), // 1秒后重试
            SecurityError::SessionExpired { .. } => Some(60), // 60秒后重试
            SecurityError::TokenExpired { .. } => Some(300), // 5分钟后重试
            _ => None,
        }
    }
}

impl From<SecurityError> for ProgramError {
    fn from(err: SecurityError) -> Self {
        ProgramError::Security(err)
    }
}

impl From<SecurityError> for Error {
    fn from(err: SecurityError) -> Self {
        Error::from(err)
    }
}

/// 安全错误扩展方法
impl SecurityError {
    /// 检查是否为可恢复错误
    pub fn can_retry(&self) -> bool {
        self.is_recoverable()
    }
    
    /// 获取错误严重程度
    pub fn severity(&self) -> SecurityErrorSeverity {
        match self {
            SecurityError::RateLimitExceeded { .. } => SecurityErrorSeverity::Warning,
            SecurityError::TimestampValidationFailed { .. } => SecurityErrorSeverity::Warning,
            SecurityError::SessionExpired { .. } => SecurityErrorSeverity::Warning,
            SecurityError::TokenExpired { .. } => SecurityErrorSeverity::Warning,
            SecurityError::InsufficientPermissions { .. } => SecurityErrorSeverity::Error,
            SecurityError::AccessDenied { .. } => SecurityErrorSeverity::Error,
            SecurityError::SignatureExpired { .. } => SecurityErrorSeverity::Error,
            SecurityError::InvalidSignatureFormat { .. } => SecurityErrorSeverity::Error,
            SecurityError::InvalidPublicKey { .. } => SecurityErrorSeverity::Error,
            SecurityError::InvalidPrivateKey { .. } => SecurityErrorSeverity::Error,
            SecurityError::EncryptionFailed { .. } => SecurityErrorSeverity::Error,
            SecurityError::DecryptionFailed { .. } => SecurityErrorSeverity::Error,
            SecurityError::KeyMismatch { .. } => SecurityErrorSeverity::Error,
            SecurityError::HashVerificationFailed { .. } => SecurityErrorSeverity::Error,
            SecurityError::UnsupportedHashAlgorithm { .. } => SecurityErrorSeverity::Error,
            SecurityError::UnsupportedEncryptionAlgorithm { .. } => SecurityErrorSeverity::Error,
            SecurityError::RandomNumberGenerationFailed { .. } => SecurityErrorSeverity::Error,
            SecurityError::NonceExpired { .. } => SecurityErrorSeverity::Error,
            SecurityError::InvalidSession { .. } => SecurityErrorSeverity::Error,
            SecurityError::InvalidToken { .. } => SecurityErrorSeverity::Error,
            SecurityError::CertificateExpired { .. } => SecurityErrorSeverity::Error,
            SecurityError::SecurityConfigurationError { .. } => SecurityErrorSeverity::Error,
            SecurityError::SignatureVerificationFailed { .. } => SecurityErrorSeverity::Critical,
            SecurityError::NonceReuse { .. } => SecurityErrorSeverity::Critical,
            SecurityError::ReplayAttackDetected { .. } => SecurityErrorSeverity::Critical,
            SecurityError::BruteForceAttackDetected { .. } => SecurityErrorSeverity::Critical,
            SecurityError::CertificateVerificationFailed { .. } => SecurityErrorSeverity::Critical,
            SecurityError::CertificateChainVerificationFailed { .. } => SecurityErrorSeverity::Critical,
            SecurityError::SecurityPolicyViolation { .. } => SecurityErrorSeverity::Critical,
            SecurityError::AuditLogFailure { .. } => SecurityErrorSeverity::Critical,
        }
    }
    
    /// 获取错误分类
    pub fn category(&self) -> &'static str {
        match self {
            SecurityError::InsufficientPermissions { .. } => "Authorization",
            SecurityError::AccessDenied { .. } => "Authorization",
            SecurityError::SignatureVerificationFailed { .. } => "Cryptography",
            SecurityError::SignatureExpired { .. } => "Cryptography",
            SecurityError::InvalidSignatureFormat { .. } => "Cryptography",
            SecurityError::InvalidPublicKey { .. } => "Cryptography",
            SecurityError::InvalidPrivateKey { .. } => "Cryptography",
            SecurityError::EncryptionFailed { .. } => "Cryptography",
            SecurityError::DecryptionFailed { .. } => "Cryptography",
            SecurityError::KeyMismatch { .. } => "Cryptography",
            SecurityError::HashVerificationFailed { .. } => "Cryptography",
            SecurityError::UnsupportedHashAlgorithm { .. } => "Cryptography",
            SecurityError::UnsupportedEncryptionAlgorithm { .. } => "Cryptography",
            SecurityError::RandomNumberGenerationFailed { .. } => "Cryptography",
            SecurityError::NonceReuse { .. } => "Cryptography",
            SecurityError::NonceExpired { .. } => "Cryptography",
            SecurityError::TimestampValidationFailed { .. } => "Validation",
            SecurityError::ReplayAttackDetected { .. } => "Attack",
            SecurityError::BruteForceAttackDetected { .. } => "Attack",
            SecurityError::RateLimitExceeded { .. } => "RateLimit",
            SecurityError::SessionExpired { .. } => "Session",
            SecurityError::InvalidSession { .. } => "Session",
            SecurityError::InvalidToken { .. } => "Token",
            SecurityError::TokenExpired { .. } => "Token",
            SecurityError::CertificateVerificationFailed { .. } => "Certificate",
            SecurityError::CertificateExpired { .. } => "Certificate",
            SecurityError::CertificateChainVerificationFailed { .. } => "Certificate",
            SecurityError::SecurityPolicyViolation { .. } => "Policy",
            SecurityError::AuditLogFailure { .. } => "Audit",
            SecurityError::SecurityConfigurationError { .. } => "Configuration",
        }
    }
    
    /// 检查是否为攻击相关错误
    pub fn is_attack_related(&self) -> bool {
        matches!(
            self,
            SecurityError::ReplayAttackDetected { .. } |
            SecurityError::BruteForceAttackDetected { .. } |
            SecurityError::NonceReuse { .. }
        )
    }
    
    /// 检查是否为加密相关错误
    pub fn is_cryptography_related(&self) -> bool {
        matches!(
            self,
            SecurityError::SignatureVerificationFailed { .. } |
            SecurityError::SignatureExpired { .. } |
            SecurityError::InvalidSignatureFormat { .. } |
            SecurityError::InvalidPublicKey { .. } |
            SecurityError::InvalidPrivateKey { .. } |
            SecurityError::EncryptionFailed { .. } |
            SecurityError::DecryptionFailed { .. } |
            SecurityError::KeyMismatch { .. } |
            SecurityError::HashVerificationFailed { .. } |
            SecurityError::UnsupportedHashAlgorithm { .. } |
            SecurityError::UnsupportedEncryptionAlgorithm { .. } |
            SecurityError::RandomNumberGenerationFailed { .. }
        )
    }
    
    /// 检查是否为授权相关错误
    pub fn is_authorization_related(&self) -> bool {
        matches!(
            self,
            SecurityError::InsufficientPermissions { .. } |
            SecurityError::AccessDenied { .. }
        )
    }
}

/// 安全错误严重程度
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityErrorSeverity {
    /// 警告级别
    Warning,
    /// 错误级别
    Error,
    /// 严重级别
    Critical,
}

/// 安全错误统计
#[derive(Debug, Default)]
pub struct SecurityErrorStats {
    /// 总错误数
    pub total_errors: u64,
    /// 可恢复错误数
    pub recoverable_errors: u64,
    /// 严重错误数
    pub critical_errors: u64,
    /// 按类型分组的错误数
    pub errors_by_type: std::collections::HashMap<String, u64>,
    /// 按严重程度分组的错误数
    pub errors_by_severity: std::collections::HashMap<SecurityErrorSeverity, u64>,
    /// 攻击相关错误数
    pub attack_related_errors: u64,
    /// 加密相关错误数
    pub cryptography_related_errors: u64,
    /// 授权相关错误数
    pub authorization_related_errors: u64,
}

impl SecurityErrorStats {
    /// 记录错误
    pub fn record_error(&mut self, error: &SecurityError) {
        self.total_errors += 1;
        
        if error.is_recoverable() {
            self.recoverable_errors += 1;
        }
        
        if error.severity() == SecurityErrorSeverity::Critical {
            self.critical_errors += 1;
        }
        
        if error.is_attack_related() {
            self.attack_related_errors += 1;
        }
        
        if error.is_cryptography_related() {
            self.cryptography_related_errors += 1;
        }
        
        if error.is_authorization_related() {
            self.authorization_related_errors += 1;
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
    
    /// 获取攻击错误率
    pub fn attack_error_rate(&self) -> f64 {
        if self.total_errors == 0 {
            return 0.0;
        }
        self.attack_related_errors as f64 / self.total_errors as f64
    }
    
    /// 获取加密错误率
    pub fn cryptography_error_rate(&self) -> f64 {
        if self.total_errors == 0 {
            return 0.0;
        }
        self.cryptography_related_errors as f64 / self.total_errors as f64
    }
    
    /// 获取授权错误率
    pub fn authorization_error_rate(&self) -> f64 {
        if self.total_errors == 0 {
            return 0.0;
        }
        self.authorization_related_errors as f64 / self.total_errors as f64
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
        self.errors_by_type.clear();
        self.errors_by_severity.clear();
        self.attack_related_errors = 0;
        self.cryptography_related_errors = 0;
        self.authorization_related_errors = 0;
    }
} 