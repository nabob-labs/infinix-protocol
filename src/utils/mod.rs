//!
//! Utilities Module
//!
//! 本模块提供核心工具函数集合，包含数学运算、验证、缓存、性能监控、价格计算、简化工具、批量优化等子模块，为整个系统提供基础功能支持。

use anchor_lang::prelude::*;
// anchor_lang::msg not available, removing import

// 导出所有子模块，确保外部可访问。
pub mod cache;      // 缓存工具模块
pub mod math;       // 数学运算模块
pub mod performance;// 性能监控模块
pub mod price;      // 价格计算模块
pub mod simplified; // 简化工具模块
pub mod validation; // 验证工具模块
pub mod batch_optimizer; // 批量操作优化模块

// 重新导出常用结构体和函数，提供便捷访问。
pub use cache::*;           // 导出缓存相关
pub use math::*;            // 导出数学运算相关
pub use performance::*;     // 导出性能监控相关
pub use price::*;           // 导出价格计算相关
pub use simplified::*;      // 导出简化工具相关
pub use validation::*;      // 导出验证工具相关
pub use batch_optimizer::*; // 导出批量优化相关

/// 工具模块版本信息。
pub const UTILS_VERSION: &str = "1.0.0";

/// 工具模块常量定义。
pub const UTILS_MODULE_NAME: &str = "utils";

/// 工具模块初始化函数。
pub fn initialize_utils() -> anchor_lang::Result<()> {
    // 初始化所有子模块
    msg!("Initializing utils module v{}", UTILS_VERSION);
    Ok(())
}

/// 工具模块清理函数。
pub fn cleanup_utils() -> anchor_lang::Result<()> {
    // 清理所有子模块资源
    msg!("Cleaning up utils module");
    Ok(())
}

/// 工具模块状态检查函数。
pub fn check_utils_status() -> bool {
    // 检查所有子模块状态
    true // 简化实现，实际应检查各子模块状态
}

/// 工具模块版本兼容性检查。
pub fn check_version_compatibility(required_version: &str) -> bool {
    // 检查版本兼容性
    UTILS_VERSION == required_version
}

/// 工具模块配置结构体。
#[derive(Debug, Clone)]
pub struct UtilsConfig {
    pub enable_cache: bool,           // 是否启用缓存
    pub enable_performance_monitoring: bool, // 是否启用性能监控
    pub enable_validation: bool,      // 是否启用验证
    pub max_cache_size: usize,        // 最大缓存大小
    pub cache_ttl_seconds: i64,      // 缓存生存时间
    pub performance_thresholds: PerformanceThresholds, // 性能阈值
}

impl Default for UtilsConfig {
    fn default() -> Self {
        Self {
            enable_cache: true,
            enable_performance_monitoring: true,
            enable_validation: true,
            max_cache_size: 1000,
            cache_ttl_seconds: 300, // 5 分钟
            performance_thresholds: PerformanceThresholds::default(),
        }
    }
}

/// 工具模块管理器结构体。
pub struct UtilsManager {
    config: UtilsConfig,              // 配置信息
    cache_manager: Option<CacheManager>, // 缓存管理器
    performance_monitor: Option<PerformanceMonitor>, // 性能监控器
}

impl UtilsManager {
    /// 创建新的工具模块管理器。
    pub fn new(config: UtilsConfig) -> Self {
        Self {
            config,
            cache_manager: None,
            performance_monitor: None,
        }
    }
    
    /// 初始化工具模块管理器。
    pub fn initialize(&mut self) -> anchor_lang::Result<()> {
        // 根据配置初始化各组件
        if self.config.enable_cache {
            self.cache_manager = Some(CacheManager::new(
                self.config.max_cache_size,
                self.config.cache_ttl_seconds,
            ));
        }
        
        if self.config.enable_performance_monitoring {
            self.performance_monitor = Some(PerformanceMonitor);
        }
        
        msg!("UtilsManager initialized successfully");
        Ok(())
    }
    
    /// 获取缓存管理器。
    pub fn get_cache_manager(&self) -> Option<&CacheManager> {
        self.cache_manager.as_ref()
    }
    
    /// 获取性能监控器。
    pub fn get_performance_monitor(&self) -> Option<&PerformanceMonitor> {
        self.performance_monitor.as_ref()
    }
    
    /// 检查工具模块是否可用。
    pub fn is_available(&self) -> bool {
        self.config.enable_validation
    }
    
    /// 执行工具模块健康检查。
    pub fn health_check(&self) -> bool {
        // 检查各组件状态
        let cache_ok = self.cache_manager.is_some() || !self.config.enable_cache;
        let performance_ok = self.performance_monitor.is_some() || !self.config.enable_performance_monitoring;
        
        cache_ok && performance_ok
    }
}

/// 工具模块错误类型枚举。
#[derive(Debug)]
pub enum UtilsError {
    InitializationFailed,    // 初始化失败
    ConfigurationInvalid,    // 配置无效
    CacheError,             // 缓存错误
    PerformanceError,        // 性能错误
    ValidationError,         // 验证错误
}

impl std::fmt::Display for UtilsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UtilsError::InitializationFailed => write!(f, "Utils initialization failed"),
            UtilsError::ConfigurationInvalid => write!(f, "Invalid utils configuration"),
            UtilsError::CacheError => write!(f, "Cache operation failed"),
            UtilsError::PerformanceError => write!(f, "Performance monitoring failed"),
            UtilsError::ValidationError => write!(f, "Validation operation failed"),
        }
    }
}

impl std::error::Error for UtilsError {}

/// 工具模块统计信息结构体。
#[derive(Debug, Clone)]
pub struct UtilsStats {
    pub cache_hits: u64,           // 缓存命中次数
    pub cache_misses: u64,         // 缓存未命中次数
    pub validation_checks: u64,    // 验证检查次数
    pub performance_measurements: u64, // 性能测量次数
    pub errors: u64,               // 错误次数
}

impl Default for UtilsStats {
    fn default() -> Self {
        Self {
            cache_hits: 0,
            cache_misses: 0,
            validation_checks: 0,
            performance_measurements: 0,
            errors: 0,
        }
    }
}

/// 工具模块统计收集器结构体。
pub struct UtilsStatsCollector {
    stats: UtilsStats,             // 统计信息
}

impl UtilsStatsCollector {
    /// 创建新的统计收集器。
    pub fn new() -> Self {
        Self {
            stats: UtilsStats::default(),
        }
    }
    
    /// 记录缓存命中。
    pub fn record_cache_hit(&mut self) {
        self.stats.cache_hits += 1;
    }
    
    /// 记录缓存未命中。
    pub fn record_cache_miss(&mut self) {
        self.stats.cache_misses += 1;
    }
    
    /// 记录验证检查。
    pub fn record_validation_check(&mut self) {
        self.stats.validation_checks += 1;
    }
    
    /// 记录性能测量。
    pub fn record_performance_measurement(&mut self) {
        self.stats.performance_measurements += 1;
    }
    
    /// 记录错误。
    pub fn record_error(&mut self) {
        self.stats.errors += 1;
    }
    
    /// 获取统计信息。
    pub fn get_stats(&self) -> &UtilsStats {
        &self.stats
    }
    
    /// 重置统计信息。
    pub fn reset_stats(&mut self) {
        self.stats = UtilsStats::default();
    }
    
    /// 计算缓存命中率。
    pub fn calculate_cache_hit_rate(&self) -> f64 {
        let total = self.stats.cache_hits + self.stats.cache_misses;
        if total == 0 {
            0.0
        } else {
            self.stats.cache_hits as f64 / total as f64
        }
    }
}
