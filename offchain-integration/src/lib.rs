//!
//! Offchain Integration Library Entry
//!
//! 本模块为离线集成服务库主入口，统一导出所有子模块，便于外部集成与调用。

// 导入并公开所有子模块。
pub mod dex;      // DEX 相关模块
pub mod error;    // 错误类型模块
pub mod oracles;  // 预言机相关模块
pub mod traits;   // 通用 trait 模块

// 重新导出常用类型和函数，便于外部访问。
pub use dex::*;
pub use error::*;
pub use oracles::*;
pub use traits::*;

/// 离线集成服务库版本号。
pub const OFFCHAIN_INTEGRATION_VERSION: &str = "1.0.0";
