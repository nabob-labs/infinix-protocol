/*!
 * Advanced Core Traits Module
 *
 * Defines comprehensive traits for consistent behavior across the entire system.
 * These traits provide a foundation for:
 * - Strategy lifecycle management
 * - Risk assessment and monitoring
 * - Performance optimization
 * - Security and authorization
 * - Data validation and integrity
 * - Event handling and notifications
 * - Resource management and cleanup
 */

use crate::error::StrategyError;
use anchor_lang::prelude::*;
use std::cmp::Ordering;
use std::collections::HashMap;

/// Common result type for strategy operations
pub type StrategyResult<T> = Result<T>;

/// Validatable trait - 可验证对象
pub trait Validatable {
    fn validate(&self) -> Result<()>;
}

/// Initializable trait - 可初始化对象
pub trait Initializable {
    fn initialize(&mut self, params: InitializationParams) -> Result<()>;
}

/// Executable trait - 可执行对象
pub trait Executable {
    type Input;
    type Output;
    fn execute(&mut self, input: Self::Input) -> Result<Self::Output>;
    fn can_execute(&self) -> bool;
}

/// Optimizable trait - 可优化对象
pub trait Optimizable {
    fn optimize(&mut self) -> Result<()>;
    fn optimization_score(&self) -> u32;
}

/// Cacheable trait - 可缓存对象
pub trait Cacheable {
    type Key;
    fn cache_key(&self) -> Self::Key;
    fn is_cache_valid(&self, timestamp: i64) -> bool;
}

/// Monitorable trait - 可监控对象
pub trait Monitorable {
    fn collect_metrics(&self) -> Metrics;
}

/// Lockable trait - 可加锁对象
pub trait Lockable {
    fn is_locked(&self) -> bool;
    fn lock(&mut self, authority: &Pubkey) -> Result<()>;
    fn unlock(&mut self, authority: &Pubkey) -> Result<()>;
    fn lock_owner(&self) -> Option<Pubkey>;
}

/// RateLimited trait - 限流对象
pub trait RateLimited {
    fn check_rate_limit(&mut self) -> Result<()>;
    fn rate_limit_config(&self) -> RateLimitConfig;
    fn update_rate_limit(&mut self, config: RateLimitConfig, authority: &Pubkey) -> Result<()>;
}

/// CircuitBreakable trait - 支持熔断
pub trait CircuitBreakable {
    fn check_circuit_breaker(&self) -> Result<()>;
    fn circuit_breaker_config(&self) -> CircuitBreakerConfig;
    fn update_circuit_breaker(
        &mut self,
        config: CircuitBreakerConfig,
        authority: &Pubkey,
    ) -> Result<()>;
}

/// EventEmitter trait - 事件发布
pub trait EventEmitter {
    type Event;
    fn emit_event(&self, event: Self::Event) -> Result<()>;
}

/// NotificationHandler trait - 通知处理
pub trait NotificationHandler {
    type Notification;
    fn handle_notification(&mut self, notification: Self::Notification) -> Result<()>;
    fn can_handle(&self, notification: &Self::Notification) -> bool;
}

/// Versioned trait - 版本化对象
pub trait Versioned {
    fn version(&self) -> u32;
}

/// Metrics 结构体
#[derive(Debug, Clone, Default)]
pub struct Metrics {
    pub execution_count: u64,
    pub error_count: u64,
    pub avg_execution_time_ms: u64,
    pub last_updated: i64,
}

/// RateLimitConfig 结构体
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub max_operations: u32,
    pub time_window_seconds: u64,
    pub burst_allowance: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_operations: 100,
            time_window_seconds: 60,
            burst_allowance: 10,
        }
    }
}

/// CircuitBreakerConfig 结构体
#[derive(Debug, Clone, Default)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub recovery_timeout_seconds: u64,
}

/// InitializationParams 结构体
#[derive(Debug, Clone)]
pub struct InitializationParams {
    pub authority: Pubkey,
    pub bump: u8,
}

/// Trait for types that can be initialized
pub trait Initializable {
    /// Initialize the object with given parameters
    fn initialize(&mut self, params: InitializationParams) -> StrategyResult<()>;
}

/// Trait for types that can be executed
pub trait Executable {
    type Input;
    type Output;

    /// Execute the operation with given input
    fn execute(&mut self, input: Self::Input) -> StrategyResult<Self::Output>;

    /// Check if execution is allowed
    fn can_execute(&self) -> bool;
}

/// Trait for types that can be optimized
pub trait Optimizable {
    /// Optimize the current configuration
    fn optimize(&mut self) -> StrategyResult<()>;

    /// Get optimization score (0-10000)
    fn optimization_score(&self) -> u32;
}

