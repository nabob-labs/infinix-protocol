//!
//! validation.rs（已拆分）
//!
//! 本文件已细粒度拆分为src/core/validation/目录下的traits.rs、validator.rs、functions.rs、business.rs、data.rs、performance.rs等最小功能单元。
//! 请直接使用`crate::core::validation::*`进行校验相关操作。

// 兼容性保留，防止旧代码直接引用本文件类型。
pub use crate::core::validation::*;
