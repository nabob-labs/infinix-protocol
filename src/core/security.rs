/*!
 * Security Module
 *
 * This module provides comprehensive security measures for:
 * - Access control and authorization
 * - Rate limiting and circuit breakers
 * - MEV protection mechanisms
 * - Reentrancy guards
 * - Input sanitization
 * - Audit logging
 */

use crate::error::StrategyError;
use anchor_lang::prelude::*;

/// AccessControl trait - 可插拔权限控制
pub trait AccessControl: Send + Sync {
    fn verify_authority(&self, expected: &Pubkey, provided: &Pubkey) -> Result<()>;
}

/// RateLimiter trait - 可插拔限流器
pub trait RateLimiter: Send + Sync {
    fn check_rate_limit(&mut self, current_timestamp: i64) -> Result<()>;
    fn remaining_operations(&self) -> u32;
}

/// CircuitBreaker trait - 可插拔熔断器
pub trait CircuitBreaker: Send + Sync {
    fn check_operation_allowed(&mut self, current_timestamp: i64) -> Result<()>;
    fn record_failure(&mut self, current_timestamp: i64);
    fn record_success(&mut self);
}

/// ReentrancyGuard trait - 可插拔重入保护
pub trait ReentrancyGuard: Send + Sync {
    fn enter(&mut self, account: &Pubkey) -> Result<()>;
    fn exit(&mut self, account: &Pubkey);
}

/// AuditLogger trait - 可插拔审计日志
pub trait AuditLogger: Send + Sync {
    fn log_security_event(
        &self,
        event_type: SecurityEventType,
        account: &Pubkey,
        details: &str,
        timestamp: i64,
    );
    fn log_access_attempt(&self, account: &Pubkey, operation: &str, success: bool, timestamp: i64);
}

/// 默认实现
pub struct DefaultAccessControl;
impl AccessControl for DefaultAccessControl {
    fn verify_authority(&self, expected: &Pubkey, provided: &Pubkey) -> Result<()> {
        if expected != provided {
            return Err(crate::error::StrategyError::Unauthorized.into());
        }
        Ok(())
    }
}

pub struct DefaultRateLimiter {
    pub max_operations: u32,
    pub time_window: u64,
    pub current_count: u32,
    pub window_start: i64,
}
impl RateLimiter for DefaultRateLimiter {
    fn check_rate_limit(&mut self, current_timestamp: i64) -> Result<()> {
        if current_timestamp - self.window_start >= self.time_window as i64 {
            self.current_count = 0;
            self.window_start = current_timestamp;
        }
        if self.current_count >= self.max_operations {
            return Err(crate::error::StrategyError::RiskLimitsExceeded.into());
        }
        self.current_count += 1;
        Ok(())
    }
    fn remaining_operations(&self) -> u32 {
        self.max_operations.saturating_sub(self.current_count)
    }
}

pub struct DefaultCircuitBreaker {
    pub failure_threshold: u32,
    pub failure_count: u32,
    pub last_failure: i64,
    pub recovery_timeout: u64,
}
impl CircuitBreaker for DefaultCircuitBreaker {
    fn check_operation_allowed(&mut self, current_timestamp: i64) -> Result<()> {
        if self.failure_count >= self.failure_threshold {
            if current_timestamp - self.last_failure < self.recovery_timeout as i64 {
                return Err(crate::error::StrategyError::CircuitBreakerActivated.into());
            } else {
                self.failure_count = 0;
            }
        }
        Ok(())
    }
    fn record_failure(&mut self, current_timestamp: i64) {
        self.failure_count += 1;
        self.last_failure = current_timestamp;
    }
    fn record_success(&mut self) {
        if self.failure_count > 0 {
            self.failure_count -= 1;
        }
    }
}

pub struct DefaultReentrancyGuard {
    pub active_operations: std::collections::HashMap<Pubkey, bool>,
}
impl ReentrancyGuard for DefaultReentrancyGuard {
    fn enter(&mut self, account: &Pubkey) -> Result<()> {
        if self.active_operations.get(account).unwrap_or(&false) {
            return Err(crate::error::StrategyError::StrategyExecutionFailed.into());
        }
        self.active_operations.insert(*account, true);
        Ok(())
    }
    fn exit(&mut self, account: &Pubkey) {
        self.active_operations.insert(*account, false);
    }
}

pub struct DefaultAuditLogger;
impl AuditLogger for DefaultAuditLogger {
    fn log_security_event(
        &self,
        event_type: SecurityEventType,
        account: &Pubkey,
        details: &str,
        timestamp: i64,
    ) {
        msg!(
            "SECURITY_EVENT: {:?} | Account: {} | Details: {} | Timestamp: {}",
            event_type,
            account,
            details,
            timestamp
        );
    }
    fn log_access_attempt(&self, account: &Pubkey, operation: &str, success: bool, timestamp: i64) {
        let status = if success { "SUCCESS" } else { "FAILED" };
        msg!(
            "ACCESS_ATTEMPT: {} | Account: {} | Operation: {} | Timestamp: {}",
            status,
            account,
            operation,
            timestamp
        );
    }
}

