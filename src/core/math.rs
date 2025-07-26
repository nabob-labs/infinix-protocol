//!
//! math.rs（已拆分）
//!
//! 本文件已细粒度拆分为src/core/math/目录下的advanced.rs、statistics.rs、timeseries.rs、linear_algebra.rs、optimization.rs、safe_math.rs等最小功能单元。
//! 请直接使用`crate::core::math::*`进行数学相关操作。

// 兼容性保留，防止旧代码直接引用本文件类型。
pub use crate::core::math::*;
