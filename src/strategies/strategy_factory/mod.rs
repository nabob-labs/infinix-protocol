//!
//! mod.rs - 策略工厂模块统一入口
//!
//! 统一re-export所有细粒度策略工厂功能单元，便于主程序融合调用。

pub mod factory;
pub mod adapter;

pub use factory::*;
pub use adapter::*; 