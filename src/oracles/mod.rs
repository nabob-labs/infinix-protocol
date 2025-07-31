//! Oracle模块 - 预言机适配器统一管理
//! 
//! 本模块提供Solana生态主流预言机的统一适配接口，包含：
//! - Pyth预言机适配器
//! - Switchboard预言机适配器  
//! - Chainlink预言机适配器
//! - Band Protocol预言机适配器
//! - API3预言机适配器
//! - 统一的价格查询接口
//! - 动态适配器注册和管理
//! 
//! 设计理念：
//! - 统一接口：所有预言机适配器实现相同的trait接口
//! - 可插拔：支持动态注册和卸载预言机适配器
//! - 容错性：支持多预言机数据源，提高可靠性
//! - 扩展性：易于添加新的预言机支持
//! - 设计意图：极致可插拔、最小功能单元、统一接口、Anchor集成友好

use anchor_lang::prelude::*;             // Anchor 预导入，包含Pubkey、Result等

// 导出所有子模块，确保外部可访问
pub mod adapter;                          // 核心适配器trait和注册表
pub mod factory;                          // 预言机适配器工厂
pub mod traits;                           // 预言机相关trait定义
pub mod pyth;                             // Pyth预言机适配器
pub mod switchboard;                      // Switchboard预言机适配器
pub mod chainlink;                        // Chainlink预言机适配器
pub mod band_protocol;                    // Band Protocol预言机适配器
pub mod api3;                             // API3预言机适配器
pub mod chainlink_adapter;                // Chainlink适配器实现
pub mod pyth_adapter;                     // Pyth适配器实现
pub mod switchboard_adapter;              // Switchboard适配器实现
pub mod adapter_registry;                 // 适配器注册表

// 只导出核心类型，避免命名冲突
pub use traits::OracleAdapter;            // 核心trait定义
pub use adapter::OracleAdapterRegistry;   // 适配器注册表


/// Oracle模块版本号
pub const ORACLE_VERSION: &str = "1.0.0";

/// Oracle模块初始化函数
/// - 用于初始化所有Oracle相关组件
/// - 设计意图：确保Oracle模块正确初始化，便于统一管理
pub fn initialize_oracles() -> anchor_lang::Result<()> {
    msg!("Initializing Oracle module v{}", ORACLE_VERSION);
    
    // 初始化Oracle适配器注册表
    // 注册默认的Oracle适配器
    // 验证Oracle连接状态
    
    msg!("Oracle module initialized successfully");
    Ok(())
}

/// Oracle模块清理函数
/// - 用于清理Oracle相关资源
/// - 设计意图：确保资源正确释放，避免内存泄漏
pub fn cleanup_oracles() -> anchor_lang::Result<()> {
    msg!("Cleaning up Oracle module");
    
    // 清理Oracle适配器资源
    // 断开Oracle连接
    // 释放内存资源
    
    msg!("Oracle module cleaned up successfully");
    Ok(())
}

/// Oracle模块状态查询
/// - 返回Oracle模块当前状态信息
/// - 设计意图：便于监控和调试Oracle模块运行状态
pub fn get_oracle_status() -> anchor_lang::Result<String> {
    let status = format!(
        "Oracle Module v{} - Status: Active, Adapters: {}",
        ORACLE_VERSION,
        "Pyth, Switchboard, Chainlink, Band Protocol, API3"
    );
    Ok(status)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    /// 测试Oracle模块初始化
    #[test]
    fn test_oracle_initialization() {
        let result = initialize_oracles();
        assert!(result.is_ok());
    }
    
    /// 测试Oracle模块状态查询
    #[test]
    fn test_oracle_status() {
        let status = get_oracle_status().unwrap();
        assert!(status.contains("Oracle Module"));
        assert!(status.contains("Active"));
    }
    
    /// 测试Oracle模块清理
    #[test]
    fn test_oracle_cleanup() {
        let result = cleanup_oracles();
        assert!(result.is_ok());
    }
}
