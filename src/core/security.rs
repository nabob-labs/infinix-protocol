/*!
 * 安全模块
 *
 * 本模块为系统提供全面的安全措施，包括：
 * - 访问控制与权限校验
 * - 限流与熔断机制
 * - MEV 防护机制
 * - 重入保护
 * - 输入清洗与校验
 * - 审计日志
 */

use crate::errors::strategy_error::StrategyError;
use anchor_lang::prelude::*;

/// 权限控制 trait
/// - 可插拔权限校验，支持多种授权模型
pub trait AccessControl: Send + Sync {
    /// 校验 provided 是否为 expected 授权者
    fn verify_authority(&self, expected: &Pubkey, provided: &Pubkey) -> anchor_lang::Result<()>;
}

/// 限流器 trait
/// - 支持速率限制、剩余操作数查询
pub trait RateLimiter: Send + Sync {
    /// 检查当前是否允许操作，超限返回 Err
    fn check_rate_limit(&mut self, current_timestamp: i64) -> anchor_lang::Result<()>;
    /// 查询剩余可用操作数
    fn remaining_operations(&self) -> u32;
}

/// 熔断器 trait
/// - 支持操作熔断、失败计数、恢复等
pub trait CircuitBreaker: Send + Sync {
    /// 检查当前是否允许操作，熔断时返回 Err
    fn check_operation_allowed(&mut self, current_timestamp: i64) -> anchor_lang::Result<()>;
    /// 记录一次失败
    fn record_failure(&mut self, current_timestamp: i64);
    /// 记录一次成功
    fn record_success(&mut self);
}

/// 重入保护 trait
/// - 防止同一账户/操作重入
pub trait ReentrancyGuard: Send + Sync {
    /// 进入重入保护区，已重入则返回 Err
    fn enter(&mut self, account: &Pubkey) -> anchor_lang::Result<()>;
    /// 退出重入保护区
    fn exit(&mut self, account: &Pubkey);
}

/// 审计日志 trait
/// - 支持安全事件、访问尝试等日志记录
pub trait AuditLogger: Send + Sync {
    /// 记录安全事件
    fn log_security_event(
        &self,
        event_type: SecurityEventType,
        account: &Pubkey,
        details: &str,
        timestamp: i64,
    );
    /// 记录访问尝试
    fn log_access_attempt(&self, account: &Pubkey, operation: &str, success: bool, timestamp: i64);
}

/// 默认权限控制实现
pub struct DefaultAccessControl;
impl AccessControl for DefaultAccessControl {
    fn verify_authority(&self, expected: &Pubkey, provided: &Pubkey) -> anchor_lang::Result<()> {
        if expected != provided {
            return Err(crate::error::StrategyError::Unauthorized.into());
        }
        Ok(())
    }
}

/// 默认限流器实现
pub struct DefaultRateLimiter {
    /// 最大操作数限制
    pub max_operations: u32,
    /// 时间窗口（秒）
    pub time_window: u64,
    /// 当前窗口内的操作计数
    pub current_count: u32,
    /// 当前窗口的开始时间戳
    pub window_start: i64,
}
impl RateLimiter for DefaultRateLimiter {
    /// 检查当前操作是否超限
    fn check_rate_limit(&mut self, current_timestamp: i64) -> anchor_lang::Result<()> {
        /// 如果当前时间戳与窗口开始时间戳的差值大于等于时间窗口，则重置计数器和窗口开始时间
        if current_timestamp - self.window_start >= self.time_window as i64 {
            self.current_count = 0;
            self.window_start = current_timestamp;
        }
        /// 如果当前计数已达到最大操作数限制，则返回错误
        if self.current_count >= self.max_operations {
            return Err(crate::error::StrategyError::RiskLimitsExceeded.into());
        }
        /// 增加当前计数
        self.current_count += 1;
        Ok(())
    }
    /// 查询剩余可用操作数
    fn remaining_operations(&self) -> u32 {
        /// 返回最大操作数与当前计数之间的差值
        self.max_operations.saturating_sub(self.current_count)
    }
}