/// MEV protection mechanisms
pub struct MEVProtection;

impl MEVProtection {
    /// Detect potential sandwich attack
    pub fn detect_sandwich_attack(
        pre_trade_price: u64,
        post_trade_price: u64,
        expected_slippage_bps: u64,
    ) -> Result<()> {
        if pre_trade_price == 0 {
            return Err(StrategyError::InvalidMarketData.into());
        }

        let actual_slippage_bps = if post_trade_price > pre_trade_price {
            ((post_trade_price - pre_trade_price) * BASIS_POINTS_MAX) / pre_trade_price
        } else {
            ((pre_trade_price - post_trade_price) * BASIS_POINTS_MAX) / pre_trade_price
        };

        // If actual slippage significantly exceeds expected, potential sandwich attack
        if actual_slippage_bps > expected_slippage_bps * 2 {
            return Err(StrategyError::SlippageExceeded.into());
        }

        Ok(())
    }

    /// Implement commit-reveal scheme for trade protection
    pub fn verify_commit_reveal(
        commitment: &[u8; 32],
        revealed_data: &[u8],
        nonce: &[u8],
    ) -> Result<()> {
        use solana_program::hash::{hash, Hash};

        let mut combined_data = Vec::new();
        combined_data.extend_from_slice(revealed_data);
        combined_data.extend_from_slice(nonce);

        let computed_hash = hash(&combined_data);

        if computed_hash.to_bytes() != *commitment {
            return Err(StrategyError::Unauthorized.into());
        }

        Ok(())
    }

    /// Calculate minimum delay to prevent front-running
    pub fn calculate_minimum_delay(
        trade_size: u64,
        market_liquidity: u64,
        base_delay_slots: u64,
    ) -> u64 {
        if market_liquidity == 0 {
            return base_delay_slots * 2;
        }

        // Larger trades relative to liquidity need longer delays
        let size_ratio = (trade_size * 100) / market_liquidity;
        let additional_delay = size_ratio / 10; // 1 slot per 10% of liquidity

        base_delay_slots + additional_delay
    }

    /// Validate transaction timing to prevent MEV
    pub fn validate_transaction_timing(
        submission_slot: u64,
        current_slot: u64,
        min_delay_slots: u64,
    ) -> Result<()> {
        if current_slot < submission_slot + min_delay_slots {
            return Err(StrategyError::InvalidTimeWindow.into());
        }
        Ok(())
    }
}

/// Input sanitization utilities
pub struct InputSanitizer;

