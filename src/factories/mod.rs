//!
//! mod.rs - 工厂模块统一入口
//!
//! 统一re-export所有细粒度工厂功能单元，便于主程序融合调用。

pub mod weight_strategy_factory;
pub mod rebalancing_strategy_factory;
pub mod asset_factory;
pub mod factory_utils;
pub mod rebalance_utils;

pub use weight_strategy_factory::*;
pub use rebalancing_strategy_factory::*;
pub use asset_factory::*;
pub use factory_utils::*;
pub use rebalance_utils::*; 