/// 默认熔断器实现
pub struct DefaultCircuitBreaker {
    /// 熔断器失败阈值
    pub failure_threshold: u32,
    /// 当前失败计数
    pub failure_count: u32,
    /// 上次失败的时间戳
    pub last_failure: i64,
    /// 熔断恢复超时时间（秒）
    pub recovery_timeout: u64,
}
impl CircuitBreaker for DefaultCircuitBreaker {
    /// 检查当前操作是否允许
    fn check_operation_allowed(&mut self, current_timestamp: i64) -> anchor_lang::Result<()> {
        /// 如果失败计数超过阈值，且当前时间戳与上次失败时间戳的差值小于恢复超时时间，则返回错误
        if self.failure_count >= self.failure_threshold {
            if current_timestamp - self.last_failure < self.recovery_timeout as i64 {
                return Err(crate::error::StrategyError::CircuitBreakerActivated.into());
            } else {
                /// 如果超过恢复超时时间，则重置失败计数
                self.failure_count = 0;
            }
        }
        Ok(())
    }
    /// 记录一次失败
    fn record_failure(&mut self, current_timestamp: i64) {
        /// 增加失败计数并更新上次失败时间
        self.failure_count += 1;
        self.last_failure = current_timestamp;
    }
    /// 记录一次成功
    fn record_success(&mut self) {
        /// 如果失败计数大于0，则减少失败计数
        if self.failure_count > 0 {
            self.failure_count -= 1;
        }
    }
}

/// 默认重入保护实现
pub struct DefaultReentrancyGuard {
    /// 记录当前账户是否处于重入保护状态
    pub active_operations: std::collections::HashMap<Pubkey, bool>,
}
impl ReentrancyGuard for DefaultReentrancyGuard {
    /// 尝试进入重入保护区，如果已重入则返回错误
    fn enter(&mut self, account: &Pubkey) -> anchor_lang::Result<()> {
        /// 检查当前账户是否已处于重入状态
        if self.active_operations.get(account).unwrap_or(&false) {
            return Err(crate::error::StrategyError::StrategyExecutionFailed.into());
        }
        /// 将当前账户标记为已重入
        self.active_operations.insert(*account, true);
        Ok(())
    }
    /// 退出重入保护区
    fn exit(&mut self, account: &Pubkey) {
        /// 将当前账户标记为未重入
        self.active_operations.insert(*account, false);
    }
}

