//!
//! types.rs（已拆分）
//!
//! 本文件已细粒度拆分为src/core/types/目录下的trade.rs、algo.rs、strategy.rs、oracle.rs、dex.rs、risk.rs、market.rs、token.rs、validatable.rs等最小功能单元。
//! 请直接使用`crate::core::types::*`进行类型相关操作。

// 兼容性保留，防止旧代码直接引用本文件类型。
pub use crate::core::types::*;