/// Trait for types that can be cached
pub trait Cacheable {
    type Key;

    /// Get cache key for this object
    fn cache_key(&self) -> Self::Key;

    /// Check if cache is valid
    fn is_cache_valid(&self, timestamp: i64) -> bool;
}

/// Trait for types that can be monitored
pub trait Monitorable {
    /// Get current performance metrics
    fn performance_metrics(&self) -> PerformanceMetrics;

    /// Get current health status
    fn health_status(&self) -> HealthStatus;
}

/// Health status enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
    Offline,
}

/// Trait for pausable operations
pub trait Pausable {
    /// Check if currently paused
    fn is_paused(&self) -> bool;

    /// Pause operations
    fn pause(&mut self) -> StrategyResult<()>;

    /// Resume operations (alias for unpause)
    fn resume(&mut self) -> StrategyResult<()> {
        self.unpause()
    }

    /// Unpause operations
    fn unpause(&mut self) -> StrategyResult<()>;
}

/// Trait for activatable objects
pub trait Activatable {
    /// Check if currently active
    fn is_active(&self) -> bool;

    /// Activate the object
    fn activate(&mut self) -> StrategyResult<()>;

    /// Deactivate the object
    fn deactivate(&mut self) -> StrategyResult<()>;
}

/// Trait for authorizable operations
pub trait Authorizable {
    /// Get current authority
    fn authority(&self) -> Pubkey;

    /// Check if given pubkey is authorized
    fn is_authorized(&self, authority: &Pubkey) -> bool {
        self.authority() == *authority
    }

    /// Transfer authority to new pubkey
    fn transfer_authority(&mut self, new_authority: Pubkey) -> StrategyResult<()>;
}

/// Trait for configurable objects
pub trait Configurable<T> {
    /// Get current configuration
    fn config(&self) -> &T;

    /// Update configuration
    fn update_config(&mut self, config: T) -> StrategyResult<()>;

    /// Validate configuration
    fn validate_config(config: &T) -> StrategyResult<()>;
}

/// Trait for objects with lifecycle management
pub trait Lifecycle {
    /// Initialize the object
    fn init(&mut self) -> StrategyResult<()>;

    /// Start operations
    fn start(&mut self) -> StrategyResult<()>;

    /// Stop operations
    fn stop(&mut self) -> StrategyResult<()>;

    /// Cleanup resources
    fn cleanup(&mut self) -> StrategyResult<()>;
}

/// Trait for risk-aware operations
pub trait RiskAware {
    /// Assess current risk level (0-10000)
    fn risk_level(&self) -> u32;

    /// Check if operation is within risk limits
    fn within_risk_limits(&self, operation_risk: u32) -> bool;

    /// Get risk metrics
    fn risk_metrics(&self) -> RiskMetrics;
}

/// Trait for fee-aware operations
pub trait FeeAware {
    /// Calculate fees for given amount
    fn calculate_fees(&self, amount: u64) -> StrategyResult<u64>;

    /// Get fee configuration
    fn fee_config(&self) -> FeeConfig;

    /// Update fee configuration
    fn update_fee_config(&mut self, config: FeeConfig) -> StrategyResult<()>;
}

/// Trait for time-aware operations
pub trait TimeAware {
    /// Get creation timestamp
    fn created_at(&self) -> i64;

    /// Get last update timestamp
    fn updated_at(&self) -> i64;

    /// Update timestamp to current time
    fn touch(&mut self) -> StrategyResult<()>;

    /// Check if object is stale
    fn is_stale(&self, max_age_seconds: i64) -> bool {
        let current_time = Clock::get().unwrap().unix_timestamp;
        current_time - self.updated_at() > max_age_seconds
    }
}

/// Trait for serializable objects
pub trait Serializable {
    /// Serialize to bytes
    fn serialize(&self) -> StrategyResult<Vec<u8>>;

    /// Deserialize from bytes
    fn deserialize(data: &[u8]) -> StrategyResult<Self>
    where
        Self: Sized;
}

/// Trait for objects that can be reset
pub trait Resettable {
    /// Reset to default state
    fn reset(&mut self) -> StrategyResult<()>;

    /// Reset to specific state
    fn reset_to(&mut self, state: &Self) -> StrategyResult<()>
    where
        Self: Clone,
    {
        *self = state.clone();
        Ok(())
    }
}

/// Trait for objects that can be compared for equality
pub trait Comparable<T> {
    /// Compare with another object
    fn compare(&self, other: &T) -> Ordering;

    /// Check if equal to another object
    fn equals(&self, other: &T) -> bool {
        self.compare(other) == Ordering::Equal
    }
}

use std::cmp::Ordering;