/// 默认审计日志实现
pub struct DefaultAuditLogger;
impl AuditLogger for DefaultAuditLogger {
    /// 记录安全事件
    fn log_security_event(
        &self,
        event_type: SecurityEventType,
        account: &Pubkey,
        details: &str,
        timestamp: i64,
    ) {
        /// 记录安全事件到链上
        msg!(
            "SECURITY_EVENT: {:?} | Account: {} | Details: {} | Timestamp: {}",
            event_type,
            account,
            details,
            timestamp
        );
    }
    /// 记录访问尝试
    fn log_access_attempt(&self, account: &Pubkey, operation: &str, success: bool, timestamp: i64) {
        /// 记录访问尝试到链上
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

/// MEV 防护机制
pub struct MEVProtection;

impl MEVProtection {
    /// 检测潜在三明治攻击
    pub fn detect_sandwich_attack(
        pre_trade_price: u64,
        post_trade_price: u64,
        expected_slippage_bps: u64,
    ) -> anchor_lang::Result<()> {
        /// 如果前交易价格为0，则返回错误
        if pre_trade_price == 0 {
            return Err(StrategyError::InvalidMarketData.into());
        }
        /// 计算实际滑点（以基点为单位）
        let actual_slippage_bps = if post_trade_price > pre_trade_price {
            ((post_trade_price - pre_trade_price) * BASIS_POINTS_MAX) / pre_trade_price
        } else {
            ((pre_trade_price - post_trade_price) * BASIS_POINTS_MAX) / pre_trade_price
        };
        /// 如果实际滑点远超预期，则判定为潜在三明治攻击
        if actual_slippage_bps > expected_slippage_bps * 2 {
            return Err(StrategyError::SlippageExceeded.into());
        }
        Ok(())
    }
    /// 提交-揭示机制校验
    pub fn verify_commit_reveal(
        commitment: &[u8; 32],
        revealed_data: &[u8],
        nonce: &[u8],
    ) -> anchor_lang::Result<()> {
        /// 使用 Solana 的 hash 函数计算组合数据的哈希
        use solana_program::hash::{hash, Hash};
        let mut combined_data = Vec::new();
        combined_data.extend_from_slice(revealed_data);
        combined_data.extend_from_slice(nonce);
        let computed_hash = hash(&combined_data);
        /// 如果计算出的哈希与承诺不匹配，则返回错误
        if computed_hash.to_bytes() != *commitment {
            return Err(StrategyError::Unauthorized.into());
        }
        Ok(())
    }
    /// 计算防抢跑最小延迟
    pub fn calculate_minimum_delay(
        trade_size: u64,
        market_liquidity: u64,
        base_delay_slots: u64,
    ) -> u64 {
        /// 如果市场流动性为0，则返回基础延迟的两倍
        if market_liquidity == 0 {
            return base_delay_slots * 2;
        }
        /// 大额交易需要更长的延迟
        let size_ratio = (trade_size * 100) / market_liquidity;
        let additional_delay = size_ratio / 10; // 每10%流动性加1 slot
        base_delay_slots + additional_delay
    }
    /// 校验交易时序防止 MEV
    pub fn validate_transaction_timing(
        submission_slot: u64,
        current_slot: u64,
        min_delay_slots: u64,
    ) -> anchor_lang::Result<()> {
        /// 如果当前时间戳小于提交时间戳加上最小延迟，则返回错误
        if current_slot < submission_slot + min_delay_slots {
            return Err(StrategyError::InvalidTimeWindow.into());
        }
        Ok(())
    }
}

/// 输入清洗工具
pub struct InputSanitizer;

impl InputSanitizer {
    /// 清洗并校验字符串输入
    pub fn sanitize_string(input: &str, max_length: usize) -> anchor_lang::Result<String> {
        /// 如果输入长度超过最大长度，则返回错误
        if input.len() > max_length {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        /// 移除控制字符，校验UTF-8
        let sanitized: String = input
            .chars()
            .filter(|c| !c.is_control() || c.is_whitespace())
            .collect();
        /// 如果清洗后的字符串为空，但输入不为空，则返回错误
        if sanitized.is_empty() && !input.is_empty() {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(sanitized)
    }
    /// 清洗数值输入
    pub fn sanitize_numeric(value: u64, min: u64, max: u64) -> anchor_lang::Result<u64> {
        /// 如果值小于最小值或大于最大值，则返回错误
        if value < min || value > max {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(value)
    }
    /// 清洗数组输入
    pub fn sanitize_array<T: Clone>(
        input: &[T],
        min_length: usize,
        max_length: usize,
    ) -> anchor_lang::Result<Vec<T>> {
        /// 如果输入数组长度小于最小长度或大于最大长度，则返回错误
        if input.len() < min_length || input.len() > max_length {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(input.to_vec())
    }
    /// 清洗 pubkey 输入
    pub fn sanitize_pubkey(pubkey: &Pubkey) -> anchor_lang::Result<Pubkey> {
        /// 如果 pubkey 是默认值，则返回错误
        if *pubkey == Pubkey::default() {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(*pubkey)
    }
}

/// 安全事件类型
#[derive(Debug, Clone)]
pub enum SecurityEventType {
    /// 未授权访问
    UnauthorizedAccess,
    /// 限流超限
    RateLimitExceeded,
    /// 熔断触发
    CircuitBreakerTriggered,
    /// 可疑行为
    SuspiciousActivity,
    /// 检测到MEV
    MEVDetected,
    /// 重入尝试
    ReentrancyAttempt,
    /// 非法输入
    InvalidInput,
    /// 权限拒绝
    PermissionDenied,
}

/// 安全配置
pub struct SecurityConfig {
    /// 是否启用限流
    pub enable_rate_limiting: bool,
    /// 是否启用熔断
    pub enable_circuit_breaker: bool,
    /// 是否启用MEV防护
    pub enable_mev_protection: bool,
    /// 是否启用重入保护
    pub enable_reentrancy_protection: bool,
    /// 是否启用审计日志
    pub enable_audit_logging: bool,
    /// 每分钟最大操作数
    pub max_operations_per_minute: u32,
    /// 熔断器失败阈值
    pub circuit_breaker_threshold: u32,
    /// MEV防护延迟slots
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

/// 安全管理器
pub struct SecurityManager;

impl SecurityManager {
    /// 记录安全错误
    pub fn log_error(context: &str, error: &str) {
        /// 记录安全错误到链上
        msg!("[SECURITY][ERROR] {}: {}", context, error);
        // 可扩展：写入链上事件、外部监控、报警等
    }
    /// 记录安全事件
    pub fn log_event(context: &str, event: &str) {
        /// 记录安全事件到链上
        msg!("[SECURITY][EVENT] {}: {}", context, event);
        // 可扩展：写入链上事件、外部监控、报警等
    }
    /// 监控安全指标
    pub fn monitor_metric(context: &str, metric: &str, value: u64) {
        /// 记录安全指标到链上
        msg!("[SECURITY][METRIC] {}: {}={}", context, metric, value);
        // 可扩展：写入链上事件、外部监控、报警等
    }
}

/// 记录安全事件日志
pub fn log_security_event(event: &str, user: &Pubkey, details: &str) {
    /// 记录安全事件到链上
    msg!("[Security] {} by {}, details: {}", event, user, details);
}

/// 记录授权检查日志
pub fn log_authorization_check(user: &Pubkey, authorized: bool) {
    /// 记录授权检查到链上
    msg!("[Security] Authorization check for {}: {}", user, authorized);
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
