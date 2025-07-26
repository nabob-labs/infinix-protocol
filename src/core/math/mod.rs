//!
//! mod.rs - core::math模块统一入口
//!
//! 统一re-export所有最小功能单元，便于主模块融合调用。

pub mod advanced;
pub mod statistics;
pub mod timeseries;
pub mod linear_algebra;
pub mod optimization;
pub mod safe_math;

pub use advanced::*;
pub use statistics::*;
pub use timeseries::*;
pub use linear_algebra::*;
pub use optimization::*;
pub use safe_math::*; 