//!
//! traits.rs（已拆分）
//!
//! 本文件已细粒度拆分为src/core/traits/目录下的behavior.rs、types.rs、dex_oracle.rs、anchor_impl.rs等最小功能单元。
//! 请直接使用`crate::core::traits::*`进行trait相关操作。

// 兼容性保留，防止旧代码直接引用本文件类型。
pub use crate::core::traits::*;
