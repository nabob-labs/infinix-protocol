//!
//! DEX Module Entry
//!
//! 本模块为 DEX 适配器与工厂主入口，统一导出所有子模块，便于外部集成与调用。

// 导入并公开所有子模块。
pub mod adapter;            // 适配器基类与通用逻辑
pub mod adapter_registry;   // 适配器注册表
pub mod factory;            // 工厂模式实现
pub mod jupiter;            // Jupiter 适配器
pub mod lifinity;           // Lifinity 适配器
pub mod lifinity_adapter;   // Lifinity 适配器实现
pub mod logging;            // 日志工具
pub mod mango_adapter;      // Mango 适配器
pub mod meteora;            // Meteora 适配器
pub mod meteora_adapter;    // Meteora 适配器实现
pub mod openbook;           // OpenBook 适配器
pub mod openbook_adapter;   // OpenBook 适配器实现
pub mod orca;               // Orca 适配器
pub mod orca_adapter;       // Orca 适配器实现
pub mod phoenix;            // Phoenix 适配器
pub mod phoenix_adapter;    // Phoenix 适配器实现
pub mod raydium;            // Raydium 适配器
pub mod raydium_adapter;    // Raydium 适配器实现
pub mod traits;             // 通用 trait

// 重新导出常用类型和函数，便于外部访问。
pub use adapter::*;
pub use adapter_registry::*;
pub use factory::*;
pub use jupiter::*;
pub use lifinity::*;
pub use lifinity_adapter::*;
pub use logging::*;
pub use mango_adapter::*;
pub use meteora::*;
pub use meteora_adapter::*;
pub use openbook::*;
pub use openbook_adapter::*;
pub use orca::*;
pub use orca_adapter::*;
pub use phoenix::*;
pub use phoenix_adapter::*;
pub use raydium::*;
pub use raydium_adapter::*;
pub use traits::*;

/// DEX 模块版本号。
pub const DEX_MODULE_VERSION: &str = "1.0.0";