/// Trait for objects that can be merged
pub trait Mergeable<T> {
    /// Merge with another object
    fn merge(&mut self, other: &T) -> StrategyResult<()>;

    /// Check if can merge with another object
    fn can_merge(&self, other: &T) -> bool;
}

/// Trait for objects that can be split
pub trait Splittable {
    /// Split into multiple objects
    fn split(&self, parts: usize) -> StrategyResult<Vec<Self>>
    where
        Self: Sized;

    /// Check if can be split
    fn can_split(&self, parts: usize) -> bool;
}

/// Advanced trait for event-driven objects
pub trait EventEmitter {
    type Event;

    /// Emit an event
    fn emit_event(&self, event: Self::Event) -> StrategyResult<()>;

    /// Subscribe to events
    fn subscribe(&mut self, callback: Box<dyn Fn(&Self::Event)>) -> StrategyResult<u64>;

    /// Unsubscribe from events
    fn unsubscribe(&mut self, subscription_id: u64) -> StrategyResult<()>;
}

/// Trait for objects that can handle notifications
pub trait NotificationHandler {
    type Notification;

    /// Handle incoming notification
    fn handle_notification(&mut self, notification: Self::Notification) -> StrategyResult<()>;

    /// Check if can handle specific notification type
    fn can_handle(&self, notification: &Self::Notification) -> bool;
}

/// Trait for objects with versioning support
pub trait Versioned {
    /// Get current version
    fn version(&self) -> Version;

    /// Check if compatible with another version
    fn is_compatible(&self, other_version: &Version) -> bool;

    /// Migrate to new version
    fn migrate(&mut self, target_version: Version) -> StrategyResult<()>;
}

/// Version structure for compatibility checking
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, AnchorSerialize, AnchorDeserialize)]
pub struct Version {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

impl Version {
    pub fn new(major: u16, minor: u16, patch: u16) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    pub fn is_breaking_change(&self, other: &Version) -> bool {
        self.major != other.major
    }
}

/// Trait for objects that can be backed up and restored
pub trait Backup {
    type BackupData;

    /// Create backup of current state
    fn create_backup(&self) -> StrategyResult<Self::BackupData>;

    /// Restore from backup
    fn restore_from_backup(&mut self, backup: Self::BackupData) -> StrategyResult<()>;

    /// Validate backup integrity
    fn validate_backup(&self, backup: &Self::BackupData) -> StrategyResult<()>;
}

/// Trait for objects with audit trail capabilities
pub trait Auditable {
    type AuditEntry;

    /// Record audit entry
    fn record_audit(&mut self, entry: Self::AuditEntry) -> StrategyResult<()>;

    /// Get audit trail
    fn audit_trail(&self) -> Vec<Self::AuditEntry>;

    /// Clear audit trail (with proper authorization)
    fn clear_audit_trail(&mut self, authority: &Pubkey) -> StrategyResult<()>;
}

/// Trait for objects with quota management
pub trait QuotaManaged {
    /// Get current usage
    fn current_usage(&self) -> u64;

    /// Get quota limit
    fn quota_limit(&self) -> u64;

    /// Check if within quota
    fn within_quota(&self, additional_usage: u64) -> bool {
        self.current_usage() + additional_usage <= self.quota_limit()
    }

    /// Update quota limit
    fn update_quota(&mut self, new_limit: u64, authority: &Pubkey) -> StrategyResult<()>;
}

/// Circuit breaker states
#[derive(Debug, Clone, PartialEq)]
pub enum CircuitState {
    Closed,   // Normal operation
    Open,     // Circuit breaker triggered
    HalfOpen, // Testing if service recovered
}

/// Trait for objects with circuit breaker functionality
pub trait CircuitBreaker {
    /// Check if circuit breaker is open
    fn is_circuit_open(&self) -> bool;

    /// Record successful operation
    fn record_success(&mut self);

    /// Record failed operation
    fn record_failure(&mut self);

    /// Get circuit breaker state
    fn circuit_state(&self) -> CircuitState;

    /// Reset circuit breaker
    fn reset_circuit(&mut self, authority: &Pubkey) -> StrategyResult<()>;
}

/// Health check result
#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    pub status: HealthStatus,
    pub message: String,
    pub timestamp: i64,
    pub metrics: HashMap<String, f64>,
}

/// Individual health metric
#[derive(Debug, Clone)]
pub struct HealthMetric {
    pub value: f64,
    pub threshold: f64,
    pub status: HealthStatus,
    pub description: String,
}

/// Trait for objects with health checking
pub trait HealthCheck {
    /// Perform health check
    fn health_check(&self) -> HealthCheckResult;

    /// Get detailed health information
    fn detailed_health(&self) -> HashMap<String, HealthMetric>;

