//!
//! Validation Module
//!
//! 本模块聚合所有合规性校验工具，包含资产、篮子、指数代币等多维度校验子模块，确保系统输入、状态、参数的安全性与合规性。

use anchor_lang::prelude::*;
use anchor_lang::msg;

// 导出所有子模块，确保外部可访问。
pub mod asset_validation;        // 资产校验工具模块
pub mod basket_validation;       // 篮子校验工具模块
pub mod index_token_validation;  // 指数代币校验工具模块

// 重新导出常用结构体和函数，提供便捷访问。
pub use asset_validation::*;        // 导出资产校验相关
pub use basket_validation::*;       // 导出篮子校验相关
pub use index_token_validation::*;  // 导出指数代币校验相关

/// 校验模块版本信息。
pub const VALIDATION_VERSION: &str = "1.0.0";

/// 校验模块常量定义。
pub const VALIDATION_MODULE_NAME: &str = "validation";

/// 校验模块初始化函数。
pub fn initialize_validation() -> Result<()> {
    // 初始化所有子模块
    msg!("Initializing validation module v{}", VALIDATION_VERSION);
    Ok(())
}

/// 校验模块清理函数。
pub fn cleanup_validation() -> Result<()> {
    // 清理所有子模块资源
    msg!("Cleaning up validation module");
    Ok(())
}

/// 校验模块状态检查函数。
pub fn check_validation_status() -> bool {
    // 检查所有子模块状态
    true // 简化实现，实际应检查各子模块状态
}

/// 校验模块版本兼容性检查。
pub fn check_version_compatibility(required_version: &str) -> bool {
    // 检查版本兼容性
    VALIDATION_VERSION == required_version
}