impl InputSanitizer {
    /// Sanitize and validate string input
    pub fn sanitize_string(input: &str, max_length: usize) -> Result<String> {
        if input.len() > max_length {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        // Remove control characters and validate UTF-8
        let sanitized: String = input
            .chars()
            .filter(|c| !c.is_control() || c.is_whitespace())
            .collect();

        if sanitized.is_empty() && !input.is_empty() {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        Ok(sanitized)
    }

    /// Sanitize numeric input
    pub fn sanitize_numeric(value: u64, min: u64, max: u64) -> Result<u64> {
        if value < min || value > max {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(value)
    }

    /// Sanitize array input
    pub fn sanitize_array<T: Clone>(
        input: &[T],
        min_length: usize,
        max_length: usize,
    ) -> Result<Vec<T>> {
        if input.len() < min_length || input.len() > max_length {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(input.to_vec())
    }

    /// Sanitize pubkey input
    pub fn sanitize_pubkey(pubkey: &Pubkey) -> Result<Pubkey> {
        if *pubkey == Pubkey::default() {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(*pubkey)
    }
}

/// 安全事件类型
#[derive(Debug, Clone)]
pub enum SecurityEventType {
    UnauthorizedAccess,
    RateLimitExceeded,
    CircuitBreakerTriggered,
    SuspiciousActivity,
    MEVDetected,
    ReentrancyAttempt,
    InvalidInput,
    PermissionDenied,
}

/// Security configuration
pub struct SecurityConfig {
    /// Enable rate limiting
    pub enable_rate_limiting: bool,
    /// Enable circuit breaker
    pub enable_circuit_breaker: bool,
    /// Enable MEV protection
    pub enable_mev_protection: bool,
    /// Enable reentrancy protection
    pub enable_reentrancy_protection: bool,
    /// Enable audit logging
    pub enable_audit_logging: bool,
    /// Maximum operations per minute
    pub max_operations_per_minute: u32,
    /// Circuit breaker failure threshold
    pub circuit_breaker_threshold: u32,
    /// MEV protection delay slots
    pub mev_protection_delay_slots: u64,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enable_rate_limiting: true,
            enable_circuit_breaker: true,
            enable_mev_protection: true,
            enable_reentrancy_protection: true,
            enable_audit_logging: true,
            max_operations_per_minute: 60,
            circuit_breaker_threshold: 5,
            mev_protection_delay_slots: MEV_PROTECTION_DELAY_SLOTS,
        }
    }
}

/// Comprehensive security manager
pub struct SecurityManager {
    pub config: SecurityConfig,
    pub rate_limiter: DefaultRateLimiter,
    pub circuit_breaker: DefaultCircuitBreaker,
    pub reentrancy_guard: DefaultReentrancyGuard,
}

impl SecurityManager {
    /// Create new security manager
    pub fn new(config: SecurityConfig) -> Self {
        Self {
            rate_limiter: DefaultRateLimiter {
                max_operations: config.max_operations_per_minute,
                time_window: 60,
                current_count: 0,
                window_start: 0,
            },
            circuit_breaker: DefaultCircuitBreaker {
                failure_threshold: config.circuit_breaker_threshold,
                failure_count: 0,
                last_failure: 0,
                recovery_timeout: 300,
            },
            reentrancy_guard: DefaultReentrancyGuard {
                active_operations: HashMap::new(),
            },
            config,
        }
    }

    /// Perform comprehensive security check
    pub fn security_check(
        &mut self,
        account: &Pubkey,
        operation: &str,
        current_timestamp: i64,
    ) -> Result<()> {
        // Rate limiting check
        if self.config.enable_rate_limiting {
            self.rate_limiter.check_rate_limit(current_timestamp)?;
        }

        // Circuit breaker check
        if self.config.enable_circuit_breaker {
            self.circuit_breaker
                .check_operation_allowed(current_timestamp)?;
        }

        // Reentrancy check
        if self.config.enable_reentrancy_protection {
            self.reentrancy_guard.enter(account)?;
        }

        // Log access attempt
        if self.config.enable_audit_logging {
            DefaultAuditLogger.log_access_attempt(account, operation, true, current_timestamp);
        }

        Ok(())
    }

    /// Record operation success
    pub fn record_success(&mut self, account: &Pubkey) {
        if self.config.enable_circuit_breaker {
            self.circuit_breaker.record_success();
        }

        if self.config.enable_reentrancy_protection {
            self.reentrancy_guard.exit(account);
        }
    }

    /// Record operation failure
    pub fn record_failure(&mut self, account: &Pubkey, current_timestamp: i64) {
        if self.config.enable_circuit_breaker {
            self.circuit_breaker.record_failure(current_timestamp);
        }

        if self.config.enable_reentrancy_protection {
            self.reentrancy_guard.exit(account);
        }

        if self.config.enable_audit_logging {
            DefaultAuditLogger.log_security_event(
                SecurityEventType::SuspiciousActivity,
                account,
                "Operation failed",
                current_timestamp,
            );
        }
    }
}

// 单元测试
#[cfg(test)]
mod tests {
    use super::*;
    use anchor_lang::prelude::Pubkey;

    #[test]
    fn test_access_control() {
        let ac = DefaultAccessControl;
        let a = Pubkey::default();
        let b = Pubkey::default();
        assert!(ac.verify_authority(&a, &b).is_ok());
        let c = Pubkey::new_unique();
        assert!(ac.verify_authority(&a, &c).is_err());
    }
    #[test]
    fn test_rate_limiter() {
        let mut rl = DefaultRateLimiter {
            max_operations: 2,
            time_window: 60,
            current_count: 0,
            window_start: 0,
        };
        assert!(rl.check_rate_limit(0).is_ok());
        assert!(rl.check_rate_limit(0).is_ok());
        assert!(rl.check_rate_limit(0).is_err());
    }
    #[test]
    fn test_circuit_breaker() {
        let mut cb = DefaultCircuitBreaker {
            failure_threshold: 2,
            failure_count: 0,
            last_failure: 0,
            recovery_timeout: 10,
        };
        assert!(cb.check_operation_allowed(0).is_ok());
        cb.record_failure(1);
        cb.record_failure(2);
        assert!(cb.check_operation_allowed(3).is_err());
        assert!(cb.check_operation_allowed(15).is_ok());
    }
    #[test]
    fn test_reentrancy_guard() {
        let mut rg = DefaultReentrancyGuard {
            active_operations: std::collections::HashMap::new(),
        };
        let a = Pubkey::default();
        assert!(rg.enter(&a).is_ok());
        assert!(rg.enter(&a).is_err());
        rg.exit(&a);
        assert!(rg.enter(&a).is_ok());
    }
}