    /// Register health check callback
    fn register_health_callback(
        &mut self,
        callback: Box<dyn Fn(&HealthCheckResult)>,
    ) -> StrategyResult<u64>;
}

/// Trait for objects with dependency management
pub trait DependencyManaged {
    type Dependency;

    /// Add dependency
    fn add_dependency(&mut self, dependency: Self::Dependency) -> StrategyResult<()>;

    /// Remove dependency
    fn remove_dependency(&mut self, dependency_id: &str) -> StrategyResult<()>;

    /// Check if all dependencies are healthy
    fn dependencies_healthy(&self) -> bool;

    /// Get dependency status
    fn dependency_status(&self) -> HashMap<String, HealthStatus>;
}

/// Resource usage information
#[derive(Debug, Clone)]
pub struct ResourceUsage {
    pub memory_bytes: u64,
    pub compute_units: u32,
    pub storage_bytes: u64,
    pub network_bandwidth: u64,
}

/// Resource requirement specification
#[derive(Debug, Clone)]
pub struct ResourceRequirement {
    pub memory_bytes: u64,
    pub compute_units: u32,
    pub storage_bytes: u64,
    pub network_bandwidth: u64,
    pub duration_seconds: u64,
}

/// Resource reservation identifier
pub type ReservationId = u64;

/// Common fee configuration structure
#[derive(Debug, Clone)]
pub struct FeeConfig {
    pub management_fee_bps: u16,
    pub performance_fee_bps: u16,
    pub creation_fee_bps: u16,
    pub redemption_fee_bps: u16,
    pub fee_collector: Pubkey,
}

impl Default for FeeConfig {
    fn default() -> Self {
        Self {
            management_fee_bps: 100,   // 1%
            performance_fee_bps: 1000, // 10%
            creation_fee_bps: 10,      // 0.1%
            redemption_fee_bps: 10,    // 0.1%
            fee_collector: Pubkey::default(),
        }
    }
}

/// Trait for strategy-specific operations
pub trait StrategyExecutor {
    type StrategyInput;
    type StrategyOutput;
    type StrategyConfig;

    /// Execute strategy with given input
    fn execute_strategy(
        &mut self,
        input: Self::StrategyInput,
    ) -> StrategyResult<Self::StrategyOutput>;

    /// Validate strategy configuration
    fn validate_strategy_config(&self, config: &Self::StrategyConfig) -> StrategyResult<()>;

    /// Get strategy performance metrics
    fn strategy_performance(&self) -> StrategyPerformanceMetrics;

    /// Optimize strategy parameters
    fn optimize_strategy(&mut self) -> StrategyResult<()>;
}

/// Strategy performance metrics
#[derive(Debug, Clone)]
pub struct StrategyPerformanceMetrics {
    pub total_executions: u64,
    pub successful_executions: u64,
    pub average_execution_time_ms: u64,
    pub total_value_processed: u64,
    pub total_fees_collected: u64,
    pub success_rate_bps: u64,
    pub average_slippage_bps: u64,
    pub risk_adjusted_return_bps: i64,
}

/// Trait for market data consumers
pub trait MarketDataConsumer {
    /// Process market data update
    fn process_market_data(&mut self, data: &MarketData) -> StrategyResult<()>;

    /// Get required market data types
    fn required_data_types(&self) -> Vec<MarketDataType>;

    /// Check if market data is stale
    fn is_market_data_stale(&self) -> bool;
}

/// Market data types
#[derive(Debug, Clone, PartialEq)]
pub enum MarketDataType {
    Price,
    Volume,
    Liquidity,
    Volatility,
    TechnicalIndicators,
    OrderBook,
    Trades,
}

// RiskMetrics and PerformanceMetrics are defined in types.rs to avoid duplication

// 单元测试
#[cfg(test)]
mod tests {
    use super::*;
    use anchor_lang::prelude::Pubkey;

    struct DummyValidatable;
    impl Validatable for DummyValidatable {
        fn validate(&self) -> Result<()> {
            Ok(())
        }
    }
    #[test]
    fn test_validatable() {
        let v = DummyValidatable;
        assert!(v.validate().is_ok());
    }

    struct DummyInitializable {
        pub initialized: bool,
    }
    impl Initializable for DummyInitializable {
        fn initialize(&mut self, _params: InitializationParams) -> Result<()> {
            self.initialized = true;
            Ok(())
        }
    }
    #[test]
    fn test_initializable() {
        let mut i = DummyInitializable { initialized: false };
        let params = InitializationParams {
            authority: Pubkey::default(),
            bump: 1,
        };
        assert!(i.initialize(params).is_ok());
        assert!(i.initialized);
    }
}
